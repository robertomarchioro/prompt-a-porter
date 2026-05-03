# Todo Fase 2 — Distribuzione e Collaborazione

> **Obiettivo**: portare Prompt a Porter da app desktop standalone a piattaforma collaborativa multi-superficie. L'utente ha già il client desktop funzionante (Fase 1); ora aggiungiamo le superfici per consumare i prompt **dove servono** (browser, web) e gli strumenti per **lavorare insieme** (commenti, versioning, import/export).
>
> **Deliverable finale**: tag release `v0.2.0-fase2`.

---

## Prerequisiti (Step 0)

- [ ] Fase 1 chiusa: `v0.1.0-fase1` taggata, CI green su tutti gli OS, smoke test passato
- [ ] Repo clean, branch `main` allineato
- [ ] Schema dati Fase 1 conferma supporto Versioning (`PromptVersions` tabella già presente)
- [ ] Server Go di sync deployato e funzionante in ambiente di test
- [ ] Nessun bug bloccante aperto in `docs/todo-fase-1.md` o issue GitHub critici
- [ ] Crea branch `fase-2` da `main`, lavoro su feature branch da lì

---

## Scope Feature Fase 2

| # | Feature | Modulo principale |
|---|---------|-------------------|
| 1 | Versioning completo prompt + rollback | client + server |
| 2 | Audit log query-able dall'admin | client + server |
| 3 | Import/export Markdown e JSON | client |
| 4 | Commenti thread + reazioni 👍👎 | client + server |
| 5 | Web app (subset funzionalità client) | nuovo modulo `apps/web` |
| 6 | Browser extension Manifest V3 | nuovo modulo `apps/extension` |
| 7 | Native Messaging Host extension ↔ desktop | bridge in `src-tauri` |

---

## Step 1 — Versioning completo dei prompt

Lo schema `PromptVersions` è già presente da Fase 1, ma non veniva popolato. Ora ogni save crea una nuova versione, e va aggiunto rollback.

- [ ] Hook su update prompt: insert in `PromptVersions` con snapshot completo (Title, Description, Body, Visibility, TargetModel)
- [ ] Indice composito `(PromptId, Version DESC)` per recupero veloce storia
- [ ] Comando Tauri `prompt_get_history(promptId)` → lista versioni con metadati (autore, timestamp, diff summary)
- [ ] Comando Tauri `prompt_rollback(promptId, targetVersion)` → applica versione storica come nuova testa (mantiene audit trail)
- [ ] **UI**: pannello "Cronologia" nel dettaglio prompt (Libreria) con lista versioni cliccabili
- [ ] **UI**: modale "Versione X — anteprima" con confronto fianco a fianco (preview only, diff visivo arriva in Fase 4)
- [ ] **UI**: pulsante "Ripristina questa versione" con doppia conferma
- [ ] Limite versioni mantenute: ultime 100 per prompt, oltre cui rolling delete (configurabile in Impostazioni > Vault)
- [ ] **Sync server**: endpoint `/sync/prompt-versions/{promptId}` per allineare storia tra client team
- [ ] Test: scrittura → 5 versioni → rollback alla v3 → verifica testa = v3 + nuova v6 generata

## Step 2 — Audit log query-able

Il log esiste già da Fase 1, ma è "scrivi e dimentica". Ora va reso consultabile.

- [ ] **UI**: nuova sezione "Audit log" in Impostazioni (visibile solo a Admin)
- [ ] Filtri: range date, utente, tipo azione, tipo entità, testo libero
- [ ] Tabella virtualizzata (per gestire 10k+ righe senza lag)
- [ ] Export CSV del filtro corrente
- [ ] **Server**: endpoint `/audit/query` con filtri lato server (per workspace team)
- [ ] Performance: aggiungere indici su `(WorkspaceId, OccurredAt DESC)`, `(UserId, OccurredAt DESC)`, `(EntityType, EntityId)`
- [ ] Retention policy: configurabile in Impostazioni, default 365 giorni, cleanup nightly
- [ ] **Privacy**: in modalità personale l'audit log resta locale e ha senso più tecnico (debug); per team è strumento di compliance

