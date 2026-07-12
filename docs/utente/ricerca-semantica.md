# Ricerca semantica

> Come funziona la ricerca ibrida (lessicale + embedding), come attivarla e bilanciarla, cosa comporta per memoria e privacy. Disponibile da `v0.3.0` (Fase 3 — Intelligenza & authoring).

La ricerca semantica permette di trovare prompt **per significato**,
non solo per match esatto delle parole. Cerca "scrivere email
formale" e ti trova anche prompt che parlano di "redigere comunicazione
professionale", anche se non hanno nessuna parola in comune.

## Come funziona

Sotto il cofano la ricerca si chiama **ibrida**: combina due segnali
indipendenti via Reciprocal Rank Fusion (RRF) pesata.

| Segnale | Cosa fa | Quando vince |
|---|---|---|
| **Lessicale (FTS5)** | Match esatto di parole/prefissi | Termini tecnici, nomi propri, keyword specifiche |
| **Semantico (vec0)** | Distanza cosine tra embedding | Sinonimi, parafrasi, descrizione concettuale |

La fusione produce un ranking unico in cui i prompt che appaiono in
entrambe le pipeline vengono premiati.

### Modello di embedding

`paraphrase-multilingual-MiniLM-L12-v2` (Xenova fork ONNX, 384 dim,
~118 MB). Funziona bene su **italiano + inglese mescolati** (50+
lingue supportate dal modello base).

Il modello si scarica al primo uso da HuggingFace. Tutto resta
**locale**: nessuna chiamata a server esterni durante le ricerche.

> Razionale completo: `docs/architettura/decisioni/embedding-model.md`.

## Attivare la feature

1. Apri **Impostazioni** → **Ricerca & Embeddings** (gruppo AI).
2. Premi **Abilita**: il client scarica il modello (~118 MB) e il
   runtime ONNX (~10 MB) sotto `${data_dir}/models/` e
   `${data_dir}/onnxruntime/`.
3. Il primo backfill calcola gli embedding di tutti i prompt e tag
   esistenti — questa è una tantum, in background, con avanzamento
   nello stesso pannello.
4. Da qui in poi la Command Palette usa la ricerca ibrida automaticamente.

## Bilanciare lessicale ↔ semantico (`α`)

Lo slider **α** in Impostazioni → Ricerca & Embeddings regola il peso
fra le due pipeline:

| α | Effetto |
|---|---|
| `0.0` | Solo FTS5 — comportamento legacy. Equivale alla ricerca pre-v0.3 |
| `0.5` (default) | Bilanciato. Coglie sia keyword sia significato |
| `1.0` | Solo semantico — utile per esplorare "prompt simili" senza match testuali |

Sopra ogni risultato ci sono due chip che indicano la posizione del
prompt in ciascuna pipeline (lex/sem). Se vedi `lex 1` ma `sem -`,
significa "matcha letteralmente ma il senso è distante".

## Performance

Bench su Intel Xeon E-2288G (numeri completi:
[`docs/operativo/bench-ricerca-ibrida.md`](../operativo/bench-ricerca-ibrida.md)):

| N prompt | Encoding query | Lex+Sem+RRF | Totale |
|---|---|---|---|
| 1 000 | ~30 ms | <1 ms | ~31 ms |
| 10 000 | ~30 ms | ~8 ms | ~38 ms |

Sotto i 100 ms anche su workspace molto grandi.

## Memoria & idle-unload

Il modello + runtime occupano ~150 MB in RAM. Se non usi la ricerca
semantica per un po', il client li **scarica automaticamente** per
liberare memoria.

In **Impostazioni → Ricerca & Embeddings → Scarica modello dopo
inattività** scegli la soglia (default 5 minuti, 0=mai). Dopo lo
scarico la ricerca torna a FTS-only fino al prossimo riavvio del
client (il riload automatico arriverà in una versione successiva).

## Tag suggeriti

L'editor del prompt usa lo stesso embedding del modello per
suggerirti **tag pertinenti** in base al testo che stai scrivendo.
Funziona quando il workspace ha già almeno 10 tag con embedding
calcolato; sotto questa soglia, si torna ai "tag più frequenti".

## Privacy

Tutto resta nel vault locale cifrato. **Nessun testo lascia mai la
tua macchina**: il modello gira in ONNX Runtime sulla CPU,
l'embedding è un vettore di 384 numeri float salvato accanto al
prompt nello stesso file SQLite.

## Limiti noti

- Embedding query ricalcolato a ogni ricerca (~30 ms): non c'è
  cache per query identiche di seguito.
- Riload automatico post-idle-unload non ancora implementato.
- KNN è brute force su sqlite-vec (lineare in N): a 100k+ prompt
  potrebbe essere necessario passare a HNSW. Vedi roadmap Fase 4+.

## Disabilitare

In **Impostazioni → Ricerca & Embeddings** togli la spunta a "Usa
ricerca semantica nelle query": la Command Palette torna a FTS-only.
I file modello/runtime restano su disco — per cancellarli del tutto
elimina manualmente le cartelle `models/` e `onnxruntime/` dentro
`${data_dir}` del client.

## Vedi anche

- [`embedding-model.md`](../architettura/decisioni/embedding-model.md) — ADR sulla scelta del modello di embedding.
- [`sqlite-vec-sqlcipher.md`](../architettura/decisioni/sqlite-vec-sqlcipher.md) — ADR su sqlite-vec dentro il vault SQLCipher.
- [`onnx-bundle.md`](../architettura/decisioni/onnx-bundle.md) — ADR sul bundling del runtime ONNX.
- [`bench-ricerca-ibrida.md`](../operativo/bench-ricerca-ibrida.md) — numeri completi dei bench di performance.
