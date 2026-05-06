# Coverage — Quality gate

> **Step 10**: floor 60% line coverage globale del client Rust, applicato
> in CI come gate di regressione. Roadmap verso 70%.

## Stato attuale

Coverage misurato il 2026-05-06 con `cargo-llvm-cov` (toolchain stable,
profilo test):

- **Line coverage globale: 60.12%** (5 893 linee strumentate, 2 350 non
  coperte)
- **Region coverage globale: 63.33%**
- **Function coverage globale: 67.64%**

CI gate: `--fail-under-lines 60`. Sotto questa soglia il workflow
`rust-test` fallisce e blocca il merge.

## Per modulo (snapshot 2026-05-06)

Ordinato per coverage discendente:

| Modulo | Lines | Coverage |
|---|---|---|
| `linting.rs` | 573 | 95.99% ✅ |
| `embeddings_store.rs` | 329 | 94.53% ✅ |
| `prompt_componibili.rs` | 361 | 91.69% ✅ |
| `migrazione.rs` | 87 | 88.51% ✅ |
| `errore.rs` | 54 | 90.74% ✅ |
| `statistiche.rs` | 250 | 81.20% ✅ |
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

## Roadmap verso 70%

I 4 file sotto 50% pesano ~1 800 linee (30% del totale strumentato).
Portarli a 60% recupererebbe ~180 linee → +3% globale.

Priorità per v0.4.x:

1. **`embeddings.rs` 28% → 50%**: estrarre helper testabili dalla logica
   di download (`scarica_modello`, `scarica_runtime`) con HTTP mockato,
   testare `mean_pooling`/`l2_normalize` con tensori sintetici. Stima
   +250 linee.
2. **`vault.rs` 44% → 60%**: scenari mancanti su `vault_lock_then_unlock`,
   `vault_re_key`, gestione errori sblocco con password lunga/corta.
   Stima +60 linee.
3. **`audit.rs` 52% → 65%**: edge case CSV escape (virgolette annidate,
   newline embed), filtri combinati `tipo + range temporale + utente`.
   Stima +50 linee.
4. **`import_export.rs` 29% → 50%**: scenari di round-trip JSON/CSV
   con segnaposti complessi, gestione versione schema mismatch.
   Stima +75 linee.
5. **`embeddings_backfill.rs` 10% → 40%**: dopo il punto 1, i path di
   batching diventano testabili senza dipendenza dal modello reale.

Target intermedio: **65% line coverage globale entro v0.4.0**, **70%**
entro v0.5.

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