## Step 3 — Import/export Markdown e JSON

La portabilità è non-negoziabile (stai usando GPL 2.0, niente lock-in).

- [ ] **Export JSON**: schema documentato `docs/formato-export-json.md`, versionato (`schemaVersion: 1`), include prompts, versioni, tag, metadati workspace
- [ ] **Export Markdown**: un file `.md` per prompt con front-matter YAML (title, description, tags, visibility, target-model), body con segnaposti `{{...}}` preservati, oppure un singolo `.md` aggregato
- [ ] **Import JSON**: validazione schema, dry-run con report (N prompt nuovi, M aggiornati, K conflitti), modalità "skip / overwrite / rinomina"
- [ ] **Import Markdown**: parser front-matter (YAML), creazione tag mancanti automatica, fallback per file senza front-matter (titolo = filename)
- [ ] **Bulk operations**: zip di multipli `.md` accettato in import
- [ ] **UI**: pulsanti dedicati in Impostazioni > Vault, drag-and-drop su Libreria
- [ ] **CLI helper opzionale**: piccolo script Node/Deno in `scripts/` per import batch fuori dall'app (utile per migrazioni da altri tool)
- [ ] Test: round-trip completo (export → wipe vault → import → verifica equivalenza)

## Step 4 — Commenti sui prompt (server)

Estensione del server di sync. Sviluppare prima il backend, poi UI.

- [ ] Nuova tabella server `Comments`: `Id, PromptId, AuthorUserId, ParentCommentId (NULL per root), Body, Reactions (JSON), CreatedAt, UpdatedAt, DeletedAt`
- [ ] Endpoint `POST /prompts/{id}/comments` (crea), `GET /prompts/{id}/comments` (lista thread), `PATCH /comments/{id}` (edit), `DELETE /comments/{id}` (soft delete), `POST /comments/{id}/reactions` (toggle reaction)
- [ ] WebSocket: nuovo channel `comments:{promptId}` per push real-time
- [ ] **Permessi**: tutti i ruoli (Admin/Editor/User) possono commentare e reagire; solo l'autore o Admin possono editare/cancellare
- [ ] **Mention**: parser `@nomeutente` nel body, notifica via WebSocket all'utente menzionato
- [ ] Rate limit: max 10 commenti/min per utente per evitare flood
- [ ] **Solo per workspace team** (i commenti su prompt privati non hanno senso)

## Step 5 — Commenti sui prompt (client UI)

- [ ] Nuovo componente `CommentThread.svelte` nel pannello dettaglio Libreria
- [ ] Editor commento con supporto Markdown base (bold, italic, code inline, code block, link) — riusa CodeMirror 6 in modalità minimal
- [ ] Mention picker con autocomplete utenti workspace (`@`)
- [ ] Reazioni rapide (👍 funziona, 👎 no, 💡 idea, ❓ domanda) — emoji minimalista, non eccessiva
- [ ] Indicatore "N commenti" nella card della lista
- [ ] Notifica in-app per mention (toast) + dot rosso su tray icon
- [ ] **Sync**: WebSocket subscription a `comments:{promptId}` quando il prompt è aperto
- [ ] **i18n**: tutte le stringhe in `it.json` e `en.json`
- [ ] Test: round-trip commento, edit, delete, reazione, mention con notifica

## Step 6 — Web app (Fase 2)

Nuovo modulo `apps/web/`. Subset di funzionalità del client desktop, **read-heavy** (consultare e usare prompt). Editing pesante resta su desktop.

**Stack**:
- **SvelteKit** (server-side rendering opzionale, ma in pratica useremo solo SPA mode)
- **Tailwind v4** (questa volta sì, perché siamo in SPA pubblica e i tokens vivono in CSS variables comunque)
- Oppure: **stesse CSS variables del client** + componenti Svelte 5 condivisi via package `packages/ui-shared`
- Hosting: SPA buildata + servita dallo stesso binario Go (embed con `embed.FS`)

