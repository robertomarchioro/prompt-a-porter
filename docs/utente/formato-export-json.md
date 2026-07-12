# Formato export JSON

> **Versione corrente**: `1`
> **Status**: stabile, breaking change richiede bump `schemaVersion`

Il formato JSON di export è il contratto pubblico del vault Prompt a Porter. Permette portabilità completa dei dati utente — nessun lock-in. È pensato per essere generato e consumato sia dall'app desktop sia da tool esterni (CLI, script, migrazioni da/verso altri tool).

## Filosofia

- **Round-trip lossless**: export → import → confronto must produce dati equivalenti
- **Versioning esplicito**: `schemaVersion` cambia solo per breaking change
- **Forward compatibility**: campi opzionali (Fase 3+: `folder_id`, `target_model`) presenti come `null` o omessi se non popolati
- **Backward compatibility**: import di v1 deve continuare a funzionare in versioni future del formato

## Schema v1

```json
{
  "schemaVersion": 1,
  "exportedAt": "2026-05-03T18:00:00Z",
  "workspace": {
    "id": "ws-personale",
    "name": "Personale",
    "type": "personal"
  },
  "prompts": [
    {
      "id": "prm-abc123",
      "title": "Riassunto bug report",
      "description": "Distilla un report tecnico in 3 punti chiave",
      "body": "Analizza il seguente bug report:\n\n{{bug}}\n\nProduci 3 punti.",
      "visibility": "private",
      "target_model": "claude-sonnet",
      "folder_id": null,
      "is_favorite": false,
      "use_count": 12,
      "last_used_at": "2026-05-02T14:30:00Z",
      "version": 3,
      "created_at": "2026-04-01T10:00:00Z",
      "updated_at": "2026-05-01T09:00:00Z",
      "tag_ids": ["tag-bug", "tag-eng"]
    }
  ],
  "versions": [
    {
      "id": "pv-abc123-001",
      "prompt_id": "prm-abc123",
      "version": 1,
      "title": "Riassunto bug report",
      "description": "Distilla un report tecnico",
      "body": "Analizza il bug:\n\n{{bug}}",
      "visibility": "private",
      "target_model": null,
      "created_at": "2026-04-01T10:00:00Z",
      "created_by_user_id": "usr-locale"
    }
  ],
  "tags": [
    {
      "id": "tag-bug",
      "name": "bug",
      "color": "#dc2626",
      "created_at": "2026-04-01T09:00:00Z"
    }
  ],
  "folders": [],
  "global_placeholders": [
    {
      "name": "autore",
      "value": "Mario Rossi"
    }
  ]
}
```

## Campi

### Top-level

| Campo | Tipo | Note |
|---|---|---|
| `schemaVersion` | integer | Versione del formato. Attualmente `1`. |
| `exportedAt` | string ISO 8601 | Timestamp dell'export. UTC. |
| `workspace` | object | Metadata del workspace esportato. |
| `prompts` | array | Tutti i prompt del workspace (esclusi tombstoned). |
| `versions` | array | Tutte le versioni storiche di tutti i prompt esportati. |
| `tags` | array | Tag del workspace. |
| `folders` | array | Cartelle del workspace, ordinate per `path` (parent prima dei figli). Ricreate dall'import → round-trip lossless. |
| `global_placeholders` | array | Segnaposti globali del vault, oggetti `{name, value}`. Campo opzionale: assente nei vecchi export (retrocompatibile, default lista vuota). |

### `prompts[]`

Struttura di un singolo prompt:

| Campo | Tipo | Note |
|---|---|---|
| `id` | string | ID univoco, formato `prm-{16hex}`. |
| `title` | string | Titolo human-readable. |
| `description` | string \| null | Descrizione breve. |
| `body` | string | Corpo del template, può contenere `{{segnaposti}}`. |
| `visibility` | enum | `private` \| `workspace`. |
| `target_model` | string \| null | Modello AI target (es. `claude-sonnet`). Anticipato da Fase 3. |
| `folder_id` | string \| null | ID cartella di appartenenza (`null` = root). Se la cartella referenziata non è presente nell'import, il prompt va a root. |
| `is_favorite` | boolean | Flag preferiti. |
| `use_count` | integer | Contatore d'uso. |
| `last_used_at` | string ISO 8601 \| null | Ultimo uso. |
| `version` | integer | Versione corrente (testa). Inizia da `1`. |
| `created_at` | string ISO 8601 | Creazione. |
| `updated_at` | string ISO 8601 | Ultimo aggiornamento. |
| `tag_ids` | string[] | Array di ID tag associati (referenziano `tags[].id`). |
| `parent_prompt_id` | string \| null | Se è una **variante**, ID del prompt principale (referenzia un altro `prompts[].id`). Opzionale: assente negli export pre-varianti. |
| `is_variant` | boolean | `true` se il prompt è una variante. Default `false` se assente. |
| `variant_label` | string \| null | Etichetta della variante (es. `B`, `C`). |
| `fork_of_prompt_id` | string \| null | Se è un **fork**, ID del prompt originale. La tracciabilità sopravvive alla cancellazione dell'originale. |

