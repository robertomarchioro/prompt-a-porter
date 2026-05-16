# Cluster — Contribuire

Documentazione per **chi vuole contribuire codice** al progetto.

## Documenti

| Doc | Descrizione |
|---|---|
| [`setup-sviluppo.md`](./setup-sviluppo.md) | Prerequisiti, installazione toolchain (Rust, Node, pnpm, Go), comandi di build, struttura della monorepo |
| [`setup-repo-github.md`](./setup-repo-github.md) | Branch strategy, naming convention dei branch, configurazione iniziale del repo GitHub |
| [`ci-workflows.md`](./ci-workflows.md) | Mappa esaustiva path → workflow CI attivati, anti-pattern (PR doc-only, workflow non auto-listati), checklist operativa, comandi di debug |
| [`setup-tauri-updater-keys.md`](./setup-tauri-updater-keys.md) | Procedura una-tantum maintainer per generare e configurare la coppia di chiavi Ed25519 del Tauri Updater (v1.0 M1.4): generate, backup, GitHub Secrets, sostituzione `pubkey` in `tauri.conf.json`, recovery se chiave persa |

## Per cominciare

1. Leggi prima [`CONTRIBUTING.md`](../../CONTRIBUTING.md) di repository per: DCO sign-off, Conventional Commits, flusso PR.
2. Segui [`setup-sviluppo.md`](./setup-sviluppo.md) per preparare l'ambiente locale.
3. Per orientarti nel codice, leggi [`../architettura/overview.md`](../architettura/overview.md).
4. Se stai lavorando su una feature pianificata, controlla [`../roadmap/`](../roadmap/) per il contesto.

## Aspetti correlati in altri cluster

- **Architettura del sistema**: [`../architettura/`](../architettura/)
- **Decisioni tecniche già prese** (ADR): [`../architettura/decisioni/`](../architettura/decisioni/)
- **Roadmap e item rinviati**: [`../roadmap/`](../roadmap/)
