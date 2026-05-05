# Spike 3 — Modello embedding qualitative IT/EN

> **Stato**: ✅ **PASSED v2** — confermata decisione v1 dopo riapertura 2026-05-05.
>
> **Verdict corrente**: la scelta resta **`paraphrase-multilingual-MiniLM-L12-v2`** (118 MB ONNX). La riapertura 2026-05-05 ha valutato 3 candidati 2024-2025: `multilingual-e5-small` performa **peggio** di MiniLM, `gte-multilingual-base` non ha ONNX ufficiale, `EmbeddingGemma-300m` ottiene 100% recall@5 ma con trade-off non giustificati (+180 MB download, 3.7× tempo per-embedding, 15× tempo load) per un guadagno di soli +2.5 punti recall@5. EmbeddingGemma è documentato come alternativa futura se i vincoli di size/performance si rilassano.

## Contesto

Fase 3 Step 1 deve scegliere un modello di embedding per ricerca semantica locale sul vault PaP. L'utenza target è italofona ma scrive prompt spesso in inglese (modelli AI commerciali, codice, paper tech). Il modello deve gestire IT/EN/mixed senza degradare significativamente.

Vincoli tecnici:
- Distribuibile via ONNX (Tauri client carica via crate `ort` o equivalente).
- Caricabile lazy on-demand al primo uso, cached localmente.
- Footprint accettabile sul disco — **preferenza dichiarata: minimizzare trade-off** (download, RAM, tempi).

## Strategia testata

Spike Node.js (`spikes/embedding-models/`) che:

1. Definisce **30 prompt** realistici per casi d'uso PaP: email business, code/dev, analysis, summarization, creative writing, technical/structured output, plus rumore (ricette, viaggi, spesa). Bilanciati IT/EN.
2. Definisce **10 query** di test con `expected_match` ideale, alcune crosslingue (IT query → EN expected) per stressare la capacità multilingue.
3. Per ogni modello: carica via `@huggingface/transformers` (ONNX runtime in Node), calcola embedding di tutti i prompt (con prefissi `query:`/`passage:` o `task:` quando il modello li richiede), poi per ogni query calcola cosine similarity, ordina top-K.
4. Calcola **recall@3** e **recall@5** medi sul set di query.

Il dataset è piccolo per costruzione (qualitative spike, non benchmark): 30 prompt su un vault reale è realistico per un utente alla prima settimana, e amplifica le differenze fra modelli.

## Risultati v2 (esecuzione 2026-05-05)

```
=== Spike 3 v2 — Embedding models 2026 (IT/EN mixed) ===
Dataset: 30 prompt, 10 query, 4 modelli

| Modello                       | Anno | Size   | Load (ms) | Avg embed (ms) | Recall@3 | Recall@5 |
|-------------------------------|------|--------|-----------|----------------|----------|----------|
| multilingual-MiniLM-L12-v2    | 2021 | 118 MB |      1563 |            9.2 |    85.8% |    97.5% |
| multilingual-e5-small         | 2024 | 118 MB |      4346 |            8.7 |    75.0% |    88.3% |
| gte-multilingual-base         | 2024 | 305 MB |      FAIL: ONNX non disponibile su HuggingFace ufficiale         |
| embeddinggemma-300m           | 2025 | 300 MB |     22856 |           33.6 |    89.2% |   100.0% |
```

### Sorprese principali

- **`multilingual-e5-small` perde** contro MiniLM nonostante sia 3 anni più recente e abbia ricevuto i prefissi `query:`/`passage:` richiesti dal paper. Possibili spiegazioni: dataset (30 prompt, 10 query) troppo piccolo per i benefici dei prefissi, o il modello è ottimizzato per retrieval di documenti lunghi più che per prompt brevi. **Non è un drop-in upgrade**, contrariamente all'aspettativa iniziale.
- **`gte-multilingual-base`** non ha ONNX ufficiale: né `onnx-community/gte-multilingual-base-ONNX` (404) né `Alibaba-NLP/gte-multilingual-base` (no ONNX nel repo). Conversione manuale via `optimum-cli` fuori scope spike.
- **`EmbeddingGemma-300m` vince in qualità** ma con costi proporzionalmente alti.

## Analisi del trade-off EmbeddingGemma vs MiniLM

| Metrica | MiniLM-L12-v2 | EmbeddingGemma-300m | Δ |
|---|---|---|---|
| Recall@5 | 97.5% | 100.0% | +2.5 pt |
| Recall@3 | 85.8% | 89.2% | +3.4 pt |
| Size download | 118 MB | 300 MB | **+180 MB (+153%)** |
| Tempo per-embedding | 9.2 ms | 33.6 ms | **+24.4 ms (3.7×)** |
| Tempo load iniziale | 1.6 s | 22.8 s | **+21.2 s (15×)** |
| RAM con modello caricato | ~50 MB | ~250 MB | **+200 MB** |
| Output dim default | 384 | 768 (truncabile a 256 via MRL) | flessibile |

Considerazioni:
- **+2.5 punti recall@5** su un dataset di 10 query significa letteralmente 1 query in più chiusa (passare da "9.75 query con tutti gli expected nei top-5" a "10/10"). La portata pratica è bassa quando MiniLM è già al 97.5%.
- **+180 MB download** è una barriera reale per utenti con connessioni lente o limitate.
- **3.7× più lento per embedding**: su un backfill iniziale di 10k prompt = ~6 min vs ~1.5 min. Non un dramma, ma non gratis.
- **15× più lento al load**: pagato ad ogni unlock del vault con feature attivata. Meno problematico (async, invisibile).
- **MRL truncation a 256 dim** è un punto a favore di EmbeddingGemma per lo storage in `vec0`, ma non basta a compensare gli altri costi.