> I riferimenti `parent_prompt_id` e `fork_of_prompt_id` puntano ad altri prompt dell'export. In import vengono risolti in una seconda passata (i prompt sono inseriti tutti prima); se il prompt referenziato non è presente né già nel vault, il riferimento resta `null`.

### `versions[]`

Struttura di una versione storica:

| Campo | Tipo | Note |
|---|---|---|
| `id` | string | ID univoco, formato `pv-{20hex}`. |
| `prompt_id` | string | FK al prompt corrente. |
| `version` | integer | Numero versione. |
| `title`, `description`, `body`, `visibility`, `target_model` | come in `prompts[]` | Snapshot dei campi alla versione N. |
| `created_at` | string ISO 8601 | Quando la versione è stata creata. |
| `created_by_user_id` | string | ID utente che ha generato la versione. |

### `tags[]`

| Campo | Tipo | Note |
|---|---|---|
| `id` | string | ID univoco, formato `tag-{...}`. |
| `name` | string | Nome del tag. |
| `color` | string \| null | Colore hex `#rrggbb` opzionale. |
| `created_at` | string ISO 8601 | Creazione. |

### `folders[]`

Le cartelle sono incluse nell'export e ricreate dall'import (round-trip lossless). Sono ordinate per `path` così che ogni cartella padre preceda i propri figli (requisito per il vincolo `parent_folder_id`). In import, se un `parent_folder_id` non è risolvibile (es. export di un solo sotto-albero), la cartella viene creata come root.

```json
{
  "id": "fld-abc",
  "parent_folder_id": null,
  "name": "Marketing",
  "path": "/Marketing",
  "created_at": "...",
  "updated_at": "..."
}
```

## Modalità di import

L'import accetta una modalità di gestione dei conflitti:

| Modalità | Comportamento su ID già esistente |
|---|---|
| `skip` | Mantiene la versione locale, ignora quella in import |
| `overwrite` | Sostituisce la versione locale con quella in import |
| `rename` | Crea una copia del prompt importato con nuovo ID + suffisso `(importato)` |

## Garanzie e limiti

- **Encoding**: sempre UTF-8.
- **Date**: sempre ISO 8601 con timezone Z (UTC). Timestamp locali NON ammessi nel formato.
- **Audit log**: NON è incluso nell'export (rimane locale per workspace personali; per workspace team passa per il server).
- **Vault metadata**: NON è incluso (chiavi crittografiche, salt, parametri Argon2 — sono dati di sicurezza, non di contenuto).
- **Allegati / file binari**: non supportati. Se un prompt fa riferimento a file esterni, il riferimento è solo testuale.
- **Dimensione massima**: nessun limite hard, ma consigliato `< 50 MB` per import singolo (parser deve materializzare tutto in memoria).

## Esempi di uso

### Backup completo del workspace

```
Da app desktop: Impostazioni → Dati, card "Esporta Vault → JSON"
Salva il file generato in posizione sicura (cifrata se sensibile).
```

### Migrazione da altro tool

Convertire l'export del tool sorgente nello schema v1 sopra, poi import via Impostazioni → Dati, card "Importa JSON", con modalità `skip` (sicura) o `rename` (esplorativa).

### Round-trip di sicurezza

```
1. Esporta JSON
2. Cancella un prompt nel vault
3. Importa lo stesso JSON con modalità `skip` (non sovrascrive nulla esistente)
   o `overwrite` (recupera la versione persa)
```

## Versioning del formato

- v1 (corrente): schema descritto sopra
- v2 (futuro): bump richiesto solo per breaking change. Campi aggiuntivi non rompono v1.
- Cambi non-breaking compatibili con v1: aggiungere nuovi campi opzionali ai prompt/versions, aggiungere nuovi top-level keys non documentati.

L'app deve **rifiutare** import con `schemaVersion` superiore a quello supportato (forward incompatibility) e **leggere** import con `schemaVersion` inferiore (backward compatibility).
