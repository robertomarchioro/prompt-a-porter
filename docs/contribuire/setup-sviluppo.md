# Setup Sviluppo — Prompt a Porter

## Prerequisiti

| Strumento | Versione | Note |
|-----------|----------|------|
| Node.js | 22.x LTS | Runtime per frontend Svelte e tooling |
| pnpm | 9.x+ | Package manager (workspace monorepo) |
| Rust | stable (≥1.77) | Per Tauri 2 core + SQLCipher |
| Go | 1.22+ | Per il server sync (opzionale) |

### Dipendenze sistema (Linux)

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  build-essential
```

Su Windows e macOS, Tauri gestisce le dipendenze native automaticamente.

## Installazione

```bash
git clone https://github.com/<org>/prompt-a-porter.git
cd prompt-a-porter
pnpm install
```

## Comandi sviluppo

### Client (Tauri + Svelte)

```bash
# Dev mode con hot reload
pnpm --filter @pap/client dev

# Solo frontend (senza Tauri, per sviluppo UI rapido)
pnpm --filter @pap/client dev:frontend

# Type check
pnpm --filter @pap/client check

# Test TypeScript (vitest)
pnpm --filter @pap/client test

# Test Rust
cd apps/client/src-tauri && cargo test

# Build release
pnpm --filter @pap/client build
```

### Server sync (Go)

```bash
cd apps/server

# Avvia il server
PAP_ADMIN_EMAIL=admin@test.com PAP_ADMIN_PASSWORD=password123 go run ./cmd/papsync

# Test
go test ./... -v

# Test con coverage
go test -race -coverprofile=coverage.out ./...
go tool cover -func=coverage.out

# Build binario
go build -o papsync ./cmd/papsync

# Docker
docker build -t pap/sync-server .
```

## Struttura workspace

```
prompt-a-porter/
├── apps/
│   ├── client/                 ← @pap/client (Tauri + Svelte)
│   │   ├── src/
│   │   │   ├── lib/
│   │   │   │   ├── components/     16 primitive UI
│   │   │   │   ├── superfici/      10 superfici (pagine/modali)
│   │   │   │   ├── i18n/           it.json + en.json
│   │   │   │   ├── sync.ts         Store sync singleton
│   │   │   │   └── template.ts     Parser segnaposti
│   │   │   └── app.css             Token CSS + utility
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   │   ├── lib.rs          Entry point + tray + hotkey
│   │   │   │   ├── vault.rs        SQLCipher + Argon2id
│   │   │   │   ├── editor.rs       CRUD prompt + FTS
│   │   │   │   ├── libreria.rs     Query lista/dettaglio
│   │   │   │   ├── prompt.rs       Ricerca FTS5
│   │   │   │   ├── sync.rs         Applica delta sync
│   │   │   │   ├── audit.rs        Audit trail
│   │   │   │   ├── preferenze.rs   Gestione preferenze JSON
│   │   │   │   ├── migrazione.rs   Schema versioning
│   │   │   │   └── errore.rs       Tipo errore unificato
│   │   │   ├── migrations/         SQL embedded
│   │   │   └── capabilities/       ACL Tauri
│   │   ├── vitest.config.ts
│   │   └── package.json
│   └── server/                 ← Go sync server
│       ├── cmd/papsync/            Entry point
│       ├── internal/
│       │   ├── auth/               Login + JWT
│       │   ├── database/           SQLite + Argon2id
│       │   ├── middleware/         JWT + logger
│       │   ├── sync/               Pull + Push handlers
│       │   └── ws/                 WebSocket hub
│       ├── Dockerfile
│       └── go.mod
├── docs/                       ← Documentazione
├── design_handoff/             ← Prototipi HTML + token CSS
└── .github/workflows/          ← CI/CD
```

## Test

### Copertura test (Fase 1)

| Area | Test | Moduli coperti |
|------|------|----------------|
| Rust | 37 | vault (7), migrazione (3), preferenze (2), prompt (9), editor (7), libreria (7), audit (5), sync (4), errore (5) |
| TypeScript | 22 | template.ts (estraiSegnaposti, compila, contaCompilati) |
| Go (server) | 12 | auth, sync, database, password hashing |

### Eseguire tutti i test

```bash
# Client TypeScript
pnpm --filter @pap/client test

# Client Rust
cd apps/client/src-tauri && cargo test

# Server Go
cd apps/server && go test ./... -v
```
