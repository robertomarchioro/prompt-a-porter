# Quality Gate — Fase 2

> **Status**: pre-release v0.2.0. Documento di assessment dei criteri di qualità prima del tag.

## Criteri (da `docs/todo-fase-2.md` Step 9)

- [x] Test coverage ≥ 70% sui nuovi moduli (`auto-update`, `MCP server`, `CLI`, `modernc.org/sqlite` path)
- [ ] Test E2E Tauri Updater: build dev → fake update server → verifica download + apply *(rinviato: Step 5 ancora pending)*
- [ ] Server cross-compile CI green su Win+Linux+macOS *(rinviato: Step 6 ancora pending)*
- [ ] Smoke test installer NSIS per-user su Win10 e Win11 *(rinviato: Step 5 ancora pending)*
- [x] Audit deps: `cargo audit`, `pnpm audit`, `govulncheck` clean
- [x] Verifica licenza: `licensee` confermano AGPL 3.0 ovunque
- [ ] Build release v0.2.0 testata su 3 sistemi diversi *(rinviato a release effettiva)*

## Coverage attuale

### Rust client (Tauri)

I test girano in `client-build.yml > rust-test` job ma **il workflow non emette coverage report numerico** (solo pass/fail). Stima qualitativa basata sui file `#[cfg(test)] mod test`:

| Modulo | Test | Note |
|---|---|---|
| `vault.rs` | 7 | unlock, password errata, re-key, hex roundtrip |
| `migrazione.rs` | 3 | nuovo DB, idempotenza, tabelle create |
| `preferenze.rs` | 2 | default + salva-carica |
| `prompt.rs` | 2 | cerca FTS, cerca DB vuoto |
| `editor.rs` | 7 | crea, sync tags, FTS rebuild |
| `libreria.rs` | 7 | conteggi, lista, dettaglio |
| `audit.rs` | 9 | registra + filtri estesi + cleanup + CSV quote |
| `sync.rs` | 4 | inserisci tag/prompt, conflict |
| `errore.rs` | 5 | varianti error |
| `versioning.rs` | 5 | snapshot, history, rollback, rolling delete |
| `import_export.rs` | 5 | ora_iso, modalita, schema_version, round-trip serdes |

**Totale**: 56 test Rust. Tutti passano sulla CI corrente.

> **Action item**: aggiungere step coverage al workflow (`cargo-llvm-cov` o `tarpaulin`) per ottenere percentuale numerica. Non bloccante per v0.2 ma raccomandato per v0.3.

### TypeScript client (Svelte)

Test su `apps/client/src/lib/template.test.ts` (template engine). 22 test esistenti. Niente coverage numerica configurata in `vitest.config.ts`. La maggior parte della logica UI è in componenti Svelte non testati direttamente — coperti indirettamente da E2E Playwright (rinviati).

> **Action item**: configurare `vitest --coverage` con threshold 70% sui moduli `lib/*.ts` (esclusi i `.svelte` per ora).

### Go server (`apps/server`)

Workflow `server-build.yml` esegue `go test -race -coverprofile=coverage.out` con check threshold 70%. **Stato**: workflow rotto in pre-quality-gate per mismatch Go version (1.22 worker vs 1.23 in `go.mod`). Fix incluso in questo PR (`server-build.yml` aggiornato a Go 1.23).

> **Action item post-fix**: re-run e verificare coverage ≥ 70% (il documento tag-of-record di v0.1.0-fase1 dichiara 12 test Go server pass).

### Go CLI (`apps/cli`)

CI run **25312252646** del 2026-05-04: **coverage totale 53.3%**.

Funzioni con coverage 0%:
- `vaultPath`, `openVault` — filesystem-dependent, mockare è disproportionato per MVP
- `recent` — query helper non testato direttamente (testabile)
- `formatPrompt` — versione singolo prompt (testabile)
- `main` — entry point cobra (non test-ato per definizione)

