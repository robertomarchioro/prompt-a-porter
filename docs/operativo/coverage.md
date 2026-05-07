# Coverage — Quality gate

> **Fase 3 Step 10**: floor 60% line coverage globale del client Rust,
> applicato in CI come gate di regressione.
>
> **Fase 4 Step 9**: target ≥ 70% sui moduli nuovi (varianti, rating,
> fork, regression, similarity, provider_ai). Tutti i moduli Fase 4
> sono **sopra il target** con margine — vedi tabella sotto.

## Stato attuale

Coverage misurato il 2026-05-07 con `cargo-llvm-cov` (toolchain stable,
profilo test) post v0.6.0 Step 1 (hardening):

- **Line coverage globale: 71.02%** (9 525 linee strumentate, 2 760 non
  coperte) — era 70.27% post v0.5.0, 60.12% post v0.3.0
- **Region coverage globale: 72.81%**
- **Function coverage globale: 75.61%**

CI gate: `--fail-under-lines 65` (alzato da 60 in v0.6.0 Step 1). Sotto
questa soglia il workflow `rust-test` fallisce e blocca il merge.
Margine corrente vs gate: ~+6 punti, sicuro contro regressioni.

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

## Roadmap verso 75%

I 2 file ancora sotto 50% pesano ~940 linee (10% del totale strumentato).
Portarli a 60% recupererebbe ~150 linee → +1.5% globale.

Priorità per v0.6.x → v0.7:

1. **`embeddings.rs` 28% → 50%**: estrarre helper testabili dalla logica
   di download (`scarica_modello`, `scarica_runtime`) con HTTP mockato,
   testare `mean_pooling`/`l2_normalize` con tensori sintetici. Stima
   +250 linee.
2. **`import_export.rs` 29% → 50%**: scenari di round-trip JSON/CSV
   con segnaposti complessi, gestione versione schema mismatch.
   Stima +75 linee.
3. **`embeddings_backfill.rs` 10% → 40%**: dopo il punto 1, i path di
   batching diventano testabili senza dipendenza dal modello reale.
4. **`libreria.rs` 59% → 72%** ⏳ in progresso v0.6 Step 1:
   filtri vista (preferiti, privati, team), ordinamento, ricerca testo.

Target raggiunti / aggiornati:
- ~~65% line coverage globale entro v0.4.0~~ ✅ raggiunto al **69.91%** (Fase 4 client-first track)
- ~~70% line coverage globale entro v0.5~~ ✅ raggiunto al **70.27%** (v0.5.0 Step 6 Gemini test)
- **75% globale entro v0.7**, gate CI ora a **65%** (alzato in v0.6 Step 1)

I 5 file in arancio/rosso (≤ 50%) restano la zavorra principale: senza
intervento dedicato su `embeddings.rs`, `vault.rs`, `audit.rs`,
`libreria.rs`, `import_export.rs` (più i casi terminali `embeddings_backfill.rs`
e `lib.rs`) la coverage globale rimane ferma intorno al 70%.

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
