# Release plan — Timeline completa

> **Fonte autorevole** della pianificazione di rilascio. Una sola tabella per tutta la vita del progetto, dalla v0.1 alla v1.0+. Aggiornato a ogni tag o decisione di rinvio.
>
> **Aggiornato al**: 2026-05-05.

## Concetti

Distinguiamo deliberatamente:

- **Fase** = macro-tema *per capability*. Stabilisce cosa il prodotto sarà capace di fare al termine. Ne abbiamo 5 (`fase-1-mvp.md` … `fase-5-enterprise.md`). Le fasi si succedono in ordine.
- **Rilascio (release)** = unità di shipping con un tag git. Un rilascio può:
  - chiudere una fase (es. `v0.1.0-fase1` chiude Fase 1);
  - chiudere parzialmente una fase con sblocchi rinviati (es. `v0.2.0-foundations` = 6/8 step di Fase 2);
  - essere un **patch line** (es. `v0.2.x` = fix + step deferred di Fase 2);
  - essere un **rilascio speciale** che non chiude alcuna fase ma paga debito o fa polish trasversale (es. `v0.5.0`, `v0.6.0` previsti).

Una fase può ospitare 0..N rilasci; un rilascio può attraversare 0..N fasi.

## Timeline

| Tag | Data | Tipo | Tema | Stato | Riferimento |
|---|---|---|---|---|---|
| `v0.1.0-fase1` | 2026-05-03 | fase | MVP — client desktop standalone (15 step, 71 test) | ✅ rilasciato | [`fase-1-mvp.md`](./fase-1-mvp.md) |
| `v0.2.0-foundations` | 2026-05-04 | fase (parziale) | Foundations & Distribuzione — AGPL 3.0, MCP, CLI, versioning, audit, import/export. Step 5 → patch line, Step 6 → Fase 5 | ✅ rilasciato (6/8 step controllabili) | [`fase-2-foundations.md`](./fase-2-foundations.md) |
| `v0.2.1` | 2026-05-05 | patch + quick wins | Quick wins anticipati di Fase 3 (modello target, Insight, cartelle) + portable Windows agli asset | ✅ rilasciato | [`fase-2-foundations.md`](./fase-2-foundations.md) (patch line) |
| `v0.2.1-fix1` | 2026-05-05 | patch fix | Bug 1 vault loop portable + bug 2 parziale tray icon | ✅ prerelease | (fix branch) |
| `v0.2.x` | TBD | patch line | **Step 5 — Auto-update silenzioso** (NSIS per-user, Tauri Updater, Authenticode signing) | 🔒 bloccato cert Certum (KYC in corso) | [`fase-2-foundations.md`](./fase-2-foundations.md) Step 5 deferred |
| `v0.3.0` | TBD | fase | Intelligenza & Authoring — embeddings ONNX locali, ricerca ibrida FTS+vettoriale, linting, prompt componibili `{{import}}` | ⏳ in preparazione (3 spike chiusi) | [`fase-3-intelligence.md`](./fase-3-intelligence.md) |
| `v0.4.0` | TBD | fase | Workflow Avanzati & Quality Assurance — varianti, approval workflow, ACL cartelle, regression testing | 📋 pianificata | [`fase-4-workflow.md`](./fase-4-workflow.md) |
| **`v0.5.0`** | TBD | **rilascio speciale** | **Recupero ritardi** — paga il debito accumulato in v0.1-v0.4 (item da `rinvii.md`). Stabilizzazione pre-1.0 | 📋 pianificata | sezione [Rilascio v0.5.0](#rilascio-v050--recupero-ritardi) |
| **`v0.6.0`** | TBD | **rilascio speciale** | **Pulizia interfaccia grafica** — pass cosmetico cross-cutting su tutte le superfici, niente nuove capability | 📋 pianificata | sezione [Rilascio v0.6.0](#rilascio-v060--pulizia-interfaccia-grafica) |
| `v1.0.0` | TBD | fase | Ecosistema Enterprise — SSO, E2E encryption, server cross-OS senza Docker, web app, browser extension. Uscita beta | 📋 pianificata, domanda-driven | [`fase-5-enterprise.md`](./fase-5-enterprise.md) |

---

## Rilascio v0.5.0 — Recupero ritardi

> **Tipo**: rilascio speciale, non lega a una singola fase.
> **Scope**: pagamento del debito accumulato durante Fase 1-4. Niente nuove feature, solo *graduazione* degli item che fino a quel momento sono in `rinvii.md` con marker 📋 (sub-step) o 🎨 (polish).
> **Quando**: dopo Fase 4 (`v0.4.0`), prima di Fase 5. È una porta di stabilizzazione.

### Criteri di ingresso

Un item entra in v0.5.0 se soddisfa **tutte** queste condizioni:
1. È in [`rinvii.md`](./rinvii.md) con marker 📋 o 🎨 (no 🔒 esterni che restano bloccati).
2. Il blocker tecnico/di dipendenza è caduto (es. è arrivata la fase prerequisita).
3. Vale la pena pagarlo prima di v1.0.0 (cioè ha utilizzo reale, non è speculativo).

Item con marker 🔒 (cert Certum, Apple Developer) restano nel patch line `v0.2.x` per quando atterrano i rispettivi sblocchi — non vengono spostati a v0.5.

### Pool di candidati

Il pool è costruito dinamicamente da `rinvii.md`. Al momento dell'apertura del branch `feat/v0.5-recupero` si fa un audit completo e si selezionano i candidati. Esempi probabili (da rinvii.md attuale):

- **Sub-step di feature già atterrate** (📋): markdown import/export, custom free-text target model, statistiche per cartella, MCP HTTP/SSE, MCP `pap_create_draft` (se Fase 4 ha portato approval workflow), CLI `import`/`export`, esporta singola cartella.
- **Coverage gap** chiusi prima di v1.0.0: TS client `vitest --coverage`, server 70%, MCP unit test minimi, Rust client report numerico CI.
- **Cosmetic / debiti tecnici minori** (🎨): bumpare versione `Cargo.toml`/`tauri.conf.json` (oggi ferma a 0.1.0), riattivare golangci-lint quando v2 stabile.

### Quality gate v0.5.0

Definito in fase di apertura branch. Bozza:
- [ ] Tutti i 📋/🎨 di `rinvii.md` valutati: ognuno è "atterrato" o "rinviato a 1.x" con razionale documentato
- [ ] Coverage TS client ≥ 70% su `lib/*.ts`
- [ ] Coverage server ≥ 70%
- [ ] MCP server con almeno 5 unit test
- [ ] `Cargo.toml`/`tauri.conf.json` allineati al tag

---

## Rilascio v0.6.0 — Pulizia interfaccia grafica

> **Tipo**: rilascio speciale, cross-cutting cosmetico.
> **Scope**: pass cosmetico su tutte le superfici del client desktop. Niente nuove capability funzionali, niente schema migration, niente Tauri command nuovi. Solo CSS, layout, micro-interaction, copy.
> **Quando**: dopo `v0.5.0` (recupero), prima di `v1.0.0`. Funge da "rifinitura pre-uscita beta".

### Scope tipico

- **Coerenza visiva**: ricognizione su tutte le superfici (`Libreria`, `EditorPrompt`, `CompilatorePrompt`, `CommandPalette`, `Impostazioni`, `Insight`, `OnboardingWizard`, etc.) per uniformare spaziature, colori accent, transizioni.
- **Stati di feedback**: empty state, loading, errore, success — uniformare presentazione e copy.
- **Micro-interaction**: hover, focus, active, drag, drop, animazioni di entrata/uscita modali.
- **Dark / light theme parity**: verifica che tutti i componenti rispettino entrambi i temi.
- **Accessibilità**: focus visibili, contrasti, semantica HTML, label associate (oggi ci sono ~30 warning a11y noti da `svelte-check`).
- **Responsive minore**: comportamento window resize tra 800x600 e 1920x1080.
- **Copy editing**: stringhe italiane uniformate (terminologia, capitalizzazione, vs/vs.).

### Out of scope per v0.6.0

- Nuove feature funzionali → restano nelle fasi
- Refactor architetturale → restano in `rinvii.md`
- Localizzazione (i18n EN/altre) → eventualmente v0.7+ o Fase 5
- Branding / logo redesign → fuori scope

### Backlog

Si costruisce mano mano fra v0.4.0 e v0.6.0 raccogliendo feedback durante l'uso. Riserviamo qui una sezione che cresce nel tempo:

- [ ] (placeholder — popolare con click-path-audit prima dell'apertura branch)
- [ ] Risolvere il residuo doppia tray icon Windows (vedi memoria `tray_icon_doppia_windows.md`)
- [ ] Pulire i ~37 warning a11y di `svelte-check` (autofocus, label senza control, role/aria mismatch)
- [ ] Verificare coerenza spaziature dei `meta-sezione` nell'editor

### Quality gate v0.6.0

Bozza:
- [ ] Tutte le superfici verificate manualmente in dark + light
- [ ] svelte-check warning a11y < 10 (oggi 37)
- [ ] Nessuna regressione funzionale rispetto a v0.5 (smoke test E2E)

---

## Manutenzione di questo documento

- **Quando taggi un release**: aggiungi/aggiorna la riga in tabella, sposta lo stato a ✅, popola la data.
- **Quando rinvii qualcosa**: registra in [`rinvii.md`](./rinvii.md). Se ha senso candidarlo a v0.5.0, aggiungilo nel pool sopra.
- **Quando emerge un'idea cosmetica**: aggiungila al backlog v0.6.0.
- **Quando una fase si sposta**: aggiorna la riga corrispondente in tabella + il doc fase.
