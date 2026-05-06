# Architettura — Prompt a Porter

## Panoramica

```
┌─────────────────────────────────────────────────────┐
│               App Desktop (Tauri 2)                 │
│                                                     │
│  ┌──────────────────┐    ┌───────────────────────┐  │
│  │   Svelte 5 UI    │    │     Rust Core         │  │
│  │                  │    │                       │  │
│  │  8 superfici:    │    │  vault.rs    SQLCipher │  │
│  │  - Libreria      │◄──┤  editor.rs   FTS5     │  │
│  │  - Palette       │    │  libreria.rs Argon2id │  │
│  │  - Editor        │ ──►│  prompt.rs   Tray     │  │
│  │  - Compilatore   │IPC │  sync.rs     Hotkey   │  │
│  │  - Impostazioni  │    │  audit.rs    Prefs    │  │
│  │  - Onboarding    │    │  migrazione.rs        │  │
│  │  - Auth (3)      │    │  preferenze.rs        │  │
│  └──────────────────┘    └───────────────────────┘  │
│         ▲                         │                  │
│         │ WebView (HTML/CSS/JS)   │ SQLite file      │
│         │                         ▼                  │
│         │                  ┌─────────────┐           │
│         │                  │ pap-vault.db│           │
│         │                  │ (SQLCipher) │           │
│         │                  └─────────────┘           │
└─────────┼───────────────────────────────────────────┘
          │
          │ REST + WebSocket (opzionale)
          │
┌─────────▼──────────────────────────┐
│       Server Sync (Go)             │
│                                    │
│  chi router + middleware chain     │
│  - POST /auth/login  (Argon2id)   │
│  - POST /auth/refresh (JWT)       │
│  - GET  /sync/pull   (delta)      │
│  - POST /sync/push   (LWW)       │
│  - GET  /ws          (broadcast)  │
│                                    │
│  SQLite (WAL) + SyncChangelog      │
└────────────────────────────────────┘
```

## Moduli Rust (client)

| Modulo | Responsabilità | Note |
|--------|---------------|---------------|
| `vault.rs` | Cifratura SQLCipher, Argon2id, lifecycle DB | |
| `editor.rs` | CRUD prompt, tag sync, FTS rebuild, hook embedding | embedding al save |
| `libreria.rs` | Query lista/dettaglio, preferiti, tag, seed dati base | |
| `prompt.rs` | Ricerca FTS5, sanitizzazione query | path legacy lessicale |
| `ricerca_ibrida.rs` | RRF pesata FTS5 + sqlite-vec, search ibrida | Fase 3 |
| `cartelle.rs` | Folders gerarchici, Path denormalizzato, sposta/rinomina cascata | Fase 3 |
| `embeddings.rs` | ONNX Runtime + tokenizer + idle-unload Session | Fase 3 |
| `embeddings_store.rs` | Wrapper sqlite-vec (vec0), upsert/search nearest | Fase 3 |
| `embeddings_backfill.rs` | Riprocessa prompt/tag senza embedding in batch | Fase 3 |
| `tags_suggest.rs` | Suggeritore tag semantico + fallback frequenza | Fase 3 |
| `linting.rs` | 11 regole lint (LEN/PH/PII/STY/IMP) body-only + DB-aware | Fase 3 |
| `prompt_componibili.rs` | Parser/resolver `{{import "path"}}` con cycle/depth | Fase 3 |
| `statistiche.rs` | Aggregazione passiva top/non-usati/per-tag | Fase 3 |
| `versioning.rs` | Snapshot storico in `PromptVersions`, rollback | |
| `import_export.rs` | Round-trip JSON formato v1 | |
| `sync.rs` | Applicazione delta dal server | |
| `audit.rs` | Audit trail fire-and-forget | |
| `preferenze.rs` | JSON prefs con serde default | |
| `migrazione.rs` | Schema versioning embedded | V001-V007 |
| `errore.rs` | Tipo errore unificato serializzabile | |
| `lib.rs` | Tray, hotkey, routing comandi, idle-unload thread | |

## Superfici Svelte

