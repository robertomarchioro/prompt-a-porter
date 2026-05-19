---
layout: home

hero:
  name: Prompt a Porter
  text: Libreria locale per i tuoi prompt AI
  tagline: Template parametrici, vault cifrato, hotkey globale. Privacy-first.
  actions:
    - theme: brand
      text: Inizia in 5 minuti
      link: /utente/getting-started
    - theme: alt
      text: Vedi su GitHub
      link: https://github.com/robertomarchioro/prompt-a-porter
    - theme: alt
      text: Scarica v0.8.11
      link: https://github.com/robertomarchioro/prompt-a-porter/releases/latest

features:
  - icon: 🔐
    title: Local-first, vault cifrato
    details: I tuoi prompt restano sul tuo disco. SQLCipher AES-256, password mai persistita. Nessun account, nessun cloud forzato.

  - icon: ⌨️
    title: Hotkey globale
    details: Ctrl+Shift+P apre la Command Palette ovunque ti trovi, anche fuori dall'app. Cerchi, compili, incolli — in pochi secondi.

  - icon: 🧩
    title: Template parametrici
    details: Segnaposti `{{nome}}`, valori globali, import fra prompt. Costruisci prompt modulari riusabili.

  - icon: 🔄
    title: Cronologia + varianti
    details: Ogni modifica versionata. Crea varianti A/B per testare formulazioni diverse. Rollback in un click.

  - icon: 🤖
    title: Integra con Claude, Cursor, agenti AI
    details: Server MCP integrato espone il vault agli agenti via stdio. CLI `pap` per accesso da terminale.

  - icon: 📦
    title: Open source, AGPL-3.0
    details: Codice trasparente, contribuzioni benvenute. Stack Tauri 2 + Svelte 5 + Rust + SQLite.
---

## Inizio rapido

Tre passi per il primo prompt:

1. **Installa** — [scarica il bundle Windows](https://github.com/robertomarchioro/prompt-a-porter/releases/latest) firmato Authenticode.
2. **Crea il vault** — l'onboarding ti guida (password robusta + hotkey).
3. **Primo prompt** — crea un template, premi `Ctrl+Shift+P`, compila, copia. ✅

Vedi la [guida getting started](/utente/getting-started) per il dettaglio passo-passo.

## Per chi è

- **Knowledge worker** che usano LLM quotidianamente e vogliono mantenere una libreria personale.
- **Sviluppatori** che integrano prompt nei flussi (CLI, MCP, agenti).
- **Team piccoli** (≤10 persone) che condividono prompt via sync opzionale.

Non è pensato per: piattaforme enterprise multi-tenant, SSO obbligatorio, audit log compliance — quelle sono nella roadmap v2.0.

## Casi d'uso pronti

Sette ricette pronte all'uso per partire subito:

- [Email professionale](/utente/casi-uso/email-professionale) — richieste, follow-up, reclami parametrici.
- [Code review](/utente/casi-uso/code-review) — review automatica con severità 🔴🟡🔵.
- [Summarize articolo](/utente/casi-uso/summarize-articolo) — N punti chiave + TL;DR + verdetto.
- [Riscrittura tono](/utente/casi-uso/riscrittura-tono) — cambio registro mantenendo contenuto.
- [Brainstorm idee](/utente/casi-uso/brainstorm-idee) — con criteri + devil's advocate.
- [Traduzione tecnica](/utente/casi-uso/traduzione-tecnica) — glossario custom + reverse check.
- [Commit message](/utente/casi-uso/commit-message) — Conventional Commits da diff.

[Vedi tutte le ricette →](/utente/casi-uso/)
