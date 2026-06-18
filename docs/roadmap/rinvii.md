# Roadmap dei rinvii

> Censimento unificato di **tutto ciò che è stato deliberatamente rinviato** durante lo sviluppo. Singola fonte di verità: nuovi rinvii vengono aggiunti qui ad ogni PR che li introduce, e gli item vengono spostati nell'archivio storico quando atterrano.
>
> **Aggiornato al**: 2026-06-16.
>
> **Stato macro**: PaP **Personale v1.0.0 è completa** (chiusa con la patch line v0.8.11, milestone M1-M8 tutte atterrate il 2026-05-19). Da allora la patch line **v0.8.x** prosegue con feature e manutenzione incrementale (fino a v0.8.20). Lo stream **Enterprise v2.0** non è ancora aperto (gate domanda-driven).

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
- 📋 → v1.x: **CLI `pap test <promptId> [--golden=...]`** per CI/CD — `apps/cli` esistente, manca il subcommand.
- 📋 → v1.x: **Doc utente `docs/utente/prompt-componibili.md`** aggiornato con esempi `with`/`version`/intellisense.
- 📋 → v1.x: **Distribuzione statistiche per cartella** (priorità bassa, top-importati copre l'80%).

---

## 2. Patch line v0.8.x (post-v1.0) — aperti

Rinvii introdotti dopo il completamento v1.0, nella patch line attuale.

| Item | Marker | Note |
|---|---|---|
| **#334 — CLI Go 1.25 + modernc/sqlite 1.52** | 🔧 (blocked) | `modernc.org/sqlite` 1.52 esige go1.25, ma nessuna release di golangci-lint è ancora buildata con go1.25 → rifiuta di lintare un modulo go1.25. Build/test/vet passano: blocca solo il lint. Sblocca quando golangci-lint rilascia un build go1.25 (vedi Dependabot #339 `golangci-lint-action` 6→9 da valutare). Issue #334 etichettata `blocked`. |
| **Migrazione a vitest 4** (Dependabot #341 `@vitest/coverage-v8` + #345 `vitest`) | 🔧 | Major coordinato (i due vanno insieme); vitest 4 rompe l'attuale setup di test → migrazione config/API rinviata a un ciclo dedicato. PR lasciate aperte/rosse. |
| **#303 opzione 3 — sostituzione import via dropdown** | 📋 | Cancellando un prompt referenziato, oltre a "rimuovi gli import" offrire "sostituisci con un altro prompt da dropdown". Stessa macchina di `import_rimuovi_da_dipendenti` ma riscrive `ImportRef.path` invece di rimuovere → futuro `import_sostituisci_in_dipendenti(target, replacement)`. Primo taglio (annulla + rimozione massiva) atterrato in v0.8.20. |
| **Embedding stale dopo rimozione import di massa** (#303) | 🎨 → v1.x | `import_rimuovi_da_dipendenti` non ricalcola gli embedding dei prompt modificati → restano stale fino al prossimo save/backfill. |
| **Release matrix macOS / Linux** | 📋 / 🔒 | I target macOS/Linux sono ancora commentati nella build matrix di `release.yml` (solo Windows NSIS + portable in produzione). macOS notarization resta 🔒 (Apple Developer cert separato, non richiesto). |

---

## 3. Feature progettate (blueprint pronti) — non implementate ✏️

Design completato, **nessun codice**. Candidati per i prossimi cicli, ognuno con blueprint dedicato.

| Blueprint | Scope | Dimensione |
|---|---|---|
| [`guida-interattiva.md`](./guida-interattiva.md) | Guida/tutorial interattivo in-app (help system a strati: tour spotlight + "?" contestuali + checklist), profondità sul sito | Media / a fasi |
| [`menu-contestuale.md`](./menu-contestuale.md) | Menu tasto-destro context-aware nell'app | Piccola / self-contained |
| [`linter-personalizzabile.md`](./linter-personalizzabile.md) | Visibilità + tuning regole linter, in 2 fasi rilasciabili indipendenti | Media |
| [`vault-a-cartella.md`](./vault-a-cartella.md) | Storage plain-text `.md` no-lock-in + sidecar `.pap/` (#258) | Grande / strategica |
| [`prompts-as-code.md`](./prompts-as-code.md) | Idea strategica storage/posizionamento — sola ideazione, da decidere | Da definire |

Il blueprint [`cestino-e-cancellazione-import.md`](./cestino-e-cancellazione-import.md) è **atterrato** in v0.8.20 (#302 cestino + #303 warning import); resta fuori solo l'opzione 3 (vedi §2).

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
- 📋 → v2.0: **MCP integration** `pap_test_prompt(promptId)` come tool MCP per agenti — richiede MCP HTTP/SSE

### Cross-cutting / opzionali
- 🔒 → v2.0: **Server cross-compile CI matrix** Linux/Windows/macOS (sblocca con Step 0a)

---

## 5. Tecnici / debiti minori — non legati a uno stream specifico

| Item | Marker | Note |
|---|---|---|
| **golangci-lint sul server / CLI bloccato a go1.24** | 🔧 | L'action `golangci/golangci-lint-action` non linta moduli che targettano go1.25 finché il suo binario non è buildato con go1.25. Stesso blocco di #334. Verificare con Dependabot #339 (action 6→9). Sul server è oggi sostituito con `go vet` (PR #17). |
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
- ✅ **v0.8.20** cestino prompt (#302) + warning cancellazione import (#303) + colorazione globali/import parametrizzati (#353/#304) + manutenzione dipendenze

---

## Come mantenere questo documento

1. **Quando rinvii qualcosa in una PR**: aggiungi una riga nella sezione appropriata (v1.x §1, patch line §2, Enterprise §4, tecnici §5) con marker motivazionale (🔒/🔧/📋/🎨/⛔/✏️) + marker destinazione + link al doc.
2. **Quando un item atterra**: spostalo in §"Cronologia". Quando la sezione cresce troppo, vecchi item vanno in `CHANGELOG.md` e qui si rimuovono.
3. **Verifica trimestrale**: rileggere tutto, vedere se item 🔒/🔧 sono ancora bloccati dalla stessa cosa o se la situazione è cambiata.
4. **Quando si apre un branch feature** (blueprint §3 o `feat/v2.0-enterprise`): spuntare gli item che entrano nel branch.
