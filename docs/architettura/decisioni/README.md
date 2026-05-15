# Decisioni architetturali (ADR)

Questa cartella raccoglie le **Architecture Decision Records** del progetto: scelte tecniche significative con razionale, alternative considerate ed evidenza empirica quando disponibile (spike, benchmark, test).

Gli ADR servono per:
- ricostruire il **perché** di una scelta a distanza di tempo;
- documentare cosa è stato **scartato** e perché (così non si rifà la stessa valutazione due volte);
- tenere traccia delle **assunzioni di design** che condizionano implementazioni future.

## Convenzioni

- Un file per decisione, naming descrittivo (es. `sqlite-vec-sqlcipher.md`).
- Ogni doc include: stato (PASSED/FAILED/PARTIAL/SUPERSEDED), data, contesto, strategia testata, risultati, decisione finale, item rinviati.
- Quando una decisione viene rivista, si crea un nuovo ADR e si marca il vecchio come SUPERSEDED senza cancellarlo.

## ADR registrati

| Doc | Stato | Data | Decisione |
|---|---|---|---|
| [`sqlite-vec-sqlcipher.md`](./sqlite-vec-sqlcipher.md) | ✅ PASSED | 2026-05-04 | sqlite-vec è compatibile con SQLCipher via auto-extension statico — Step 2 di Fase 3 procede col path standard, niente fallback architetturali |
| [`onnx-bundle.md`](./onnx-bundle.md) | ✅ PARTIAL | 2026-05-04 | ONNX Runtime aggiunge ~14-22 MB al bundle Tauri (4-8× crescita) — accettabile, si procede con bundle inclusivo via `ort` 2.x default |
| [`embedding-model.md`](./embedding-model.md) | ✅ PASSED v2 (2026-05-05) | 2026-05-04 → 2026-05-05 | Modello embedding scelto: **`paraphrase-multilingual-MiniLM-L12-v2`** (118 MB ONNX, decisione v1 **confermata** in v2). EmbeddingGemma-300m valutato e scartato per trade-off (+180 MB / 3.7× per-embed) non giustificati su +2.5 pt recall@5 |
| [`authenticode-signing.md`](./authenticode-signing.md) | ✅ PASSED | 2026-05-15 | Certum SimplySign Cloud Open Source per firma Authenticode release Windows. Approccio A — Windows runner GitHub Actions con SimplySign Desktop + TOTP automatizzata da seed in GitHub Secret. Apre M1 v1.0 sub-PR M1.2-M1.7 |

## Aspetti correlati in altri cluster

- **Roadmap delle fasi che dipendono da queste decisioni**: [`../../roadmap/`](../../roadmap/)
- **Item rinviati che si sbloccano con queste decisioni**: [`../../roadmap/rinvii.md`](../../roadmap/rinvii.md)
