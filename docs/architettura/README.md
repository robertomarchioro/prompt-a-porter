# Cluster — Architettura

Documentazione per **chi vuole capire o estendere** il sistema. Comprende overview tecnica, schema dati, API server, design system e tutte le decisioni architetturali documentate (ADR).

## Documenti

| Doc | Descrizione |
|---|---|
| [`overview.md`](./overview.md) | Panoramica architetturale: moduli del client desktop, server di sync, MCP server, CLI; flussi principali; tecnologie scelte |
| [`schema-dati.md`](./schema-dati.md) | Schema SQLite del vault locale: tabelle, indici, FTS5, vincoli; descrizione di ogni colonna |
| [`api-server.md`](./api-server.md) | API HTTP del server di sync (papsync). ⚠️ Documento fermo a Fase 1; aggiornamento programmato in Fase 5 (vedi [`../roadmap/fase-5-enterprise.md`](../roadmap/fase-5-enterprise.md)) |
| [`design-system.md`](./design-system.md) | Punto di accesso al bundle di design Fase 1 (mockup HTML, token CSS, componenti) |

## Sottocartelle

| Path | Descrizione |
|---|---|
| [`decisioni/`](./decisioni/) | Architecture Decision Records (ADR) e spike tecnici: scelte architetturali con razionale, alternative scartate, evidenza empirica quando disponibile |
| [`design-handoff/`](./design-handoff/) | Bundle Fase 1: design system in HTML statico, token semantici (`tokens.css`/`tokens.json`), 8 superfici principali ad alta fedeltà. Asset di riferimento per implementazione UI |

## Per orientarsi rapidamente

- **"Come funziona X?"** → [`overview.md`](./overview.md)
- **"Come è fatto il database?"** → [`schema-dati.md`](./schema-dati.md)
- **"Perché abbiamo scelto Y invece di Z?"** → [`decisioni/`](./decisioni/)
- **"Che colore usa il bottone primary?"** → [`design-handoff/tokens.css`](./design-handoff/tokens.css)
