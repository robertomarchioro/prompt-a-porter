# CI Workflows — Guida operativa

> Riferimento veloce per capire **quale workflow scatta su quale modifica** e come gestire correttamente l'apertura di una PR senza aspettare CI che non si attiveranno.

## Workflow esistenti (10)

| File | Trigger | Job principali |
|---|---|---|
| `bootstrap.yml` | `workflow_dispatch` (manuale) | Genera `pnpm-lock.yaml` + icone Tauri |
| `cli-build.yml` | `push:main` + `pull_request` su `apps/cli/**` (no md) + `workflow_dispatch` | `lint-and-test` (Go + golangci-lint + coverage report, no gate), `cross-compile` matrix 6 OS/arch |
| `client-build.yml` | `push:main` + `pull_request` su `apps/client/**` ∪ `packages/**` ∪ `pnpm-workspace.yaml` ∪ `pnpm-lock.yaml` ∪ `.github/workflows/client-build.yml` (no md) + `workflow_dispatch` | `lint-and-test` (TypeScript, **coverage gate 70%**), `rust-test` (cargo-llvm-cov **gate 80% line**, `--locked`) |
| `mcp-server-build.yml` | `push:main` + `pull_request` su `apps/mcp-server/**` ∪ `pnpm-workspace.yaml` ∪ `pnpm-lock.yaml` (no md) + `workflow_dispatch` | `lint-and-build` (TS type check + build + **coverage gate 80%** su `@pap/shared-schema` e `@pap/mcp-server`) |
| `server-build.yml` | `push:main` + `pull_request` su `apps/server/**` (no md) + `workflow_dispatch` | `lint-and-test` (Go vet + test `-race` + **coverage gate 50%**) |
| `release.yml` | `push:tags:v*` | `build` matrix 3 runner / 4 target: Windows (NSIS + portable zip), Linux (deb + AppImage), macOS **universale** arm64+x86_64 (app + dmg) → asset draft release via `tauri-action`; job `cli` allega i binari della CLI `pap` |
| `security-audit.yml` | `workflow_dispatch` + `schedule cron giornaliero 05:23 UTC` | `cargo audit`, `govulncheck` server+CLI, `pnpm audit`; su failure il job `notifica-fallimento` apre/aggiorna la issue "Security Audit fallito: intervenire" |
| `dep-canary.yml` | `workflow_dispatch` + `schedule cron mar/ven 05:17 UTC` | Canary **non bloccante**: toolchain Rust latest senza pin + `cargo update` (ignora il lock) per scoprire rotture upstream; su failure apre/aggiorna una issue con label `dep-canary` |
| `macos-smoke.yml` | `workflow_dispatch` (solo manuale) | Smoke build Tauri unsigned su `macos-14` (artifact di workflow, retention 7 gg, mai su release). Nato pre-certificato Apple; utile per validare cambi alla toolchain macOS senza taggare |
| `site-deploy.yml` | `push:main` su `apps/site/**` ∪ `docs/**` ∪ `pnpm-workspace.yaml` ∪ `pnpm-lock.yaml` ∪ `.github/workflows/site-deploy.yml` + `workflow_dispatch` — **niente trigger su PR** | `build` (VitePress) → `deploy` su GitHub Pages (www.promptaporter.it) |

## Path → workflow attivati (mappa esaustiva)

