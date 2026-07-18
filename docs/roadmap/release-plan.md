# Release plan — Timeline completa

> **Fonte autorevole** della pianificazione di rilascio. Una sola tabella per tutta la vita del progetto, dalla v0.1 alla v2.0+. Aggiornato a ogni tag o decisione di rinvio.
>
> **Aggiornato al**: 2026-05-12 (post v0.8.8 hotfix + adozione strategia SKU v1.0/v2.0).

## Concetti

Distinguiamo deliberatamente:

- **Fase** = macro-tema *per capability*. Stabilisce cosa il prodotto sarà capace di fare al termine. Ne abbiamo 5 (`fase-1-mvp.md` … `fase-5-enterprise.md`). Le fasi si succedono in ordine.
- **Rilascio (release)** = unità di shipping con un tag git. Un rilascio può:
  - chiudere una fase (es. `v0.1.0-fase1` chiude Fase 1);
  - chiudere parzialmente una fase con sblocchi rinviati (es. `v0.2.0-foundations` = 6/8 step di Fase 2);
  - essere un **patch line** (es. `v0.2.x` = fix + step deferred di Fase 2);
  - essere un **rilascio speciale** che non chiude alcuna fase ma paga debito o fa polish trasversale (es. `v0.5.0`, `v0.6.0` già rilasciati).
- **SKU** = linea di prodotto distinta con audience e gating diverso. PaP ha 2 SKU pianificati:
  - **PaP Personale** → target `v1.0.0` GA. Single-user, local-first. Scope chiuso in [`v1.0-personale.md`](./v1.0-personale.md).
  - **PaP Enterprise** → target `v2.0.0` GA. Multi-user, server-driven. **Gate domanda-driven**, nessuna data finché non c'è cliente reale. Scope in [`v2.0-enterprise.md`](./v2.0-enterprise.md) + dettaglio tecnico in [`fase-5-enterprise.md`](./fase-5-enterprise.md).

Una fase può ospitare 0..N rilasci; un rilascio può attraversare 0..N fasi.

## Strategia SKU v1.0 / v2.0

Adottata il 2026-05-12. PaP si separa in due linee di rilascio:

| Linea | Scope | Status oggi | Prossimo tag |
|---|---|---|---|
| **Personale (v1.x)** | Recupero rinvii Fase 1-4 + auto-update + polish UX/docs/coverage | 90% completo (v0.8.8 Latest) | `v0.9.x` patch line → `v1.0.0` GA |
| **Enterprise (v2.x)** | Fase 5 Step 0a-8 (server cross-OS, SSO, E2E, web, extension, etc.) | Non avviato (gate domanda-driven) | Nessuno finché non c'è cliente reale; consentito stream design parallelo |

**Sequenzialità macro**: v1.0 prima, v2.0 dopo. Vietato parallelo "due agenti su codice di produzione" prima di v1.0 chiusa (rischio sovra-ingegnerizzazione + conflitti su file condivisi come `lib.rs`, `ImpostazioniModal.svelte`, migration numbering).

**Parallelismo permesso**:
- *Dentro* v1.0: agenti su layer disgiunti (UI / auto-update / docs / coverage). Vedi [`v1.0-personale.md`](./v1.0-personale.md) §"Strategia di branching".
- *Stream design v2.0*: un agente può produrre design doc, ADR, spike (non in `apps/`) mentre v1.0 procede. Vedi [`v2.0-enterprise.md`](./v2.0-enterprise.md) §"Stream design parallelo a v1.0".

**Post-v2.0 GA**: due binari (`pap` personal + `pap-enterprise`) da single codebase con feature flag Cargo + dynamic import Svelte. CI build matrix raddoppiata. Rilasci di pari passo (stesso giorno entrambi i binari).

## Thread parallelo — Landing page (website)

Indipendente dai due stream prodotto. Asset di marketing/onboarding utenti:

