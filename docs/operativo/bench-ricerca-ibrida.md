# Bench ricerca ibrida — Quality gate Fase 3

> **Step 10**: validazione P95 ricerca ibrida < 100 ms su 10k prompt.

## Contesto

La ricerca ibrida (`prompt_cerca_ibrida`) combina FTS5 (lessicale) e sqlite-vec
(semantica) via Reciprocal Rank Fusion pesata. Lo Step 10 di Fase 3 fissa
come obiettivo di accettazione **P95 < 100 ms su un workspace da 10 000
prompt**.

Il bench harness vive in `apps/client/src-tauri/benches/ricerca_ibrida.rs`,
costruito con [criterion](https://crates.io/crates/criterion). Il dataset è
generato in modo riproducibile (seed fisso) da
`apps/client/src-tauri/examples/genera_dataset.rs`.

## Cosa misuriamo

I componenti in ordine di dipendenza:

1. **`cerca_lessicale_fts5`** — query FTS5 con `MATCH` + `ORDER BY rank` +
   `LIMIT 50`.
2. **`search_nearest_vec0`** — KNN brute force su sqlite-vec (`vec0`
   virtual table), embedding query 384-dim L2-normalized.
3. **`ricerca_completa_lex_sem_rrf`** — lex + sem + RRF fusion, l'intero
   path eccetto il `compute_embedding` del query (richiede modello ONNX
   scaricato in CI; il costo dell'encoding è caratterizzato a parte dalla
   doc del modello).

## Esecuzione

```bash
cd apps/client/src-tauri
cargo bench --bench ricerca_ibrida
```

Output HTML report in `target/criterion/`.

Per generare un dataset out-of-band (utile per smoke test manuali sul
client):

```bash
cargo run --example genera_dataset --release -- --output /tmp/pap.db --prompts 10000
```

## Risultati

**Hardware di riferimento**: Intel Xeon E-2288G @ 3.70 GHz, 4 core, Linux
6.8.

Numeri rilevati il 2026-05-06, profilo `release`, criterion sample
size 50.

### `cerca_lessicale_fts5`

| N prompt | Tempo (mediana ± low/high) |
|---|---|
| 1 000 | 341 µs (336 / 349) |
| 10 000 | **2.06 ms** (2.04 / 2.08) |

Scala ≈ lineare con N: 6× tempo per 10× dati. Atteso per FTS5 con `LIMIT`.

### `search_nearest_vec0`

| N prompt | Tempo (mediana ± low/high) |
|---|---|
| 1 000 | 597 µs (594 / 601) |
| 10 000 | **6.25 ms** (6.17 / 6.34) |

KNN brute force lineare in N. Costo dominante: 384 dot product per
embedding nel pool. Comportamento prevedibile e in linea con i numeri
documentati per sqlite-vec.

### `ricerca_completa_lex_sem_rrf`

| N prompt | Tempo (mediana ± low/high) |
|---|---|
| 1 000 | 978 µs (972 / 985) |
| 10 000 | **8.29 ms** (8.19 / 8.39) |

Somma lex + sem + RRF, escluso encoding del query.

## Compute embedding del query (caratterizzazione esterna)

Il path completo che il command Tauri esegue include un
`compute_embedding_opt` sul testo della query — un'inference ONNX
MiniLM-L12-v2 su 384-dim. Stima da spike Step 1 (vedi
`docs/architettura/decisioni/embedding-model.md`):

- CPU desktop (i5/Ryzen 5+ moderno): **20–40 ms** per query (cold runs
  più lenti, warm runs ~25 ms tipici).

## Bilancio rispetto al target

Sommando il bench misurato + la stima encoding query in CI/desktop:

| Componente | 10k |
|---|---|
| Encoding query (MiniLM) | ~30 ms |
| Lex + Sem + RRF | ~8 ms |
| **Totale ricerca ibrida** | **~38 ms** |

Target P95 Step 10: **< 100 ms**. Margine reale ≥ 2.5× sull'hardware
di riferimento. Su hardware più lento (laptop ARM, cloud micro-instance) si
mantiene comunque sotto soglia con buona probabilità.

## Note di realismo

- Embedding del dataset: random Gauss + L2-normalize. **Non rappresenta
  similarità semantica reale**, ma riproduce le proprietà di shape e
  norma — quindi il costo del `search_nearest` è realistico (lo stesso
  numero di operazioni floating-point per embedding di un modello vero).
- Body dei prompt: 15–60 parole italiane/inglesi miste, distribuzione
  realistica per un prompt LLM tipico.
- Cache disco / FS warm: il primo bench può essere più lento del
  cold-start tipico. Criterion fa warm-up 3 s prima del campione utile.

## Quando rileggere questo doc

- Se il numero di prompt mediano per workspace cresce > 10 000 in
  produzione.
- Se cambia il modello di embedding (dimensione ≠ 384 → altro costo
  KNN).
- Prima di tag `v0.4.x` o major release con cambi al search path.
- Quando si aggiunge una pipeline aggiuntiva (es. re-rank LLM) che
  modifica il bilancio di latenza.

## Riferimenti

- Roadmap: `docs/roadmap/fase-3-intelligence.md` Step 10
- Modello embedding: `docs/architettura/decisioni/embedding-model.md`
- Auto-extension sqlite-vec: `docs/architettura/decisioni/sqlite-vec-sqlcipher.md`
- Bench harness: `apps/client/src-tauri/benches/ricerca_ibrida.rs`
- Generator dataset: `apps/client/src-tauri/examples/genera_dataset.rs`