**Funzionalità incluse**:
- [ ] Login (riusa stessi endpoint `/auth/login`)
- [ ] Libreria read-only con tutte le viste (sidebar + lista + dettaglio)
- [ ] Renderer/Compilatore funzionante (form + copy)
- [ ] Commenti (read + write)
- [ ] Ricerca full-text via API server (`/search?q=`)
- [ ] **Differenza chiave**: niente vault locale, niente cifratura at-rest sul client web — la web app è **read-only sui prompt cifrati** che il server custodisce in chiaro per i workspace team

**Funzionalità escluse dalla web app**:
- ❌ Creazione/modifica prompt (rimando a desktop con CTA "Apri in Prompt a Porter")
- ❌ Impostazioni vault, hotkey, tema avanzato
- ❌ Tray icon, command palette globale (è un'app web, non un'app desktop)

- [ ] Embedding statico nel server: `go:embed dist/web/*` → servito da `/`
- [ ] CSP stretta, nessuna risorsa esterna
- [ ] Build CI: `pnpm -F @pap/web build` → output in `apps/server/internal/web/dist/`
- [ ] Test E2E con Playwright sui flussi principali

## Step 7 — Browser extension (Manifest V3)

Nuovo modulo `apps/extension/`. Compatibilità: **Chrome 120+, Edge 120+, Firefox 121+**.

**Stack**:
- Manifest V3 puro (TS compilato in JS)
- UI popup in **Svelte 5** compilato standalone (bundle separato dal client desktop)
- Stessi CSS variables/tokens
- Lucide Icons
- Niente framework UI esterni

**Architettura**:
- **Service worker**: gestisce messaging, storage, fetch verso desktop client (Native Messaging) o server (fallback)
- **Popup**: command palette compatta (~360×480), variante mobile-friendly del design Fase 1
- **Content script**: iniettato sui domini AI (claude.ai, chatgpt.com, gemini.google.com, copilot.microsoft.com), rileva textarea attiva e abilita injection
- **Storage**: nessun dato cifrato sensibile in `chrome.storage` — solo cache leggera dei prompt più usati per uso offline (con TTL)

- [ ] Scaffolding Manifest V3 con build Vite multi-target (Chrome/Firefox build separate per piccole differenze)
- [ ] UI popup minimal: search → lista risultati → seleziona prompt → form segnaposti → "Inietta nella chat" o "Copia"
- [ ] Permission minime: `activeTab`, `nativeMessaging`, host permissions per i domini AI
- [ ] Logo + icone (16/32/48/128) — derivati dal glifo tray
- [ ] Settings UI minimal: server URL (per fallback), preferenze visualizzazione
- [ ] **Privacy**: nessuna telemetria, nessun analytics, nessun fetch di terze parti
- [ ] Pubblicazione store **rimandata a fine Fase 2** (prima validazione interna)

## Step 8 — Native Messaging Host

Bridge tra extension browser e client desktop. Quando il client desktop è in running, l'extension parla con lui (zero latenza, accesso al vault locale cifrato). Se non c'è, fallback su API server.

