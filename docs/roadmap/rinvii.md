# Roadmap dei rinvii

> Censimento unificato di **tutto ciò che è stato deliberatamente rinviato** durante lo sviluppo. Singola fonte di verità: nuovi rinvii vengono aggiunti qui ad ogni PR che li introduce, e gli item vengono spostati nell'archivio storico quando atterrano.
>
> **Aggiornato al**: 2026-05-12. Adozione strategia SKU v1.0 (Personale) / v2.0 (Enterprise) — gli item sono ora classificati per stream di destinazione.

## Convenzioni

**Marker di motivazione**:

- 🔒 **Bloccato da fattore esterno** (certificato, KYC, hardware, decisione di prodotto) → resta dove si trova finché il blocco esterno non cade
- 🔧 **Bloccato da fattore tecnico** (libreria instabile, dipendenza non pronta) → si sblocca quando il fattore tecnico cade
- 📋 **Sub-step di feature già atterrata** → da fare quando si presenta finestra di lavoro, o quando il padre arriva
- 🎨 **Polish / cosmetico** → non bloccante per nessuno
- ⛔ **Scelto deliberatamente di NON implementare** (rapporto costo/beneficio sfavorevole). Resta qui solo come traccia decisionale

**Marker di destinazione (stream)**:

- → `v1.0` = entra nel piano [`v1.0-personale.md`](./v1.0-personale.md), tipicamente come sub-deliverable di un M-block
- → `v2.0` = entra nel piano [`v2.0-enterprise.md`](./v2.0-enterprise.md) / [`fase-5-enterprise.md`](./fase-5-enterprise.md)
- → `v0.2.x` = patch line storica Fase 2 (auto-update)
- → `v1.x` = patch line post-v1.0 (refinamento naturale)
- → archivio = atterrato, vedi §"Cronologia" in fondo

---

## 1. Patch line `v0.2.x` — sblocco esterno