- **Scope**: landing page IT su GitHub Pages (Astro statico), analytics Matomo self-hosted, no costo continuativo.
- **Sequenzialità rispetto a v1.0/v2.0**: nessuna, può partire in qualunque momento. Suggerito parallelo a M2-M3 di v1.0 così è pronta a v1.0.0 GA.
- **Assegnazione**: agente parallelo dedicato (TBD), distinto dall'agente principale. Vedi [`website/archivio/README.md`](./website/archivio/README.md).
- **Fasi**: 1 setup tecnico → 2 handoff Claude Design → 3 sviluppo → 4 analytics → 5 lancio. Effort effettivo ~6-10 giorni in 3-4 settimane.

## Timeline

| Tag | Data | Tipo | Tema | Stato | Riferimento |
|---|---|---|---|---|---|
| `v0.1.0-fase1` | 2026-05-03 | fase | MVP — client desktop standalone (15 step, 71 test) | ✅ rilasciato | [`fase-1-mvp.md`](./fase-1-mvp.md) |
| `v0.2.0-foundations` | 2026-05-04 | fase (parziale) | Foundations & Distribuzione — AGPL 3.0, MCP, CLI, versioning, audit, import/export. Step 5 → patch line, Step 6 → Fase 5 | ✅ rilasciato (6/8 step controllabili) | [`fase-2-foundations.md`](./fase-2-foundations.md) |
| `v0.2.1` | 2026-05-05 | patch + quick wins | Quick wins anticipati di Fase 3 (modello target, Insight, cartelle) + portable Windows agli asset | ✅ rilasciato | [`fase-2-foundations.md`](./fase-2-foundations.md) (patch line) |
| `v0.2.1-fix1` | 2026-05-05 | patch fix | Bug 1 vault loop portable + bug 2 parziale tray icon | ✅ prerelease | (fix branch) |
| `v0.2.2` / `v0.2.3` | imminente | patch line | **Step 5 — Auto-update silenzioso** (NSIS per-user, Tauri Updater, Authenticode signing) | 🚧 cert Certum SimplySign Cloud arrivato 2026-05-15, M1 avviabile | [`v1.0-personale.md`](./v1.0-personale.md) M1 (sub-PR M1.1-M1.7) |
| `v0.3.0` | 2026-05-06 | fase | Intelligenza & Authoring — embeddings ONNX locali, ricerca ibrida FTS+vettoriale, linting, cartelle, prompt componibili `{{import}}`, statistiche, idle-unload | ✅ rilasciato (11/11 step) | [`fase-3-intelligence.md`](./fase-3-intelligence.md) |
| `v0.4.0` | 2026-05-07 | fase (parziale) | Workflow Avanzati & Quality Assurance — golden+regression, varianti, diff, confronto, fork, rating. Step 6+7 → Fase 5/v2.0 | ✅ rilasciato (6/8 step client-first) | [`fase-4-workflow.md`](./fase-4-workflow.md) |
| `v0.5.0` | 2026-05-07 | rilascio speciale | Quick wins UX (6 PR) + 5° provider AI Gemini + 351 test backend | ✅ rilasciato | memoria `sprint_v05_chiuso.md` |
| `v0.6.0` | 2026-05-07 | rilascio speciale | Hardening + quick wins (6 PR) + 382 test backend, coverage 71% | ✅ rilasciato | memoria `sprint_v06_chiuso.md` |
| `v0.7.0` | 2026-05-08 | rilascio speciale | Refactor coverage + import/cartelle quick wins (6 PR) + 416 test, coverage 74% | ✅ rilasciato | memoria `sprint_v07_chiuso.md` |
| `v0.8.0` | 2026-05-09 | redesign | Redesign UI completo F0-F11 (17 PR) — sezione Modale, Onboarding, Shell, a11y WCAG 2.1 AA | ✅ rilasciato | memoria `release_v080.md` |
| `v0.8.1`–`v0.8.4` | 2026-05-09/10 | patch line | Bugfix Win11 multi-issue + retry release CI | ✅ rilasciati | CHANGELOG |
| `v0.8.5` | 2026-05-10 | patch + feature | Editor UX + tray fix + segnaposti globali `{{global nome}}` | ✅ rilasciato (Draft GitHub) | memoria `sprint_v085_chiuso.md` |
| `v0.8.6` | 2026-05-10 | ⚠️ DIFETTOSA | Fix data-loss switch prompt + hardening Go 1.25.10 + chi v5.2.5 — **regressione editor input** | ⚠️ Draft, banner di warning | release notes |
| `v0.8.7` | 2026-05-10 | ⚠️ DIFETTOSA | Sezione Sviluppo + Debug log Telescope-like — **regressione editor input ereditata** | ⚠️ Draft, banner di warning | memoria `sprint_v087_chiuso.md` |
| `v0.8.8` | 2026-05-11 | hotfix | Fix #170 editor input bloccato (`untrack()` su `$effect`) | ✅ rilasciato + Latest published | CHANGELOG v0.8.8 |
| `v0.9.x` | TBD | patch line | **Recupero finale verso v1.0** — pulizia a11y, sub-step Fase 4, Markdown import, import scopati, doppia vista editor, docs utente, coverage 80%/70% | 🚧 pianificata | [`v1.0-personale.md`](./v1.0-personale.md) M1-M8 |
| **`v1.0.0`** | TBD (~mese 3) | **fase + SKU GA** | **PaP Personale GA** — uscita beta lato utente individuale. Quality gate completo. | 📋 pianificata | [`v1.0-personale.md`](./v1.0-personale.md) |
| `v1.x.x` | post-1.0 | patch line | Patch + minor feature PaP Personale. Vive su `main` (single codebase con `v2.x`) | 📋 futuro | TBD |
| **`v2.0.0`** | TBD (domanda-driven) | **fase + SKU GA** | **PaP Enterprise GA** — Server cross-OS + SSO + RBAC + Webhook + API pubblica. Gate cliente reale | 🟡 in attesa di domanda | [`v2.0-enterprise.md`](./v2.0-enterprise.md) + [`fase-5-enterprise.md`](./fase-5-enterprise.md) |
| `v2.x.x` | post-2.0 | patch line | Patch + minor feature PaP Enterprise | 📋 futuro | TBD |