| Superficie | Finestra | Dimensione | Note |
|-----------|----------|------------|------|
| Libreria | libreria | 1200×800 | 3 pannelli CSS Grid, finestra principale |
| CommandPalette | palette | 640×480 | Frameless, alwaysOnTop, fuzzy search FTS5 |
| Editor | modale su libreria | 960×720 | CodeMirror 6, tag picker, autosave |
| Compilatore | modale su libreria | 880×640 | Form auto-generato, anteprima live |
| Impostazioni | modale su libreria | 800×600 | 8 sezioni sidebar, incluso audit log |
| Onboarding | modale su libreria | — | Wizard 3 step, prima esecuzione |
| AuthLogin | modale su libreria | 440px | Login server sync |
| AuthResetPassword | modale su libreria | 440px | Reset password |
| AuthRecuperaWorkspace | modale su libreria | 440px | Cerca workspace |
| ConflittoSync | modale su libreria | 600px | Risoluzione conflitti locale/server |

## Decisioni architetturali

### Local-first
Il vault SQLite cifrato è la fonte di verità. Il server sync è opzionale e aggiunge solo la condivisione team. L'app funziona completamente offline.

### SQLCipher per cifratura
AES-256 trasparente a livello di pagina SQLite. La chiave viene derivata dalla password utente con Argon2id e mai persistita su disco. Solo il salt è salvato in `vault-meta.json`.

### FTS5 contentless
La tabella `PromptsFts` usa `content=''` per non duplicare dati. Viene ricostruita (`delete-all` + bulk `INSERT`) dopo ogni operazione di scrittura. Semplice e robusto per dataset fino a ~10k prompt.

### Last-write-wins sync
Il conflict resolution usa il timestamp `UpdatedAt`. Se il server ha un dato più recente del client push, il dato viene rifiutato. Il client riceve i conflitti e li mostra in una UI dedicata.

### Fire-and-forget audit
L'audit log usa `let _ = conn.execute(...)` — errori di scrittura audit vengono ignorati silenziosamente. L'audit non deve mai bloccare l'operazione primaria.

### Svelte 5 runes
Tutto il frontend usa la nuova API runes (`$state`, `$derived`, `$effect`, `$props`, `$bindable`). Nessun uso delle API legacy (writable, readable, derived store).

### Singleton sync store
Il modulo `lib/sync.ts` usa stato privato a livello di modulo con pattern callback (`syncOnChange`), non un Svelte store, perché le runes non sono utilizzabili a top-level di moduli non-componente.

### Ricerca ibrida (Fase 3)

```
Query utente
     │
     ├──────► FTS5 MATCH + ORDER BY rank ──► top-K lessicali (rank_lex)
     │
     │       compute_embedding (MiniLM ONNX)
     ├──────► search_nearest in vec0 ──────► top-K semantici (rank_sem)
     │
     ▼
Reciprocal Rank Fusion pesata
score(d) = (1-α)/(60 + rank_lex) + α/(60 + rank_sem)
     │
     ▼
top-N risultati ordinati
```

`α` configurabile dall'utente (0=solo FTS, 1=solo semantico, default 0.5).
Fallback graceful: se la Session ort non è caricata, `cerca_semantica`
ritorna `vec![]` e il punteggio degenera a solo FTS — la ricerca
continua a funzionare.

### Embedding lifecycle (Fase 3)

```
Boot client
     │
     ▼
auto-init Session (se modello+runtime su disco)
     │
     ▼
Session in RAM (~150 MB)
     │
     ├── compute al save di prompt/tag (hook editor)
     ├── compute al backfill bulk
     └── compute alle ricerche/suggerimenti
     │
     ▼
Background thread (ogni 30s)
controlla last_used vs idle_unload_secondi pref
     │
     ▼
se idle > soglia → drop Session (libera RAM)
```

### Resolver import (Fase 3)

`{{import "path"}}` → parser regex → `resolve_path` (cartella+titolo
o solo titolo, fallback case-insensitive) → DFS ricorsivo con
`HashSet<String>` di id visitati per cycle detection + depth check
contro `MAX_DEPTH=5`. Tabella `PromptImports` aggiornata su ogni
save del prompt parent (popolata da `aggiorna_imports`), permette
query "chi importa X" senza scan dei body.