Item che vivono nel branch della release v0.2 ma aspettavano il cert Authenticode. **Confluiscono in v1.0 M1** ([`v1.0-personale.md`](./v1.0-personale.md#m1-auto-update--authenticode-signing-)).

**Aggiornamento 2026-05-15**: cert Certum **SimplySign Cloud** (variante API-based, CI-friendly) **arrivato**. Tutti gli item Certum-bloccati passano da 🔒 a 🚧 (in lavorazione). Resta 🔒 solo macOS notarization (cert Apple Developer separato, non richiesto).

| Item | Stato | Sblocca con |
|---|---|---|
| **Step 5 — Auto-update silenzioso completo** (NSIS per-user, Tauri Updater, `latest.json` firmato, downgrade refuse, signature mismatch refuse) | 🚧 → v1.0 M1 | ✅ Cert Certum SimplySign Cloud arrivato 2026-05-15 |
| Firma Authenticode su tutti gli `.exe` portable di release | 🚧 → v1.0 M1 | ✅ Cert Certum SimplySign Cloud arrivato 2026-05-15 |
| Test reale **macOS notarization** del bundle Tauri con `libonnxruntime.dylib` inclusa | 🔒 → v1.0 M1 | Apple Developer certificate (separato, non richiesto) |
| Test reale **Authenticode signing** del bundle Windows con `onnxruntime.dll` inclusa | 🚧 → v1.0 M1 | ✅ Cert Certum arrivato |
| `docs/utente/auto-update.md` (meccanica + troubleshooting + recovery update corrotto) | 📋 → v1.0 M1.7 | Sub-deliverable di M1 |
| `docs/architettura/decisioni/authenticode-signing.md` (provider considerati, criteri attivazione) | 📋 → v1.0 M1.1 | Primo sub-PR di M1 |
| Test E2E Tauri Updater (build dev → fake update server → download + apply, downgrade refuse, signature mismatch refuse) | 🚧 → v1.0 M1.6 | Sub-deliverable di M1 |
| Smoke test installer NSIS per-user su Win10 e Win11 (verifica nessun UAC) | 🚧 → v1.0 M1.6 | Sub-deliverable di M1 |

**Quando atterrano**: lavoro avviabile in qualunque momento, sequenza sub-PR M1.1-M1.7 documentata in [`v1.0-personale.md`](./v1.0-personale.md#m1-auto-update--authenticode-signing-) §"Sequenza sub-PR raccomandata". Parallelo a M2/M3 senza conflitti.

---

## 2. Stream Personale — v1.0

Sub-step di feature già atterrate, recuperati come must-have v1.0. Vedi [`v1.0-personale.md`](./v1.0-personale.md) per il piano completo.

### Da Fase 2 Step 4 — Import/export
- ✅ M6 PR-1..4 (#216-219, 2026-05-19): **Markdown import/export** — parser front-matter, walker directory ricorsivo, export zip vault, UI Impostazioni → Dati, doc utente Obsidian/Foam compatibility (`docs/utente/markdown-import-export.md`).

### Da Fase 3 Step 5 — Linting
- 📋 → v1.0 nice-to-have: **regole linting nuove** per validare sintassi import scopati (`with k=v`, `version=N`)

### Da Fase 3 Step 8 — Prompt componibili
- ✅ M4 PR-1 (#209, 2026-05-16): **Sintassi `{{import "x" with k=v}}`** per variabili scopate per import
- ✅ M4 PR-2 (#210, 2026-05-16): **Pinning a versione storica** `{{import "x" version=N}}` — letto da `PromptVersions`, combinabile con `with`
- ✅ M4 PR-3 (#211, 2026-05-16): **Intellisense autocomplete `{{import "...`** in EditorTab (richiesta utente, bonus M4)
- 📋 → **v1.0 M4.x backlog** (post-M4): **Lint rules IMP004/005** per `with`/`version` (variabile dichiarata non usata, version inesistente). Richiede refactor lint per accesso DB.
- 📋 → **v1.0 M4.x backlog** (post-M4): **Doc utente `docs/utente/prompt-componibili.md`** aggiornato con esempi `with`/`version`/intellisense + screenshot.
- ✅ M5 PR-1+PR-2 (#213-214, 2026-05-16): **Editor doppia vista Sorgente/Compilato integrata** — AnteprimaTab ora split-view con form valori inline + espansione import live via `prompt_compila_inline`. CompilaModal mantenuto come fallback palette.

### Da Fase 4 Step 1 — Varianti
- ✅ M3 PR-1 (#203, 2026-05-16): **UI Editor "Crea variante"** (modale in RightRail)
- ✅ M3 PR-4 (#206, 2026-05-16): **Renderer dropdown variante** con switch al volo mantenendo i valori del form
- ✅ M3 PR-5 (#207, 2026-05-16): **Promuovi variante a principale** (swap main ↔ variant con preservazione storia)
- 📋 → **v1.0 M3.x backlog** (post-M3): **Migration automatica backreference su promozione variante**.
  Quando una variante viene promossa a principale (M3 PR-5, mergiato 2026-05-16), gli `{{import "id-vecchia-main"}}` esistenti nei prompt terzi continuano a puntare alla vecchia main (ora variante). Una migration automatica che li riscrive richiede una decisione semantica: silenziosa (cambio invisibile, può sorprendere) o con notifica utente (lista import affetti + scelta esplicita). Rinviato per evitare decisioni di prodotto inadeguate sotto pressione M3.

### Da Fase 4 Step 2 — Rating
- ✅ M3 PR-2 (#204, 2026-05-16): **Modale "Aggiungi nota" su voto negativo** in CompilaModal
- ✅ pre-M3 (F11 ListPane redesign): **Sort by quality "Migliori prompt"** in Libreria — già implementato (`libreria.rs:172-180` + `ListPane.svelte:220`)

### Da Fase 4 Step 8 — Golden + regression
- ✅ M3 PR-3 (#205, 2026-05-16): **"Esegui tutti i golden" batch** con riassunto pass/fail in GoldenTab
- 📋 → v1.x (nice-to-have): **CLI integration** `pap test <promptId> [--golden=...]` per CI/CD — `apps/cli` esistente, manca subcommand
- 📋 → v1.0 (audit): **Security review formale** chiavi API provider AI (security-review agent)

### Coverage gap → v1.0 M7
- 🔧 → v1.0 M7: **Coverage TS client** `vitest --coverage` su `lib/*.ts` — soglia 70%
- 🔧 → v1.0 M7: **Coverage client Rust 74.14% → 80%** (oggi gate CI 70%)
- 🔧 → v1.0 M7: **Test unit MCP server** (almeno `sanitizzaFts`, `compila`, `estraiSegnaposti`, parsing import, linting hook)

### Polish v1.0 M2
- 🎨 → v1.0 M2: **~37 warning a11y di `svelte-check`** (autofocus, label senza control, role/aria mismatch). Era target v0.6.0, slittato. Gate CI `--fail-on-warnings` su a11y.

### Docs v1.0 M8
- 📋 → v1.0 M8: Documentazione utente completa — getting-started, casi d'uso, troubleshooting, glossario sintassi, scorciatoie

---

## 3. Stream Enterprise — v2.0

Vedi [`v2.0-enterprise.md`](./v2.0-enterprise.md) per scope SKU completo + [`fase-5-enterprise.md`](./fase-5-enterprise.md) per dettaglio tecnico.

### Da Fase 2 Step 6 (spostato)
- 🔒 → v2.0 Step 0a: **Server Go cross-OS senza Docker** (pure-Go SQLite, single binary, Windows Service nativo + systemd unit, .deb/.rpm/NSIS server)

### Da Fase 2 Step 7 — MCP server
- 📋 → v2.0: **Trasporto HTTP/SSE** (oggi solo stdio)
- 📋 → v2.0: **Tool `pap_create_draft`** — richiede approval workflow

### Da Fase 2 Step 8 — CLI `pap`
- 📋 → v2.0: **Comandi `login` / `new` / `import` / `export`** — richiedono server API o IPC client desktop

### Da Fase 3 Step 5 — Linting
- 📋 → v2.0: **Linting PII block-by-default** (oggi warn-only). Per workspace ad alta sensibilità, naturale con E2E

### Da Fase 3 Step 6 — Modello target
- 📋 → v2.0: **Server endpoint `?target=...`** filter su endpoint search

### Da Fase 3 Step 7 — Cartelle
- 📋 → v2.0: **Server endpoint `/sync/folders`**

### Da Fase 3 Step 9 — Statistiche
- 📋 → v2.0: **Distribuzione per autore** (richiede multi-user team)
- 📋 → v1.x (nice-to-have): **Distribuzione per cartella** (priorità bassa, top-importati copre l'80%)

### Da Fase 4 Step 5 — Fork
- 📋 → v2.0: **Contatore "N fork attivi"** lato originale per workspace team — schema già pronto via `idx_prompts_fork_of`
- 📋 → v2.0: **Pull request leggera** dal fork verso l'originale — dipende da Step 6 approval

### Da Fase 4 Step 2 — Rating
- 📋 → v2.0: **Privacy team su rating personali** — admin vede aggregati ma non singole note. Richiede E2E

### Da Fase 4 Step 6+7 (spostati a Fase 5)
- 📋 → v2.0 Step 6: **Approval workflow** (status `pending_review`, ReviewedByUserId, notifiche WS)
- 📋 → v2.0 Step 7: **RBAC cartelle** (FolderPermissions con additive permissions, inheritance)

### Da Fase 4 Step 8 — Golden + regression
- 📋 → v2.0: **MCP integration** `pap_test_prompt(promptId)` come tool MCP per agenti — richiede MCP HTTP/SSE

### Cross-cutting / opzionali
- 🔒 → v2.0: **Server cross-compile CI matrix** Linux/Windows/macOS (sblocca con Step 0a)

---

## 4. Tecnici / debiti minori — non legati a uno stream specifico

| Item | Marker | Note |
|---|---|---|
| **golangci-lint reattivare sul server** dopo che l'action `golangci/golangci-lint-action@v6+` supporta stabilmente `golangci-lint` v2.x. Oggi sostituito con `go vet` (PR #17) | 🔧 | Verificare ogni 3 mesi. Sostituto attuale è soddisfacente, no priorità. |
| **Workflow CI non auto-listati nei propri path filter**: `cli-build.yml`, `mcp-server-build.yml`, `server-build.yml`. Modifiche a questi YAML non triggerano una run di validazione | 🎨 → v1.0 nice-to-have | Quick win 1 ora. Vedi `docs/contribuire/ci-workflows.md`. |
| **Fallback `candle-core` per ONNX** se `ort` torna instabile | 🔧 | Piano B documentato in `docs/architettura/decisioni/onnx-bundle.md`. Non attivato (ort è stabile da v0.3.0). |
| **Workflow `bootstrap-go.yml`** che rigenera `go.sum` server + CLI on-demand (pattern simile a `bootstrap.yml` per pnpm) | 🔧 | Considerato e non implementato in PR #17. Se `go.sum` torna a divergere, valuteremo. |
| **Tray icon doppia su Windows residuo** | 🔧 → verifica | Memoria `tray_icon_doppia_windows.md` segnala residuo dopo `v0.2.1-fix1`. PR #161 in v0.8.5 ha rimosso `app.trayIcon` da `tauri.conf.json` (root cause): **verificare se ancora presente** su Win11 e aggiornare memoria. |

---

## 5. Bug residui non bloccanti

| Item | Stato | Note |
|---|---|---|
| (vuoto al 2026-05-12) | | Issue #170 (catastrofica) chiusa con v0.8.8. Issue #167 chiusa con v0.8.5. |

---

## 6. Non implementeremo — decisione finale ⛔

Item con razionale costo/beneficio sfavorevole, conservati come traccia decisionale.

### Da Fase 3 Step 5 — Linting
- ⛔ **PH002** (segnaposto dichiarato non usato) — semantica ambigua: difficile distinguere "dichiarato ma non usato per scelta" vs "errore di battitura". Falsi positivi alti.
- ⛔ **PII002** (codice fiscale italiano) — regex complessa low-priority; PII0 (email) e PII1 (carta credito) coprono i casi critici.
- ⛔ **STY002** (mancanza istruzioni chiare) — richiede NLP IT/EN troppo fragile a regex. Falsi positivi disturberebbero gli utenti.

---

## Cronologia PR e item chiusi (archivio storico)

> Item che SONO ATTERRATI ma erano stati rinviati. Conservati per tracciabilità. Quando la sezione cresce troppo, si può consolidare in `CHANGELOG.md` e rimuovere da qui.

### Fase 1-2 (v0.2.x)
- ✅ CLI coverage 53.3% → ≥ 70% — PR #18 (`v0.2.1` ciclo). Achieved 72.7%
- ✅ Server `go.sum` regen + bump Go 1.25 + chi 5.2.2 — PR #17 (`v0.2.0-foundations`). govulncheck server clean (0 vulns)
- ✅ Spike sqlite-vec ⊕ SQLCipher — PR #20
- ✅ Spike ONNX bundle size — PR #21. Crescita 4-8× accettabile
- ✅ Spike modello embedding IT/EN (v1 + v2 riapertura) — PR #22. **MiniLM confermato**. EmbeddingGemma-300m scartato (+180 MB / 3.7× per-embed, +2.5 pt recall@5 non giustificano)
- ✅ Riposizionamento Step 5 (→ patch line) e Step 6 (→ Fase 5) — PR #19
- ✅ Tag `v0.2.0-foundations` (parziale 6/8 step) — manual
- ✅ Versione portable Windows agli asset release — PR #27 + #28
- ✅ Step 6 modello target (Fase 3 anticipato in v0.2.1) — PR #23
- ✅ Step 9 statistiche / Insight (Fase 3 anticipato) — PR #24
- ✅ Step 7 cartelle backend + UI base — PR #25
- ✅ Step 7 cartelle drag&drop + filter chip + rinomina inline — PR #26
- ✅ Bug 1 vault loop portable + bug 2 parziale tray icon — PR #29 / `v0.2.1-fix1`

### Fase 3 (v0.3.0)
- ✅ Step 1 ONNX Runtime + MiniLM-L12-v2 — `ort 2.0.0-rc.12 default-features=false + api-23 + load-dynamic`
- ✅ Step 2 sqlite-vec integration (V005, V006) — Auto-extension + vec0 384-dim
- ✅ Step 3 ricerca ibrida RRF pesata — P95 8.29 ms su 10k prompt
- ✅ Step 4 tag suggeriti semantici — Top-K vec0 nearest + fallback frequenza
- ✅ Step 5 linting (11 regole su 14) — PR #45, #48 / v0.3.0
- ✅ Step 8 prompt componibili `{{import "..."}}` — Parser + resolver + cycle detection + depth 5 + V007 grafo
- ✅ Auto-init Session embeddings al boot — PR #47
- ✅ Idle-unload Session embeddings — PR #51
- ✅ Quality gate Step 10 (5 sub-step) — PR #49-#53
- ✅ Tag `v0.3.0` con build cross-OS (8 asset) — 2026-05-06
- ✅ Bumpare versione `Cargo.toml`/`tauri.conf.json` 0.1.0 → 0.3.0 — PR #55
- ✅ CI Rust client coverage report numerico — PR #53. Gate 60% line

### Fase 4 (v0.4.0)
- ✅ Step 8a Schema golden + run observations + CRUD — PR #58
- ✅ Step 8b Provider abstraction + Ollama — PR #59
- ✅ Step 8c Similarity functions base — PR #60
- ✅ Step 8d `golden_esegui` end-to-end — PR #61
- ✅ Step 8e UI Editor pannello Test — PR #62
- ✅ Step 8f Provider Anthropic + OpenAI + llm-judge — PR #63
- ✅ Step 8g UI Libreria vista Regressioni — PR #64
- ✅ Step 3 Diff tra versioni in CronologiaPrompt — PR #65
- ✅ Step 1 Varianti A/B testing — PR #66
- ✅ Step 4 Confronto fianco-a-fianco — PR #67
- ✅ Step 5 Fork / Clone con tracciabilità — PR #68
- ✅ Step 2 Rating discreto post-uso — PR #69

### Sprint speciali (v0.5.0 - v0.8.x)
- ✅ Sprint v0.5.0 quick wins UX + 5° provider AI Gemini (6 PR) — memoria `sprint_v05_chiuso.md`
- ✅ Sprint v0.6.0 hardening + quick wins (6 PR), coverage 71% — memoria `sprint_v06_chiuso.md`
- ✅ Inline marker CodeMirror sui lint issue (Fase 3 sub-step) — v0.6.0 Step 3
- ✅ Configurazione linter per-categoria (toggle LEN/PH/PII/STY/IMP) — v0.6.0 Step 6
- ✅ Vista "Confronto varianti" dedicata (Fase 4 sub-step) — v0.6.0 Step 5
- ✅ Prompt più importati statistiche (Fase 3 sub-step) — v0.6.0 Step 4
- ✅ Lint health % + top categorie — v0.6.0 Step 4
- ✅ Riload Session embeddings post idle-unload — v0.6.0 Step 2
- ✅ Sprint v0.7.0 refactor coverage + import/cartelle quick wins (6 PR), coverage 74% — memoria `sprint_v07_chiuso.md`
- ✅ Custom free-text target model (Fase 3 sub-step) — v0.7.0 Step 3
- ✅ Esporta singola cartella (Fase 3 sub-step) — v0.7.0 Step 2
- ✅ Hover preview import + Ctrl+click navigazione (Fase 3 sub-step) — v0.7.0 Step 4
- ✅ Cross-prompt linting "chi importa X" (Fase 3 sub-step) — v0.7.0 Step 5 (regola IMP004)
- ✅ Markdown export con front-matter `imports` (Fase 3 sub-step) — v0.7.0 Step 6
- ✅ Redesign UI completo F0-F11 (17 PR) — v0.8.0 (2026-05-09)
- ✅ Patch line v0.8.1-v0.8.4 — bugfix Win11 multi-issue + retry release CI
- ✅ Sprint v0.8.5 (#160 editor UX + #161 tray/modelli + #162 segnaposti globali `{{globale nome}}`) — memoria `sprint_v085_chiuso.md`
- ✅ v0.8.6 hardening sicurezza Go 1.25.10 + chi v5.2.5 + x/crypto v0.51.0 (#164, #166); issue #165 tracking deps chiusa
- ✅ Sprint v0.8.7 sezione Sviluppo + Debug log Telescope-like (#171 backbone, #172 UI, #173 viewer) — memoria `sprint_v087_chiuso.md`
- ✅ Hotfix v0.8.8 fix #170 editor input bloccato (`untrack()` su `$effect`) — memoria `feedback_svelte5_untrack_effect.md`

---

## Come mantenere questo documento

1. **Quando rinvii qualcosa in una PR**: aggiungi una riga nella sezione appropriata (Personale §2 o Enterprise §3), con marker motivazionale (🔒/🔧/📋/🎨) + marker destinazione (→ v1.0 M*, → v2.0, → v1.x, etc.) + link al doc di destinazione.
2. **Quando un item atterra**: spostalo in §"Cronologia PR e item chiusi". Quando la sezione cresce troppo, vecchi item vanno in `CHANGELOG.md` e qui si rimuovono.
3. **Verifica trimestrale**: rileggere tutto, vedere se item 🔒 sono ancora bloccati dalla stessa cosa o se la situazione è cambiata.
4. **Quando si apre un branch v1.0 M*** o `feat/v2.0-enterprise`: spuntare gli item che entrano nel branch.
