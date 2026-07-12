# Formato export JSON

> Lo schema del formato JSON di export: campi, gestione dei conflitti in import, garanzie di round-trip. **Versione corrente**: `1`.

I tuoi prompt ti appartengono, e un giorno potresti volerli portare altrove: su un'altra macchina, in un altro strumento, in uno script tutto tuo. Per questo il formato di export JSON è **documentato pubblicamente**, campo per campo: è la garanzia concreta che i tuoi dati non restano prigionieri dell'app. Nessun lock-in — se domani Prompt a Porter non ti servisse più, i tuoi dati sarebbero già in un formato che qualunque strumento sa leggere.

Questo formato è il contratto pubblico del vault: viene generato e consumato dall'app desktop, ma è pensato perché anche tool esterni — script, migrazioni da o verso altri strumenti — possano produrlo e leggerlo senza sorprese. Un file di export contiene tutto il contenuto del workspace: prompt con il loro storico versioni, tag, cartelle e segnaposti globali.

## Le garanzie del formato

Quattro promesse tengono in piedi il contratto:

- **Round-trip lossless**: esportare e poi reimportare deve produrre dati equivalenti, senza perdite.
- **Versioning esplicito**: il campo `schemaVersion` cambia solo per modifiche incompatibili (breaking change).
- **Compatibilità in avanti**: i campi opzionali compaiono come `null` o vengono omessi se non popolati; aggiungerne di nuovi non rompe il formato.
- **Compatibilità all'indietro**: un export v1 continuerà a essere importabile anche nelle versioni future del formato.

## Schema v1

Questo è un export completo in miniatura — un workspace con un prompt, una versione storica, un tag e un segnaposto globale:

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
| `prompts` | array | Tutti i prompt del workspace (esclusi quelli cancellati). |
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
| `target_model` | string \| null | Modello AI target (es. `claude-sonnet`). |
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
| `prompt_id` | string | Riferimento al prompt corrente. |
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

Le cartelle sono incluse nell'export e ricreate dall'import (round-trip lossless). Sono ordinate per `path` così che ogni cartella padre preceda i propri figli. In import, se un `parent_folder_id` non è risolvibile (es. export di un solo sotto-albero), la cartella viene creata come root.

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

Cosa succede se importi un file che contiene prompt già presenti nel vault? Lo decidi tu: nell'app, la card **Importa JSON** (in **Impostazioni → Dati**) offre tre modalità nel campo **Conflitti**:

| Modalità | Nell'app | Comportamento su ID già esistente |
|---|---|---|
| `skip` | **Salta esistenti** | Mantiene la versione locale, ignora quella in import |
| `overwrite` | **Sovrascrivi** | Sostituisce la versione locale con quella in import |
| `rename` | **Rinomina duplicati** | Crea una copia del prompt importato con nuovo ID + suffisso `(importato)` |

Se non sei sicuro, `skip` è la scelta prudente: non tocca nulla di esistente.

## Garanzie e limiti

- **Encoding**: sempre UTF-8.
- **Date**: sempre ISO 8601 con timezone Z (UTC). Timestamp locali NON ammessi nel formato.
- **Audit log**: NON è incluso nell'export; resta nel vault locale.
- **Vault metadata**: NON è incluso (chiavi crittografiche, salt, parametri di derivazione — sono dati di sicurezza, non di contenuto).
- **Allegati / file binari**: non supportati. Se un prompt fa riferimento a file esterni, il riferimento è solo testuale.
- **Dimensione massima**: nessun limite hard, ma consigliato `< 50 MB` per import singolo (l'import carica l'intero file in memoria).

## Esempi d'uso

### Backup completo del workspace

Dall'app desktop: **Impostazioni → Dati**, card **Esporta Vault → JSON**. Salva il file generato in una posizione sicura (cifrata, se il contenuto è sensibile). Un backup periodico di questo file è tutto quello che serve per poter ricostruire il workspace.

### Migrazione da un altro strumento

Se arrivi da un altro tool, converti il suo export nello schema v1 descritto sopra, poi importa da **Impostazioni → Dati**, card **Importa JSON**. Usa la modalità **Salta esistenti** (sicura) o **Rinomina duplicati** (esplorativa: vedi tutto, non perdi nulla).

### Round-trip di sicurezza

Un modo semplice per toccare con mano la garanzia di round-trip:

1. Esporta il JSON.
2. Cancella un prompt nel vault.
3. Reimporta lo stesso JSON: con **Salta esistenti** non viene sovrascritto nulla di esistente e il prompt cancellato torna al suo posto; con **Sovrascrivi** recuperi anche eventuali modifiche fatte per errore.

## Versioning del formato

- **v1** (corrente): lo schema descritto sopra.
- **v2** (eventuale futuro): il bump è richiesto solo per breaking change. Campi aggiuntivi non rompono v1.
- Cambi non-breaking compatibili con v1: nuovi campi opzionali su prompt/versioni, nuove chiavi top-level non documentate.

L'app **rifiuta** import con `schemaVersion` superiore a quello che supporta (non può garantire di interpretarli correttamente) e **legge** import con `schemaVersion` inferiore (i vecchi export restano validi per sempre).

## Vedi anche

- [`markdown-import-export.md`](./markdown-import-export.md) — l'alternativa a file Markdown: più leggibile, meno metadati.
- [`cli.md`](./cli.md) — leggere il vault da script e terminale, senza passare da un export.
