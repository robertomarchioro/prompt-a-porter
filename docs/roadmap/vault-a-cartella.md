# Blueprint — Vault "a cartella" (file plain-text, no lock-in)

> **Stato**: design / analisi d'impatto. Nessun codice ancora.
> **Obiettivo utente**: evitare il lock-in tecnologico e poter modificare i
> prompt con editor esterni (Obsidian, VS Code, vim, …).
> **Scope**: solo versione Personale (single-user, no server obbligatorio).

## Decisioni fondative (confermate dall'utente)

1. **Sidecar `.pap/`** per tutto ciò che non è contenuto puro (cronologia
   versioni, rating, palette tag, segnaposti globali). Niente delega a git:
   PaP resta agnostico, l'utente può comunque versionare la cartella con git
   se vuole. Massimo no-lock-in.
2. **Modalità a livello di vault, scelta in creazione**: o **tutto SQLite
   cifrato** (come oggi) o **tutto a cartella**. Mutuamente esclusive, niente
   modalità mista. All'avvio della procedura di creazione si chiede all'utente
   quale vuole, mostrando pro/contro di ciascuna.
3. **Vault a cartella = tutto in chiaro**: nessuna cifratura possibile (è il
   prezzo per poter editare i file da fuori).

## Principio architetturale

> Nel vault a cartella, **i file `.md` + il sidecar `.pap/` sono la sorgente
> di verità**. SQLite NON sparisce: resta come **cache/indice derivato e
> ricostruibile** per full-text (FTS5) e ricerca semantica (sqlite-vec).
> Anche l'app scrive attraverso i file (single writer channel).

