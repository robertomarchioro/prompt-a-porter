# Cluster — Utente

Documentazione per **chi usa** Prompt a Porter, sia il client desktop che gli strumenti integrati.

## Documenti

| Doc | Descrizione |
|---|---|
| [`cli.md`](./cli.md) | Reference della CLI `pap` (Go + cobra): 5 comandi read-only per accedere al vault dal terminale, output in `table`/`json`/`yaml`/`plain`, completion bash/zsh/fish/powershell |
| [`mcp.md`](./mcp.md) | Integrazione Model Context Protocol: setup Claude Desktop, Cursor, custom client; 4 tool esposti (`pap_search`, `pap_get`, `pap_list_recent`, `pap_render`); trasporto stdio; troubleshooting |
| [`formato-export-json.md`](./formato-export-json.md) | Schema del formato di export JSON v1: campi obbligatori/opzionali, modalità conflitti import (skip/overwrite/rename), garanzie round-trip lossless |
| [`ricerca-semantica.md`](./ricerca-semantica.md) | Ricerca ibrida (FTS + embedding), modello scelto, alpha bilanciamento, idle-unload, performance e privacy |
| [`linting-regole.md`](./linting-regole.md) | Catalogo completo delle 11 regole di linting con esempi pratici (LEN/PH/PII/STY/IMP) |
| [`cartelle.md`](./cartelle.md) | Modello dati e UX cartelle (anticipa permessi Fase 4) |
| [`prompt-componibili.md`](./prompt-componibili.md) | Sintassi `{{import "..."}}`, esempi, anti-pattern, depth limits e cicli |
| [`varianti-prompt.md`](./varianti-prompt.md) | A/B testing dei prompt: come creare varianti B/C/Z, navigazione, differenza con i fork (Fase 4) |
| [`fork-prompt.md`](./fork-prompt.md) | Clone indipendente con tracciabilità: quando fare fork, banner "Fork di X", chain (Fase 4) |
| [`rating-prompt.md`](./rating-prompt.md) | Rating discreto post-uso 👎/😐/👍, badge percentuale colorato, traiettoria nel tempo (Fase 4) |
| [`regression-testing.md`](./regression-testing.md) | Golden examples + provider AI (Ollama/Anthropic/OpenAI) + similarity functions + drift report — differenziatore strategico (Fase 4) |
| [`auto-update.md`](./auto-update.md) | Aggiornamenti automatici (v1.0+): policy on-demand, firma Ed25519 + Authenticode, privacy, troubleshooting, FAQ |

## Per iniziare

Se sei un nuovo utente:

1. Installa il client desktop (vedi [`README.md`](../../README.md) di repository per i bundle disponibili).
2. Al primo avvio, l'onboarding ti guida nella creazione del vault (locale, SQLCipher cifrato).
3. Imposta una hotkey globale (default: `Ctrl+Shift+P`) per evocare la Command Palette ovunque.

Per integrazioni avanzate:

- **Riga di comando**: vedi [`cli.md`](./cli.md). La CLI `pap` legge lo stesso vault del client.
- **Agenti AI** (Claude, Cursor, ...): vedi [`mcp.md`](./mcp.md).
- **Backup / migrazione**: vedi [`formato-export-json.md`](./formato-export-json.md) per il formato dell'export.
