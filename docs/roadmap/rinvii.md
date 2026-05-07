# Roadmap dei rinvii

> Censimento unificato di **tutto ciò che è stato deliberatamente rinviato** durante lo sviluppo. Questo documento è la singola fonte di verità: nuovi rinvii vengono aggiunti qui ad ogni PR che li introduce, e gli item vengono rimossi/spuntati quando atterrano.
>
> **Stato**: aggiornato al 2026-05-07 dopo Fase 4 client-first chiusa (6/8 step). Tag `v0.4.0` in arrivo dopo Step 9-10.
>
> **Convenzioni**:
> - 🔒 = bloccato da fattore esterno (cert, KYC, hardware, decisione di prodotto) → resta nel patch line di destinazione (es. `v0.2.x`)
> - 🔧 = bloccato da fattore tecnico (libreria instabile, dipendenza non pronta) → si sblocca quando il fattore tecnico cade
> - 📋 = sub-step già pianificato di una feature, da fare quando il padre arriva → **candidato naturale per `v0.5.0` (recupero ritardi)**
> - 🎨 = polish / cosmetico, non bloccante per nessuno → **candidato naturale per `v0.5.0` (recupero ritardi)** o per `v0.6.0` (pulizia UI) se è puramente visuale
> - ⛔ = scelto deliberatamente di **non** implementare (rapporto costo/beneficio sfavorevole). Resta qui solo come traccia decisionale
> - ✅ = item che è atterrato ed è qui solo per archivio storico
>
> Vedi [`release-plan.md`](./release-plan.md) per il calendario completo di v0.5.0/v0.6.0 e i criteri di ingresso.

---

## 1. Patch line `v0.2.x` — sblocco esterno

Item che vivono nel branch della release v0.2 ma aspettano qualcosa di non-tecnico.

| Item | Stato | Sblocca con |
|---|---|---|
| **Step 5 — Auto-update silenzioso completo** (NSIS per-user, Tauri Updater, `latest.json` firmato, downgrade refuse, signature mismatch refuse) | 🔒 | Cert Authenticode Certum OSS (procedura KYC in corso) |
| Firma Authenticode su tutti gli `.exe` portable di release | 🔒 | Stesso cert Certum |
| Test reale **macOS notarization** del bundle Tauri con `libonnxruntime.dylib` inclusa | 🔒 | Apple Developer certificate |
| Test reale **Authenticode signing** del bundle Windows con `onnxruntime.dll` inclusa | 🔒 | Cert Certum |
| `docs/auto-update.md` (meccanica + troubleshooting + recovery update corrotto) | 📋 | Step 5 |
| `docs/decisioni/authenticode-signing.md` (provider considerati, criteri attivazione) | 📋 | Step 5 |

**Quando atterrano**: cert Certum arriva → branch `feat/auto-update`, due PR (`v0.2.2` NSIS per-user; `v0.2.3` updater + signing).

---

## 2. Bug residui non bloccanti

| Item | PR origine | Memoria |
|---|---|---|
| **Tray icon doppia su Windows** — fix `v0.2.1-fix1` rende visibile la nostra icona ma ne resta una seconda di origine ignota. Possibili cause: global-shortcut plugin con proprio tray nascosto, indicator "running app" del taskbar Windows, doppia chiamata a `TrayIconBuilder::build`. Da indagare quando si tocca tray/visibility. | #29 | `tray_icon_doppia_windows.md` |

---

## 3. Cosmetici / debiti tecnici minori

