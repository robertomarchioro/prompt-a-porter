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

| Modulo | Responsabilità | Comandi Tauri |
|--------|---------------|---------------|
| `vault.rs` | Cifratura SQLCipher, Argon2id, lifecycle DB | 10 (esiste, aperto, crea, crea_aperto, cifrato, unlock, lock, cambia_password, percorso, elimina) |
| `editor.rs` | CRUD prompt, tag sync, FTS rebuild | 4 (crea, aggiorna, registra_uso, elimina) |
| `libreria.rs` | Query lista/dettaglio, preferiti, tag, seed dati base | 5 (conteggi, lista, dettaglio, toggle_preferito, tag_lista) |
| `prompt.rs` | Ricerca FTS5, sanitizzazione query | 1 (cerca) |
| `sync.rs` | Applicazione delta dal server | 1 (applica_delta) |
| `audit.rs` | Audit trail fire-and-forget | 1 (lista) |
| `preferenze.rs` | JSON prefs con serde default | 2 (carica, salva) |
| `migrazione.rs` | Schema versioning embedded | 0 (interno) |
| `errore.rs` | Tipo errore unificato serializzabile | 0 (tipo) |
| `lib.rs` | Tray, hotkey, routing comandi | 1 (registra_hotkey) |

**Totale: 25 comandi Tauri + 1 hotkey**

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
