# Cluster — Roadmap

Documentazione di **pianificazione**: timeline dei rilasci, fasi del progetto, item rinviati, quality gate, lessons learned.

## Distinzione fra "fase" e "rilascio"

- **Fase** = macro-tema *per capability*. 5 in totale, in ordine progressivo.
- **Rilascio (release)** = unità di shipping con un tag git. Può chiudere una fase, parzialmente una fase, essere un patch line, oppure un rilascio speciale trasversale (recupero ritardi, polish UI).

Una fase può ospitare 0..N rilasci; un rilascio può attraversare 0..N fasi.

La **fonte autorevole della pianificazione** è [`release-plan.md`](./release-plan.md): un'unica timeline che mappa tag → fase/tema → stato.

## Documenti

| Doc | Descrizione |
|---|---|
| [`release-plan.md`](./release-plan.md) | **Timeline completa di tutti i rilasci** (v0.1 → v1.0+), incluso piano dei rilasci speciali `v0.5.0` (recupero ritardi) e `v0.6.0` (pulizia UI). Fonte autorevole della pianificazione |
| [`fase-1-mvp.md`](./fase-1-mvp.md) | Step 0-15 della Fase 1 (chiusa) — MVP client desktop standalone |
| [`fase-2-foundations.md`](./fase-2-foundations.md) | Step 1-10 della Fase 2 — Foundations & Distribuzione (chiusa parziale: Step 5 → patch line `v0.2.x`, Step 6 → Fase 5) |
| [`fase-3-intelligence.md`](./fase-3-intelligence.md) | Step 1-11 della Fase 3 — Intelligenza & Authoring (✅ chiusa, tag `v0.3.0` rilasciato 2026-05-06) |
| [`fase-4-workflow.md`](./fase-4-workflow.md) | Step della Fase 4 — Workflow Avanzati & Quality Assurance (🚧 client-first chiusa 6/8 step, Step 6+7 → Fase 5, tag `v0.4.0` in arrivo) |
| [`fase-5-enterprise.md`](./fase-5-enterprise.md) | Step della Fase 5 — Ecosistema Enterprise → `v1.0.0` |
| [`rinvii.md`](./rinvii.md) | **Censimento unificato di tutto ciò che è stato deliberatamente rinviato.** Singola fonte di verità degli item rinviati; pool di candidati per `v0.5.0` |
| [`quality-gate-fase-2.md`](./quality-gate-fase-2.md) | Assessment dei criteri di qualità per il tag `v0.2.0-foundations`: coverage per modulo, audit security, verifica licenza |
| [`lessons-learned-fase-1.md`](./lessons-learned-fase-1.md) | Lezioni apprese durante l'implementazione della Fase 1, utili per le fasi successive |

## Convenzioni

- Ogni **fase** ha Step numerati con TODO list e criteri di completamento; stato `[x]` chiuso / `[ ]` aperto / strikethrough per item spostati.
- I **rilasci** vivono in `release-plan.md`. Quelli speciali (`v0.5.0`, `v0.6.0`) hanno una loro sezione dedicata lì dentro.
- `rinvii.md` è il **glue** del debt tracking: ogni rinvio registrato lì con marker (🔒 esterno / 🔧 tecnico / 📋 sub-step / 🎨 polish). Il pool 📋/🎨 è naturalmente il candidate pool per `v0.5.0`.

## Aspetti correlati in altri cluster

- **Decisioni architetturali che condizionano la roadmap**: [`../architettura/decisioni/`](../architettura/decisioni/)
- **Doc utente di feature già atterrate**: [`../utente/`](../utente/)
