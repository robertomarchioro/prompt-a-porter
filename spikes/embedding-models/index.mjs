// Spike 3 — qualitative test embedding models su prompt PaP misti IT/EN.
//
// Strategia: caricare 2 modelli embedding via @huggingface/transformers (ONNX
// runtime web, scarica i .onnx e li runna in Node.js). Calcolare embedding di
// 30 prompt + 10 query. Per ogni query: cosine similarity vs tutti i prompt,
// rank top-5. Confrontare i ranking dei modelli.
//
// Esecuzione (prima volta scarica ~150 MB di modelli):
//   pnpm install
//   pnpm spike

import { pipeline, env } from '@huggingface/transformers';
import { prompts, queries } from './dataset.mjs';

// Cache locale per evitare ri-scarico ad ogni esecuzione.
env.cacheDir = './models';
// Disabilita la versione browser-native fetch per stabilità su Node 22.
env.allowLocalModels = true;
env.useFSCache = true;

// Modelli da confrontare. ONNX quantizzato dove disponibile per dimensione
// realistica del bundle.
const MODELS = [
  {
    id: 'bge-small-en',
    hf: 'Xenova/bge-small-en-v1.5',
    size: '33 MB',
    note: 'EN-focused, multilingue passabile, raccomandato dal doc Fase 3',
  },
  {
    id: 'multilingual-MiniLM',
    hf: 'Xenova/paraphrase-multilingual-MiniLM-L12-v2',
    size: '118 MB',
    note: 'Multilingue forte, 50+ lingue inclusi IT',
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
  // Recall@k: frazione degli expected che appaiono nei primi k risultati.
  const topK = predictions.slice(0, k).map((p) => p.id);
  const found = expected.filter((e) => topK.includes(e));
  return { found: found.length, total: expected.length, recall: found.length / expected.length };
}

async function runModel(model) {
  console.log(`\n=== Modello: ${model.id} (${model.hf}, ${model.size}) ===`);
  console.log(`    ${model.note}`);
  console.log('Carico modello...');
  const t0 = Date.now();
  const extractor = await pipeline('feature-extraction', model.hf, {
    quantized: true,
  });
  const tLoad = Date.now() - t0;
  console.log(`    caricato in ${tLoad} ms`);

  console.log(`\nCalcolo embeddings di ${prompts.length} prompt...`);
  const tEmbStart = Date.now();
  const promptEmb = {};
  for (const p of prompts) {
    promptEmb[p.id] = await embed(extractor, p.body);
  }
  const tEmb = Date.now() - tEmbStart;
  console.log(`    ${prompts.length} embeddings in ${tEmb} ms (avg ${(tEmb / prompts.length).toFixed(1)} ms/embedding)`);

  console.log(`\nQuery e ranking top-5:\n`);
  const queryResults = [];
  let recall3sum = 0;
  let recall5sum = 0;
  for (const query of queries) {
    const qEmb = await embed(extractor, query.q);
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
  console.log('=== Spike 3 — Embedding models qualitative test (IT/EN mixed) ===\n');
  console.log(`Dataset: ${prompts.length} prompt, ${queries.length} query`);

  const results = [];
  for (const m of MODELS) {
    const r = await runModel(m);
    results.push(r);
  }

  console.log('\n\n=== Riepilogo confronto ===\n');
  console.log('| Modello                     | Size  | Load (ms) | Avg embed (ms) | Recall@3 | Recall@5 |');
  console.log('|-----------------------------|-------|-----------|----------------|----------|----------|');
  for (const r of results) {
    console.log(
      `| ${r.model.id.padEnd(27)} | ${r.model.size.padEnd(5)} | ${String(r.tLoad).padStart(9)} | ${r.avgPerEmb.toFixed(1).padStart(14)} | ${(r.avgR3 * 100).toFixed(1).padStart(7)}% | ${(r.avgR5 * 100).toFixed(1).padStart(7)}% |`
    );
  }
}

main().catch((e) => {
  console.error('FAIL:', e);
  process.exit(1);
});
