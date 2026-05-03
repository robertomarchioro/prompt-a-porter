# Changelog — Prompt a Porter

## v0.1.0-fase1 (2026-05-03)

Prima release MVP. Tutte le funzionalità core implementate.

### Step 0 — Bootstrap repo
- Inizializzazione repo con LICENSE GPL 2.0, README, .gitignore
- Setup pnpm workspace monorepo (`apps/client`, `apps/server`, `packages/`)
- GitHub Actions baseline (lint check client + server)

### Step 1 — Client Tauri + Svelte
- Scaffolding Tauri 2 + Svelte 5 + TypeScript
- Configurazione multi-window (libreria 1200×800 + palette 640×480 frameless)
- Struttura directory: components, superfici, stores, i18n
- File i18n: it.json + en.json con stringhe per tutte le superfici
- Icone SVG sorgenti (Lucide `braces`)

### Step 2 — Vault SQLite + SQLCipher
- Integrazione `rusqlite` con `bundled-sqlcipher` (AES-256)
- Schema V001: 8 tabelle + FTS5 + 8 indici
- Migration system embedded via `include_str!()`
- Comandi: vault_crea, vault_unlock, vault_lock, vault_cambia_password
- Derivazione chiave Argon2id (m=32MiB, t=3, p=4)
- 7 test unitari

### Step 3 — Componenti UI base
- 16 primitive Svelte 5: Button, Input, Textarea, Select, Field, Switch, Kbd, Tag, Badge, Placeholder, NavItem, ListItem, EmptyState, Toast, Skeleton, Tooltip
- Classi utility globali in app.css
- Pagina demo `?demo` con switch tema/tono
- Accessibilità: aria attributes, focus-visible, keyboard nav

### Step 4 — Onboarding
- Wizard 3 step (Profilo → Password vault → Hotkey)
- Strength meter password (4 livelli, calcolo entropia)
- Supporto vault non cifrato ("Salta cifratura")
- Navigazione tastiera (Enter=avanti, Esc=reset)

### Step 5 — Tray icon + global hotkey
- Tray con menu contestuale (5 voci)
- Hotkey globale registrabile a runtime
- Toggle palette: show+center+focus / hide
- Caricamento hotkey da preferenze all'avvio

### Step 6 — Command Palette
- Window frameless dedicata, fuzzy search FTS5
- Navigazione tastiera (↑↓ naviga, Enter seleziona, Escape chiudi)
- Espansione inline form segnaposti
- Ctrl+Enter = compila e copia negli appunti

### Step 7 — Libreria
- Layout 3 pannelli CSS Grid (sidebar + lista + dettaglio)
- Sidebar con workspace switcher, viste, tag dinamici
- Lista con search debounced, sort (recente/popolare/A-Z)
- Status bar con sync dot, versione, hotkey

### Step 8 — Editor prompt
- Modale 2 colonne con CodeMirror 6
- Highlight {{segnaposti}} con ViewPlugin + Decoration
- Tag picker con autocomplete
- Autosave con debounce (2s)

### Step 9 — Compilatore
- Vista 2 colonne (form + preview)
- Form auto-generato dai segnaposti
- Progress bar compilazione
- Toggle output Testo / Markdown / JSON
- Copy to clipboard + toast

### Step 10 — Impostazioni
- Layout sidebar + content con 7 sezioni
- Hotkey configurabile con HotkeyInput
- Tema scuro/chiaro + tono zinc/slate/stone
- Gestione vault: percorso, cifratura, cambio password, elimina
- Toggle lingua it/en

### Step 11 — Server Go
- chi router con middleware (CORS, logger, JWT, recoverer)
- Schema SQLite server + SyncChangelog
- Auth: Argon2id + JWT HS256 (login + refresh)
- Sync: pull delta + push con last-write-wins
- WebSocket broadcast per workspace
- Dockerfile multistage (golang:1.23-alpine → alpine:3.20)
- 12 test di integrazione

### Step 12 — Auth e Sync client
- 3 schermate auth: Login, Reset password, Recupera workspace
- Store sync singleton (polling + WebSocket reconnect)
- Conflict UI con scelta locale/server per entità
- Preferenze estese con campi sync (serde default backward compat)
- Sezione Sync in Impostazioni con stato live

### Step 13 — Audit log
- Modulo `audit.rs` con `registra()` fire-and-forget
- Hook su editor, libreria, vault, sync (9 azioni tracciate)
- Vista "Registro attività" in Impostazioni con filtro per tipo
- Comando `audit_lista` con limite e filtro tipo entità

### Step 14 — Quality gate
- 37 test Rust su 8 moduli
- 22 test TypeScript per template.ts (vitest)
- CI aggiornata: job rust-test + vitest + coverage 70% server

### Step 15 — Documentazione
- Architettura completa con diagrammi e tabelle moduli
- Setup sviluppo con comandi e struttura directory
- Deploy produzione con Docker e variabili d'ambiente
- Prompt di ricostruzione con lezioni apprese
- Changelog completo
- API server aggiornata
