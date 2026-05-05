# Cluster — Roadmap

Documentazione di **pianificazione**: fasi del progetto, item rinviati, quality gate, lessons learned.

## Visione complessiva

Il progetto attraversa **5 fasi**:

| Fase | Tag deliverable | Tema | Stato |
|---|---|---|---|
| 1 | `v0.1.0-fase1` | MVP — client desktop standalone | ✅ chiusa (2026-05-03) |
| 2 | `v0.2.0-foundations` | Foundations & Distribuzione (AGPL, MCP, CLI, versioning, audit, import/export) | ✅ chiusa parziale (2026-05-04, 6/8 step controllabili) |
| 3 | `v0.3.0` | Intelligenza & Authoring (embeddings, ricerca semantica, linting, cartelle, prompt componibili) | ⏳ in preparazione (3 spike chiusi, quick wins anticipati in `v0.2.1`) |
| 4 | `v0.4.0` | Workflow Avanzati & Quality Assurance | 📋 pianificata |
| 5 | `v1.0.0` | Ecosistema Enterprise (SSO, E2E encryption, web app, browser extension) | 📋 pianificata, domanda-driven |

## Documenti

| Doc | Descrizione |
|---|---|
| [`fase-1-mvp.md`](./fase-1-mvp.md) | Step 0-15 della Fase 1 (chiusa) |
| [`fase-2-foundations.md`](./fase-2-foundations.md) | Step 1-10 della Fase 2 (chiusa parziale: Step 5 → patch line `v0.2.x`, Step 6 → Fase 5) |
| [`fase-3-intelligence.md`](./fase-3-intelligence.md) | Step 1-11 della Fase 3 (in preparazione) |
| [`fase-4-workflow.md`](./fase-4-workflow.md) | Step della Fase 4 |
| [`fase-5-enterprise.md`](./fase-5-enterprise.md) | Step della Fase 5 |
| [`rinvii.md`](./rinvii.md) | **Censimento unificato di tutto ciò che è stato deliberatamente rinviato.** Singola fonte di verità: aggiornato ad ogni PR che rinvia qualcosa, item spostati in cronologia quando atterrano |
| [`quality-gate-fase-2.md`](./quality-gate-fase-2.md) | Assessment dei criteri di qualità per il tag `v0.2.0-foundations`: coverage per modulo, audit security, verifica licenza |
| [`lessons-learned-fase-1.md`](./lessons-learned-fase-1.md) | Lezioni apprese durante l'implementazione della Fase 1, utili per le fasi successive |

## Convenzioni

- Ogni fase ha **Step numerati** con TODO list e criteri di completamento.
- Lo stato è marcato con `[x]` quando lo Step è chiuso, `[ ]` quando aperto, strikethrough per item spostati altrove.
- `rinvii.md` è il **glue** che tiene traccia degli item rinviati: ogni rinvio viene registrato lì con un marker (🔒 esterno / 🔧 tecnico / 📋 sub-step / 🎨 polish).

## Aspetti correlati in altri cluster

- **Decisioni architetturali che condizionano la roadmap**: [`../architettura/decisioni/`](../architettura/decisioni/)
- **Doc utente di feature già atterrate**: [`../utente/`](../utente/)