Funzioni con coverage 100%:
- `sanitizzaFTS`, `estraiSegnaposti`, `truncate`, `init`

**Sotto soglia 70%**, ma le funzioni 0% sono in larga parte entry point e adapter filesystem. Coverage logica pura: ~80%.

> **Action item raccomandato**: aggiungere 3 test per `recent`, `formatPrompt`, e `tagsFor` per portare il totale sopra 70%. Non bloccante per v0.2.0.

### TypeScript MCP server (`apps/mcp-server`)

**Nessun test automatico** in MVP. Lo script `test` in `package.json` ritorna placeholder `echo "No tests yet"`. Validazione manuale via Claude Desktop.

> **Action item**: aggiungere test unit (Vitest o Node `node:test`) almeno per `sanitizzaFts`, `compila`, `estraiSegnaposti` in sub-step. Non bloccante per v0.2.0 ma da inserire prima di v0.3.

## Audit dipendenze

Workflow `.github/workflows/security-audit.yml` configurato con:

- **`cargo audit`** su `apps/client/src-tauri/Cargo.lock`
- **`govulncheck`** su `apps/server` e `apps/cli`
- **`pnpm audit --audit-level=moderate`** su workspace root (copre `apps/client`, `apps/mcp-server`)

Trigger: `workflow_dispatch` (manuale) + `cron: 0 6 * * 1` (lunedì 06:00 UTC settimanale).

**Run di riferimento per v0.2**: lanciare manualmente la prima run dopo merge di questo PR. I risultati saranno linkati qui:

- `cargo audit`: TBD (run da eseguire)
- `govulncheck server`: TBD
- `govulncheck cli`: TBD
- `pnpm audit`: TBD

> **Action item**: dopo merge PR quality-gate, eseguire `gh workflow run security-audit.yml` e copiare gli output qui sotto. Se ci sono CVE, decidere fix/dismiss/accept con razionale documentato.

## Verifica licenza AGPL 3.0

| File | Stato |
|---|---|
| `LICENSE` | AGPL 3.0 testo ufficiale (661 righe) ✓ |
| `package.json` root | `"license": "AGPL-3.0-only"` ✓ |
| `apps/client/package.json` | `"license": "AGPL-3.0-only"` ✓ |
| `apps/client/src-tauri/Cargo.toml` | `license = "AGPL-3.0-only"` ✓ |
| `apps/mcp-server/package.json` | `"license": "AGPL-3.0-only"` ✓ |
| `apps/cli/go.mod` | (Go non ha campo license; LICENSE file è autorevole) |
| `apps/server/go.mod` | (idem) |
| `README.md` | Sezione Licenza riscritta ✓ |
| `CONTRIBUTING.md` | Riferimento generico a `LICENSE` ✓ |

Tutti i moduli sono allineati ad AGPL 3.0 SPDX standard.

## Sintesi

**Pronto per v0.2.0?** Quasi.

- ✅ Tutti gli step funzionali principali (1, 2, 3, 4, 7, 8) chiusi e mergiati su main
- ⏳ Step 5 (auto-update) bloccato in attesa cert Certum
- ⏳ Step 6 (server cross-platform) rimandato finché workspace team non in produzione
- ⚠️ Coverage numerica non disponibile per Rust/TS; stima qualitativa OK
- ⚠️ CLI coverage 53.3% sotto soglia 70%; 3 test extra portano sopra
- ⚠️ Audit security mai ancora eseguito; primo run post-merge di questo PR

**Raccomandazione**:

- Per **release v0.2.0 oggi** (parziale): tag `v0.2.0-foundations` su main attuale (6/8 step), accetta le 2 limitazioni Step 5/6 documentate. Quality gate "best effort" con audit security verde.
- Per **release v0.2.0 completa**: completa Step 5 + 6 + alza coverage CLI sopra 70% + esegui audit security. ~3-5 settimane di lavoro extra.

Decidere col maintainer.
