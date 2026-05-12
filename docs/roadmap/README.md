# Cluster — Roadmap

Documentazione di **pianificazione**: timeline dei rilasci, fasi del progetto, item rinviati, quality gate, lessons learned.

## Distinzione fra "fase", "rilascio" e "SKU"

- **Fase** = macro-tema *per capability*. 5 in totale, in ordine progressivo.
- **Rilascio (release)** = unità di shipping con un tag git. Può chiudere una fase, parzialmente una fase, essere un patch line, oppure un rilascio speciale trasversale (recupero ritardi, polish UI).
- **SKU** = linea di prodotto distinta. PaP ha 2 SKU pianificati:
  - **Personale** → `v1.x` (single user, local-first). Scope chiuso in [`v1.0-personale.md`](./v1.0-personale.md).
  - **Enterprise** → `v2.x` (multi-user, server). Gate domanda-driven, scope in [`v2.0-enterprise.md`](./v2.0-enterprise.md).

Una fase può ospitare 0..N rilasci; un rilascio può attraversare 0..N fasi. Le fasi sono ortogonali agli SKU: Fasi 1-4 sono confluite nello SKU Personale; Fase 5 corrisponde allo SKU Enterprise.

La **fonte autorevole della pianificazione** è [`release-plan.md`](./release-plan.md): un'unica timeline che mappa tag → fase/tema/SKU → stato.

## Documenti

| Doc | Descrizione |
|---|---|
| [`release-plan.md`](./release-plan.md) | **Timeline completa di tutti i rilasci** (v0.1 → v2.0+). Fonte autorevole della pianificazione. Include strategia SKU v1.0 Personale / v2.0 Enterprise |
| [`v1.0-personale.md`](./v1.0-personale.md) | **Scope chiuso PaP Personale v1.0.0** — must-have M1-M8 (auto-update, a11y, sub-step Fase 4, import scopati, doppia vista editor, Markdown, coverage, docs), quality gate, timeline indicativa, strategia branching |
| [`v2.0-enterprise.md`](./v2.0-enterprise.md) | **Scope SKU PaP Enterprise v2.0.0** — gating domanda-driven, prerequisiti apertura branch, stream design parallelo a v1.0, strategia multi-SKU post-v2.0 |
| [`fase-1-mvp.md`](./fase-1-mvp.md) | Step 0-15 della Fase 1 (chiusa) — MVP client desktop standalone |
| [`fase-2-foundations.md`](./fase-2-foundations.md) | Step 1-10 della Fase 2 — Foundations & Distribuzione (chiusa parziale: Step 5 → patch line `v0.2.x`/M1 v1.0, Step 6 → Fase 5/v2.0) |
| [`fase-3-intelligence.md`](./fase-3-intelligence.md) | Step 1-11 della Fase 3 — Intelligenza & Authoring (✅ chiusa, tag `v0.3.0` rilasciato 2026-05-06) |
| [`fase-4-workflow.md`](./fase-4-workflow.md) | Step della Fase 4 — Workflow Avanzati & Quality Assurance (✅ client-first chiusa, tag `v0.4.0` 2026-05-07. Sub-step UI residui → v1.0 M3) |
| [`fase-5-enterprise.md`](./fase-5-enterprise.md) | Step della Fase 5 — dettaglio tecnico Step 0a-8 per `v2.0.0` (PaP Enterprise) |
| [`rinvii.md`](./rinvii.md) | **Censimento unificato di tutto ciò che è stato deliberatamente rinviato.** Singola fonte di verità degli item rinviati; classificati per stream (v1.0 / v2.0 / patch line `v0.2.x` / scartato) |
| [`SAL_2026_05_17.MD`](./SAL_2026_05_17.MD) | Stato Avanzamento Lavori snapshot 2026-05-17 — recap todo/bloccanti/rinvii con motivazione |
| [`website/`](./website/README.md) | **Thread parallelo — Landing page PaP**. Sottocartella con 6 doc per agente parallelo: handoff Claude Design (contenuti IT), stack Astro, hosting GitHub Pages, analytics Matomo self-hosted su Gigantto, roadmap fasi |
| [`quality-gate-fase-2.md`](./quality-gate-fase-2.md) | Assessment dei criteri di qualità per il tag `v0.2.0-foundations`: coverage per modulo, audit security, verifica licenza |
| [`lessons-learned-fase-1.md`](./lessons-learned-fase-1.md) | Lezioni apprese durante l'implementazione della Fase 1, utili per le fasi successive |

## Convenzioni

- Ogni **fase** ha Step numerati con TODO list e criteri di completamento; stato `[x]` chiuso / `[ ]` aperto / strikethrough per item spostati.
- I **rilasci** vivono in `release-plan.md`. Quelli speciali (`v0.5.0`, `v0.6.0`) hanno una loro sezione dedicata lì dentro.
- `rinvii.md` è il **glue** del debt tracking: ogni rinvio registrato lì con marker (🔒 esterno / 🔧 tecnico / 📋 sub-step / 🎨 polish). Il pool 📋/🎨 è naturalmente il candidate pool per `v0.5.0`.

## Aspetti correlati in altri cluster

- **Decisioni architetturali che condizionano la roadmap**: [`../architettura/decisioni/`](../architettura/decisioni/)
- **Doc utente di feature già atterrate**: [`../utente/`](../utente/)
