# Todo — Fase 1 (MVP)

> Aggiornato al: 2026-05-02

## Step 0 — Bootstrap repo
- [x] Inizializza repo con `LICENSE` GPL 2.0, `README.md`, `.gitignore`
- [x] Setup pnpm workspace, layout directory completo
- [x] Copia `design_handoff/` nel repo
- [x] Setup GitHub Actions baseline (lint check)

## Step 1 — Setup client Tauri + Svelte
- [ ] `pnpm create tauri-app` con template Svelte + TypeScript
- [ ] Aggiungi dipendenze: CodeMirror 6, Lucide Svelte, vitest
- [ ] Importa `tokens.css` in entry point
- [ ] Verifica build su Win, macOS, Linux
- [ ] Setup `tauri.conf.json` per: tray icon, global shortcut, multiple windows

## Step 2 — Setup vault SQLite + SQLCipher
- [ ] Integra SQLCipher (tauri-plugin-sql o rusqlite bundled-sqlcipher)
- [ ] Crea schema iniziale
- [ ] Migration system (versioned SQL files in src-tauri/migrations/)
- [ ] Comando Tauri `vault_unlock(password)`
- [ ] Comando Tauri `vault_lock()`
- [ ] Comando Tauri `vault_change_password(old, new)` con re-key
- [ ] Test vault: apertura, chiusura, password errata

## Step 3 — Componenti UI base (porting design)
- [ ] Porta primitive da app.css a componenti Svelte
- [ ] Pagina demo /components per test visivo dark/light e 3 toni
- [ ] Verifica accessibilità tastiera + focus ring + ARIA

## Step 4 — Onboarding
- [ ] Wizard 3 step (Profilo → Password vault → Hotkey)
- [ ] Strength meter password
- [ ] Salva profilo e config in file prefs

## Step 5 — Tray icon + global hotkey
- [ ] Implementa tray con glifo SVG
- [ ] Menu contestuale
- [ ] Registra hotkey globale
- [ ] Test hotkey in background

## Step 6 — Command Palette (Variante B Raycast)
- [ ] Window frameless dedicata
- [ ] Componente Svelte custom
- [ ] Fuzzy search FTS5
- [ ] Navigation tastiera
- [ ] Espansione inline form segnaposti
- [ ] Cmd/Ctrl+Enter = compila e copia

## Step 7 — Libreria (finestra principale)
- [ ] Layout 3 pannelli responsive
- [ ] Sidebar workspace switcher + viste + tag
- [ ] Lista centrale con search, sort, card
- [ ] Pannello dettaglio
- [ ] Status bar

## Step 8 — Editor prompt
- [ ] Modale 2 colonne
- [ ] CodeMirror 6 con highlight {{...}}
- [ ] Parser segnaposti live
- [ ] Tabella segnaposti reattiva
- [ ] Tag picker con autocomplete
- [ ] Switch privato/team
- [ ] Anteprima rendering
- [ ] Autosave con debounce
- [ ] Hotkey editor

## Step 9 — Renderer / Compilatore
- [ ] Vista 2 colonne (form + preview)
- [ ] Form auto-generato dai segnaposti
- [ ] Progress bar compilazione
- [ ] Highlight valori sostituiti
- [ ] Toggle output Markdown / Plain / JSON
- [ ] Copy to clipboard + toast
- [ ] Conteggio token approssimativo

## Step 10 — Impostazioni
- [ ] Layout sidebar + content
- [ ] Tutte le sezioni (Account, Sync, Hotkey, Aspetto, Vault, Lingua, Info)
- [ ] Hotkey input con registrazione
- [ ] Vault: cambio password, esporta/importa, elimina
- [ ] Tema/tono live preview

## Step 11 — Setup server Go
- [ ] Scaffolding cmd/papsync con chi router
- [ ] Schema SQLite server
- [ ] Endpoint /auth/login con Argon2id + JWT
- [ ] Endpoint /sync/pull e /sync/push
- [ ] WebSocket /ws
- [ ] Dockerfile multistage
- [ ] Test integrazione sync

## Step 12 — Auth e Sync client
- [ ] Schermate Login / Reset password / Recupera workspace
- [ ] Store sync con stato
- [ ] Polling sync + WS push
- [ ] Conflict UI

## Step 13 — Audit log
- [ ] Hook su operazioni di scrittura → AuditLog
- [ ] Vista admin in Impostazioni

## Step 14 — Quality gate
- [ ] Test coverage ≥ 70% moduli core
- [ ] Build cross-platform pulita (CI green)
- [ ] Smoke test manuale 8 superfici dark/light + 3 toni
- [ ] Test accessibilità screen reader

## Step 15 — Documentazione
- [ ] Aggiorna docs/
- [ ] Genera changelog
- [ ] Tag release v0.1.0-fase1