## Diagnosi

In benchmark MTEB pubblici, EmbeddingGemma supera MiniLM-L12-v2 di margini più ampi (specie su task di retrieval con documenti più lunghi). Sul **nostro caso d'uso** (prompt brevi, dataset utente piccolo), il guadagno reale si schiaccia a ~2.5 punti recall@5.

Il principio di **"no premature optimization"** applicato qui: MiniLM è già ottimo (97.5%), il prossimo modello deve giustificare i suoi costi con un guadagno proporzionato. EmbeddingGemma non lo fa per il nostro use case.

## Decisione

✅ **Modello scelto: `paraphrase-multilingual-MiniLM-L12-v2`** (decisione v1 confermata in v2).

ID HuggingFace: `Xenova/paraphrase-multilingual-MiniLM-L12-v2` (distribuzione ONNX quantizzata).

### Implementazione di produzione (per Step 1 di Fase 3)

1. Comando Tauri `embeddings_init()`:
   - Verifica esistenza modello in `${data_dir}/models/multilingual-MiniLM-L12-v2.onnx`
   - Se assente, scarica da `https://huggingface.co/Xenova/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/onnx/model_quantized.onnx` con progress bar
   - Carica in memoria via `ort`
2. Comando Tauri `embeddings_compute(text)`: ritorna `Vec<f32>` di 384 dimensioni (output pooled mean + L2-normalized)
3. Setting in Impostazioni > Ricerca: `[ ] Ricerca semantica (richiede download modello ~118 MB al primo uso)`
4. Cache modello con eviction dopo 5 min di idle (per liberare ~50 MB RAM quando inutilizzato).

### Migration path se in futuro si vuole cambiare modello

L'embedding store in `vec0` è dipendente dal modello. Se si cambia modello:
- Marcare la versione del modello in `vault-meta.json` (campo `embedding_model_id` + `embedding_dim`)
- Al boot, se il modello in meta != modello corrente, mostrare prompt utente per re-indicizzazione (lavoro grosso ma reversibile)

### Quando rivalutare EmbeddingGemma

Documentato qui per non perdere il filo: **EmbeddingGemma-300m diventa la scelta giusta se** uno o più dei seguenti cambiano:

- Connessioni a banda larga diventano lo standard universale per l'utenza target → +180 MB non è più una barriera.
- Hardware desktop dell'utenza si stabilizza su CPU recenti che riducono il gap di 3.7× → tempo per-embedding torna sotto i 15 ms.
- Lo use case si sposta su prompt più lunghi (chat history, documenti) dove il vantaggio di EmbeddingGemma diventa più marcato.
- Si aggiunge ricerca su un corpus esterno (non solo vault utente) dove ogni punto di recall conta.

In quel caso si esegue Spike 3 v3 con le condizioni aggiornate. Il dataset di test e il runner restano disponibili in `spikes/embedding-models/`.

## Item rinviati (out of scope per spike)

- **Quantizzazione int8/int4** ulteriore di MiniLM per ridurre footprint sotto 80 MB. Considerare se il download iniziale è troppo lungo per utenti su connessione lenta.
- **Test gte-multilingual-base via conversione manuale ONNX**: skip per ora. Se serve un'opzione 305 MB nel futuro, conversione fattibile via `optimum-cli`.
- **Benchmark P95 latency** su 10k embeddings reali: rimandato a Step 10 quality gate Fase 3.

## Cronologia decisioni

| Versione | Data | Modello scelto | Recall@5 | Stato |
|---|---|---|---|---|
| v1 | 2026-05-04 | `paraphrase-multilingual-MiniLM-L12-v2` | 97.5% | ✅ scelto |
| **v2 (corrente)** | **2026-05-05** | **`paraphrase-multilingual-MiniLM-L12-v2`** (decisione v1 confermata) | **97.5%** | **✅ confermato dopo valutazione di 3 alternative 2024-2025** |

Razionale del confirm: EmbeddingGemma-300m ottiene 100% recall@5 ma con trade-off non giustificati (+180 MB download, 3.7× tempo per-embedding, 15× tempo load) per un guadagno marginale di +2.5 punti su un dataset piccolo. MiniLM resta il miglior trade-off qualità/costo per il nostro use case (prompt brevi, vault locale).

## Riferimenti

- Modello scelto: <https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2>
- ONNX quantizzato (Xenova): <https://huggingface.co/Xenova/paraphrase-multilingual-MiniLM-L12-v2>
- Modelli valutati e scartati in v2:
  - `intfloat/multilingual-e5-small` (2024) — perde contro MiniLM in questo dataset
  - `Alibaba-NLP/gte-multilingual-base` (2024) — no ONNX ufficiale
  - `google/embeddinggemma-300m` (2025) — vince qualità ma trade-off non giustificati
- Google blog EmbeddingGemma: <https://developers.googleblog.com/introducing-embeddinggemma/>
- HuggingFace blog tecnico EmbeddingGemma: <https://huggingface.co/blog/embeddinggemma>
- @huggingface/transformers (transformers.js v3): <https://github.com/huggingface/transformers.js>
