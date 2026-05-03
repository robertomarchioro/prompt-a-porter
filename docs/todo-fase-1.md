# Todo — Fase 1 (MVP)

> Aggiornato al: 2026-05-03 (Step 11)

## Step 0 — Bootstrap repo
- [x] Inizializza repo con `LICENSE` GPL 2.0, `README.md`, `.gitignore`
- [x] Setup pnpm workspace, layout directory completo
- [x] Copia `design_handoff/` nel repo
- [x] Setup GitHub Actions baseline (lint check)

## Step 1 — Setup client Tauri + Svelte
- [x] Scaffolding manuale Tauri 2 + Svelte 5 + TypeScript (più preciso di create-tauri-app)
- [x] Aggiungi dipendenze: CodeMirror 6, Lucide Svelte, vitest
- [x] Importa `tokens.css` in entry point (index.html + src/styles/)
- [ ] Verifica build su Win, macOS, Linux ⚠️ richiede `pnpm install` + toolchain locale
- [x] Setup `tauri.conf.json` per: tray icon, global shortcut, multiple windows (libreria + palette)
- [x] Struttura directory Svelte: components, superfici, stores, lib_template, lib_vault, lib_sync, i18n
- [x] File i18n: it.json + en.json con tutte le stringhe delle 8 superfici
- [x] Rust: Cargo.toml, lib.rs con tray + menu contestuale, capabilities ACL
- [x] Icone: SVG sorgenti (Lucide `braces`, ISC license) + tool HTML per generare PNG
- [ ] Genera PNG finali con `genera-icone.html` o `pnpm tauri icon` ⚠️ richiede browser/toolchain

## Step 2 — Setup vault SQLite + SQLCipher
- [x] Integra `rusqlite` con `bundled-sqlcipher` (rimosso `tauri-plugin-sql`)
- [x] Schema iniziale V001: 8 tabelle + FTS5 + 8 indici
- [x] Migration system: SQL embedded via `include_str!()`, tabella `_Migrazioni`
- [x] Comando Tauri `vault_crea(password)` — genera salt, Argon2id, crea DB
- [x] Comando Tauri `vault_unlock(password)` — deriva chiave, apre DB, verifica
- [x] Comando Tauri `vault_lock()` — chiude connessione
- [x] Comando Tauri `vault_cambia_password(old, new)` — re-key con nuovo salt
- [x] Comandi ausiliari: `vault_esiste()`, `vault_aperto()`
- [x] Tipo errore `PapErrore` serializzabile per Tauri
- [x] Test: creazione, unlock, password errata, re-key, hex roundtrip, migrazioni idempotenti
- [x] Documentazione schema in `docs/schema-dati.md` con diagramma ER Mermaid
- [ ] Test con `cargo test` ⚠️ richiede toolchain Rust locale

## Step 3 — Componenti UI base (porting design)
- [x] Porta 16 primitive da app.css a componenti Svelte 5 con props tipizzate
      Button, Input, Textarea, Select, Field, Switch, Kbd, Tag, Badge,
      Placeholder, NavItem, ListItem, EmptyState, Toast, Skeleton, Tooltip
- [x] Barrel export in `components/index.ts`
- [x] Classi utility globali in app.css (.eyebrow, .muted, .row, .spacer, ecc.)
- [x] Pagina demo `?demo` con switch tema/tono per test visivo dark/light × 3 toni
- [x] Accessibilità: aria-checked (Switch), aria-current (NavItem), aria-selected (ListItem),
      aria-invalid (Input/Textarea), role=status (Toast), focus-visible ring, keyboard nav
- [ ] Verifica visiva nel browser ⚠️ richiede `pnpm dev`

## Step 4 — Onboarding
- [x] Wizard 3 step (Profilo → Password vault → Hotkey)
- [x] Strength meter password (4 livelli, calcolo entropia)
- [x] Salva profilo e config in file prefs (preferenze.json via Tauri command)
- [x] Componenti: StrengthMeter, ProfileCard, HotkeyInput
- [x] Supporto vault non cifrato (opzione "Salta cifratura")
- [x] "Salta tour" con vault non cifrato e preferenze default
- [x] Navigazione tastiera (Enter=avanti, Esc=reset hotkey)
- [x] Rust: modulo preferenze con preferenze_carica/preferenze_salva
- [x] Rust: vault_crea_aperto, vault_cifrato, vault_unlock aggiornato

## Step 5 — Tray icon + global hotkey
- [x] Implementa tray con glifo SVG (da Step 1, PNG generation deferred)
- [x] Menu contestuale (5 voci: palette, nuovo, libreria, impostazioni, esci)
- [x] Registra hotkey globale (Rust-side con parse_hotkey, toggle palette show/hide)
- [x] Comando registra_hotkey per cambio a runtime (onboarding, impostazioni)
- [x] Toggle palette: show+center+focus / hide, Escape chiude palette
- [x] Caricamento hotkey da preferenze.json all'avvio
- [ ] Test hotkey in background ⚠️ richiede `pnpm dev` + toolchain

