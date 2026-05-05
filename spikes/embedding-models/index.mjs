// Spike 3 — qualitative test embedding models su prompt PaP misti IT/EN.
//
// Riapertura 2026-05-05: la decisione originale (Spike 3 v1) cadeva su
// `paraphrase-multilingual-MiniLM-L12-v2` (2021). Modello vecchio rispetto
// allo stato dell'arte 2025. Aggiungiamo al confronto 3 candidati moderni:
//   - intfloat/multilingual-e5-small (2024, 118MB, drop-in size)
//   - Alibaba-NLP/gte-multilingual-base (2024, 305MB, mid-tier upgrade)
//   - google/embeddinggemma-300m (2025, 300MB, on-device first, MRL)
//
// Strategia: caricare ogni modello via @huggingface/transformers (ONNX
// runtime, scarica .onnx e li runna in Node.js). Calcolare embedding di
// 30 prompt + 10 query. Per ogni query: cosine similarity vs tutti i prompt,
// rank top-5. Confrontare i ranking dei modelli.
//
// Esecuzione (prima volta scarica ~750 MB di modelli):
//   npm install
//   node index.mjs

import { pipeline, env } from '@huggingface/transformers';
import { prompts, queries } from './dataset.mjs';

env.cacheDir = './models';
env.allowLocalModels = true;
env.useFSCache = true;

// Modelli da confrontare. Includono i prefissi specifici richiesti dal
// modello per separare query da documento (e5 e EmbeddingGemma li usano).
const MODELS = [
  {
    id: 'multilingual-MiniLM-L12-v2',
    hf: 'Xenova/paraphrase-multilingual-MiniLM-L12-v2',
    size: '118 MB',
    year: '2021',
    note: 'Baseline storico (vincitore Spike 3 v1)',
    prepQuery: (q) => q,
    prepDoc: (d) => d,
  },
  {
    id: 'multilingual-e5-small',
    hf: 'Xenova/multilingual-e5-small',
    size: '118 MB',
    year: '2024',
    note: 'Microsoft, drop-in stesso budget MiniLM, prefissi query/passage',
    // e5 family: docs ufficiali raccomandano i prefissi.
    prepQuery: (q) => `query: ${q}`,
    prepDoc: (d) => `passage: ${d}`,
  },
  {
    id: 'gte-multilingual-base',
    hf: 'onnx-community/gte-multilingual-base-ONNX',
    size: '305 MB',
    year: '2024',
    note: 'Alibaba, 70+ lingue, mid-tier upgrade',
    prepQuery: (q) => q,
    prepDoc: (d) => d,
  },
  {
    id: 'embeddinggemma-300m',
    hf: 'onnx-community/embeddinggemma-300m-ONNX',
    size: '300 MB',
    year: '2025',
    note: 'Google DeepMind, on-device first, MRL flessibile (768/512/256/128)',
    // EmbeddingGemma usa task-specific prompts (vedi model card Google).
    prepQuery: (q) => `task: search result | query: ${q}`,
    prepDoc: (d) => `title: none | text: ${d}`,
  },
];

function cosineSimilarity(a, b) {
  let dot = 0,
    na = 0,
    nb = 0;
  for (let i = 0; i < a.length; i++) {
    dot += a[i] * b[i];
    na += a[i] * a[i];
    nb += b[i] * b[i];
  }
  return dot / (Math.sqrt(na) * Math.sqrt(nb));
}

async function embed(extractor, text) {
  const out = await extractor(text, { pooling: 'mean', normalize: true });
  return Array.from(out.data);
}

function rankAt(predictions, expected, k) {
  const topK = predictions.slice(0, k).map((p) => p.id);
  const found = expected.filter((e) => topK.includes(e));
  return { found: found.length, total: expected.length, recall: found.length / expected.length };
}

