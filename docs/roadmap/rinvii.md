# Roadmap dei rinvii

> Censimento unificato di **tutto ciò che è stato deliberatamente rinviato** durante lo sviluppo. Singola fonte di verità: nuovi rinvii vengono aggiunti qui ad ogni PR che li introduce, e gli item vengono spostati nell'archivio storico quando atterrano.
>
> **Aggiornato al**: 2026-07-04.
>
> **Stato macro**: PaP **Personale v1.0.0 è completa** (chiusa con la patch line v0.8.11, milestone M1-M8 tutte atterrate il 2026-05-19). Da allora la patch line **v0.8.x** prosegue con feature e manutenzione incrementale (fino a **v0.8.31**: linter personalizzabile, riordino Impostazioni, **Test Golden completo lato UI**). Lo stream **Enterprise v2.0** non è ancora aperto (gate domanda-driven).

## Convenzioni

**Marker di motivazione**:

- 🔒 **Bloccato da fattore esterno** (certificato, KYC, hardware, decisione di prodotto) → resta dove si trova finché il blocco esterno non cade
- 🔧 **Bloccato da fattore tecnico** (libreria instabile, dipendenza non pronta) → si sblocca quando il fattore tecnico cade
- 📋 **Sub-step di feature già atterrata** → da fare quando si presenta finestra di lavoro, o quando il padre arriva
- 🎨 **Polish / cosmetico** → non bloccante per nessuno
- ⛔ **Scelto deliberatamente di NON implementare** (rapporto costo/beneficio sfavorevole). Resta qui solo come traccia decisionale
- ✏️ **Feature progettata (blueprint pronto) ma non implementata** → in attesa di un ciclo dedicato

**Marker di destinazione (stream)**:

- → `v0.8.x` = patch line attuale Personale (post-v1.0)
- → `v1.x` = refinamento naturale post-v1.0
- → `v2.0` = entra nel piano [`v2.0-enterprise.md`](./v2.0-enterprise.md) / [`fase-5-enterprise.md`](./fase-5-enterprise.md)
- → archivio = atterrato, vedi §"Cronologia" in fondo

---

## 1. Stream Personale — v1.0 ✅ COMPLETO

Tutto lo stream v1.0 (recupero rinvii Fase 1-4 + signing/updater) è **atterrato** nelle milestone M1-M8 (2026-05-16 → 2026-05-19), chiuso con la release v0.8.11.

- **M1 — Auto-update + Authenticode signing**: ✅ cert Certum SimplySign Cloud, NSIS per-user, Tauri Updater Ed25519, `latest.json` firmato, downgrade/signature-mismatch refuse, E2E updater, smoke test NSIS, docs. Vedi memoria `release_v0810_m1_signing`.
- **M2 — a11y svelte-check**: ✅ 18→0 warning, gate CI `--fail-on-warnings` attivo (memoria `m2_chiusa_a11y_svelte_check`).
- **M3 — recupero UI Fase 4** (varianti/rating/golden): ✅ (memoria `m3_chiusa_recupero_ui_fase4`).
- **M4 — import evoluta + intellisense** (`{{import with k=v}}`, `version=N`, autocomplete): ✅ (memoria `m4_chiusa_import_evoluta`).
- **M5 — editor doppia vista** Sorgente/Compilato: ✅ (memoria `m5_chiusa_editor_doppia_vista`).
- **M6 — markdown import/export**: ✅ (memoria `m6_chiusa_markdown_import_export`).
- **M7 — coverage gap**: ✅ Rust 74→80%, TS 38→81% (gate 70), MCP 100% (memoria `m7_chiusa_coverage_gap`).
- **M8 — documentazione utente**: ✅ getting-started, glossario, scorciatoie, troubleshooting, ricette (memoria `m8_chiusa_documentazione_utente`).

**Residui v1.x ancora aperti** (nice-to-have, non bloccanti per nessuno):

