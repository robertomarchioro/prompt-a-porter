# Changelog — Prompt a Porter

## v0.2.0-foundations (2026-05-04)

> **Status**: Fase 2 chiusa sui 6 step controllabili (1, 2, 3, 4, 7, 8). Step 5 (auto-update silenzioso) riposizionato a patch line `v0.2.x` — sblocca con cert Authenticode Certum (KYC in corso). Step 6 (server cross-platform senza Docker) spostato a Fase 5 Step 0a — domanda-driven, riprende con workspace team enterprise. Razionale completo in `docs/todo-fase-2.md` e `docs/quality-gate-fase-2.md`.

### Breaking changes

- **Licenza**: GPL 2.0 → **AGPL 3.0** (`LICENSE`, `package.json`, `Cargo.toml`). Chiude il loophole SaaS: chi ospita il codice come servizio è obbligato a pubblicare le modifiche. Fork e redistribution restano liberi sotto AGPL 3.0. Vedi commit `4e365c9`.

### Step 1 — Cambio licenza GPL 2.0 → AGPL 3.0
- `LICENSE` sostituito con testo ufficiale AGPL 3.0
- SPDX `AGPL-3.0-only` in tutti i manifest (`package.json` root + client, `Cargo.toml` Tauri)
- README sezione Licenza riscritta con razionale anti-SaaS-loophole

### Step 2 — Versioning completo prompt + rollback
- **Migration V002**: `PromptVersions` esteso con `Visibility` + `TargetModel`, indice composito `(PromptId, Version DESC)`, backfill v1 per prompt esistenti
- Nuovo modulo Rust `versioning.rs`: `snapshot_versione` (idempotente via INSERT OR IGNORE), `prompt_get_history`, `prompt_rollback` (soft, preserva storia)
- Hook in `prompt_crea`/`prompt_aggiorna`: snapshot automatico ad ogni create/update
- Rolling delete oltre 100 versioni per prompt (configurabile in futuro)
- UI Svelte `CronologiaPrompt.svelte`: modale split pane con lista versioni + preview + ripristino con doppia conferma
- Bottone "Cronologia" nel pannello dettaglio Libreria
- 5 test Rust nuovi
- PR #6, commit `ee0c4e3`

### Step 3 — Audit log query-able
- **Migration V003**: 3 indici performance su `AuditLog` (`(WorkspaceId, OccurredAt)`, `(UserId, OccurredAt)`, `(EntityType, EntityId)`)
- Tauri commands `audit_query` (filtri date+user+action+text+entity, paginazione), `audit_export_csv` (RFC 4180 con quoting), `audit_cleanup_oltre_giorni` (retention manuale)
- UI Impostazioni > Registro attività: filtri estesi (segmented entity, search action/testo, range date), paginazione 50/pag, bottone "Esporta CSV", inline-confirm cleanup
- 4 test Rust nuovi
- PR #7, commit `6af4bd9`

### Step 4 — Import/export JSON con schema v1
- **Schema documentato**: `docs/formato-export-json.md` — versionato (`schemaVersion: 1`), forward/backward compatible, round-trip lossless
- Tauri commands `vault_export_json` (workspace completo), `vault_import_json` con modalità conflitti (`skip`/`overwrite`/`rename`)
- Helper `ora_iso()` in pure Rust (zero `chrono`, algoritmo Howard Hinnant)
- UI Impostazioni > Vault: bottoni Esporta/Importa con segmented modalità, report inline post-import (nuovi/aggiornati/conflitti/errori)
- Audit log: `vault.exported`, `vault.imported`
- 5 test Rust nuovi
- Markdown export/import rinviato a sub-step
- PR #8, commit `1eda4f8`

### Step 7 — MCP server (Model Context Protocol)
- **Nuovo modulo `apps/mcp-server/`** in TypeScript con `@modelcontextprotocol/sdk` + `better-sqlite3`
- Trasporto stdio (Claude Desktop, Cursor)
- 4 tool read-only: `pap_search`, `pap_get`, `pap_list_recent`, `pap_render`
- Vault discovery via env `PAP_VAULT_PATH` o default per piattaforma
- Solo vault non cifrati in MVP (SQLCipher in arrivo)
- Documentazione completa `docs/mcp-integration.md` (Claude Desktop, Cursor, troubleshooting)
- Workflow CI dedicato `mcp-server-build.yml` (lint + build TS) con `workflow_dispatch` manuale
- HTTP/SSE transport e `pap_create_draft` rinviati a sub-step
- PR #9, commit `cfbe546`

