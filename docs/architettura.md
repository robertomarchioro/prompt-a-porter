# Architettura — Prompt a Porter

> Documento in costruzione. Sarà completato durante la Fase 1.

## Panoramica

```
┌─────────────────────────────────────────┐
│            App Desktop (Tauri 2)        │
│  ┌──────────────┐  ┌────────────────┐   │
│  │  Svelte 5 UI │  │ Rust Core      │   │
│  │  (web view)  │◄─┤ - SQLCipher    │   │
│  │              │  │ - Tray/Hotkey  │   │
│  │              │  │ - Filesystem   │   │
│  └──────────────┘  └────────────────┘   │
└──────────────────┬──────────────────────┘
                   │ REST + WebSocket (opzionale)
         ┌─────────▼──────────┐
         │  Server Sync (Go)  │
         │  - Auth JWT        │
         │  - SQLite storage  │
         │  - WebSocket push  │
         └────────────────────┘
```

## Decisioni architetturali

Vedi il brief di sviluppo per le decisioni consolidate.
Questo documento sarà espanso con diagrammi Mermaid dettagliati.
