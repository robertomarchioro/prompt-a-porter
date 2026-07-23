# Cluster — Operativo

Documentazione di **operations**: deploy del server di sync, configurazione, runbook.

> **Nota**: in questa fase del progetto il server `papsync` non è in produzione presso clienti reali. La documentazione di deploy esiste come riferimento e si arricchirà significativamente con Fase 5 (Step 0a — server cross-OS senza Docker).

## Documenti

| Doc | Descrizione |
|---|---|
| [`deploy-produzione.md`](./deploy-produzione.md) | Deploy del client desktop e del server di sync via Docker; variabili d'ambiente; raccomandazioni di base |
| [`bench-ricerca-ibrida.md`](./bench-ricerca-ibrida.md) | Quality gate Fase 3: bench P95 ricerca ibrida su dataset realistico 1k/10k prompt |
| [`coverage.md`](./coverage.md) | Quality gate Fase 3: coverage line del client Rust, gate CI 60%, roadmap verso 70% |
| [`firma-macos.md`](./firma-macos.md) | Firma `Developer ID Application` + notarizzazione Apple degli asset macOS, interamente in CI dentro `release.yml`. Procedura eseguita e validata (2026-07-08/09). |

## Aspetti correlati in altri cluster

- **API del server** (per integrazioni esterne): [`../architettura/api-server.md`](../architettura/api-server.md) ⚠️ stale
- **Roadmap del server cross-OS senza Docker**: [`../roadmap/fase-5-enterprise.md`](../roadmap/fase-5-enterprise.md) Step 0a