| Path modificato | Workflow che scatta |
|---|---|
| `apps/cli/**` (escluso md/CHANGELOG/LICENSE) | **cli-build** |
| `apps/client/**` (escluso md/CHANGELOG/LICENSE) | **client-build** |
| `apps/mcp-server/**` (escluso md/CHANGELOG/LICENSE) | **mcp-server-build** |
| `apps/server/**` (escluso md/CHANGELOG/LICENSE) | **server-build** |
| `packages/**` (escluso md) | **client-build** |
| `pnpm-workspace.yaml` o `pnpm-lock.yaml` | **client-build** + **mcp-server-build** (su PR); + **site-deploy** al merge su main |
| `.github/workflows/client-build.yml` | **client-build** (è auto-listato nei suoi `paths`) |
| `.github/workflows/cli-build.yml` | ⚠️ **nessuno** (non auto-listato) |
| `.github/workflows/mcp-server-build.yml` | ⚠️ **nessuno** (non auto-listato) |
| `.github/workflows/server-build.yml` | ⚠️ **nessuno** (non auto-listato) |
| `.github/workflows/release.yml` | ⚠️ **nessuno** su PR — solo su push di tag `v*` |
| `.github/workflows/bootstrap.yml` | ⚠️ **nessuno** (solo manuale) |
| `.github/workflows/security-audit.yml` | ⚠️ **nessuno** su PR (schedule + manuale) |
| `.github/workflows/dep-canary.yml` | ⚠️ **nessuno** su PR (schedule + manuale) |
| `.github/workflows/macos-smoke.yml` | ⚠️ **nessuno** (solo manuale) |
| `.github/workflows/site-deploy.yml` | ⚠️ **nessuno** su PR — **site-deploy** al merge su main (auto-listato) |
| `apps/site/**` | ⚠️ **nessuno** su PR — **site-deploy** al merge su main |
| `docs/**` | ⚠️ **nessuno** su PR — **site-deploy** al merge su main |
| `CHANGELOG.md` (root) | ⚠️ **nessuno** |
| `README.md`, `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (root) | ⚠️ **nessuno** |
| `Cargo.lock`, `apps/client/src-tauri/Cargo.toml/lock` | **client-build** (sotto `apps/client/**`) |
| Tag `v*` (push) | **release** (build cross-OS + draft release) |

⚠️ = nessun job CI di PR validation. Significa **PR mergiabile senza green check**: vanno valutate manualmente o con audit locale.

## Anti-pattern: cosa NON fare

### ❌ Aspettare CI su PR doc-only

Se la PR cambia **solo** file in `docs/**`, `apps/site/**` o `CHANGELOG.md`, non scatta nulla **sulla PR**. Aspettare il monitor è tempo sprecato. **Vissuto**: PR #54 (chiusura Step 11 doc) e PR #56 (roadmap update) — in entrambi i casi ho atteso CI inesistente.

Attenzione però: **al merge su main** dei path `docs/**` / `apps/site/**` parte `site-deploy` che ripubblica il sito. Se la PR tocca contenuti visibili sul sito, verificare l'esito del deploy post-merge (`gh run list --workflow=site-deploy.yml --limit 1`).

### ❌ Modificare un workflow YAML pensando che si auto-validi

Solo `client-build.yml` è auto-listato nei propri `paths`. Per gli altri workflow, una modifica al loro YAML **non scatta una run di validazione**. **Vissuto**: PR #53 (coverage CI) — modificai solo `client-build.yml` + `docs/operativo/coverage.md`, e il workflow non partiva. Risolto aggiungendo `.github/workflows/client-build.yml` ai propri `paths`.

Per testare modifiche a workflow non auto-listati: usare `workflow_dispatch` manuale dopo merge, oppure includere nella stessa PR un cambio dummy al codice coperto dai `paths` (anche solo un commento), oppure estendere temporaneamente i `paths` per testare.

### ❌ Tag senza prima passare via PR + main

Il workflow `release.yml` triggera **solo** su push di tag `v*`. Se taggo un commit non ancora su main, la build cross-OS gira sulla versione "ufficiosa". Sequenza corretta: PR → merge su main → bump version (PR aggiuntiva se serve) → tag su main → push tag.

## Checklist operativa per ogni PR

Da seguire **prima** di aprire la PR:

1. **`git diff --stat origin/main...HEAD` → lista file modificati**
2. Per ogni path, mappa contro la tabella sopra → ottieni la **lista dei workflow attesi**
3. Se lista vuota:
   - PR doc-only / config-only senza impatto code → segnalo subito all'utente "nessun CI atteso, PR mergiabile a vista"
   - Se invece c'erano modifiche di codice ma il path filter le esclude (es. workflow YAML non auto-listato) → considera se estendere i `paths` o testare con `workflow_dispatch` post-merge
4. Se lista non vuota:
   - Apri PR, attendi via `Monitor` solo i workflow attesi (non aspettare un `rust-test` se la PR tocca solo `apps/server/`)
5. Dopo che i workflow attesi sono tutti pass:
   - `gh pr merge --squash --delete-branch`
   - `git checkout main && git pull --ff-only`

## Formula per stimare l'attesa CI

| Workflow | Tempo medio (run completo) |
|---|---|
| `lint-and-test` (TS) | ~30 s |
| `lint-and-build` (MCP) | ~30 s |
| `rust-test` (cargo-llvm-cov) | ~3-5 min |
| `lint-and-test` (server Go) | ~1-2 min |
| `cli-build` `lint-and-test` + `cross-compile` matrix 6 | ~5-8 min totali |
| `site-deploy` (build VitePress + Pages, post-merge) | ~1-4 min |
| `release.yml` cross-OS | ~15-20 min |

Se il monitor sta polling da > 2× il tempo medio del workflow atteso, **probabilmente non è triggerato** — fare check con `gh pr checks <num>` e se "no checks reported" rivedere i path filter.

## Comando rapido per debug "perché non scatta?"

```bash
# Lista i file modificati in un PR rispetto a main
gh pr diff <num> --name-only

# Verifica workflow attivi sulla PR
gh pr checks <num>

# Vedi i run più recenti del repo
gh run list --limit 10

# Filtra per workflow specifico
gh run list --workflow=client-build.yml --limit 5

# Vedi il log di un job
gh run view <id> --log-failed
```

## Possibili miglioramenti (technical debt)

I tre workflow non auto-listati (`cli-build.yml`, `mcp-server-build.yml`, `server-build.yml`) andrebbero estesi per auto-testarsi su modifica:

```yaml
# In cli-build.yml, aggiungere ai paths:
- ".github/workflows/cli-build.yml"
# Idem per mcp-server-build.yml e server-build.yml
```

Tracking: nuovo bullet in `docs/roadmap/rinvii.md` § 3 cosmetici.

## Riferimenti

- File workflow: `.github/workflows/`
- Quality gate Fase 3: [`docs/operativo/coverage.md`](../operativo/coverage.md)
- Bench performance: [`docs/operativo/bench-ricerca-ibrida.md`](../operativo/bench-ricerca-ibrida.md)
- Roadmap rinvii: [`docs/roadmap/rinvii.md`](../roadmap/rinvii.md)