- 📋 → v1.x: **Lint rules IMP004/IMP005** per la sintassi scopata (`with k=v`: variabile dichiarata non usata; `version=N`: versione inesistente). Richiede refactor del linter per accesso DB.
- 📋 → v1.x: **Migration automatica backreference** alla promozione di una variante a principale (M3 PR-5). Gli `{{import "id-vecchia-main"}}` nei prompt terzi continuano a puntare alla vecchia main. Riscriverli richiede una decisione semantica (silenziosa vs notifica utente) → rinviato per non decidere sotto pressione.
- 📋 → v1.x: **CLI `pap test <promptId> [--golden=...]`** per CI/CD — il subcommand manca, ma (esplorazione 2026-07-04) **non è un'aggiunta piccola**: CLI (Go) e MCP (TS) aprono il vault SQLite **direttamente, read-only, solo non cifrato** e non hanno accesso a provider/embeddings/similarità (solo-Rust). Non esiste una `pap-core` né FFI, e `cosine` non è replicabile fuori da Rust (serve ONNX+MiniLM). Percorso corretto = **nuovo binario Rust headless** che avvolge `regression::esegui_pure_con_ctx` (già Tauri-free), invocato da CLI/MCP; + blocchi cifratura (SQLCipher/Argon2) e scrittura osservazioni. Vedi memoria `golden_ui_completa`.
- ✏️ → v1.x: **CLI su vault CIFRATI** — oggi la CLI Go è puro-Go (`modernc`, `CGO_ENABLED=0`) e legge **solo vault in chiaro**, ma i vault reali del desktop sono **tutti SQLCipher** → la CLI di fatto non legge nessun vault reale. **Spike 2026-07-04 (memoria `cli_vault_cifrati_spike`): FATTIBILE** — `go-sqlcipher/v4` legge i vault SQLCipher 4.14 del desktop via DSN `_pragma_key`, e `argon2.IDKey` in Go dà la stessa chiave del Rust. **Contro**: è CGO (perde il puro-Go/cross-compile banale). **Decisione rinviata** (utente, 2026-07-04) tra **A** (CLI tutta CGO, un binario, provata) e **C** (sidecar Rust che riusa `vault.rs`, sblocca anche `pap test`/MCP). Legato all'item CLI `pap test` sopra.
- 📋 → v1.x (o v0.9): **Multi-vault** — `WorkspaceSwitcher.svelte` mostra lo switcher ma la selezione di più vault è rinviata (commento `WorkspaceSwitcher.svelte:17`). Oggi vault singolo.
- 📋 → v1.x: **Doc utente `docs/utente/prompt-componibili.md`** aggiornato con esempi `with`/`version`/intellisense.
- 📋 → v1.x: **Distribuzione statistiche per cartella** (priorità bassa, top-importati copre l'80%).

---

## 2. Patch line v0.8.x (post-v1.0) — aperti

Rinvii introdotti dopo il completamento v1.0, nella patch line attuale.

| Item | Marker | Note |
|---|---|---|
| **#303 opzione 3 — sostituzione import via dropdown** | 📋 | Cancellando un prompt referenziato, oltre a "rimuovi gli import" offrire "sostituisci con un altro prompt da dropdown". Stessa macchina di `import_rimuovi_da_dipendenti` ma riscrive `ImportRef.path` invece di rimuovere → futuro `import_sostituisci_in_dipendenti(target, replacement)`. Primo taglio (annulla + rimozione massiva) atterrato in v0.8.20. |
| **Embedding stale dopo rimozione import di massa** (#303) | 🎨 → v1.x | `import_rimuovi_da_dipendenti` non ricalcola gli embedding dei prompt modificati → restano stale fino al prossimo save/backfill. |
| **Release matrix macOS** | 🔒 | **Linux riabilitato** (PR #432, 2026-07-04): `ubuntu-22.04` → `.deb` + `.AppImage` (x86_64) + updater `.sig`; validato con tag di prova (job verde, `latest.json` con entry `linux-x86_64`). Resta solo **macOS**, 🔒 per notarization (Apple Developer cert separato, non richiesto). Da verificare al primo rilascio vero: che `sign-release.ps1` preservi le entry Linux nel `latest.json`. |
| **Menu contestuale — gap residui** (#374-#380) | 📋 → v1.x | Taglio principale completo (v0.8.23). Restano: apertura da tastiera **Shift+F10** (primitivo pronto, manca il wiring per-superficie; i chip-tag `<span>` non sono raggiungibili da tastiera); **Vai alla riga** dall'editor (serve un input numerico, no `prompt()` in webview); **Rinomina etichetta variante** e **Duplica variante** (semantica/backend); unificare l'**Elimina dal menu** col warning import-dipendenze #303 (oggi soft-delete diretto); menu su **pannelli segnaposti/import** (§6.7 del blueprint). |
| **Backend `tags.rs` — gestione tag globali** | ✏️ → v1.x | Comandi rinomina/unisci/colore/elimina tag a livello vault (PR-7 del blueprint menu-contestuale). Abilita le voci tag-management oggi assenti. Diverso dall'associazione tag-su-prompt (`prompt_tag_aggiungi`/`rimuovi`, già fatta in v0.8.23). |
| **Comando batch tag** (perf) | 🎨 → v1.x | `prompt_tag_aggiungi` ricostruisce l'INTERA FTS a ogni chiamata; nel bulk "Aggiungi tag a N" = N rebuild. Accettabile per vault personale; un `prompt_tags_aggiungi_bulk(coppie)` con un solo rebuild è il fix futuro. |

---

## 3. Feature progettate (blueprint pronti) — non implementate ✏️

Design completato, **nessun codice**. Candidati per i prossimi cicli, ognuno con blueprint dedicato.

| Blueprint | Scope | Dimensione |
|---|---|---|
| [`vault-a-cartella.md`](./vault-a-cartella.md) | Storage plain-text `.md` no-lock-in + sidecar `.pap/` (#258) | Grande / strategica |
| [`prompts-as-code.md`](./prompts-as-code.md) | Idea strategica storage/posizionamento — sola ideazione, da decidere | Da definire |

**Blueprint atterrati** (spostati qui dalla tabella):

- [`cestino-e-cancellazione-import.md`](./cestino-e-cancellazione-import.md) → v0.8.20 (#302 cestino + #303 warning import); resta solo l'opzione 3 (§2).
- [`linter-personalizzabile.md`](./linter-personalizzabile.md) → **Fase 1** (#381/#383 toggle per-regola) + **Fase 2** (#384 backend `ConfigLinter`/`SoglieLinter`, #385 frontend UI severità + soglie editabili). Resta solo la **Fase 3** (regole custom utente) — **congelata** per scelta esplicita (blueprint §7), nessun debito.
- [`guida-interattiva.md`](./guida-interattiva.md) → Fase 0/1/3 in v0.8.20-v0.8.23 (#364-#372: hub + "?" ovunque, tour benvenuto + auto-offerta + micro-tour, checklist "Primi passi"). Resta solo la **Fase 4** (deep-link al sito: quando il sito VitePress `apps/site` sarà pubblicato basta cambiare `SITO_BASE` in `docs-links.ts` — le pagine docs esistono già) + l'esplorazione guidata del vault demo.
- [`menu-contestuale.md`](./menu-contestuale.md) → taglio principale in v0.8.23 (#374-#380 + tag): card/cartelle/editor/chip-tag/varianti/selezione-multipla + Gestisci tag / Aggiungi tag a N. Gap residui in §2.

---

## 4. Stream Enterprise — v2.0

Non ancora aperto (gate domanda-driven). Vedi [`v2.0-enterprise.md`](./v2.0-enterprise.md) per lo scope SKU completo + [`fase-5-enterprise.md`](./fase-5-enterprise.md) per il dettaglio tecnico.

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

### Da Fase 4 Step 5 — Fork
- 📋 → v2.0: **Contatore "N fork attivi"** lato originale per workspace team — schema già pronto via `idx_prompts_fork_of`
- 📋 → v2.0: **Pull request leggera** dal fork verso l'originale — dipende da Step 6 approval

### Da Fase 4 Step 2 — Rating
- 📋 → v2.0: **Privacy team su rating personali** — admin vede aggregati ma non singole note. Richiede E2E

### Da Fase 4 Step 6+7 (spostati a Fase 5)
- 📋 → v2.0 Step 6: **Approval workflow** (status `pending_review`, ReviewedByUserId, notifiche WS)
- 📋 → v2.0 Step 7: **RBAC cartelle** (FolderPermissions con additive permissions, inheritance)

### Da Fase 4 Step 8 — Golden + regression
- La **UI desktop dei golden è completa** dal v0.8.30 (esecuzione singola + dettaglio, modifica, giudice llm-judge, costo stimato — #418-421). Restano solo le integrazioni CLI/MCP sotto.
- 📋 → v2.0: **MCP integration** `pap_test_prompt(promptId)` come tool MCP per agenti — richiede MCP HTTP/SSE (stesso runner Rust headless della CLI `pap test`, vedi §1)

### Cross-cutting / opzionali
- 🔒 → v2.0: **Server cross-compile CI matrix** Linux/Windows/macOS (sblocca con Step 0a)

---

## 5. Tecnici / debiti minori — non legati a uno stream specifico

| Item | Marker | Note |
|---|---|---|
| **Segreti (`sync_token`) nel keychain OS** | 🔒 → v1.x | Oggi `sync_token` è salvato in chiaro in `preferenze.json` (mitigato: `0600` su Unix, AppData per-utente su Windows). `preferenze.rs:193` prevede lo spostamento nel keychain OS (crate `keyring`). **Unico debito con implicazioni di sicurezza.** |
| **Tabella prezzi `pricing.rs` indicativa** | 🎨 → manutenzione | La stima del `costo_stimato` dei golden usa prezzi hardcoded (USD/1M token) + token di input stimati euristicamente; da aggiornare quando cambiano i listini dei provider. |
| **Conteggio token euristico** (`statistiche.rs`) | 🎨 | "~N token medi" via euristica lunghezza; tokenizer reale rinviato (bassa priorità). |
| **Workflow CI non auto-listati nei propri path filter**: `cli-build.yml`, `mcp-server-build.yml`, `server-build.yml`. Modifiche a questi YAML non triggerano una run di validazione | 🎨 → v1.x | Quick win ~1 ora. Vedi `docs/contribuire/ci-workflows.md`. |
| **Fallback `candle-core` per ONNX** se `ort` torna instabile | 🔧 | Piano B documentato in `docs/architettura/decisioni/onnx-bundle.md`. Non attivato (ort stabile da v0.3.0). |
| **Workflow `bootstrap-go.yml`** che rigenera `go.sum` server + CLI on-demand | 🔧 | Considerato e non implementato. Se `go.sum` torna a divergere, valuteremo. |
| **Tray icon doppia su Windows residuo** | 🔧 → verifica | Memoria `tray_icon_doppia_windows.md` segnala residuo dopo `v0.2.1-fix1`. PR #161 (v0.8.5) ha rimosso `app.trayIcon` da `tauri.conf.json`: **verificare se ancora presente** su Win11 e aggiornare memoria. |
| **Pin ecosistema brotli in `Cargo.toml`** | ✅ rimosso | Era un workaround temporaneo (#306) per `alloc-no-stdlib 3.0.0`; **rimosso in #352** quando l'upstream si è allineato (brotli 8.0.4). Il canary `dep-canary` lo aveva auto-segnalato. |

---

## 6. Bug residui non bloccanti

| Item | Stato | Note |
|---|---|---|
| (vuoto al 2026-06-16) | | I bug da gate test (onboarding/tray/errori vault, import, palette, creazione cartelle) sono stati chiusi nei cicli triage v0.8.15-v0.8.20. |

---

## 7. Non implementeremo — decisione finale ⛔

Item con razionale costo/beneficio sfavorevole, conservati come traccia decisionale.

### Da Fase 3 Step 5 — Linting
- ⛔ **PH002** (segnaposto dichiarato non usato) — semantica ambigua: difficile distinguere "dichiarato ma non usato per scelta" vs "errore di battitura". Falsi positivi alti.
- ⛔ **PII002** (codice fiscale italiano) — regex complessa low-priority; PII0 (email) e PII1 (carta credito) coprono i casi critici.
- ⛔ **STY002** (mancanza istruzioni chiare) — richiede NLP IT/EN troppo fragile a regex. Falsi positivi disturberebbero gli utenti.

---

## Cronologia PR e item chiusi (archivio storico)

> Item che SONO ATTERRATI ma erano stati rinviati. Conservati per tracciabilità. Quando la sezione cresce troppo, vecchi item vanno in `CHANGELOG.md` e qui si rimuovono.

### Fase 1-2 (v0.2.x)
- ✅ CLI coverage 53.3% → 72.7% — PR #18
- ✅ Server `go.sum` regen + bump Go 1.25 + chi 5.2.2 — PR #17 (govulncheck clean)
- ✅ Spike sqlite-vec ⊕ SQLCipher — PR #20; ONNX bundle size — PR #21; modello embedding (MiniLM confermato) — PR #22
- ✅ Riposizionamento Step 5/6 — PR #19; Step 6 modello target — PR #23; Step 9 statistiche — PR #24; Step 7 cartelle — PR #25/#26
- ✅ Versione portable Windows agli asset — PR #27/#28; Tag `v0.2.0-foundations`; bug vault loop + tray parziale — PR #29 (`v0.2.1-fix1`)

### Fase 3 (v0.3.0)
- ✅ Step 1 ONNX+MiniLM; Step 2 sqlite-vec (V005/V006); Step 3 ricerca ibrida RRF (P95 8.29ms); Step 4 tag suggeriti semantici; Step 5 linting (11/14 regole); Step 8 prompt componibili `{{import}}`; auto-init + idle-unload embeddings; quality gate Step 10 — PR #45-#53
- ✅ Tag `v0.3.0` cross-OS (8 asset) 2026-05-06; bump versione → 0.3.0 (PR #55)

### Fase 4 (v0.4.0)
- ✅ Step 8a-g Golden + provider (Ollama/Anthropic/OpenAI/Gemini) + llm-judge — PR #58-#64; Step 3 diff versioni — PR #65; Step 1 varianti A/B — PR #66; Step 4 confronto — PR #67; Step 5 fork — PR #68; Step 2 rating — PR #69

### Sprint v0.5.0 - v0.8.0
- ✅ v0.5.0 quick wins UX + 5° provider Gemini; v0.6.0 hardening + linter per-categoria + inline marker; v0.7.0 refactor coverage + import/cartelle quick wins; Redesign UI completo F0-F11 (17 PR) → v0.8.0 (2026-05-09)
- ✅ Patch v0.8.1-v0.8.4 bugfix Win11; v0.8.5 (editor UX, tray/modelli, `{{global nome}}`); v0.8.6 hardening sicurezza Go; v0.8.7 sezione Sviluppo + Debug log; v0.8.8 hotfix #170 (`untrack()`)

### v1.0 Personale — M1-M8 (v0.8.10 → v0.8.11, 2026-05-16/19)
- ✅ **M1** signing+updater (Authenticode Certum + Tauri Updater Ed25519); **M2** a11y 18→0 warning; **M3** recupero UI Fase 4; **M4** import evoluta + intellisense; **M5** editor doppia vista; **M6** markdown import/export; **M7** coverage gap (Rust 80%, TS 81%, MCP 100%); **M8** documentazione utente
- ✅ Release **v0.8.11 = v1.0.0 Personale completa** (2026-05-19)

### Patch line v0.8.x (post-v1.0)
- ✅ **v0.8.12** audit sicurezza + export lossless (#184-186) + drop MSI (#254, solo NSIS per-user + portable)
- ✅ **v0.8.15** triage gate test (onboarding/tray/errori vault); **v0.8.16** triage (compila/import, demo globali, errori vault, updater notes); **v0.8.17** import da palette (#299)
- ✅ **v0.8.18** creazione cartelle (#301) + hardening CI (Cargo.lock committato + `--locked` + toolchain pin + Dependabot + canary, #309/#332) + pin brotli (#306) + 19 dipendenze (2 sicurezza)
- ✅ **v0.8.19** migrazione rand 0.9 fail-closed (#333) + rimozione pin brotli (#352, upstream allineato)
- ✅ **v0.8.20** cestino prompt (#302) + warning cancellazione import (#303) + colorazione globali/import parametrizzati (#353/#304) + guida interattiva Fase 0+1-PR1 (#364-#367) + manutenzione dipendenze
- ✅ **v0.8.21** guida: auto-offerta tour post-onboarding (#368) + micro-tour per-feature (#369)
- ✅ **v0.8.22** hotfix: il tour di benvenuto non partiva (#370, cleanup `$effect` annullava i rAF)
- ✅ **v0.8.23** menu contestuale completo (#374-#380, 7 PR: card/cartelle/editor/chip-tag/varianti/bulk + Gestisci tag) + checklist "Primi passi" (#372) + sintassi `{{globale}}`→`{{global}}` (#371) + fix ancora glossario (#373). Lungo la strada: scoperto e corretto bug serde `prompt_sposta` (campi struct camelCase) già presente in main.
- ✅ **v0.8.24** linter personalizzabile Fasi 1+2 (#381/#383 toggle per-regola, #384/#385 tuning severità+soglie) + fix icona Diagnosi (#386).
- ✅ **v0.8.25** triage: rename identifier `com.pap.client` + migrazione one-shot, tab Valutazioni (#390), creazione golden (#382), `checkout@v5` (#388), warning Rust test (#387) — PR #391-#394.
- ✅ **v0.8.26** angolo alto-sx/codename (#404), bottoni editor (#402), connettore varianti (#403) + **catena Go 1.25 sbloccata** (#405 — chiude **#334**: il blocco era `golangci-lint-action` v6, non il linter; action→v9 + toolchain 1.25 + modernc 1.53) + **migrazione vitest 4** (#401) + rand 0.10/zip 8/ndarray + 19 dipendenze.
- ✅ **v0.8.27** riordino sidebar Impostazioni (6 gruppi, #413) + ordinamento "Migliori" pesa-voti + rifiniture header/onboarding.
- ✅ **v0.8.28** scroll indipendente sidebar/dettaglio Impostazioni (#414, prop `corpoFisso` su `Modale`).
- ✅ **v0.8.29** voto medio 90gg in "Migliori" (#415) + fix connettore varianti in lista (#416, regressione #412: `parent_prompt_id` non esposto).
- ✅ **v0.8.30** **Test Golden completo lato UI** (#418 esecuzione singola + dettaglio, #419 modifica, #420 giudice llm-judge, #421 costo stimato `pricing.rs`) + connettore varianti solo con ordinamento "A-Z" (#417).
- ✅ **v0.8.31** rifiniture Golden: inserimento guidato variabili + descrizione similarità (#422), selettore modello a tendina per provider pubblici (#423); + editor aggiornato dopo ripristino da cronologia (#425).
- ✅ **post-v0.8.31**: fix falso positivo linter **PH003** — non segnala più `{{global nome}}` e `{{import "..."}}` come nomi illegali (#428).

---

## Come mantenere questo documento

1. **Quando rinvii qualcosa in una PR**: aggiungi una riga nella sezione appropriata (v1.x §1, patch line §2, Enterprise §4, tecnici §5) con marker motivazionale (🔒/🔧/📋/🎨/⛔/✏️) + marker destinazione + link al doc.
2. **Quando un item atterra**: spostalo in §"Cronologia". Quando la sezione cresce troppo, vecchi item vanno in `CHANGELOG.md` e qui si rimuovono.
3. **Verifica trimestrale**: rileggere tutto, vedere se item 🔒/🔧 sono ancora bloccati dalla stessa cosa o se la situazione è cambiata.
4. **Quando si apre un branch feature** (blueprint §3 o `feat/v2.0-enterprise`): spuntare gli item che entrano nel branch.
