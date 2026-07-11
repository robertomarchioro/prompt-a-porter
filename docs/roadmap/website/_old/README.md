# Thread parallelo — Landing page PaP

> **Scope**: progettazione, sviluppo, pubblicazione e manutenzione della landing page di Prompt a Porter. Thread completamente separato dal codice del prodotto.
> **Assegnazione**: agente parallelo dedicato (TBD), distinto dall'agente principale che lavora su v1.0 Personale.
> **Aggiornato al**: 2026-05-12.

## Stato attuale

🟡 **Thread aperto, lavoro non ancora avviato.** Documentazione di setup completa, in attesa di:
- Assegnazione agente parallelo dedicato.
- Handoff a Claude Design per la fase grafica.
- Decisione su quando far partire (suggerito: parallelo a M2-M3 di v1.0, non aspetta v1.0 GA).

## Perché serve un thread separato

Una landing page ha esigenze di sviluppo, deploy e manutenzione **disjoint** dal client desktop:
- Stack diverso (static site generator vs Tauri/Svelte).
- Repository / sottocartella diversa (`website/` o repo gemello).
- Pipeline CI distinta (build statico + push GitHub Pages).
- Aggiornamenti async rispetto alle release del prodotto.

Tenere il lavoro separato permette di parallelizzare senza conflitti sul codice di produzione.

## Decisioni già prese

Vedi i doc dedicati per il razionale completo:

| Tema | Decisione | Doc |
|---|---|---|
| Frame del prodotto | Solo PaP Personale (no teaser Enterprise) | [`contenuti.md`](./contenuti.md) §"Frame" |
| Lingua | IT (solo) | [`contenuti.md`](./contenuti.md) §"Lingua" |
| Stack tecnico | Astro (static site generator) | [`tech-stack.md`](./tech-stack.md) |
| Hosting | GitHub Pages | [`hosting.md`](./hosting.md) |
| Analytics | Matomo self-hosted su Gigantto | [`analytics-matomo.md`](./analytics-matomo.md) |
| Dominio | GitHub Pages subdomain inizialmente, custom domain opzionale dopo | [`hosting.md`](./hosting.md) §"Dominio" |

## Documenti del thread

| Doc | Descrizione |
|---|---|
| [`README.md`](./README.md) | (questo file) — index e stato thread |
| [`contenuti.md`](./contenuti.md) | **Handoff a Claude Design** — cosa raccontare, sezioni, copy iniziale IT, CTA, audience |
| [`tech-stack.md`](./tech-stack.md) | Astro setup, struttura repo, build pipeline |
| [`hosting.md`](./hosting.md) | GitHub Pages deploy, dominio, opzioni alternative archiviate |
| [`analytics-matomo.md`](./analytics-matomo.md) | Setup Matomo container su Gigantto, integrazione frontend, considerazioni GDPR |
| [`roadmap.md`](./roadmap.md) | Milestone del thread con dipendenze e timeline indicativa |

## Workflow per l'agente parallelo

1. **Leggi tutti i 6 doc** prima di iniziare. Sono ~20 minuti di lettura.
2. **Fase 1 — Setup tecnico**: segui [`tech-stack.md`](./tech-stack.md) e [`hosting.md`](./hosting.md) per inizializzare la struttura Astro nel repo + workflow GitHub Pages.
3. **Fase 2 — Handoff a Claude Design**: passa [`contenuti.md`](./contenuti.md) a Claude Design. Aspetta gli asset grafici (mockup, palette, tipografia).
4. **Fase 3 — Sviluppo pagina**: implementa la landing seguendo i mockup, copy in IT da [`contenuti.md`](./contenuti.md).
5. **Fase 4 — Setup analytics**: segui [`analytics-matomo.md`](./analytics-matomo.md) per setup container + integrazione.
6. **Fase 5 — Deploy** su GitHub Pages, verifica live.
7. **Manutenzione** continua: ogni release di prodotto significativa → aggiornamento screenshot/versioni sulla landing.

## Cosa l'agente parallelo NON deve fare

- Modifiche a `apps/` (codice del prodotto).
- Cambiare la strategia SKU o il branding del prodotto (decisioni dell'agente principale + utente).
- Promettere feature Enterprise (vedi [`contenuti.md`](./contenuti.md) §"Frame").
- Aggiungere analytics di terze parti diversi da Matomo on-premise (privacy first è un valore comunicato).

## Cosa fa l'agente principale rispetto a questo thread

- **Niente**, salvo:
  - Aggiornare `CHANGELOG.md` quando una nuova release del prodotto è disponibile → l'agente website aggiorna gli screenshot/versioni.
  - Notificare il thread quando v1.0.0 GA → momento del "lancio ufficiale" della landing.

## Manutenzione di questo documento

- Quando una fase del thread chiude: spuntare in [`roadmap.md`](./roadmap.md).
- Quando una decisione strategica cambia: aggiornare la tabella §"Decisioni già prese" + il doc dedicato.
- Quando l'agente parallelo cambia: aggiornare §"Assegnazione" in cima.
