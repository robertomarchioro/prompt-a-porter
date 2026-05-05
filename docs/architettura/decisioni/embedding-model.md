# Spike 3 — Modello embedding qualitative IT/EN

> **Stato**: ✅ **PASSED** (eseguito 2026-05-04 su Ubuntu Linux x86_64, Node.js 22, `@huggingface/transformers` 3.x).
>
> **Verdict**: vincitore netto è **`paraphrase-multilingual-MiniLM-L12-v2`** (118 MB ONNX). Penalizza il primo download di 85 MB rispetto a `bge-small-en-v1.5`, ma raddoppia il recall@5 medio sul dataset misto IT/EN (97.5% vs 65.0%) ed è essenziale per query crosslingue (IT→EN o viceversa, comune nell'utenza target).

## Contesto

Fase 3 Step 1 deve scegliere un modello di embedding tra:
- **`bge-small-en-v1.5`** (~33 MB ONNX): EN-focused, multilingue passabile, raccomandato dal doc originale Fase 3
- **`paraphrase-multilingual-MiniLM-L12-v2`** (~118 MB ONNX): multilingue forte, 50+ lingue inclusi italiano

L'utenza target di PaP è italofona ma scrive prompt spesso in inglese (modelli AI commerciali, codice, paper tech). Il modello deve gestire IT/EN/mixed senza degradare significativamente.

## Strategia testata

Spike Node.js (`spikes/embedding-models/`) che:

1. Definisce **30 prompt** realistici per casi d'uso PaP: email business, code/dev, analysis, summarization, creative writing, technical/structured output, plus rumore (ricette, viaggi, spesa). Bilanciati IT/EN.
2. Definisce **10 query** di test con `expected_match` ideale, alcune crosslingue (IT query → EN expected) per stressare la capacità multilingue.
3. Per ogni modello: carica via `@huggingface/transformers` (ONNX runtime in Node), calcola embedding di tutti i prompt, poi per ogni query calcola cosine similarity, ordina top-K.
4. Calcola **recall@3** e **recall@5** medi sul set di query.

Il dataset è piccolo per costruzione (qualitative spike, non benchmark): 30 prompt su un vault reale è realistico per un utente alla prima settimana, e amplifica le differenze fra modelli.

## Risultati

### Esecuzione 2026-05-04

```
=== Spike 3 — Embedding models qualitative test (IT/EN mixed) ===
Dataset: 30 prompt, 10 query

| Modello                     | Size   | Load (ms) | Avg embed (ms) | Recall@3 | Recall@5 |
|-----------------------------|--------|-----------|----------------|----------|----------|
| bge-small-en                | 33 MB  |       963 |            6.9 |    65.0% |    65.0% |
| multilingual-MiniLM         | 118 MB |      2991 |            7.0 |    85.8% |    97.5% |
```

### Esempi qualitativi che mostrano la differenza

**Q (IT) "rendi questa email più professionale" — expected: p01, p02, p05, p06**

bge-small-en non trova `p06` (`"Convert this Slack message into a polished business email"`) perché EN-only nel modello = penalizzazione su query IT.

multilingual-MiniLM trova `p06` come **primo** risultato (score 0.81), seguito dagli equivalenti IT — il senso "messaggio informale → email business" è trasferito crosslingue.

**Q (IT) "tradurre codice Python in altro linguaggio" — expected: p08, p10, p12**

bge-small-en mette `p12` (IT, "spiega questa funzione Python") al primo posto, ma `p08` (EN, "Convert Python to TypeScript") fuori top-3. Il modello capisce IT-IT ma fallisce IT→EN cross.

multilingual-MiniLM trova `p08` come **primo** (score 0.56), poi `p12` (0.54). Match crosslingue corretto.

**Q (IT) "come si fa la carbonara" — expected: p26 (rumore, deve isolarsi)**

Entrambi trovano `p26` al primo posto, recall@3 = 100% per entrambi. Il segnale di rumore funziona — buon health check.

## Diagnosi

`bge-small-en-v1.5` è OK per workspace **anglofoni puri** ma penalizza significativamente le query miste. Il modello vede il prompt IT come "stringa esotica con poca semantica" rispetto al suo training set EN.

`paraphrase-multilingual-MiniLM-L12-v2` recupera 30+ punti di recall sul mix IT/EN. Costo:
- **First download**: +85 MB (118 vs 33 MB) — una sola volta, lazy quando utente attiva la feature
- **Storage on disk**: +85 MB cache modello in `~/.local/share/prompt-a-porter/models/`
- **Memory at runtime**: ~50 MB RAM in più con modello caricato (configurabile: scaricamento dopo 5 min idle previsto in Step 1)
- **Performance per embedding**: trascurabile (7.0 vs 6.9 ms su CPU x86_64)
- **Performance load iniziale**: 3× lento (3.0 vs 1.0 s) — invisibile all'utente perché async

## Decisione

✅ **Modello scelto: `paraphrase-multilingual-MiniLM-L12-v2`** (Xenova/paraphrase-multilingual-MiniLM-L12-v2 su HuggingFace, distribuzione ONNX quantizzata).

Aggiorna `docs/roadmap/fase-3-intelligence.md` Step 1 sostituendo la "decisione consigliata" da `bge-small-en-v1.5` a multilingual-MiniLM. La crescita di download è gestita dalla strategia "**download on-demand al primo uso**" già prevista nel doc — l'utente vede una progress bar durante il primo unlock con feature attivata, dopodiché il modello è cached localmente.

### Implementazione di produzione (per Step 1)

1. Comando Tauri `embeddings_init()`:
   - Verifica esistenza modello in `${data_dir}/models/multilingual-MiniLM-L12-v2.onnx`
   - Se assente, scarica da `https://huggingface.co/Xenova/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/onnx/model_quantized.onnx` con progress bar
   - Carica in memoria via ort
2. Comando Tauri `embeddings_compute(text)`: ritorna `Vec<f32>` di 384 dimensioni (output pooled mean + L2-normalized)
3. Setting in Impostazioni > Ricerca: `[ ] Ricerca semantica (richiede download modello ~118 MB al primo uso)`

## Item rinviati (out of scope per spike)

- **Quantizzazione int8/int4** ulteriore per ridurre footprint ([dynamic_int8](https://huggingface.co/docs/transformers/main/en/main_classes/quantization) può portare il modello a ~30-40 MB con perdita ~5% di qualità). Da considerare in Step 1 implementation se il download iniziale è troppo lungo per utenti su connessione lenta.
- **Confronto con modelli IT-specifici** come `mxbai-embed-large` o variants `e5-multilingual-large`. Probabilmente over-kill (modelli >500 MB), il guadagno marginale non giustifica.
- **Benchmark P95 latency** su 10k embeddings: rimandato a Step 10 quality gate Fase 3.

## Riferimenti

- Modello scelto: <https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2>
- ONNX quantizzato (Xenova): <https://huggingface.co/Xenova/paraphrase-multilingual-MiniLM-L12-v2>
- @huggingface/transformers (transformers.js): <https://github.com/huggingface/transformers.js>
- Sentence Transformers: <https://www.sbert.net/>