## Step 6 — Command Palette (Variante B Raycast)
- [x] Window frameless dedicata (da Step 1, 640×480 alwaysOnTop)
- [x] Componente CommandPalette.svelte con due modalità (ricerca/compila)
- [x] Fuzzy search FTS5 con sanitizzazione query e prefix matching (word*)
- [x] Navigation tastiera (↑↓ naviga, Enter seleziona, Escape chiudi/indietro)
- [x] Espansione inline form segnaposti (parser template.ts, form auto-generato)
- [x] Ctrl+Enter = compila e copia negli appunti (Web Clipboard API)
- [x] Anteprima live del prompt compilato
- [x] Rust: modulo prompt con prompt_cerca (recenti + FTS5 MATCH)
- [x] TypeScript: lib/template.ts (estraiSegnaposti, compila, contaCompilati)

## Step 7 — Libreria (finestra principale)
- [x] Layout 3 pannelli responsive (CSS Grid 240px / 360px / 1fr + status bar 28px)
- [x] Sidebar workspace switcher + viste (Recenti/Preferiti/Tutti) + visibilità + tag dinamici
- [x] Lista centrale con search debounced, sort (recente/popolare/A-Z), prompt card con icone visibilità
- [x] Pannello dettaglio con header, azioni (preferiti/modifica/compila), body preview con placeholder pill, parametri grid
- [x] Status bar con sync dot, versione, hotkey combo
- [x] Rust: modulo libreria con 5 comandi (conteggi, lista, dettaglio, toggle_preferito, tag_lista)
- [x] Rust: seed dati base (workspace Personale + utente locale) su creazione/unlock vault
- [x] Query parametrizzate con named_params (LIKE + tag JOIN opzionali, nessun SQL injection)
- [x] Schermata unlock vault per vault cifrati (auto-unlock per non cifrati)
- [x] Tempo relativo italianizzato (ora, Nm fa, Nh fa, ieri, Ng fa, data breve)

## Step 8 — Editor prompt
- [x] Modale 2 colonne (960×720, grid 1.3fr/1fr, scrim + backdrop-filter)
- [x] CodeMirror 6 con highlight {{...}} (ViewPlugin + Decoration.mark, baseTheme)
- [x] Parser segnaposti live ($derived da estraiSegnaposti sul body CM)
- [x] Tabella segnaposti reattiva (pills nella colonna destra, aggiornamento live)
- [x] Tag picker con autocomplete (carica tag esistenti, suggerimenti filtrati, Enter/Backspace)
- [x] Switch privato/team (toggle segmentato con stato visuale attivo)
- [x] Anteprima rendering ({@html renderPreview} con XSS-safe escaping)
- [x] Autosave con debounce (2s, solo prompt esistenti, indicatore stato)
- [x] Hotkey editor (Ctrl+S salva, Escape chiude)
- [x] Rust: modulo editor con prompt_crea, prompt_aggiorna, prompt_elimina
- [x] Rust: sincronizza_tags (find-or-create) + ricostruisci_fts (delete-all + bulk INSERT)
- [x] Libreria: bottone "+ Nuovo", Modifica abilitato, {#key} per stato fresco editor

## Step 9 — Renderer / Compilatore
- [x] Vista 2 colonne (form 1fr + preview 1.2fr, modale 880×640)
- [x] Form auto-generato dai segnaposti (input per ciascun placeholder, label uppercase mono)
- [x] Progress bar compilazione (barra orizzontale animata, contatore X/Y)
- [x] Highlight valori sostituiti (span .val-inserito verde, .ph viola per non compilati)
- [x] Toggle output Testo / Markdown / JSON (segmented control, JSON con model e parameters)
- [x] Copy to clipboard + toast (navigator.clipboard + Toast component success)
- [x] Conteggio token approssimativo (~chars/4, font-mono in basso a destra)
- [x] Rust: prompt_registra_uso (incrementa UseCount + LastUsedAt)
- [x] Libreria: bottone Compila abilitato, {#key} per stato fresco compilatore

## Step 10 — Impostazioni
- [x] Layout sidebar (200px) + content con modale 800×600
- [x] 7 sezioni: Account, Sync, Hotkey, Aspetto, Vault, Lingua, Info
- [x] Hotkey: HotkeyInput con registra_hotkey + salva preferenze
- [x] Vault: percorso (copia), cifratura info, cambio password (form 3 campi), elimina (doppia conferma)
- [x] Aspetto: tema scuro/chiaro + tono zinc/slate/stone con segmented control e live preview
- [x] Lingua: toggle it/en con salvataggio preferenze
- [x] Rust: vault_percorso, vault_elimina + registrazione in lib.rs
- [x] Libreria: NavItem Impostazioni collegato, inizializzazione tema da preferenze

## Step 11 — Setup server Go
- [x] Scaffolding cmd/papsync con chi router (go.mod, main.go, chi + middleware)
- [x] Schema SQLite server (compatibile con client + SyncChangelog per delta tracking)
- [x] Endpoint /auth/login con Argon2id + JWT (+ /auth/refresh)
- [x] Endpoint /sync/pull e /sync/push (delta query, last-write-wins conflict detection)
- [x] WebSocket /ws (auth via query param token, broadcast per workspace)
- [x] Dockerfile multistage (golang:1.23-alpine → alpine:3.20, CGO_ENABLED=1)
- [x] Test integrazione sync (12 test: auth, sync push/pull, conflict, idempotenza, password hash)
- [ ] Test con `go test ./...` ⚠️ richiede toolchain Go locale

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
