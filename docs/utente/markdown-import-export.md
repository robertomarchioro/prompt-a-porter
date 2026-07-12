# Import / Export Markdown

> Come importare ed esportare i prompt come file Markdown con front-matter YAML, compatibili con **Obsidian** e **Foam**.

I prompt sono, in fondo, testo — e il testo sta bene nei file. Prompt a Porter sa esportare l'intero vault come una cartella di file `.md` con front-matter YAML, e sa importare file Markdown esistenti trasformandoli in prompt. È il ponte tra il vault e tutto il mondo che già lavora a file: editor di testo, git, strumenti di note come Obsidian e Foam.

Questo apre parecchie porte: un backup leggibile che puoi versionare con git e aprire con qualunque editor; la migrazione in blocco di una libreria di prompt che tieni già come note; l'editing offline con il tuo editor preferito, seguito da un re-import. E siccome il formato è Markdown standard con front-matter, i file restano utilizzabili anche senza Prompt a Porter — nessun lock-in.

## Quando usare

I casi tipici in cui il formato Markdown conviene rispetto all'export JSON:

- **Import in blocco** di prompt esistenti da una libreria Obsidian/Foam o da una cartella di file `.md`.
- **Backup leggibile** del vault: ogni prompt come singolo file `.md` versionabile (git-friendly).
- **Migrazione** verso/da altri strumenti basati su file system.
- **Editing offline** con il tuo editor Markdown preferito, poi re-import.

Se invece ti serve un backup completo con tutti i metadati (storico versioni, tag, rating), il formato giusto è il [JSON](./formato-export-json.md).

## Formato file `.md`

Ecco come appare un prompt esportato: front-matter YAML con i metadati, poi il corpo del prompt con tutta la sua sintassi:

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
{{global data_attivazione}}.

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

**Chiavi sconosciute** (es. `tags`, `aliases` di Obsidian) sono **ignorate silenziosamente** in import — il file resta compatibile con Obsidian dopo il round-trip.

Le chiavi `version`, `created_at`, `updated_at`, `imports` presenti nel front-matter prodotto da Prompt a Porter sono **informative**: in import vengono ricalcolate (versione resettata a 1, date a `now`, imports ri-derivati dal body).

### Body

Tutto il testo dopo il secondo `---` è il **body del prompt**, e supporta l'intera sintassi che useresti nell'editor dell'app:

- **Segnaposti** `{{nome}}` (compilati al momento dell'uso)
- **Segnaposti globali** `{{global autore}}` (compilati con i valori salvati nel vault)
- **Import composti** `{{import "Path/titolo"}}` per riusare altri prompt
- **Variabili scopate** `{{import "x" with k=v, k2=v2}}`
- **Pinning di versione** `{{import "x" version=N}}`

## Importare da file `.md`

L'import trasforma ogni file selezionato in un nuovo prompt del vault:

1. Vai in **Impostazioni → Dati**, card **Importa Markdown**.
2. Clicca **"Seleziona file…"**.
3. Seleziona **uno o più** file `.md`/`.markdown` (multi-selezione con Ctrl/Cmd+click).
4. Ogni file diventa un nuovo prompt nel vault.
5. Al termine, la lista dei risultati mostra i prompt creati e gli eventuali errori, file per file.

### Workflow Obsidian / Foam

Per portare in Prompt a Porter le note di un vault Obsidian:

1. Apri Esplora Risorse / Finder nella cartella del vault Obsidian.
2. Individua i file `.md` che vuoi portare in Prompt a Porter.
3. In Prompt a Porter → **Impostazioni → Dati** → **"Seleziona file…"** e selezionali (Ctrl+A per tutti quelli di una cartella).

I file Obsidian con front-matter standard (titolo, tags, aliases) sono **importati senza modifiche distruttive** — i tags/aliases vengono ignorati ma body e titolo sono preservati. Puoi re-esportare in qualsiasi momento.

## Esportare il vault → zip Markdown

L'export produce un archivio zip con un file `.md` per ogni prompt:

1. Vai in **Impostazioni → Dati**, card **Esporta Vault → Zip Markdown**.
2. Clicca **"Esporta zip"**.
3. Il file `prompt-a-porter-export-YYYY-MM-DD.zip` viene scaricato automaticamente.

### Struttura dello zip

Dentro lo zip ritrovi la stessa gerarchia di cartelle del vault:

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

File con titoli duplicati nella stessa cartella vengono disambiguati con un suffisso `-<id-short>` (es. `Duplicato-abc12345.md`).

### Caratteri non sicuri nei nomi file

I caratteri `/\<>:"|?*` (riservati su Windows) sono sostituiti con `_` nei nomi file dello zip. Il titolo del prompt nel front-matter è preservato intatto.

## Compatibilità Obsidian / Foam

| Feature | Obsidian | Foam | Prompt a Porter |
|---|---|---|---|
| Front-matter YAML | ✓ | ✓ | ✓ (subset) |
| Wikilinks `[[foo]]` | ✓ | ✓ | ✗ (usa `{{import "foo"}}`) |
| Tags `#tag` | ✓ | ✓ | ✗ (sistema tag separato) |
| `tags:` in front-matter | ✓ | ✓ | ignorato in import (nessuna perdita sul file) |

Il round-trip Obsidian → PaP → Obsidian preserva: title, body, struttura cartelle. Perde: tags front-matter, aliases (recuperabili manualmente).

## Limiti noti

- **Niente import di intere cartelle**: si selezionano file, non directory. Per una libreria con sottocartelle, ripeti la selezione multipla cartella per cartella.
- **Front-matter malformato**: se il front-matter non si chiude (manca il secondo `---`), l'intero contenuto viene trattato come body, con il titolo preso dal nome file.
- **File grandi** (> 1 MB): supportati, ma il parser non è ottimizzato per `.md` enormi.
- **Tags e aliases di Obsidian** non diventano tag del vault: vengono ignorati in import (il file, però, non viene alterato).

## Vedi anche

- [`prompt-componibili.md`](./prompt-componibili.md) — la sintassi `{{import "..."}}` con `with` e `version`, che sopravvive al round-trip.
- [`cartelle.md`](./cartelle.md) — la struttura cartelle del vault, che l'export preserva nello zip.
- [`formato-export-json.md`](./formato-export-json.md) — l'alternativa JSON con schema versionato, per backup e migrazioni complete di metadati.
