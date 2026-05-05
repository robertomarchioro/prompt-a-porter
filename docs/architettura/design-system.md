# Design System

Il design di Prompt a Porter è stato consolidato in **Fase 1** come bundle handoff statico, ed è la fonte di riferimento canonica per tutte le decisioni visive del client.

## Dove si trova

Tutto il bundle è in [`./design-handoff/`](./design-handoff/):

| Cosa | File |
|---|---|
| Entry point del bundle | [`design-handoff/index.html`](./design-handoff/index.html) |
| Design system completo (componenti + linee guida) | [`design-handoff/Design System.html`](./design-handoff/Design System.html) |
| 8 superfici principali ad alta fedeltà | `design-handoff/01..08 *.html` |
| Token semantici come CSS variables | [`design-handoff/tokens.css`](./design-handoff/tokens.css) |
| Token come JSON (per Style Dictionary, Tailwind generato, ...) | [`design-handoff/tokens.json`](./design-handoff/tokens.json) |
| Foglio di stile applicativo di riferimento | [`design-handoff/app.css`](./design-handoff/app.css) |
| Note di handoff originali (Fase 1) | [`design-handoff/README.md`](./design-handoff/README.md) |

## Come usare il bundle

I file HTML sono **prototipi visivi statici**: serve aprirli nel browser per vederli. Non sono codice di produzione da copiare.

I file CSS/JSON dei token sono invece **direttamente consumabili**:

- I componenti Svelte del client (`apps/client/src/lib/components/`) leggono CSS variables da `tokens.css`.
- Per nuove integrazioni (es. browser extension, web app in Fase 5) si parte sempre da questi token, mai da valori hardcoded.

## Decisioni di design preservate

Dal bundle handoff, tre decisioni che hanno impatto su tutto il progetto:

1. **Densità delle liste 32-40px**: superfici "lavorative" devono permettere scansione veloce di molti item.
2. **Accent semantici** ambra=privato, viola=team: distinguono visualmente la visibilità dei prompt senza testo.
3. **Monospace ovunque conta il parsing** (segnaposti `{{nome}}`, ID, hex): il font veicola il significato di "stringa strutturata".

## Aspetti correlati

- **Componenti Svelte già implementati**: `apps/client/src/lib/components/`
- **Implementazione delle superfici**: `apps/client/src/lib/superfici/`
- **Aggiornamenti del design system**: tracciati in [`../roadmap/`](../roadmap/) quando una nuova fase richiede componenti nuovi