- [ ] **Lato client desktop (Rust)**: nuovo binario companion `pap-native-host` (oppure subcommand di `pap` con flag `--native-host`)
- [ ] Protocollo Native Messaging standard: stdin/stdout con messaggi JSON length-prefixed
- [ ] Manifest registrato sul SO: file JSON in path specifico per browser e OS:
  - Chrome/Edge Linux: `~/.config/google-chrome/NativeMessagingHosts/it.promptaporter.host.json`
  - Chrome/Edge macOS: `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/`
  - Chrome/Edge Windows: registry `HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\`
  - Firefox: path simili
- [ ] Installer/uninstaller del manifest gestito dall'app desktop al primo avvio (con consenso utente)
- [ ] **Sicurezza**: solo extension con `extension_id` allowlistato (incluso nel manifest) può connettersi
- [ ] Comandi supportati: `list_prompts`, `search`, `render`, `get_recent`, `health`
- [ ] Fallback HTTP locale (es. `localhost:11811` con token random rigenerato a ogni avvio) come piano B se Native Messaging fallisce
- [ ] Test integrazione: extension → host → vault → response, su tutti e tre gli OS

## Step 9 — Injection nei siti AI

Content script che integra Prompt a Porter dentro le interfacce AI.

- [ ] **Site adapters** specifici per ogni dominio:
  - `claude.ai`: detect textarea principale, inject button "📝 PaP" accanto
  - `chatgpt.com`: idem
  - `gemini.google.com`: idem
  - `copilot.microsoft.com`: idem
- [ ] Adapter pattern: ogni adapter implementa `detectInputElement()`, `insertText(text)`, `attachUI()`
- [ ] Fallback generico: hotkey configurable (default `Ctrl+Shift+/`) apre il popup ovunque, copia output
- [ ] Floating action button discreto, posizionato dove l'utente l'aspetta (vicino al composer)
- [ ] Animazione di iniezione minimal (typing effect opzionale, off di default)
- [ ] Resilienza ai cambi DOM: ogni adapter ha selectors fallback multipli, e si auto-disabilita pulito se rileva struttura cambiata
- [ ] **Test manuale obbligatorio**: provare su tutti e 4 i siti, verificare zero break del sito host

## Step 10 — Quality gate Fase 2

- [ ] Test coverage ≥ 70% su moduli core (versioning, comments, import/export, native-messaging)
- [ ] Test E2E Playwright: flusso completo desktop ↔ web ↔ extension
- [ ] Manual smoke test su tutte le 8 superfici Fase 1 + nuove di Fase 2
- [ ] Audit di sicurezza: dipendenze (`pnpm audit`, `cargo audit`, `govulncheck`), CSP, permessi extension
- [ ] Performance: ricerca full-text su 5000 prompt < 50ms, sync delta su 1000 modifiche < 5s
- [ ] CI green su Win/macOS/Linux per client; Linux/Docker per server; build extension Chrome+Firefox

## Step 11 — Documentazione e release

- [ ] Aggiorna `docs/architettura.md` con i nuovi moduli (web, extension, native host)
- [ ] Nuovo `docs/extension-distribuzione.md`: come installare extension dev mode, come pubblicare in store (rimandato)
- [ ] Nuovo `docs/native-messaging.md`: protocollo, troubleshooting per OS
- [ ] Nuovo `docs/web-app-deploy.md`: come deployare la web app (è embedded nel server, ma documentare cache control, ecc.)
- [ ] Aggiorna `docs/api-server.md` con endpoint commenti, versioni, audit
- [ ] Aggiorna `docs/formato-export-json.md`
- [ ] Changelog v0.2.0
- [ ] Crea `docs/todo-fase-3.md` (parti dal file di Fase 3 di questo documento)
- [ ] Tag `v0.2.0-fase2`, release notes con highlights
- [ ] Merge `fase-2` → `main` con squash o merge commit (a scelta)

---

## Decisioni discrezionali da prendere prima di iniziare

1. **Web app SvelteKit o pure SPA Svelte 5?** SvelteKit dà SSR/route più strutturate, pure SPA è più snella. Preferenza mia: **pure SPA** se non servono SSR e SEO.
2. **Tailwind v4 o solo CSS variables?** Tailwind aiuta sulla velocità ma aggiunge una dipendenza. Coerente con l'app desktop sarebbe **solo CSS variables**.
3. **Embed extension popup nel monorepo o repo separato?** Monorepo è più ordinato per coerenza versioni; repo separato facilita pubblicazione store. Preferenza: **monorepo per ora**, eventuale split successivo.
4. **Modello-target del prompt arriva qui o in Fase 3?** È utile averlo già dalla web/extension per filtrare. Valuta se anticiparlo.

---

## Riferimenti

- Fase 1: `docs/todo-fase-1.md` (chiusa)
- Fase 3 (prossima): `docs/todo-fase-3.md`
- Schema dati base: `docs/schema-dati.md`
- API server: `docs/api-server.md`