| Item | Note |
|---|---|
| **golangci-lint reattivare sul server** dopo che l'action `golangci/golangci-lint-action@v6+` supporta stabilmente v2.x. Oggi è stato sostituito con `go vet` (PR #17) perché v1.64 incompatibile con Go 1.25. | 🔧 |
| **~37 warning a11y di `svelte-check`** (autofocus, label senza control, role/aria mismatch). Target `v0.6.0` (pulizia UI). | 🎨 |
| **Workflow CI non auto-listati nei propri path filter**: `cli-build.yml`, `mcp-server-build.yml`, `server-build.yml`. Modifiche a questi YAML non triggerano una run di validazione. Aggiungere ognuno nei propri `paths:` come fatto per `client-build.yml` (PR #53). Vedi `docs/contribuire/ci-workflows.md`. | 🎨 |

---

## 4. Coverage e quality gate

| Item | Soglia target | Stato oggi |
|---|---|---|
| Coverage TS client `vitest --coverage` su `lib/*.ts` | 70% | non configurata |
| Coverage server **alzare da 56% → 70%** con unit test mirati su `database`/`middleware`/`models`/`sync`/`ws` | 70% | 56.2% via test integrazione cross-package |
| Test unit MCP server (almeno `sanitizzaFts`, `compila`, `estraiSegnaposti`) | qualunque >0 | placeholder `echo "No tests yet"` |
| Coverage **client Rust 60.12% → 70%** (cargo-llvm-cov nel CI già attivo, gate floor 60% in PR #53) | 70% | 60.12% line, 67.64% function. Roadmap incrementale per file in `docs/operativo/coverage.md` |

---

## 5. Fase 3 — prerequisiti tecnici (storia)

| Item | Stato | Azione |
|---|---|---|
| ~~**`ort` 2.x compile fix su Rust stable**~~ | ✅ risolto 2026-05-05 | rc.12 compila pulito con `default-features=false` + feature `api-23`. Documentato in `architettura/decisioni/onnx-bundle.md`. |
| Fallback `candle-core` se `ort` resta instabile a lungo | 🔧 | Piano B documentato in `docs/architettura/decisioni/onnx-bundle.md`. Non attivato in Fase 3 (ort è risultato stabile). |

---

## 6. Sub-step rinviati (di feature già atterrate)

Pezzi di feature parzialmente atterrate, con il resto programmato per `v0.5.0` (recupero ritardi) o per fasi successive.

### Da Fase 2 Step 4 — Import/export
- 📋 **Markdown import/export** (oggi solo JSON con schema v1)

### Da Fase 2 Step 7 — MCP server
- 📋 **Trasporto HTTP/SSE** (oggi solo stdio)
- 📋 **Tool `pap_create_draft`** — richiede approval workflow di Fase 4

### Da Fase 2 Step 8 — CLI `pap`
- 📋 **Comandi `login` / `new` / `import` / `export`** — richiedono server API o IPC client desktop

### Da Fase 3 Step 5 — Linting
- ✅ **Inline marker CodeMirror 6** atterrati in v0.6.0 Step 3: `lib/codemirror/lint-markers.ts` con `StateField<DecorationSet>` + `setLintIssues` effect; underline wavy colorato per severità (error/warning/info) + tooltip nativo `code: messaggio`.
- 📋 **Configurazione per-categoria** in Impostazioni (abilita/disabilita LEN/PH/PII/STY/IMP). Oggi sempre attive
- ⛔ **PH002** (segnaposto dichiarato non usato) — semantica ambigua, scelta di non implementare in v0.3
- ⛔ **PII002** (codice fiscale italiano) — regex complessa low-priority, scelta di non implementare in v0.3
- ⛔ **STY002** (mancanza istruzioni chiare) — richiede NLP IT/EN troppo fragile a regex, scelta di non implementare in v0.3
- 📋 **Linting PII block-by-default** (oggi warn-only). Per workspace ad alta sensibilità, naturale in Fase 5 con E2E

### Da Fase 3 Step 6 — Modello target
- 📋 **Custom free-text target model** (oggi solo i 9 preset)
- 📋 **Server endpoint `?target=...`** — Fase 5

### Da Fase 3 Step 7 — Cartelle
- 📋 **Esporta singola cartella** (oggi solo intero vault)
- 📋 **Server endpoint `/sync/folders`** — Fase 5

### Da Fase 3 Step 8 — Prompt componibili
- 📋 **Sintassi `{{import "x" with k=v}}`** per variabili scopate per import. Schema decodifica già pensato, manca parser
- 📋 **Pinning a versione storica** `{{import "x" version=N}}` — schema `PromptVersions` già pronto, manca solo parser + lookup. Naturale in Fase 4
- 📋 **Editor doppia vista Sorgente/Compilato** integrata (oggi separato in `CompilatorePrompt` standalone)
- 📋 **Hover preview import** sul body con tooltip del prompt importato
- 📋 **Ctrl+click "Vai al prompt importato"** dentro l'editor
- 📋 **Cross-prompt linting** — quando si modifica un prompt, mostra "questo prompt è importato da N altri" usando `PromptImports` come grafo inverso
- 📋 **Markdown export con front-matter `imports`** per riproducibilità

### Da Fase 3 Step 9 — Statistiche
- ✅ **Prompt più importati** atterrato in v0.6.0 Step 4: `top_importati()` in `statistiche.rs` riusa `idx_imports_imported` (grafo inverso) con `COUNT(DISTINCT ParentPromptId)`, top 10 esposti in vista Insight.
- 📋 **Distribuzione per cartella** — Step 7 chiuso, idem
- 📋 **Distribuzione per autore** — Fase 5 (multi-user team)
- ✅ **Lint health %** + top categorie atterrato in v0.6.0 Step 4: `calcola_lint_health()` esegue `linting::analizza` body-only su tutti i prompt attivi, aggrega `prompt_senza_issue / totale * 100` + top 5 categorie per prefisso (`PH`, `PII`, `LEN`, `STY`).

### Da Fase 3 Step 10 — Quality gate
- ✅ **Riload automatico Session post idle-unload** atterrato in v0.6.0 Step 2: nuova `assicura_session_caricata(rt_state, vault_state)` chiamata da `cerca_semantica` prima di `compute_embedding_opt`; refactor `init_session_pure` idempotente.
- 📋 **Coverage line client Rust 60% → 70%** — gate CI già attivo, roadmap per file in `docs/operativo/coverage.md` (priorità: `embeddings.rs`, `vault.rs`, `audit.rs`, `import_export.rs`)

### Da Fase 4 Step 1 — Varianti
- 📋 **UI Editor "Crea variante"** (oggi solo dalla Libreria) — atterrabile come quick win in `v0.5.0`
- 📋 **Vista "Confronto varianti" dedicata** multicolonna — riusabile da `ConfrontoPrompt` (Step 4 atterrato)
- 📋 **Renderer dropdown variante** con switch al volo mantenendo i valori del form
- 📋 **Promuovi variante a principale** (swap main ↔ variant con preservazione storia)

### Da Fase 4 Step 2 — Rating
- 📋 **Modale "Aggiungi nota" su voto negativo** — campo `Note` già nello schema V013, manca solo UI prompt modale
- 📋 **Sort by quality** "Migliori prompt" in Libreria — nuova vista che ordina per rating medio recente
- 📋 **Privacy team su rating personali** — admin vede aggregati ma non singole note. Naturale Fase 5 con E2E

### Da Fase 4 Step 5 — Fork
- 📋 **Contatore "N fork attivi"** lato originale per workspace team — schema già pronto via `idx_prompts_fork_of`
- 📋 **Pull request leggera** dal fork verso l'originale — dipende da Step 6 approval, naturale Fase 5

### Da Fase 4 Step 8 — Golden + regression
- 📋 **"Esegui tutti i golden" batch** con riassunto pass/fail — quick win frontend in `v0.5.0`
- 📋 **Provider Google API (Gemini)** — non implementato in v0.4 (4 provider su 5 pianificati: Anthropic, OpenAI, OpenAI-compat, Ollama)
- 📋 **CLI integration** `pap test <promptId> [--golden=...]` per CI/CD — `apps/cli` esistente, manca subcommand
- 📋 **MCP integration** `pap_test_prompt(promptId)` come tool MCP per agenti — Fase 5 con MCP HTTP/SSE
- 📋 **Audit security** chiavi API: nessun log evidente, ma serve verifica formale (security-review agent)

---

## 7. Fase 5 — enterprise, domanda-driven

| Item | Origine |
|---|---|
| **Step 0a — Server Go cross-OS senza Docker** (pure-Go SQLite, single binary, Windows Service nativo + systemd unit, .deb/.rpm/NSIS server) | spostato da Fase 2 Step 6 |
| Sync server in produzione | gate per Fase 5 + tutti i feature server-side multi-user |
| `/sync/folders` endpoint | da Step 7 cartelle (Fase 3) |
| `?target=` filter su endpoint search | da Step 6 modello target (Fase 3) |
| Distribuzione statistiche per autore | da Step 9 (Fase 3) |
| **Step 6 Fase 4 — Approval workflow** (status `pending_review`, ReviewedByUserId, notifiche WS) | spostato da Fase 4 Step 6 — richiede multi-utente reale |
| **Step 7 Fase 4 — RBAC cartelle** (FolderPermissions con additive permissions, inheritance) | spostato da Fase 4 Step 7 — richiede multi-utente reale |

Tutti questi atterrano se e solo se c'è un workspace team in produzione che li richiede (Fase 5 è esplicitamente "domanda-driven").

---

## 8. Cross-cutting / opzionali

| Item | Note |
|---|---|
| **Workflow `bootstrap-go.yml`** che rigenera `go.sum` server + CLI on-demand (pattern simile a `bootstrap.yml` per pnpm) | Considerato e non implementato in PR #17. Se in futuro il `go.sum` torna a divergere, vale la pena. |
| **Test E2E Tauri Updater** (build dev → fake update server → download + apply, downgrade refuse, signature mismatch refuse) | 🔒 sblocca con Step 5 patch line |
| **Smoke test installer NSIS per-user** su Win10 e Win11 (verifica nessun UAC) | 🔒 sblocca con Step 5 patch line |
| **Server cross-compile CI matrix** Linux/Windows/macOS | 🔒 sblocca con Fase 5 Step 0a |

---

## Cronologia PR e item chiusi

> Solo per archivio: gli item che SONO ATTERRATI ma erano stati rinviati. Una volta consolidati i 6 mesi successivi, si possono spostare in `CHANGELOG.md` e rimuovere da qui.

| Item | Chiuso in | Note |
|---|---|---|
| ✅ CLI coverage 53.3% → ≥ 70% (target era 70%) | PR #18 (`v0.2.1` ciclo) | Achieved 72.7% |
| ✅ Server `go.sum` regen + bump Go 1.25 + chi 5.2.2 | PR #17 (`v0.2.0-foundations`) | govulncheck server clean (0 vulns) |
| ✅ Spike sqlite-vec ⊕ SQLCipher | PR #20 | Step 2 Fase 3 procede senza fallback |
| ✅ Spike ONNX bundle size | PR #21 | Crescita 4-8× accettabile, bundle inclusivo |
| ✅ Spike modello embedding IT/EN (v1) | PR #22 | `paraphrase-multilingual-MiniLM-L12-v2` (97.5% recall@5) |
| ✅ Spike modello embedding IT/EN (v2, riapertura per valutare alternative 2024-2025) | TBD (PR corrente) | **MiniLM confermato**. EmbeddingGemma-300m valutato e scartato per trade-off non giustificati (+180 MB / 3.7× per-embed) sul +2.5 pt recall@5 |
| ✅ Riposizionamento Step 5 (→ patch line) e Step 6 (→ Fase 5) | PR #19 | docs/roadmap/fase-2-foundations.md, docs/roadmap/fase-5-enterprise.md |
| ✅ Tag `v0.2.0-foundations` (parziale 6/8 step) | manual | release stable |
| ✅ Versione portable Windows agli asset release | PR #27 + #28 | `Prompt-a-Porter-portable-windows-x64-{tag}.zip` |
| ✅ Step 6 modello target (Fase 3 anticipato in v0.2.1) | PR #23 | dropdown editor, filtro libreria, badge |
| ✅ Step 9 statistiche / Insight (Fase 3 anticipato) | PR #24 | KPI grid + bar charts SVG inline |
| ✅ Step 7 cartelle backend + UI base (Fase 3 anticipato) | PR #25 | schema V004, CRUD Tauri, sidebar tree |
| ✅ Step 7 cartelle drag&drop + filter chip + rinomina inline | PR #26 | UX completa |
| ✅ Bug 1 (vault loop portable) + bug 2 parziale (tray icon visibile) | PR #29 / `v0.2.1-fix1` | residuo: doppia icona Windows |
| ✅ Step 1 ONNX Runtime + MiniLM-L12-v2 (Fase 3) | v0.3.0 | `ort 2.0.0-rc.12 default-features=false + api-23 + load-dynamic`, modello + runtime download lazy |
| ✅ Step 2 sqlite-vec integration (V005, V006) | v0.3.0 | Auto-extension + vec0 384-dim, hooks editor su create/update/delete |
| ✅ Step 3 ricerca ibrida RRF pesata | v0.3.0 | P95 8.29 ms su 10k prompt, slider alpha 0/0.5/1, fallback FTS-only |
| ✅ Step 4 tag suggeriti semantici | v0.3.0 | Top-K vec0 nearest, fallback frequenza < MIN_TAG_PER_SEMANTIC |
| ✅ Step 5 linting (11 regole su 14) | PR #45, #48 / v0.3.0 | LEN/PH/PII/STY body-only + IMP* DB-aware con cycle/depth |
| ✅ Step 8 prompt componibili `{{import "..."}}` | v0.3.0 | Parser + resolver cartella/titolo + cycle detection + depth 5 + V007 grafo |
| ✅ Auto-init Session embeddings al boot | PR #47 / v0.3.0 | Caricamento automatico se modello+runtime presenti su disco |
| ✅ Idle-unload Session embeddings | PR #51 / v0.3.0 | Soglia configurabile (5/10/30/60 min, 0=off), libera ~150 MB RAM |
| ✅ Quality gate Step 10 (5 sub-step) | PR #49-#53 / v0.3.0 | Grace degradation, bench P95, idle-unload, stress cartelle, coverage gate 60% |
| ✅ Tag `v0.3.0` con build cross-OS (8 asset) | 2026-05-06 | Linux deb/rpm/AppImage, macOS arm64, Windows NSIS/MSI/portable |
| ✅ Bumpare versione `Cargo.toml`/`tauri.conf.json` 0.1.0 → 0.3.0 | PR #55 / v0.3.0 | Allineamento al tag, debt tecnico chiuso |
| ✅ CI Rust client coverage report numerico | PR #53 / v0.3.0 | cargo-llvm-cov nel workflow rust-test, gate 60% line, doc in `docs/operativo/coverage.md` |
| ✅ Step 8a Fase 4 — Schema golden + run observations + CRUD | PR #58 (post v0.3.0) | V008/V009 + modulo `regression.rs`, 22 test |
| ✅ Step 8b Fase 4 — Provider abstraction + Ollama | PR #59 | trait `AIProvider` + `OllamaProvider`, 10 test |
| ✅ Step 8c Fase 4 — Similarity functions base | PR #60 | cosine + exact-match + regex, 25 test |
| ✅ Step 8d Fase 4 — `golden_esegui` end-to-end | PR #61 | dispatch mock-able provider, 10 test |
| ✅ Step 8e Fase 4 — UI Editor pannello Test | PR #62 | tab Golden CRUD + Esegui in EditorPrompt |
| ✅ Step 8f Fase 4 — Provider Anthropic + OpenAI + llm-judge | PR #63 | V010 ProviderConfig + 47 test |
| ✅ Step 8g Fase 4 — UI Libreria vista Regressioni | PR #64 | tabella drift + export CSV RFC 4180, 10 test |
| ✅ Step 3 Fase 4 — Diff tra versioni in CronologiaPrompt | PR #65 | jsdiff + `VersionDiff.svelte`, 16 vitest |
| ✅ Step 1 Fase 4 — Varianti A/B testing | PR #66 | V011 + modulo `varianti.rs`, 16 test |
| ✅ Step 4 Fase 4 — Confronto fianco-a-fianco di prompt diversi | PR #67 | `ConfrontoPrompt.svelte` riusa VersionDiff |
| ✅ Step 5 Fase 4 — Fork / Clone con tracciabilità | PR #68 | V012 + modulo `fork.rs`, 16 test |
| ✅ Step 2 Fase 4 — Rating discreto post-uso | PR #69 | V013 + modulo `rating.rs`, 15 test (toast 👎/😐/👍 + badge %) |

---

## Come mantenere questo documento

1. **Quando rinvii qualcosa in una PR**: aggiungi una riga nella sezione appropriata, con marker (🔒/🔧/📋/🎨), origine PR, link al doc di destinazione (Fase X Step Y).
2. **Quando un item atterra**: spostalo in *Cronologia PR e item chiusi*. Quando la sezione cresce troppo, vecchi item vanno in `CHANGELOG.md` e qui si rimuovono.
3. **Verifica trimestrale**: rileggere tutto, vedere se item 🔒 sono ancora bloccati dalla stessa cosa o se la situazione è cambiata.
