# Roadmap dei rinvii

> Censimento unificato di **tutto ciò che è stato deliberatamente rinviato** durante lo sviluppo. Questo documento è la singola fonte di verità: nuovi rinvii vengono aggiunti qui ad ogni PR che li introduce, e gli item vengono rimossi/spuntati quando atterrano.
>
> **Stato**: aggiornato al 2026-05-05 dopo `v0.2.1` + prerelease `v0.2.1-fix1`.
>
> **Convenzioni**:
> - 🔒 = bloccato da fattore esterno (cert, KYC, hardware, decisione di prodotto)
> - 🔧 = bloccato da fattore tecnico (libreria instabile, dipendenza non pronta)
> - 📋 = sub-step già pianificato di una feature, da fare quando il padre arriva
> - 🎨 = polish / cosmetico, non bloccante per nessuno
> - ✅ = item che è atterrato ed è qui solo per archivio storico

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
| **Versione in `Cargo.toml` + `tauri.conf.json` ferma a 0.1.0** → asset di release riportano `0.1.0` nei nomi anche su tag v0.2.x. Bumpare manualmente al prossimo PR feature. | 🎨 |
| **golangci-lint reattivare sul server** dopo che l'action `golangci/golangci-lint-action@v6+` supporta stabilmente v2.x. Oggi è stato sostituito con `go vet` (PR #17) perché v1.64 incompatibile con Go 1.25. | 🔧 |
| **CI Rust client coverage report numerico** via `cargo-llvm-cov` o `tarpaulin`. Oggi solo pass/fail. | 🎨 |

---

## 4. Coverage e quality gate (action item per `v0.3` quality gate)

| Item | Soglia target | Stato oggi |
|---|---|---|
| Coverage TS client `vitest --coverage` su `lib/*.ts` | 70% | non configurata |
| Coverage server **alzare da 50% → 70%** con unit test mirati su `database`/`middleware`/`models`/`sync`/`ws` | 70% | 56.2% via test integrazione cross-package |
| Test unit MCP server (almeno `sanitizzaFts`, `compila`, `estraiSegnaposti`) | qualunque >0 | placeholder `echo "No tests yet"` |
| Coverage Rust client report numerico in CI | report visibile | 56 test pass/fail, no %. |

---

## 5. Fase 3 — prerequisiti tecnici

| Item | Stato | Azione |
|---|---|---|
| **`ort` 2.x compile fix su Rust stable** | 🔧 | rc.9/.10/.12 tutti rotti (mismatch ort/ort-sys). Verificare crates.io all'inizio di Fase 3 Step 1, scegliere prima rc che compila pulito. |
| Fallback `candle-core` se `ort` resta instabile a lungo | 🔧 | Piano B documentato in `docs/architettura/decisioni/onnx-bundle.md`. ~2-5 MB bundle invece di 14-22 MB, performance leggermente inferiore. |

---

## 6. Sub-step rinviati a Fase 3 piena

Pezzi di feature già parzialmente atterrate, con il resto programmato per la fase di destinazione.

### Da Step 4 — Import/export
- 📋 **Markdown import/export** (oggi solo JSON con schema v1) — sub-step di Step 4

### Da Step 6 — Modello target
- 📋 **Custom free-text target model** (oggi solo i 9 preset) — polish UX
- 📋 **Server endpoint `/search?target=...`** — quando workspace team va in produzione (anche è in Fase 5)

### Da Step 7 — Cartelle
- 📋 **Esporta singola cartella** (oggi solo intero vault) — sub-step di Step 7
- 📋 **Server endpoint `/sync/folders`** — Fase 5 Step 0a

### Da Step 7 di Fase 2 — MCP server
- 📋 **Trasporto HTTP/SSE** (oggi solo stdio) — sub-step di Step 7 Fase 2
- 📋 **Tool `pap_create_draft`** — richiede approval workflow di Fase 4

### Da Step 8 di Fase 2 — CLI `pap`
- 📋 **Comandi `login` / `new` / `import` / `export`** — richiedono server API o IPC client desktop

### Da Step 9 — Statistiche
- 📋 **Prompt più importati** — richiede Step 8 di Fase 3 (import resolver)
- 📋 **Distribuzione per cartella** — Step 7 ora atterrato in v0.2.1, si può aggiungere subito in `v0.2.2` o all'inizio di Fase 3
- 📋 **Distribuzione per autore** — richiede multi-user (workspace team in produzione)
- 📋 **Lint health % senza issues + top categorie** — richiede Step 5 di Fase 3 (linting)

---

## 7. Fase 5 — enterprise, domanda-driven

| Item | Origine |
|---|---|
| **Step 0a — Server Go cross-OS senza Docker** (pure-Go SQLite, single binary, Windows Service nativo + systemd unit, .deb/.rpm/NSIS server) | spostato da Fase 2 Step 6 |
| Sync server in produzione | gate per Fase 5 + tutti i feature server-side multi-user |
| `/sync/folders` endpoint | da Step 7 cartelle |
| `?target=` filter su endpoint search | da Step 6 |
| Distribuzione statistiche per autore | da Step 9 |

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
| ✅ Spike modello embedding IT/EN | PR #22 | `paraphrase-multilingual-MiniLM-L12-v2` |
| ✅ Riposizionamento Step 5 (→ patch line) e Step 6 (→ Fase 5) | PR #19 | docs/roadmap/fase-2-foundations.md, docs/roadmap/fase-5-enterprise.md |
| ✅ Tag `v0.2.0-foundations` (parziale 6/8 step) | manual | release stable |
| ✅ Versione portable Windows agli asset release | PR #27 + #28 | `Prompt-a-Porter-portable-windows-x64-{tag}.zip` |
| ✅ Step 6 modello target (Fase 3 anticipato in v0.2.1) | PR #23 | dropdown editor, filtro libreria, badge |
| ✅ Step 9 statistiche / Insight (Fase 3 anticipato) | PR #24 | KPI grid + bar charts SVG inline |
| ✅ Step 7 cartelle backend + UI base (Fase 3 anticipato) | PR #25 | schema V004, CRUD Tauri, sidebar tree |
| ✅ Step 7 cartelle drag&drop + filter chip + rinomina inline | PR #26 | UX completa |
| ✅ Bug 1 (vault loop portable) + bug 2 parziale (tray icon visibile) | PR #29 / `v0.2.1-fix1` | residuo: doppia icona Windows |

---

## Come mantenere questo documento

1. **Quando rinvii qualcosa in una PR**: aggiungi una riga nella sezione appropriata, con marker (🔒/🔧/📋/🎨), origine PR, link al doc di destinazione (Fase X Step Y).
2. **Quando un item atterra**: spostalo in *Cronologia PR e item chiusi*. Quando la sezione cresce troppo, vecchi item vanno in `CHANGELOG.md` e qui si rimuovono.
3. **Verifica trimestrale**: rileggere tutto, vedere se item 🔒 sono ancora bloccati dalla stessa cosa o se la situazione è cambiata.
