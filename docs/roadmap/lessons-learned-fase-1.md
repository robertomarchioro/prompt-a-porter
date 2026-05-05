# Prompt di Ricostruzione — Prompt a Porter

> Lezioni apprese durante l'implementazione della Fase 1.
> Utile per ricostruire il progetto da zero o per onboarding nuovi sviluppatori.

## Sequenza ottimale di build

L'ordine degli step si è rivelato corretto e produce dipendenze minime tra step:

1. **Bootstrap** — repo, workspace pnpm, CI baseline
2. **Vault SQLCipher** — fondamenta: senza DB non c'è nulla da testare
3. **Componenti UI** — primitive riusabili prima delle superfici
4. **Onboarding** — primo flusso end-to-end (wizard → vault → prefs)
5. **Tray + Hotkey** — infrastruttura background
6. **Command Palette** — la killer feature, usa FTS5 dal vault
7. **Libreria** — finestra principale, dipende da vault + componenti
8. **Editor** — CRUD prompt, dipende da libreria
9. **Compilatore** — usa template parser, dipende da editor
10. **Impostazioni** — aggrega tutto: prefs, vault, hotkey, tema
11. **Server Go** — indipendente dal client, può essere sviluppato in parallelo
12. **Auth + Sync client** — collega client al server
13. **Audit** — trasversale, hook su tutti i moduli di scrittura
14. **Quality gate** — test e CI
15. **Documentazione** — riflette lo stato finale

## Scelte che hanno funzionato

- **`rusqlite` con `bundled-sqlcipher`** invece di `tauri-plugin-sql`: controllo totale su PRAGMA key/rekey, migrazioni custom, query parametrizzate con named_params
- **FTS5 contentless con rebuild** (`delete-all` + bulk INSERT): semplice, robusto, nessun trigger da mantenere
- **`#[serde(default)]`** per backward compatibility delle preferenze: aggiungere campi sync non rompe i JSON esistenti
- **Fire-and-forget audit** (`let _ =`): l'audit non blocca mai le operazioni primarie
- **Svelte 5 runes ovunque**: nessun mix con API legacy, tutto coerente
- **Singleton sync store** con callback `onChange`: evita il problema delle runes a top-level di moduli
- **`pub(crate)`** per `ricostruisci_fts`: condivisione cross-modulo senza esporre nell'API pubblica

## Gotcha da ricordare

- **WebSocket auth**: non si può usare l'header `Authorization` nel handshake WS del browser, serve query param `?token=`
- **CGO per Docker**: il container Alpine ha bisogno di `gcc musl-dev` per compilare SQLite/SQLCipher via cgo
- **Argon2id formato compatibile**: client Rust e server Go usano lo stesso formato `$argon2id$v=19$m=...,t=...,p=...$salt$hash` — i parametri devono coincidere
- **Tauri capabilities ACL**: i comandi custom (`#[tauri::command]`) non hanno bisogno di entries nelle capabilities, quelle servono solo per i plugin built-in
- **`MaxOpenConns=1`** nel server Go: SQLite non supporta scritture concorrenti, un singolo pool connection evita `SQLITE_BUSY`
