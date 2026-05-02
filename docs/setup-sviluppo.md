# Setup Sviluppo — Prompt a Porter

> Documento in costruzione. Sarà completato durante la Fase 1.

## Prerequisiti

| Strumento | Versione | Note |
|-----------|----------|------|
| Node.js | 22.x LTS | Runtime per frontend Svelte e tooling |
| pnpm | 9.x+ | Package manager (workspace monorepo) |
| Rust | stable | Per Tauri 2 core |
| Go | 1.22+ | Per il server sync |

## Installazione

```bash
# Clona il repo
git clone https://github.com/<org>/prompt-a-porter.git
cd prompt-a-porter

# Installa dipendenze Node
pnpm install

# Avvia il client in dev mode
pnpm --filter @pap/client dev

# Avvia il server sync (in un altro terminale)
cd apps/server
go run ./cmd/papsync
```

## Struttura workspace pnpm

```
prompt-a-porter/
├── apps/client/    → @pap/client (Tauri + Svelte)
├── apps/server/    → Go server (fuori dal workspace pnpm)
└── packages/shared-schema/ → @pap/shared-schema
```