---

## Rilasci speciali archiviati

Le sezioni v0.5.0 (recupero ritardi) e v0.6.0 (pulizia UI) qui originariamente pianificate sono state shippate con scope diverso da quello previsto:

- **v0.5.0** (2026-05-07) → quick wins UX + 5° provider AI Gemini (6 PR). Vedi memoria `sprint_v05_chiuso.md`.
- **v0.6.0** (2026-05-07) → hardening backend + quick wins. Vedi memoria `sprint_v06_chiuso.md`.
- **v0.7.0** (2026-05-08) → refactor coverage + import/cartelle quick wins. Vedi memoria `sprint_v07_chiuso.md`.
- **v0.8.0** (2026-05-09) → redesign UI completo F0-F11 (17 PR) — la pulizia UI che era prevista in v0.6.0 è stata fatta qui, in forma più ambiziosa.

Il **pool di item residui** (sub-step Fase 4, Markdown import, sintassi import scopati, editor doppia vista, coverage gap, warning a11y) è confluito nel piano **PaP Personale v1.0** documentato in [`v1.0-personale.md`](./v1.0-personale.md) M1-M8.

---

## Manutenzione di questo documento

- **Quando taggi un release**: aggiungi/aggiorna la riga in tabella, sposta lo stato a ✅, popola la data.
- **Quando rinvii qualcosa**: registra in [`rinvii.md`](./rinvii.md). Se ha senso candidarlo a v0.5.0, aggiungilo nel pool sopra.
- **Quando emerge un'idea cosmetica**: aggiungila al backlog v0.6.0.
- **Quando una fase si sposta**: aggiorna la riga corrispondente in tabella + il doc fase.
