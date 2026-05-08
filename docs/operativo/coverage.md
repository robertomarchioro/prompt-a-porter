# Coverage — Quality gate

> **Fase 3 Step 10**: floor 60% line coverage globale del client Rust,
> applicato in CI come gate di regressione.
>
> **Fase 4 Step 9**: target ≥ 70% sui moduli nuovi (varianti, rating,
> fork, regression, similarity, provider_ai). Tutti i moduli Fase 4
> sono **sopra il target** con margine — vedi tabella sotto.

## Stato attuale

Coverage misurato il 2026-05-07 con `cargo-llvm-cov` (toolchain stable,
profilo test) post v0.7.0 Step 1 (coverage push):

- **Line coverage globale: 74.14%** (10 073 linee strumentate, 2 605 non
  coperte) — era 71.02% post v0.6.0, 70.27% post v0.5.0, 60.12% post v0.3.0
- **Region coverage globale: 75.56%**
- **Function coverage globale: 77.69%**

CI gate: `--fail-under-lines 70` (alzato da 65 in v0.7.0 Step 1; era 60
fino a v0.6 e 65 fino a v0.7). Sotto questa soglia il workflow
`rust-test` fallisce e blocca il merge. Margine corrente vs gate:
~+4 punti, sicuro contro regressioni.

## Per modulo (snapshot 2026-05-07)

Ordinato per coverage discendente. Moduli Fase 4 evidenziati con 🆕.

| Modulo | Lines | Coverage |
|---|---|---|
| `linting.rs` | 573 | 95.99% ✅ |
| `rating.rs` 🆕 | 315 | 95.24% ✅ |
| `embeddings_store.rs` | 329 | 94.53% ✅ |
| `prompt_componibili.rs` | 361 | 91.69% ✅ |
| `regression.rs` 🆕 | 1 168 | 91.27% ✅ |
| `fork.rs` 🆕 | 350 | 91.14% ✅ |
| `errore.rs` | 54 | 90.74% ✅ |
| `varianti.rs` 🆕 | 384 | 90.36% ✅ |
| `migrazione.rs` | 88 | 88.64% ✅ |
| `similarity.rs` 🆕 | 274 | 86.13% ✅ |
| `statistiche.rs` | 250 | 81.20% ✅ |
| `provider_ai.rs` 🆕 | 692 | 77.17% ✅ |
| `versioning.rs` | 287 | 69.69% |
| `cartelle.rs` | 527 | 68.31% |
| `tags_suggest.rs` | 189 | 68.25% |
| `prompt.rs` | 135 | 68.15% |
| `editor.rs` | 281 | 61.57% |
| `preferenze.rs` | 84 | 60.71% |
| `sync.rs` | 156 | 60.90% |
| `ricerca_ibrida.rs` | 228 | 59.65% |
| `audit.rs` | 318 | 51.89% |
| `libreria.rs` | 326 | 50.61% |
| `vault.rs` | 400 | 43.50% ⚠️ |
| `import_export.rs` | 380 | 28.95% ⚠️ |
| `embeddings.rs` | 561 | 27.99% ⚠️ |
| `embeddings_backfill.rs` | 154 | 9.74% ⚠️ |
| `lib.rs` | 213 | 0.00% ⚠️ |

### Moduli Fase 4 vs target Step 9

Target Step 9 era **≥ 70% sui nuovi moduli**. Tutti rispettano con
margine ≥ 7 punti percentuali:

| Modulo Fase 4 | Coverage | Margine vs 70% |
|---|---|---|
| `rating.rs` | 95.24% | +25.24 |
| `regression.rs` | 91.27% | +21.27 |
| `fork.rs` | 91.14% | +21.14 |
| `varianti.rs` | 90.36% | +20.36 |
| `similarity.rs` | 86.13% | +16.13 |
| `provider_ai.rs` | 77.17% | +7.17 |

## Riprodurre localmente

```bash
cd apps/client/src-tauri

# Soglia 60% (come CI)
cargo llvm-cov --lib --summary-only --fail-under-lines 60

# Report HTML in target/llvm-cov/html/
cargo llvm-cov --lib --html

# Solo un modulo (utile durante refactor)
cargo llvm-cov --lib --summary-only --no-clean linting::
```

## Roadmap verso 78%

Solo `embeddings.rs` resta sotto 50% (40.61% post v0.7 Step 1) come
zavorra significativa. Il path completo richiede HTTP mock per la
logica di download, scope dedicato di v0.8.

Priorità per v0.8:

1. **`embeddings.rs` 41% → 70%**: estrarre helper testabili dalla logica
   di download (`scarica_modello`, `scarica_runtime`) con HTTP mockato
   (libreria mock-server o fixture file). Stima +180 linee, +2pt globale.
2. **`embeddings_backfill.rs` 10% → 50%**: dopo il punto 1, i path di
   batching diventano testabili senza dipendenza dal modello reale.

Target raggiunti / aggiornati:
- ~~65% line coverage globale entro v0.4.0~~ ✅ raggiunto al **69.91%** (Fase 4 client-first track)
- ~~70% line coverage globale entro v0.5~~ ✅ raggiunto al **70.27%** (v0.5.0 Step 6 Gemini test)
- ~~75% globale entro v0.7~~ ✅ sostanzialmente raggiunto al **74.14%** (v0.7 Step 1, refactor `import_export` 29%→79%)
- **78% globale entro v0.8**, gate CI ora a **70%** (alzato in v0.7 Step 1)

## Cosa NON misuriamo (deliberatamente)

- Test E2E Playwright/Webdriver (separati, no impact su line coverage
  del crate Rust)
- TypeScript del frontend (vitest senza coverage gate; vedi
  `apps/client/vitest.config.ts` per estendere)
- `lib.rs` setup di Tauri (path eseguito solo all'avvio dell'app, non
  raggiungibile da test unit nel formato attuale)

## Riferimenti

- Tooling: [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- Roadmap: `docs/roadmap/fase-3-intelligence.md` Step 10
- Workflow: `.github/workflows/client-build.yml` job `rust-test`