Conseguenza: non si perde nulla di ciò che differenzia PaP (FTS, semantica,
atomicità restano, perché SQLite c'è ancora come proiezione). La cache è
**rigenerabile**: cancellandola e ri-scansionando la cartella si ricostruisce.

### Dove vive la cache SQLite (importante)

La cache derivata **non** sta dentro `.pap/` sulla cartella, ma nella app-data
locale del SO, indicizzata per path del vault. Motivo: un file SQLite su uno
share di rete (SMB/NFS) ha locking inaffidabile → rischio corruzione (doc
ufficiale SQLite). Tenendo la cache **locale** e i dati di contenuto come file
plain-text (safe su share), si elimina il footgun: la cartella di rete contiene
solo testo, mai il `.db`.

## Cosa vive dove

| Dato | Vault SQLite (oggi) | Vault a cartella |
|---|---|---|
| Titolo, descrizione, body, visibility, target_model | colonne Prompts | front-matter + body del `.md` |
| Cartella di appartenenza | `FolderId` / Path | **percorso del file** nella directory tree |
| Tag (nome) | PromptTags + Tags | `tags:` nel front-matter |
| Tag (id + colore) | Tags | `.pap/tags.json` (palette) |
| Variante / fork (riferimenti) | self-FK | `parent_prompt_id` / `fork_of_prompt_id` nel front-matter |
| Preferito | colonna | `favorite:` front-matter |
| Cronologia versioni | PromptVersions | `.pap/versions/<id>/<n>.md` (snapshot) |
| Rating | PromptRatings | `.pap/ratings.jsonl` (append) |
| Segnaposti globali | tabella | `.pap/globals.json` |
| Audit log | AuditLog | `.pap/audit.jsonl` (append) |
| use_count / last_used | colonne | `.pap/stats.json` (derivato runtime) |
| FTS index | PromptsFts | cache SQLite locale (rigenerata) |
| Embeddings | sqlite-vec | cache SQLite locale (rigenerata) |

## Schema file `.md` (esteso, identity-aware)

Estende il front-matter M6 attuale rendendolo **non-lossy** e con **identità
stabile**. Esempio:

```markdown
---
id: prm-a1b2c3d4e5f6a7b8        # stabile, sopravvive a rename/spostamento
title: "Email professionale"
description: "Template email formale parametrica"
target_model: "claude-sonnet-4-6"
visibility: private
favorite: true
tags: ["email", "lavoro"]        # nomi; id+colore in .pap/tags.json
variant_label: null              # se variante: es. "B"
parent_prompt_id: null           # se variante: id del principale
fork_of_prompt_id: null          # se fork: id dell'originale
---

Ciao {{nome}},

{{import "firme/standard"}}
```

Regole di **identità** (il problema più delicato dell'editing esterno):
- File **senza `id`** → nuovo prompt: si assegna un `id` e si **riscrive il
  file** con l'id dentro.
- File con **`id` esistente** → *upsert* (aggiorna, non duplica). È la
  differenza chiave rispetto all'import M6 attuale (che fa sempre insert →
  duplicati).
- **`id` duplicato** (file copia-incollato) → il secondo è trattato come copia:
  nuovo id + riscrittura.
- Riferimento (`parent`/`fork`) a id inesistente → `null` (come già fa
  l'import lossless #186).
- Cartella = **percorso del file**. Spostare un file fra directory = spostare
  il prompt di cartella.

## Layout sidecar `.pap/`

```
<vault>/
├─ Scrittura/
│  ├─ email-professionale.md
│  └─ Email/
│     └─ cold-outreach.md
├─ Sviluppo/
│  └─ code-review.md
└─ .pap/
   ├─ tags.json          # [{id, name, color}]
   ├─ globals.json       # {autore: "...", azienda: "..."}
   ├─ versions/
   │  └─ prm-a1b2.../    # 1.md, 2.md, 3.md (snapshot per versione)
   ├─ ratings.jsonl      # 1 riga per voto {prompt_id, rating, note, created_at}
   ├─ audit.jsonl        # append-only
   └─ stats.json         # {prm-...: {use_count, last_used_at}}
```

Tutto JSON/JSONL/Markdown → leggibile, diffabile, safe su share di rete.

## Flusso di creazione vault (onboarding)

Schermata di scelta con pro/contro espliciti:

### Opzione A — Vault SQLite cifrato (default attuale)
- ✅ Cifratura a riposo (SQLCipher + Argon2id): i prompt non sono leggibili
  senza password.
- ✅ Atomicità transazionale, integrità garantita.
- ✅ Performance ricerca/semantica native, un solo file da gestire.
- ❌ Dati "chiusi" nel `.db`: per portarli fuori servono export.
- ❌ Non editabile con strumenti esterni.

### Opzione B — Vault a cartella (file plain-text)
- ✅ Nessun lock-in: i prompt sono `.md` leggibili/versionabili ovunque.
- ✅ Editabili con Obsidian / VS Code / qualsiasi editor.
- ✅ Backup/sync con i tuoi strumenti (git, Dropbox, NAS).
- ❌ **Nessuna cifratura**: contenuto in chiaro su disco/share.
- ❌ Editing esterno concorrente da più PC = possibili conflitti.
- ❌ Il `.db` di cache (locale, derivato) va comunque ricostruito alla prima
  apertura su una nuova macchina.

## Punti di tocco nel codice

| Area | Intervento | Sforzo |
|---|---|---|
| `import_export.rs` | front-matter esteso (`id`, `tags`, variant/fork); parser di import *upsert* su `id` | medio (cuore del lavoro) |
| nuovo `vault_cartella.rs` (o `mirror.rs`) | engine cartella: scan, serializza-su-file, ingest, watcher (`notify`), debounce | medio-alto |
| `vault.rs` + `VaultMeta` | concetto di **modalità vault** (`sqlite-encrypted` \| `folder-plaintext`); apertura "punta a cartella" | medio |
| `.pap/` sidecar | spec + lettura/scrittura `tags.json` / `globals.json` / `versions/` / `ratings.jsonl` / `stats.json` | basso-medio |
| bootstrap/cache | "ricostruisci cache SQLite dai file" all'apertura + su evento file | medio |
| `editor.rs` (write path) | salvataggio in-app → scrive il `.md` (+ sidecar) → re-ingest | basso |
| `embeddings` / `ricerca_ibrida` | trigger re-index su evento file; nessuna nuova logica di ricerca | basso |
| Onboarding (Svelte) | schermata scelta modalità con pro/contro; file picker cartella | medio |
| `audit`, `rating`, `versioning` | astrarre la destinazione (tabella SQLite vs sidecar) dietro la modalità | medio |

> Nota: serve un punto di astrazione sulla **destinazione di scrittura** per i
> dati non-contenuto (versioni/rating/audit/globali), così lo stesso comando
> scrive su tabella (vault SQLite) o su sidecar (vault cartella). È l'unico
> pezzo di "Repository-like" che il codebase oggi non ha; va introdotto in
> modo mirato, non come refactor generale.

## Fasi (incrementali, ognuna utile da sola)

- **F1 — MD lossless + identity-aware**: `id` + tag + variant/fork nel
  front-matter; import che fa *upsert* sull'id invece di sempre-insert.
  *(Già qui: "edita un file e reimporta senza duplicare". Migliora anche il
  round-trip MD per i vault SQLite esistenti.)*
- **F2 — Modalità vault + export cartella completo**: scelta in onboarding;
  serializzazione di un intero vault come directory tree + `.pap/`.
  *(Backup leggibile/versionabile: copre gran parte del "no lock-in".)*
- **F3 — Vault a cartella "vivo"**: cache SQLite locale derivata, ingest
  all'apertura, watcher per re-ingest automatico su edit esterno.
  *(Il vero editing esterno live.)*
- **F4 — Robustezza & conflitti**: id duplicati/orfani, file malformati,
  debounce, report conflitti, sanitizzazione nomi file, validazione input
  non fidato (cicli `{{import}}`, cap anti-bomba, path traversal, YAML).

## Rischi e vincoli

- **Sicurezza**: contenuto in chiaro per design (accettato). I file esterni
  sono **input non fidato** → riusare i controlli dell'import (cicli import,
  `MAX_OUTPUT_BYTES`, path traversal nel mapping cartelle, front-matter YAML
  malformato).
- **SQLite su rete = corruzione**: la cache `.db` resta **locale**, mai sullo
  share. Sulla cartella di rete solo testo.
- **Concorrenza multi-PC**: due macchine che editano lo stesso file insieme =
  conflitto last-write-wins. È il problema che il sync server (team) risolve;
  nel vault a cartella si gestisce con detection conflitti (F4) e si documenta
  il limite. Git, se l'utente lo usa sulla cartella, fa il merge.
- **Interazione con sync server team**: vault a cartella è **Personale-only** e
  mutuamente esclusivo col sync server. Da chiarire in UI (un vault non può
  essere insieme "a cartella" e sincronizzato col server team).
  La mutua esclusività vale anche per il futuro sync multi-device
  [Ordito](./ordito-sync-log.md): un vault a cartella non può essere
  sincronizzato via oplog (i `.md` replicati dal file-syncer e l'oplog
  trasporterebbero la stessa modifica su due canali → echo e due
  conflict-resolver in disaccordo). La cartella di *trasporto* `.ordito/` di
  un vault SQLite può invece stare su qualunque share: non è questa cartella.
  Un'eventuale riconciliazione (`.md` come proiezione read-mostly) è un
  design futuro separato (v. Punti aperti di Ordito).
- **Performance**: vault grandi → watcher con debounce + ingest incrementale
  per hash/mtime, non full-rescan ad ogni evento.

## Cosa NON fare

- Non trasformare PaP in un client git (commit/push restano all'utente).
- Non sostituire SQLite con un backend a file puro (perderebbe FTS + semantica
  + atomicità): SQLite resta come cache anche nel vault a cartella.
- Non mettere il `.db` sullo share di rete.
- Non introdurre una modalità mista per-prompt: la scelta è per-vault.

## Decisioni chiuse (2026-06-03)

1. **Naming file = slug leggibile** (es. `email-professionale.md`). L'identità
   reale sta nell'`id` del front-matter, quindi il nome file è **cosmetico**:
   - rename esterni innocui (l'`id` preserva l'identità);
   - collisioni di slug risolte con suffisso `-2`, `-3`;
   - cambio titolo in-app → rename del file per coerenza; cambio titolo
     *esterno* → non si forza il rename (no churn/sorprese, l'`id` resta la
     verità). Drift titolo↔nome file è accettato.
   - Scartato: `id` come nome file (stabile ma illeggibile, annulla lo scopo).

2. **Migrazione = comando wrapper, mai mutazione in-place.** "Converti vault"
   serializza il contenuto verso una **nuova** cartella (serializer della F2,
   piena fedeltà incluso `.pap/`) e la riapre; al contrario, ingest in un
   **nuovo** vault SQLite. Un solo code path (lo stesso serializer/ingest),
   zero mutazione distruttiva sullo storage esistente.
   - Scartato: conversione in-place (mutazione rischiosa, recovery complesso).

3. **Cancellazione `.pap/` = degradazione graceful.** Il contenuto `.md`
   sopravvive sempre; all'apertura l'app ricrea `.pap/` e la cache:
   - tag ri-derivati dai nomi presenti nel front-matter (colore default);
   - versioni / rating / globali / audit / stats persi (ripartono da zero);
   - warning una tantum all'utente.
   - **Bonus**: aprire una cartella Obsidian/Foam *qualsiasi* "funziona" — il
     `.pap/` nasce al primo ingest. È una proprietà no-lock-in forte (nessun
     formato proprietario richiesto per iniziare).
   - Scartato: bloccare l'apertura chiedendo ripristino (attrito alto, contro
     lo spirito "apri qualsiasi cartella").
