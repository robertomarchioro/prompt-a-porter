# Setup Repository GitHub — Prompt a Porter

> Documento in costruzione. Sarà completato durante la Fase 1.

## Branch protection

- Branch `main` protetto: richiede PR review + CI green
- Branch di sviluppo: `feature/*`, `fix/*`, `docs/*`

## GitHub Actions

- `client-build.yml` — Build Tauri multi-OS (Windows, macOS, Linux)
- `server-build.yml` — Build e test server Go

## Secret necessari

Documentare qui i secret richiesti per CI/CD.