async function runModel(model) {
  console.log(`\n=== Modello: ${model.id} (${model.year}, ${model.size}) ===`);
  console.log(`    HF: ${model.hf}`);
  console.log(`    ${model.note}`);
  console.log('Carico modello...');
  const t0 = Date.now();
  let extractor;
  try {
    extractor = await pipeline('feature-extraction', model.hf, {
      quantized: true,
    });
  } catch (e) {
    console.error(`    FAIL load: ${e.message}`);
    return { model, error: `load: ${e.message}` };
  }
  const tLoad = Date.now() - t0;
  console.log(`    caricato in ${tLoad} ms`);

  console.log(`\nCalcolo embeddings di ${prompts.length} prompt...`);
  const tEmbStart = Date.now();
  const promptEmb = {};
  try {
    for (const p of prompts) {
      promptEmb[p.id] = await embed(extractor, model.prepDoc(p.body));
    }
  } catch (e) {
    console.error(`    FAIL embed prompt: ${e.message}`);
    return { model, error: `embed: ${e.message}` };
  }
  const tEmb = Date.now() - tEmbStart;
  console.log(`    ${prompts.length} embeddings in ${tEmb} ms (avg ${(tEmb / prompts.length).toFixed(1)} ms/embedding)`);

  console.log(`\nQuery e ranking top-5:\n`);
  const queryResults = [];
  let recall3sum = 0;
  let recall5sum = 0;
  for (const query of queries) {
    const qEmb = await embed(extractor, model.prepQuery(query.q));
    const scored = prompts
      .map((p) => ({
        id: p.id,
        lang: p.lang,
        body: p.body,
        score: cosineSimilarity(qEmb, promptEmb[p.id]),
      }))
      .sort((a, b) => b.score - a.score);
    const top5 = scored.slice(0, 5);
    const r3 = rankAt(scored, query.expected, 3);
    const r5 = rankAt(scored, query.expected, 5);
    recall3sum += r3.recall;
    recall5sum += r5.recall;

    console.log(`Q [${query.lang}] "${query.q}"`);
    console.log(`    expected ⊂ ${JSON.stringify(query.expected)}, recall@3 = ${(r3.recall * 100).toFixed(0)}%, recall@5 = ${(r5.recall * 100).toFixed(0)}%`);
    for (const r of top5) {
      const inExp = query.expected.includes(r.id) ? '✓' : ' ';
      console.log(`    ${inExp} ${r.id} [${r.lang}] ${r.score.toFixed(4)}  ${r.body.slice(0, 60)}${r.body.length > 60 ? '…' : ''}`);
    }
    console.log();
    queryResults.push({ query: query.q, lang: query.lang, top5, r3, r5 });
  }

  const avgR3 = recall3sum / queries.length;
  const avgR5 = recall5sum / queries.length;
  console.log(`Recall@3 medio: ${(avgR3 * 100).toFixed(1)}%`);
  console.log(`Recall@5 medio: ${(avgR5 * 100).toFixed(1)}%`);

  return { model, tLoad, tEmb, avgPerEmb: tEmb / prompts.length, avgR3, avgR5, queryResults };
}

async function main() {
  console.log('=== Spike 3 v2 — Embedding models 2026 (IT/EN mixed) ===\n');
  console.log(`Dataset: ${prompts.length} prompt, ${queries.length} query, ${MODELS.length} modelli`);

  const results = [];
  for (const m of MODELS) {
    const r = await runModel(m);
    results.push(r);
  }

  console.log('\n\n=== Riepilogo confronto ===\n');
  console.log('| Modello                       | Anno | Size   | Load (ms) | Avg embed (ms) | Recall@3 | Recall@5 |');
  console.log('|-------------------------------|------|--------|-----------|----------------|----------|----------|');
  for (const r of results) {
    if (r.error) {
      console.log(
        `| ${r.model.id.padEnd(29)} | ${r.model.year} | ${r.model.size.padEnd(6)} | FAIL: ${r.error}`
      );
      continue;
    }
    console.log(
      `| ${r.model.id.padEnd(29)} | ${r.model.year} | ${r.model.size.padEnd(6)} | ${String(r.tLoad).padStart(9)} | ${r.avgPerEmb.toFixed(1).padStart(14)} | ${(r.avgR3 * 100).toFixed(1).padStart(7)}% | ${(r.avgR5 * 100).toFixed(1).padStart(7)}% |`
    );
  }
}

main().catch((e) => {
  console.error('FAIL:', e);
  process.exit(1);
});