### Step 8 — CLI `pap`
- **Nuovo modulo `apps/cli/`** in Go con `cobra` + `modernc.org/sqlite` (pure-Go, zero CGO) + `yaml.v3`
- 5 comandi: `pap version|search|get|recent|render` + `completion` automatico (bash/zsh/fish/powershell)
- Output formats: `table` (default, tabwriter), `json`, `yaml`, `plain` (id<TAB>title)
- Vault read-only strict (DSN `?mode=ro`)
- CI cross-compile matrix 6 build (linux/darwin/windows × amd64/arm64) con `CGO_ENABLED=0`, ldflags `-s -w`, upload-artifact
- Documentazione `docs/cli-reference.md` con esempi tab-completion per ogni shell
- 10 test unit Go
- Comandi `login`/`new`/`import`/`export` rinviati (richiedono server API o IPC client desktop)
- PR #11, commit `12a1214`

### Infrastructure & repo

- File standard di presentazione GitHub: `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1 IT), `SECURITY.md` (disclosure policy + tempi risposta), `.github/ISSUE_TEMPLATE/{config,bug_report,feature_request}.yml`, `.github/PULL_REQUEST_TEMPLATE.md`
- `CONTRIBUTING.md` esteso con DCO sign-off + Conventional Commits
- Filter-repo per unificare autori commit (rimossi commit di `robertomarchioro-bluenergy`)
- Workflow `bootstrap.yml` per generare lockfile + icone in CI senza Node locale
- Workflow `release.yml` per build multi-OS Tauri (NSIS perUser su Windows, dmg arm64 su macOS, deb/AppImage/rpm su Linux) con `tauri-action` + draft Release
- Patch CI workflow: `paths-ignore` per `*.md`/`LICENSE`/`CHANGELOG.md` dentro `apps/*` per evitare build inutili; `workflow_dispatch` su tutti i workflow per re-run manuale; `pnpm-lock.yaml` aggiunto ai trigger paths
- Nuovo workflow `security-audit.yml`: `cargo audit` + `govulncheck` (server + CLI) + `pnpm audit`, schedulato settimanalmente + dispatch manuale

### Bug fix significativi (post-v0.1.0-fase1)

- **#4 critical**: preferenze Windows non persistevano causando re-onboarding e errore "vault già aperto". Fix: `App.svelte` usa `vault_esiste()` come fallback robusto del check `onboarding_completato`.
- **#3 high**: tray menu Windows non appariva. Fix: `lib.rs` configura `show_menu_on_left_click(false)` + handler `on_tray_icon_event` per click sinistro → mostra libreria; click destro → menù contestuale.
- **#2 low**: onboarding mancava toggle tema light/dark. Fix: segmented control nel wizard, applicato live via `data-theme`.

### Quality gate (PR #17, #18, #19)

- **PR #17** — Server `go.sum` rigenerato (hash inconsistenti con `sum.golang.org` per tutti i moduli, probabile generazione originale con `GOSUMDB=off`); bump server Go 1.23 → 1.25 + `golang-jwt/jwt/v5` aggiornato + `chi/v5 v5.2.1 → v5.2.2` (fix `GO-2025-3770` open-redirect). Risultato `govulncheck`: 22 vuln (1.23.4) → 0 (1.25.9).
- **PR #18** — Coverage CLI `53.3% → 72.7%` con 3 test mirati su `recent` (70.6%), `formatPrompt` (93.5%), `tagsFor` (81.8%).
- **PR #19** — Riposizionamento Step 5 (→ patch line `v0.2.x`) e Step 6 (→ Fase 5 Step 0a). Scope `v0.2.0-foundations` formalizzato.

### Audit security finale

| Audit | Stato |
|---|---|
| `cargo audit` (Tauri client) | ✅ clean |
| `pnpm audit` (workspace) | ✅ clean |
| `govulncheck` CLI (Go 1.24) | ✅ clean |
| `govulncheck` server (Go 1.25) | ✅ clean — 0 vulnerabilità |
| `licensee` AGPL 3.0 | ✅ consistente in tutti i manifest |

### Roadmap successiva

- **Patch line `v0.2.x`** — Auto-update silenzioso (Step 5): NSIS per-user + Tauri Updater + firma Authenticode. Sblocco: cert Certum OSS.
- **Fase 5 Step 0a** — Server Go cross-platform senza Docker (`modernc.org/sqlite`, Win Service + systemd). Domanda-driven.
- **Fase 3 (`v0.3.0`)** — Intelligenza & authoring: ricerca semantica via embeddings ONNX locali, prompt componibili, linting proattivo. Vedi `docs/todo-fase-3.md`.

### Statistics

- 14 PR mergiate (#6 – #19)
- ~5500 righe di codice nuovo (Rust + TypeScript + Go + SQL)
- Coverage CLI 72.7%, server 56.2% (cross-package via test integrazione)
- 0 vulnerabilità note (audit settimanale via `security-audit.yml`)

---

## v0.1.0-fase1 (2026-05-03)

Prima release MVP. Tutte le funzionalità core implementate.

### Step 0 — Bootstrap repo
- Inizializzazione repo con LICENSE GPL 2.0, README, .gitignore
- Setup pnpm workspace monorepo (`apps/client`, `apps/server`, `packages/`)
- GitHub Actions baseline (lint check client + server)

### Step 1 — Client Tauri + Svelte
- Scaffolding Tauri 2 + Svelte 5 + TypeScript
- Configurazione multi-window (libreria 1200×800 + palette 640×480 frameless)
- Struttura directory: components, superfici, stores, i18n
- File i18n: it.json + en.json con stringhe per tutte le superfici
- Icone SVG sorgenti (Lucide `braces`)

### Step 2 — Vault SQLite + SQLCipher
- Integrazione `rusqlite` con `bundled-sqlcipher` (AES-256)
- Schema V001: 8 tabelle + FTS5 + 8 indici
- Migration system embedded via `include_str!()`
- Comandi: vault_crea, vault_unlock, vault_lock, vault_cambia_password
- Derivazione chiave Argon2id (m=32MiB, t=3, p=4)
- 7 test unitari

### Step 3 — Componenti UI base
- 16 primitive Svelte 5: Button, Input, Textarea, Select, Field, Switch, Kbd, Tag, Badge, Placeholder, NavItem, ListItem, EmptyState, Toast, Skeleton, Tooltip
- Classi utility globali in app.css
- Pagina demo `?demo` con switch tema/tono
- Accessibilità: aria attributes, focus-visible, keyboard nav

### Step 4 — Onboarding
- Wizard 3 step (Profilo → Password vault → Hotkey)
- Strength meter password (4 livelli, calcolo entropia)
- Supporto vault non cifrato ("Salta cifratura")
- Navigazione tastiera (Enter=avanti, Esc=reset)

### Step 5 — Tray icon + global hotkey
- Tray con menu contestuale (5 voci)
- Hotkey globale registrabile a runtime
- Toggle palette: show+center+focus / hide
- Caricamento hotkey da preferenze all'avvio

### Step 6 — Command Palette
- Window frameless dedicata, fuzzy search FTS5
- Navigazione tastiera (↑↓ naviga, Enter seleziona, Escape chiudi)
- Espansione inline form segnaposti
- Ctrl+Enter = compila e copia negli appunti

### Step 7 — Libreria
- Layout 3 pannelli CSS Grid (sidebar + lista + dettaglio)
- Sidebar con workspace switcher, viste, tag dinamici
- Lista con search debounced, sort (recente/popolare/A-Z)
- Status bar con sync dot, versione, hotkey

### Step 8 — Editor prompt
- Modale 2 colonne con CodeMirror 6
- Highlight {{segnaposti}} con ViewPlugin + Decoration
- Tag picker con autocomplete
- Autosave con debounce (2s)

### Step 9 — Compilatore
- Vista 2 colonne (form + preview)
- Form auto-generato dai segnaposti
- Progress bar compilazione
- Toggle output Testo / Markdown / JSON
- Copy to clipboard + toast

### Step 10 — Impostazioni
- Layout sidebar + content con 7 sezioni
- Hotkey configurabile con HotkeyInput
- Tema scuro/chiaro + tono zinc/slate/stone
- Gestione vault: percorso, cifratura, cambio password, elimina
- Toggle lingua it/en

### Step 11 — Server Go
- chi router con middleware (CORS, logger, JWT, recoverer)
- Schema SQLite server + SyncChangelog
- Auth: Argon2id + JWT HS256 (login + refresh)
- Sync: pull delta + push con last-write-wins
- WebSocket broadcast per workspace
- Dockerfile multistage (golang:1.23-alpine → alpine:3.20)
- 12 test di integrazione

### Step 12 — Auth e Sync client
- 3 schermate auth: Login, Reset password, Recupera workspace
- Store sync singleton (polling + WebSocket reconnect)
- Conflict UI con scelta locale/server per entità
- Preferenze estese con campi sync (serde default backward compat)
- Sezione Sync in Impostazioni con stato live

### Step 13 — Audit log
- Modulo `audit.rs` con `registra()` fire-and-forget
- Hook su editor, libreria, vault, sync (9 azioni tracciate)
- Vista "Registro attività" in Impostazioni con filtro per tipo
- Comando `audit_lista` con limite e filtro tipo entità

### Step 14 — Quality gate
- 37 test Rust su 8 moduli
- 22 test TypeScript per template.ts (vitest)
- CI aggiornata: job rust-test + vitest + coverage 70% server

### Step 15 — Documentazione
- Architettura completa con diagrammi e tabelle moduli
- Setup sviluppo con comandi e struttura directory
- Deploy produzione con Docker e variabili d'ambiente
- Prompt di ricostruzione con lezioni apprese
- Changelog completo
- API server aggiornata
