# Cluster — Roadmap

Documentazione di **pianificazione viva**: item rinviati, scope delle linee di prodotto, esplorazioni post-1.0, convenzione di naming dei rilasci.

> **Dov'è finito lo storico?** Il materiale di lavorazione concluso (fasi 1–4,
> blueprint del redesign v0.8 e delle feature spedite, SAL, retrospettive,
> pianificazione originale del sito) è stato spostato nell'archivio privato del
> maintainer (2026-07-23). Tutto resta comunque leggibile nella history git:
> ultima revisione completa al commit
> [`98093ce`](https://github.com/robertomarchioro/prompt-a-porter/tree/98093ce635b81a0b42b4e2ada4ffca0f7397197d/docs/roadmap).

## Distinzione fra "fase", "rilascio" e "SKU"

- **Fase** = macro-tema *per capability*. 5 in totale, in ordine progressivo (le fasi 1–4 sono chiuse; dettaglio nello storico).
- **Rilascio (release)** = unità di shipping con un tag git. Può chiudere una fase, parzialmente una fase, essere un patch line, oppure un rilascio speciale trasversale.
- **SKU** = linea di prodotto distinta. PaP ha 2 SKU pianificati:
  - **Personale** → `v1.x` (single user, local-first). Scope chiuso in [`v1.0-personale.md`](./v1.0-personale.md).
  - **Enterprise** → `v2.x` (multi-user, server). Gate domanda-driven, scope in [`v2.0-enterprise.md`](./v2.0-enterprise.md).

Le fasi sono ortogonali agli SKU: Fasi 1-4 sono confluite nello SKU Personale; Fase 5 corrisponde allo SKU Enterprise.

## Indirizzo strategico (2026-06-03)

Decisione di rotta dell'autore:

1. **Adesso: chiudere v1.0.0 Personale "senza fronzoli".** Priorità unica =
   far atterrare il tag `v1.0.0` con il quality gate di [`v1.0-personale.md`](./v1.0-personale.md).
   Niente feature nuove finché 1.0 non è fuori e non vediamo come viene recepito.
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

## Documenti di lavoro

| Doc | Descrizione |
|---|---|
| [`rinvii.md`](./rinvii.md) | **Censimento unificato di tutto ciò che è stato deliberatamente rinviato.** Singola fonte di verità degli item rinviati; classificati per stream (v1.0 / v2.0 / patch line / scartato) |
| [`v1.0-personale.md`](./v1.0-personale.md) | **Scope chiuso PaP Personale v1.0.0** — must-have M1-M8, quality gate, timeline indicativa, strategia branching |
| [`v2.0-enterprise.md`](./v2.0-enterprise.md) | **Scope SKU PaP Enterprise v2.0.0** — gating domanda-driven, prerequisiti apertura branch, strategia multi-SKU post-v2.0 |
| [`fase-5-enterprise.md`](./fase-5-enterprise.md) | Step della Fase 5 — dettaglio tecnico Step 0a-8 per `v2.0.0` (PaP Enterprise) |
| [`stagioni-e-nomi-rilascio.md`](./stagioni-e-nomi-rilascio.md) | **Convenzione di naming dei rilasci**: etichetta stagionale `Autunno-Inverno ANNO · vX.Y.Z`, doppia linea `1.x` Personale / `2.x` Enterprise, codename cardine tessile |
| [`prompts-as-code.md`](./prompts-as-code.md) | 🧭 *Esplorazione post-1.0 (Deluxe)* — idea strategica "Prompts as Code" (git=versioni, branch=varianti, golden=CI gate, pin a SHA per agent) |
| [`vault-a-cartella.md`](./vault-a-cartella.md) | 🧭 *Esplorazione post-1.0 (Deluxe)* — blueprint storage a file plain-text no-lock-in, sidecar `.pap/`, fasi F1-F4 |
| [`ordito-sync-log.md`](./ordito-sync-log.md) | 🧭 *Esplorazione post-1.0 (Deluxe→Enterprise)* — blueprint "Ordito": oplog replicato firmato (HLC, LWW per campo), sync multi-device senza server, DB come proiezione. v3: 7 decisioni chiuse |
| [`ordito/`](./ordito/blueprint-F1.md) | ✏️ *Blueprint operativi Ordito (design, no codice)* — tagli esecutivi delle 4 fasi F1-F4 |
| [`website/`](./website/istruzioni-sviluppo-landing.md) | **Thread parallelo — Landing page attiva** (www.promptaporter.it): istruzioni di sviluppo, contenuti, mockup desktop/mobile |
| [`icons/`](./icons/) | Sorgenti handoff delle icone (set violet definitivo + amber di riserva); i derivati si rigenerano con `pnpm tauri icon` |

## Convenzioni

- `rinvii.md` è il **glue** del debt tracking: ogni rinvio registrato lì con marker (🔒 esterno / 🔧 tecnico / 📋 sub-step / 🎨 polish).
- Il **naming dei rilasci** segue [`stagioni-e-nomi-rilascio.md`](./stagioni-e-nomi-rilascio.md). La timeline effettiva dei rilasci pubblicati è nel [`CHANGELOG.md`](../../CHANGELOG.md) (la timeline pianificatoria storica, `release-plan.md`, è archiviata nello storico).

## Aspetti correlati in altri cluster

- **Decisioni architetturali che condizionano la roadmap**: [`../architettura/decisioni/`](../architettura/decisioni/)
- **Doc utente di feature già atterrate**: [`../utente/`](../utente/)
