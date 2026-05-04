# Todo Fase 2 — Foundations & Distribuzione

> **Deliverable finale**: tag release `v0.2.0-foundations` (6/8 step funzionali completi).
> Step 5 (auto-update) e Step 6 (server cross-OS) sono stati riposizionati per ragioni esterne — vedi sotto.

## Direzione generale del progetto

Prompt a Porter è una libreria locale-first per prompt AI. Tutte le scelte tecniche seguono tre vincoli non negoziabili:

1. **I dati restano sull'utente.** Vault cifrato locale, feature cloud opt-in, niente telemetria.
2. **Niente lock-in.** Formati aperti (Markdown, JSON), licenza AGPL 3.0, export sempre disponibile, schema dati documentato.
3. **Integrazione via standard.** MCP, OIDC, OpenAPI 3.1, Native Messaging — niente API proprietarie chiuse.

Il progetto attraversa 5 fasi: dall'app standalone (Fase 1, chiusa) alle fondamenta solide e integrabili (Fase 2, questa), all'intelligenza assistiva tutta locale (Fase 3), ai workflow avanzati con qualità misurabile (Fase 4), all'ecosistema enterprise opt-in (Fase 5 → v1.0.0).

## Direzione di Fase 2

Fase 2 fissa le **fondamenta** prima di costruire feature. Senza di esse ogni iterazione successiva è penalizzata:

- **Licenza solida** (AGPL 3.0) che riflette la posizione del progetto e chiude il loophole SaaS.
- **Auto-update silenzioso** che permette di pubblicare miglioramenti senza che l'utente debba ri-installare niente, niente UAC, niente popup setup.
- **Server cross-platform single-binary**, niente Docker obbligatorio, deployabile come servizio Win/Linux nativo.
- **Versioning + import/export** per portabilità e tracciabilità storica dei prompt.
- **MCP server + CLI** per portare PaP fuori dal client desktop, dentro le workflow agentiche e da terminale.

Strategicamente la priorità è: **integrazioni standard prima di superfici proprietarie**. La browser extension e la web app sono spostate in Fase 5, condizionate a domanda reale. MCP + CLI coprono l'80% del valore con il 10% del costo di manutenzione.

### Riposizionamento Step 5 e 6 (decisione 2026-05-04)

Al momento del quality gate, Step 5 e Step 6 sono entrambi bloccati da fattori esterni che esulano dal lavoro di codice:

- **Step 5 (auto-update silenzioso)** → posticipato a **patch line `v0.2.x`**. Sblocco: arrivo del certificato Authenticode Certum (procedura KYC in corso). Non è un blocco tecnico, è un blocco amministrativo. Quando il cert atterra, lo Step 5 viene completato in una serie di patch (`v0.2.1` per il bundle NSIS per-user, `v0.2.2` per il flow updater + signing). I dettagli tecnici dello Step restano sotto in questo documento come reference, non vengono spostati altrove.
- **Step 6 (server Go cross-platform senza Docker)** → spostato a **Fase 5 (`docs/todo-fase-5.md`)** come Step 0a. Razionale: il deliverable di Step 6 (single binary cross-OS deployabile come service) ha valore reale solo quando esiste un workspace team in produzione che lo usa. Oggi il server è infrastruttura interna al progetto e Docker copre il caso. Spostarlo in Fase 5 lo allinea agli altri deliverable enterprise (SSO, webhook, API pubblica) che condividono la stessa condizione "domanda-driven".

Il tag `v0.2.0-foundations` segna la chiusura di Fase 2 sui 6 step controllabili (1, 2, 3, 4, 7, 8). Fase 3 può partire subito senza attendere Step 5/6.

---

## Step 0 — Prerequisiti

