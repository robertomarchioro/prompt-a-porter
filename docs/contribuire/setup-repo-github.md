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
| `actions/checkout` | v4 |
| `actions/setup-node` | v4 (Node 22) |
| `pnpm/action-setup` | v4 |
| `dtolnay/rust-toolchain` | stable |
| `actions/setup-go` | v5 (Go 1.22) |
| `golangci/golangci-lint-action` | v6 |
