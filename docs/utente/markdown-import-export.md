# Import / Export Markdown

Prompt a Porter supporta import ed export dei prompt come file Markdown con front-matter YAML, compatibile con **Obsidian** e **Foam** (pattern wiki).

## Quando usare

- **Import bulk** di prompt esistenti da una libreria Obsidian/Foam o da una cartella di file `.md`.
- **Backup leggibile** del vault: ogni prompt come singolo file `.md` versionabile (git-friendly).
- **Migrazione** verso/da altre tool basate su file system.
- **Editing offline** con il tuo editor markdown preferito, poi re-import.

## Formato file `.md`

```markdown
---
title: "Email di benvenuto"
description: "Template per onboarding nuovi utenti"
target_model: "claude-sonnet-4-6"
visibility: private
version: 3
created_at: 2026-05-10 14:23:11
updated_at: 2026-05-16 09:11:47
imports:
  - "Saluto base"
  - "intestazione/firma-corporate"
---

{{import "Saluto base"}}

Ciao {{nome_utente}},

benvenuto in {{prodotto}}. Il tuo account è stato attivato il
{{globale data_attivazione}}.

{{import "intestazione/firma-corporate"}}
```

### Front-matter (YAML)

Le seguenti chiavi vengono lette in import (tutte opzionali):

| Chiave | Default su import | Note |
|---|---|---|
| `title` | nome file senza estensione | titolo del prompt |
| `description` | vuoto | descrizione breve |
| `target_model` | vuoto | modello AI suggerito |
| `visibility` | `private` | `private` o `workspace` |

**Chiavi sconosciute** (es. `tags`, `aliases` di Obsidian) sono **ignorate silentemente** in import — il file resta compatibile con Obsidian dopo il round-trip.

Le chiavi `version`, `created_at`, `updated_at`, `imports` presenti nel front-matter prodotto da Prompt a Porter sono **informative**: in import vengono ricalcolate (versione resettata a 1, date a `now`, imports ri-derivati dal body).

### Body

Tutto il testo dopo il secondo `---` è il **body del prompt**. Supporta tutte le funzionalità native:

- **Segnaposti** `{{nome}}` (compilati al momento dell'uso)
- **Segnaposti globali** `{{globale autore}}` (compilati dal DB globale)
- **Import composti** `{{import "Path/titolo"}}` per riusare altri prompt
- **Variabili scopate** `{{import "x" with k=v, k2=v2}}` (M4)
- **Pinning versione** `{{import "x" version=N}}` (M4)

## Importare da file `.md`

1. Impostazioni → **Dati** → "Importa Markdown"
2. Clicca "Seleziona file…"
3. Seleziona **uno o più** file `.md`/`.markdown` (multi-select con Ctrl/Cmd+click)
4. Ogni file diventa un nuovo prompt nel vault
5. Lista risultati mostra successi + eventuali errori per-file

### Workflow Obsidian / Foam

Da un vault Obsidian:

1. Apri Esplora Risorse / Finder nella cartella del vault Obsidian
2. Seleziona i file `.md` che vuoi portare in Prompt a Porter (Ctrl+A per tutti)
3. In Prompt a Porter → Impostazioni → Dati → "Seleziona file…" → trascina o seleziona

I file Obsidian con front-matter standard (titolo, tags, aliases) sono **importati senza modifiche destructive** — i tags/aliases vengono ignorati ma il body+title sono preservati. Puoi re-esportare in qualsiasi momento.

### Limiti

- **Una directory completa con sottocartelle**: il file picker HTML non supporta directory recursion in WebView. Usa l'invocazione CLI futura (`pap import-markdown <dir>`) oppure trascina tutti i file in modo manuale.
- **Front-matter malformato**: se il front-matter non è chiudibile (manca il secondo `---`), il contenuto intero viene trattato come body con titolo dal nome file.
- **File grandi** (> 1 MB): supportati ma il parser non è ottimizzato per `.md` enormi.

## Esportare il vault → zip Markdown

1. Impostazioni → **Dati** → "Esporta Vault → Zip Markdown"
2. Clicca "Esporta zip"
3. Il file `prompt-a-porter-export-YYYY-MM-DD.zip` viene scaricato automaticamente

### Struttura dello zip

```
prompt-a-porter-export-2026-05-19.zip
├── Email di benvenuto.md
├── Generatore changelog.md
├── marketing/
│   ├── Email cold outreach.md
│   └── email/
│       ├── Welcome flow.md
│       └── Retention week-2.md
└── tech/
    ├── Riassunto bug report.md
    └── PR review checklist.md
```

La gerarchia cartelle del vault è preservata. File con titoli duplicati nella stessa cartella sono disambigui con suffisso `-<id-short>` (es. `Duplicato-abc12345.md`).

### Caratteri non sicuri nei nomi file

I caratteri `/\<>:"|?*` (riservati su Windows) sono sostituiti con `_` nei nomi file dello zip. Il titolo del prompt nel front-matter è preservato intatto.

## Pattern Obsidian / Foam compatibility

| Feature | Obsidian | Foam | Prompt a Porter |
|---|---|---|---|
| Front-matter YAML | ✓ | ✓ | ✓ (subset) |
| Wikilinks `[[foo]]` | ✓ | ✓ | ✗ (usa `{{import "foo"}}`) |
| Tags `#tag` | ✓ | ✓ | ✗ (sistema tag DB separato) |
| `tags:` in front-matter | ✓ | ✓ | ignorato in import (no perdita) |

Il round-trip Obsidian → PaP → Obsidian preserva: title, body, struttura cartelle. Perde: tags front-matter, aliases (recuperabili manualmente).

## CLI (futuro)

Il backend `prompt_import_markdown`, `vault_import_markdown_bulk`, `vault_export_markdown_zip` sono già esposti come Tauri command. Una sub-command CLI dedicata (`pap import-markdown <path>`, `pap export-markdown --output=vault.zip`) è in roadmap per CI/CD automatici.

## Vedi anche

- [`prompt-componibili.md`](./prompt-componibili.md) — sintassi `{{import "..."}}` + `with`/`version`
- [`cartelle.md`](./cartelle.md) — struttura cartelle del vault
- [`formato-export-json.md`](./formato-export-json.md) — alternativa JSON con schema versionato (per migrazione completa con metadati)