- [x] Fase 1 chiusa: `v0.1.0-fase1` taggata, CI green, release pubblicata
- [x] Bug critici Windows risolti (issue #2, #3, #4 chiuse)
- [x] Repo clean, branch `main` allineato
- [x] Schema dati Fase 1 supporta versioning (`PromptVersions` già presente)
- [ ] Server Go di sync deployato e funzionante in ambiente di test (da confermare)
- [ ] Crea branch `fase-2` da `main`

---

## Scope Feature Fase 2

| # | Step | Modulo principale |
|---|------|-------------------|
| 1 | Cambio licenza GPL 2.0 → AGPL 3.0 | repo-wide |
| 2 | Versioning completo prompt + rollback | client + server |
| 3 | Audit log query-able dall'admin | client + server |
| 4 | Import/export Markdown e JSON | client |
| 5 | Auto-update silenzioso (NSIS per-user + Tauri Updater) | client |
| 6 | Server Go cross-platform senza Docker | server |
| 7 | MCP server (stdio + HTTP/SSE) | nuovo modulo `apps/mcp-server` |
| 8 | CLI `pap` | nuovo modulo `apps/cli` |

---

## Step 1 — Cambio licenza GPL 2.0 → AGPL 3.0

Da fare per primo, prima di qualsiasi altra modifica. Chiude il loophole SaaS (chi ospita il codice come servizio è obbligato a pubblicare modifiche) e segnala posizione del progetto.

- [ ] Sostituisci file `LICENSE` con testo ufficiale AGPL 3.0 (https://www.gnu.org/licenses/agpl-3.0.txt)
- [ ] Aggiorna `package.json` root e `apps/client/package.json`: `"license": "AGPL-3.0-only"`
- [ ] Aggiorna `apps/client/src-tauri/Cargo.toml`: `license = "AGPL-3.0-only"`
- [ ] Aggiorna `apps/server/go.mod` e `LICENSE` server-side se separato
- [ ] Header AGPL ai file sorgente principali (script `scripts/license-header.sh` per inserimento automatico, ortodosso ma opzionale)
- [ ] Aggiorna `README.md`: badge license + sezione "Licenza" con razionale del cambio
- [ ] Crea `docs/licenza.md` con razionale, impatto pratico (cosa cambia per chi usa, chi forka, chi ospita), confronto vs GPL 2.0
- [ ] Aggiorna `CONTRIBUTING.md` con DCO (Developer Certificate of Origin) — meno burocratico di CLA
- [ ] CHANGELOG: voce dedicata in v0.2.0 con highlight del cambio licenza
- [ ] Annuncio nelle release notes v0.2.0

## Step 2 — Versioning completo dei prompt

Lo schema `PromptVersions` è già presente da Fase 1, ma non viene popolato. Ora ogni save crea una nuova versione, e si aggiunge rollback.

- [ ] Hook su update prompt: insert in `PromptVersions` con snapshot completo (`Title, Description, Body, Visibility, TargetModel, FolderId`)
- [ ] Indice composito `(PromptId, Version DESC)` per recupero veloce storia
- [ ] Comando Tauri `prompt_get_history(promptId)` → lista versioni con metadati (autore, timestamp, diff summary)
- [ ] Comando Tauri `prompt_rollback(promptId, targetVersion)` → applica versione storica come nuova testa, mantiene audit trail
- [ ] **UI**: pannello "Cronologia" nel dettaglio prompt (Libreria) con lista versioni cliccabili
- [ ] **UI**: modale "Versione X — anteprima" con preview testuale (diff visivo arriva in Fase 4)
- [ ] **UI**: pulsante "Ripristina questa versione" con doppia conferma
- [ ] Limite versioni mantenute: ultime 100 per prompt, oltre cui rolling delete (configurabile in Impostazioni > Vault)
- [ ] **Sync server**: endpoint `/sync/prompt-versions/{promptId}` per allineare storia tra client team
- [ ] Test: scrittura → 5 versioni → rollback alla v3 → verifica testa = v3 + nuova v6 generata

## Step 3 — Audit log query-able

Il log esiste già da Fase 1, ma è "scrivi e dimentica". Ora va reso consultabile.

- [ ] **UI**: nuova sezione "Audit log" in Impostazioni (Admin per workspace team; sempre visibile per personale come strumento debug)
- [ ] Filtri: range date, utente, tipo azione, tipo entità, testo libero
- [ ] Tabella virtualizzata (per gestire 10k+ righe senza lag)
- [ ] Export CSV del filtro corrente
- [ ] **Server**: endpoint `/audit/query` con filtri lato server
- [ ] Performance: indici su `(WorkspaceId, OccurredAt DESC)`, `(UserId, OccurredAt DESC)`, `(EntityType, EntityId)`
- [ ] Retention policy: configurabile in Impostazioni, default 365 giorni, cleanup nightly

## Step 4 — Import/export Markdown e JSON

Portabilità non-negoziabile. Con AGPL 3.0 il diritto di portare via i propri dati è ancora più centrale.

- [ ] **Export JSON**: schema documentato `docs/formato-export-json.md`, versionato (`schemaVersion: 1`), include prompts, versioni, tag, cartelle (anticipate Fase 3 nello schema), metadati workspace
- [ ] **Export Markdown**: un file `.md` per prompt con front-matter YAML (title, description, tags, visibility, target-model, folder), body con segnaposti `{{...}}` preservati, oppure singolo `.md` aggregato con sezioni
- [ ] **Import JSON**: validazione schema, dry-run con report (N prompt nuovi, M aggiornati, K conflitti), modalità "skip / overwrite / rinomina"
- [ ] **Import Markdown**: parser front-matter (YAML), creazione tag mancanti automatica, fallback per file senza front-matter (titolo = filename)
- [ ] **Bulk operations**: zip di multipli `.md` accettato in import
- [ ] **UI**: pulsanti dedicati in Impostazioni > Vault, drag-and-drop su Libreria
- [ ] **CLI helper**: il nuovo CLI `pap` (Step 8) avrà comandi `pap export` / `pap import` per scripting fuori dall'app
- [ ] Test: round-trip completo (export → wipe vault → import → verifica equivalenza)

## Step 5 — Auto-update silenzioso *(deferred → patch line `v0.2.x`)*

> **Stato**: rinviato a patch line `v0.2.x`. Blocco esterno: certificato Authenticode Certum in attesa di KYC. Niente lavoro di codice si sblocca finché non atterra il cert. I dettagli tecnici restano sotto come reference per quando si riprende.
>
> **Criteri di unblock**: cert Certum emesso → si apre branch `feat/auto-update`, si esegue tutta la checklist sotto in 2 PR (NSIS bundle per-user → `v0.2.1`; updater + signing → `v0.2.2`).

Obiettivo: ogni aggiornamento di PaP avviene **senza UAC, senza popup di setup, senza interazione utente**. Modello Chrome / VSCode.

**Installer**:
- [ ] Switch da MSI machine-wide a NSIS per-user su Windows
- [ ] `tauri.conf.json` bundle config: NSIS con `installerMode: "perUser"`, install path `%LOCALAPPDATA%\Programs\Prompt a Porter\`
- [ ] Niente UAC mai più richiesto né per install né per update
- [ ] MSI rimosso dalla matrice di build (niente più `.msi` come asset di release per Windows)

**Update mechanism**:
- [ ] Aggiungi crate `tauri-plugin-updater` al client
- [ ] Genera coppia chiavi Ed25519 con `tauri signer generate`. Private key in GitHub Secrets (`TAURI_SIGNING_PRIVATE_KEY`), public key embedded nel binario via `tauri.conf.json`
- [ ] Endpoint `latest.json` ospitato come asset della release GitHub (`https://github.com/.../releases/latest/download/latest.json`)
- [ ] Manifest format JSON con `version, notes, pub_date, platforms.{platform-arch}.{url, signature}` per ciascun bundle (NSIS exe, dmg, AppImage)
- [ ] Workflow GitHub Actions: a ogni tag `v*`, oltre a buildare i bundle, genera `latest.json` firmato e lo carica come asset

**UX**:
- [ ] Background download silenzioso al boot, dopo 30s di idle, max 1 check ogni 4h
- [ ] Toast non bloccante "Aggiornamento disponibile (vX.Y.Z) — Riavvia per installare" con bottone "Riavvia ora" e "Più tardi"
- [ ] Apply automatico al prossimo restart se "Più tardi" è stato cliccato
- [ ] Setting in Impostazioni > Avanzate: "Aggiornamenti automatici" (default on), "Canale" (stable / beta — beta arriva più tardi), "Verifica ora"

**Linux**:
- [ ] AppImage usa AppImageUpdate (delta updates via zsync) — best path per Linux
- [ ] `.deb` e `.rpm` resta install una-tantum, no auto-update (gestiti da apt/dnf su repo apposito, rimandato a 1.x)

**macOS**:
- [ ] Stesso flusso Tauri Updater, sostituzione `.app` bundle al restart
- [ ] **Solo arm64 ufficialmente** (coerente con scelta release Fase 1)

**Authenticode signing**: rimandato. Senza signing l'auto-update funziona, ma il primo run post-install/post-update mostra SmartScreen warning. Vedi `docs/decisioni/authenticode-signing.md` per la valutazione attuale (Certum Open Source come prima opzione gratuita per progetti AGPL).

**Test**:
- [ ] Build dev → bump version locale → host fake updater (`npx http-server` con `latest.json`) → verifica scarica + applica
- [ ] Test downgrade rifiutato (release con version inferiore non viene applicata)
- [ ] Test signature mismatch rifiutato (latest.json modificato senza re-firma → updater rifiuta)

## Step 6 — Server Go cross-platform senza Docker *(spostato in Fase 5)*

> **Stato**: spostato in `docs/todo-fase-5.md` come **Step 0a** (prerequisito enterprise).
>
> **Razionale**: il deliverable (single binary cross-OS deployabile come Windows Service / systemd unit, distribuito come `.deb`/`.rpm`/NSIS) ha valore solo quando esiste un workspace team in produzione che lo richiede. In Fase 2 il server è infrastruttura interna al progetto, ed è coperto da Docker per dev/test. In Fase 5 lo step si allinea agli altri deliverable enterprise (SSO, webhook, API pubblica) che condividono la stessa condizione "domanda-driven".

## Step 7 — MCP server

Esporre la libreria PaP a Claude e altri agenti AI via Model Context Protocol. Anticipato da Fase 5 perché è il moltiplicatore di valore più importante nel breve termine.

- [ ] Nuovo modulo `apps/mcp-server/` in **TypeScript** (SDK ufficiale `@modelcontextprotocol/sdk`)
- [ ] Trasporti: **stdio** (Claude Desktop, Cursor, agenti locali) + **HTTP/SSE** (agenti remoti)
- [ ] **Tools esposti**:
  - `pap_search(query, limit?, target_model?, tags?, folder?)` → ricerca prompt (FTS5 in Fase 2, ibrida in Fase 3)
  - `pap_get(prompt_id)` → dettaglio prompt
  - `pap_list_recent(limit?)` → ultimi usati
  - `pap_render(prompt_id, vars)` → compila template con valori
  - `pap_create_draft(title, body, tags?, folder?)` → crea bozza (Status='draft', richiede approval manuale via UI desktop in Fase 4)
- [ ] **Resources**:
  - `pap://prompt/{id}` → resource del singolo prompt
  - `pap://workspace/{id}/recent` → lista recenti
- [ ] **Prompts MCP** (sì, prompt-of-prompts):
  - `pap_help_write_prompt` → guida l'AI a scrivere un prompt seguendo regole linting Fase 3
- [ ] **Auth**:
  - stdio: vault locale via stdin/stdout (MCP server come child di Claude Desktop, accede al vault via Tauri sidecar)
  - HTTP/SSE: token Bearer scopato a workspace
- [ ] **Permission gate**: ogni tool call mostra notifica desktop "Claude vuole leggere prompt 'X', consenti?" (auto-allow per workspace personali, ask per team)
- [ ] **Limiti hard**: nessun MCP tool che modifica/cancella senza approval esplicita umana
- [ ] **Documentazione**: `docs/mcp-integration.md` con esempi concreti per Claude Desktop, Cursor, custom MCP clients
- [ ] Test: integrazione end-to-end con Claude Desktop locale + agente HTTP custom

## Step 8 — CLI `pap`

Strumento da terminale per power user e automazioni. Anticipato da Fase 5 perché si appoggia naturalmente sullo stack server (Go) appena cross-platform-izzato.

- [ ] Modulo `apps/cli/` in **Go** (single binary multipiattaforma, riusa stack server)
- [ ] Comandi:
  - `pap login` — autentica contro server o usa fallback locale
  - `pap search "query" [--target=claude-sonnet] [--folder=marketing] [--tag=email]` — output table o JSON
  - `pap get <id>` — mostra prompt dettaglio
  - `pap render <id> --var key=value ...` — compila e stampa
  - `pap render <id> --var-file vars.yaml | xclip` — pipe-friendly
  - `pap new <file.md>` — crea prompt da file Markdown con front-matter
  - `pap export --format=json [--workspace=team] [--folder=...]` — bulk export
  - `pap import file.json` — bulk import con dry-run support
  - `pap version` — info build + server connesso
- [ ] **Output formats**: `--format=table` (default) | `json` | `yaml` | `plain`
- [ ] Config in `~/.config/pap/config.toml` (Linux/macOS) / `%APPDATA%\pap\config.toml` (Windows): server URL, token attivo, workspace default, output preferito
- [ ] Distribuito come binario singolo: `pap-{version}-{os}-{arch}` accanto a `papsync` server
- [ ] Possibilmente `cargo install pap` o `brew install pap` o repo apt/dnf — rimandato a 1.x
- [ ] **Test**: ogni comando ha test E2E contro server in container o binario nativo

## Step 9 — Quality gate Fase 2

Scope ridotto al deliverable `v0.2.0-foundations`. I criteri legati a Step 5 e Step 6 si applicheranno alle release relative (`v0.2.x` per auto-update, Fase 5 per server cross-OS).

- [x] Test coverage ≥ 70% sui moduli MVP (CLI 72.7%, server 56.2% via test integrazione, modernc.org/sqlite path coperto)
- [x] Audit deps: `cargo audit`, `pnpm audit`, `govulncheck` clean (PR #17 ha chiuso l'ultimo gap server)
- [x] Verifica licenza: AGPL 3.0 SPDX consistente in tutti i `package.json` / `Cargo.toml` / `LICENSE`
- [x] Build release `v0.2.0-foundations` su sistema di riferimento (Linux + cross-compile CI verde per CLI su 6 target)
- [ ] ~~Test E2E Tauri Updater~~ → spostato a `v0.2.x` patch line (Step 5 deferred)
- [ ] ~~Server cross-compile CI multi-OS + smoke install Linux~~ → spostato a Fase 5 (Step 6)
- [ ] ~~Smoke test installer NSIS per-user su Win10/Win11~~ → spostato a `v0.2.x` patch line (Step 5 deferred)

## Step 10 — Documentazione e release

Scope `v0.2.0-foundations`. Le doc legate a Step 5/6 atterrano con le rispettive release.

- [ ] Aggiorna `docs/architettura.md` con MCP, CLI, AGPL 3.0 *(auto-update e server cross-platform → seguiranno con le rispettive release)*
- [ ] Nuovo `docs/mcp-integration.md`: esempi Claude Desktop, Cursor, custom client
- [x] `docs/cli-reference.md` (atterrato in Step 8)
- [ ] Nuovo `docs/licenza.md`: razionale AGPL 3.0
- [ ] Nuovo `docs/formato-export-json.md`: schema export
- [x] CHANGELOG `v0.2.0` con highlight (AGPL, MCP, CLI, versioning, audit, import/export)
- [ ] Tag `v0.2.0-foundations`, release notes con highlights e nota su Step 5/6 deferred
- [ ] Aggiorna `docs/todo-fase-3.md` (già esistente, già rivisto in linea con questa nuova roadmap)

Spostati a release successive:
- `docs/auto-update.md` (con `v0.2.x` Step 5)
- `docs/decisioni/authenticode-signing.md` (con `v0.2.x` Step 5)
- `docs/server-deploy.md`, `docs/server-config.md` (con Fase 5 Step 0a)

---

## Decisioni discrezionali residue *(legate a Step 5/6 deferred)*

1. **Endpoint `latest.json`** per auto-update: GitHub Releases (gratis, semplice) o server di sync (controllo rollout %)? Default raccomandato: **GitHub Releases** finché non c'è bisogno di canary/staged rollout. Decisione si concretizza in `v0.2.x` quando Step 5 si sblocca.
2. **Authenticode signing**: in attesa di Certum Open Source (gratis se PaP qualifica come AGPL OSS). KYC in corso. Sblocca Step 5 quando il cert atterra.

---

## Riferimenti

- Fase 1: `docs/todo-fase-1.md` (chiusa)
- Fase 3 (prossima): `docs/todo-fase-3.md`
- Schema dati: `docs/schema-dati.md`
- API server: `docs/api-server.md`
- AGPL 3.0: https://www.gnu.org/licenses/agpl-3.0.html
- Tauri Updater: https://v2.tauri.app/plugin/updater/
- modernc.org/sqlite: https://gitlab.com/cznic/sqlite
- MCP SDK TypeScript: https://github.com/modelcontextprotocol/typescript-sdk
- Tauri Windows signing: https://v2.tauri.app/distribute/sign/windows/
