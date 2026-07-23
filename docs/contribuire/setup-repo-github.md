# Setup Repository GitHub — Prompt a Porter

## Branch strategy

- `main` — branch principale, sempre deployabile
- Branch di sviluppo: `feature/*`, `fix/*`, `docs/*`

## GitHub Actions

### `client-build.yml`

Due job paralleli:

| Job | Runner | Step |
|-----|--------|------|
| `lint-and-test` | ubuntu-latest | pnpm install → svelte-check (lint + type check) → vitest run |
| `rust-test` | ubuntu-latest | installa libwebkit2gtk + deps → cargo test |

Trigger: push/PR su `main` quando cambiano file in `apps/client/`, `packages/`, o `pnpm-workspace.yaml`.

Build Tauri multi-OS (Windows, macOS, Linux) è predisposta nel YAML ma commentata — richiede setup signing per distribuzione.

### `server-build.yml`

Un job:

| Job | Runner | Step |
|-----|--------|------|
| `lint-and-test` | ubuntu-latest | golangci-lint → go test -race -coverprofile → coverage check (soglia 70%) |

Trigger: push/PR su `main` quando cambiano file in `apps/server/`.

## Dipendenze CI

| Azione | Versione |
|--------|----------|
| `actions/checkout` | v7 |
| `actions/setup-node` | v7 (Node 22) |
| `pnpm/action-setup` | v6 |
| `dtolnay/rust-toolchain` | pin 1.96.0 |
| `actions/setup-go` | v7 (Go 1.25) |
| `golangci/golangci-lint-action` | v9 |

> Le versioni sono pinnate per SHA nei workflow: la fonte di verità è sempre
> `.github/workflows/*.yml`, questa tabella è un riepilogo.
