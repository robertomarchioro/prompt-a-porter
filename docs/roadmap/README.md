# Cluster — Roadmap

Documentazione di **pianificazione**: timeline dei rilasci, fasi del progetto, item rinviati, quality gate, lessons learned.

## Distinzione fra "fase", "rilascio" e "SKU"

- **Fase** = macro-tema *per capability*. 5 in totale, in ordine progressivo.
- **Rilascio (release)** = unità di shipping con un tag git. Può chiudere una fase, parzialmente una fase, essere un patch line, oppure un rilascio speciale trasversale (recupero ritardi, polish UI).
- **SKU** = linea di prodotto distinta. PaP ha 2 SKU pianificati:
  - **Personale** → `v1.x` (single user, local-first). Scope chiuso in [`v1.0-personale.md`](./v1.0-personale.md).
  - **Enterprise** → `v2.x` (multi-user, server). Gate domanda-driven, scope in [`v2.0-enterprise.md`](./v2.0-enterprise.md).

Una fase può ospitare 0..N rilasci; un rilascio può attraversare 0..N fasi. Le fasi sono ortogonali agli SKU: Fasi 1-4 sono confluite nello SKU Personale; Fase 5 corrisponde allo SKU Enterprise.

## Indirizzo strategico (2026-06-03)

Decisione di rotta dell'autore:

1. **Adesso: chiudere v1.0.0 Personale "senza fronzoli".** Priorità unica =
   far atterrare il tag `v1.0.0` con il quality gate di [`v1.0-personale.md`](./v1.0-personale.md)
   (restano: build matrix macOS/Linux, auto-update E2E, smoke test, bump). Niente
   feature nuove finché 1.0 non è fuori e non vediamo come viene recepito.
2. **Poi, in base alla ricezione, biforcazione della linea Personale in due
   roadmap** (da scrivere *dopo* 1.0, non ora):
   - **Personale "Flat"** — fedele al progetto iniziale: *ambito personale ma
     con feature non banali*. Polish e completamento, niente cambi di paradigma.
   - **Personale "Deluxe"** — direzione all'avanguardia. Seme in
     [`prompts-as-code.md`](./prompts-as-code.md) (tesi "Prompts as Code") e
     [`vault-a-cartella.md`](./vault-a-cartella.md) (storage a file no-lock-in).
3. **v2.0 Enterprise resta valida e invariata** — vedi [`v2.0-enterprise.md`](./v2.0-enterprise.md).

> I doc `prompts-as-code.md` e `vault-a-cartella.md` sono **esplorazioni
> post-1.0** (materiale per la futura roadmap "Deluxe"), non impegni per 1.0.

La **fonte autorevole della pianificazione** è [`release-plan.md`](./release-plan.md): un'unica timeline che mappa tag → fase/tema/SKU → stato.

## Documenti

| Doc | Descrizione |
|---|---|
| [`release-plan.md`](./release-plan.md) | **Timeline completa di tutti i rilasci** (v0.1 → v2.0+). Fonte autorevole della pianificazione. Include strategia SKU v1.0 Personale / v2.0 Enterprise |
| [`stagioni-e-nomi-rilascio.md`](./stagioni-e-nomi-rilascio.md) | **Convenzione di naming dei rilasci** ispirata alla moda: etichetta stagionale `Autunno-Inverno ANNO · vX.Y.Z` (taglio mesi H1/H2), doppia linea `1.x` Personale / `2.x` Enterprise, codename cardine stile Ubuntu (aggettivo+sostantivo tessile allitterante) con pool di ~20 nomi |
| [`v1.0-personale.md`](./v1.0-personale.md) | **Scope chiuso PaP Personale v1.0.0** — must-have M1-M8 (auto-update, a11y, sub-step Fase 4, import scopati, doppia vista editor, Markdown, coverage, docs), quality gate, timeline indicativa, strategia branching |
| [`v2.0-enterprise.md`](./v2.0-enterprise.md) | **Scope SKU PaP Enterprise v2.0.0** — gating domanda-driven, prerequisiti apertura branch, stream design parallelo a v1.0, strategia multi-SKU post-v2.0 |
| [`vault-a-cartella.md`](./vault-a-cartella.md) | 🧭 *Esplorazione post-1.0 (Deluxe)* — blueprint storage a file plain-text no-lock-in, modalità per-vault SQLite-cifrato vs cartella-chiaro, sidecar `.pap/`, fasi F1-F4 |
| [`prompts-as-code.md`](./prompts-as-code.md) | 🧭 *Esplorazione post-1.0 (Deluxe)* — idea strategica "Prompts as Code" (git=versioni, branch=varianti, golden=CI gate, pin a SHA per agent); gap competitivo, MVP, bet alternativi |
| [`ordito-sync-log.md`](./ordito-sync-log.md) | 🧭 *Esplorazione post-1.0 (Deluxe→Enterprise)* — blueprint "Ordito": oplog replicato firmato (HLC, LWW per campo, conflitto→variante), sync multi-device senza server (cartella/LAN/relay muto), DB come proiezione → strato database-agnostic per v2.x. v3: 7 decisioni chiuse |
| [`ordito/blueprint-F1.md`](./ordito/blueprint-F1.md) | ✏️ *Blueprint operativo (design, no codice)* — taglio esecutivo della fase F1 "Fondamenta" di Ordito: DDL V016, spec HLC/`apply_change`/key-storage, indici derivati fuori TX, GC locale, push riparato verso papsync, sequenza 8 PR, piano property test |
| [`guida-interattiva.md`](./guida-interattiva.md) | ✏️ *Blueprint feature (design, no codice)* — guida/tutorial interattivo in-app: help system a strati (tour spotlight + "?" contestuali + checklist + empty-state), 7 alternative valutate, driver.js vs engine custom, 5 fasi; profondità sul sito |
| [`menu-contestuale.md`](./menu-contestuale.md) | ✏️ *Blueprint feature (design, no codice)* — menu tasto-destro context-aware nell'app |
| [`linter-personalizzabile.md`](./linter-personalizzabile.md) | ✏️ *Blueprint feature (design, no codice)* — visibilità + tuning regole linter, in 2 fasi rilasciabili indipendenti |
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
- Il **naming dei rilasci** (etichetta stagionale + eventuale codename cardine) segue [`stagioni-e-nomi-rilascio.md`](./stagioni-e-nomi-rilascio.md). La stagione colloca nel tempo, il numero `vX.Y.Z` è il riferimento tecnico, il codename battezza solo le release di svolta.

## Aspetti correlati in altri cluster

- **Decisioni architetturali che condizionano la roadmap**: [`../architettura/decisioni/`](../architettura/decisioni/)
- **Doc utente di feature già atterrate**: [`../utente/`](../utente/)
