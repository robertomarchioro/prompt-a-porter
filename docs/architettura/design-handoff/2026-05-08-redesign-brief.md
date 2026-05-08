# Prompt a Porter — Brief per redesign UX/UI

> **Versione del prodotto al momento del brief**: `v0.7.0` (2026-05-08).
> **Status**: prodotto funzionante, single-user, AGPL, distribuito come app
> desktop cross-OS via Tauri. Stratificato in 7 fasi/sprint successivi —
> il brief nasce dall'esigenza di una review organica della UI prima del
> prossimo round di feature.

## 1. Sintesi esecutiva

**Prompt a Porter** (PaP) è un **prompt manager desktop locale-first** per
chi lavora con LLM. Memorizza prompt in un vault SQLCipher cifrato sul
disco dell'utente (no cloud forzato), li organizza in cartelle/tag,
supporta versioning + diff, ricerca ibrida lessicale+semantica
client-side, linting, varianti A/B, fork, regression testing con
**golden examples** contro 5 provider AI (Anthropic, OpenAI,
OpenAI-compat, Ollama, Gemini), e import componibili
(`{{import "path"}}`).

Il prodotto ha **18+ superfici Svelte** stratificate in 7 minor release
(`v0.1` → `v0.7`) con ~95 PR. Ogni superficie funziona, ma l'esperienza
complessiva si è accumulata per addizione: il brief chiede al designer
di **rivedere l'IA, il visual hierarchy e i pattern di interazione**
in modo organico.

## 2. Persona target / use case

> **Da validare con il designer**. Le ipotesi qui sotto sono da
> confermare o rifinire in fase di kickoff.

**Persona primaria — Single power user / dev**:
- Sviluppatore o knowledge worker che usa LLM quotidianamente
  (Claude Desktop, Cursor, ChatGPT, modelli locali via Ollama).
- Gestisce **decine-centinaia di prompt** ricorrenti (ruolo expert,
  templates con variabili, helper di codice, email).
- Vuole **versionare** i prompt come fa col codice (cronologia, diff,
  varianti per A/B).
- Si aspetta **privacy** (vault locale cifrato, no upload),
  **portabilità** (export JSON/Markdown, AGPL) e
  **reliability** (regression testing per non regredire al cambio
  modello AI).

**Persona secondaria — Team marketing / content (Fase 5, non in scope ora)**:
- Workspace condivisi, approval workflow, RBAC su cartelle.
- Oggi non in produzione: il client supporta `visibility: private |
  workspace` ma lo Step "team" è rinviato a Fase 5.

**Use case principali**:
- "Ho 50 prompt sparsi tra ChatGPT, Notion, file di testo: voglio un
  posto unico, versionato, ricercabile."
- "Voglio sapere se il mio prompt 'ruolo customer service' funziona
  ancora con Claude 4.7 come funzionava con 4.6 — regression test."
- "Voglio condividere i miei prompt come file `.md` o JSON, anche
  con front-matter Jekyll/Hugo per pubblicarli su un blog statico."

## 3. Pain points specifici

- layout a colonne fisse: deve essere possibile nascondere la colonna di sinistra o ridimensionarla
- non è possibile collassare le categorie nella colonna di sinistra (a.e. TAG, Modello Target, Cartelle)
- seconda colonna "elenco prompt" poco amichevole, introdurre opzione da "impostazioni" per vedere solo titolo del prompt e tag oppure anche anteprima delle prime "n" righe del prompt stesso
- area di lavoro a destra poco utile con funzioni elencate in alto a destra ma in maniera anonima
- funzioni di creazione o modifica che si realizano in una modale troppo piccola e poco adatta a scrivere e gestire prompt anche lunghi e complessi: se possibile eliminare la modale e integrare tutte le funzioni di modifica enlla colonna di destra
- sfruttare la barra di stato della finistra per inserire qualche informazione sul prompt
- in tutte le cronologie di versioni manca l'autore della modifica

## 4. Goal e successo del redesign

- il nuovo redesign deve usare meglio lo spazio dando priorità alla semplicità di utilizzo del sistema e alla superficie dedicata a gestire i testi dei prompt e alle funzioni di ottimizzazione, lasciare uso delle modali solo alle funzioni per cui ha un senso, come impostazioni, insights e quello che riterrai più opportuno

## 5. Architettura di massima

```
┌─────────────────────────────────────────────────────────────┐
│  Client desktop Tauri 2 (cross-OS: Linux / macOS / Windows) │
│                                                              │
│  ┌──────────────────┐         ┌──────────────────────────┐ │
│  │  Frontend        │ ←IPC→   │  Backend Rust            │ │
│  │  Svelte 5 + Vite │         │  (Tauri commands)        │ │
│  │  CodeMirror 6    │         │                          │ │
│  │  Stato: $state/  │         │  • Vault SQLCipher AES   │ │
│  │  $effect (runes) │         │  • SQLite + sqlite-vec   │ │
│  │                  │         │  • ONNX Runtime + MiniLM │ │
│  │  Process:        │         │  • Provider AI HTTP      │ │
│  │  - WebView       │         │  • Audit log             │ │
│  │  - Tray icon     │         │                          │ │
│  └──────────────────┘         └──────────────────────────┘ │
│                                          ↕                  │
│                                   File system locale:       │
│                                   - pap-vault.db (cifrato) │
│                                   - vault-meta.json        │
│                                   - models/ (MiniLM ONNX)  │
│                                   - onnxruntime/ (lib)     │
└─────────────────────────────────────────────────────────────┘
                                   ↓ (HTTP, opzionale)
┌─────────────────────────────────────────────────────────────┐
│  Server di sync `papsync` (Rust, Axum)                      │
│  Workspace team. Oggi NON in produzione (Fase 5).           │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  MCP Server (Node.js, npm @pap/mcp-server)                  │
│  Espone il vault a Claude Desktop / Cursor / agenti AI.     │
│  Oggi: solo trasporto stdio. HTTP/SSE in roadmap.           │
└─────────────────────────────────────────────────────────────┘
```

**Caratteristiche chiave**:
- **Single-binary native** (no Electron). Bundle ~50 MB Linux/macOS,
  ~30 MB Windows.
- **Locale-first**: niente account obbligatorio, niente sync cloud
  forzato. Il sync server è opt-in.
- **AGPL-3.0**: codice open-source, anche il MCP server.
- **Schema dati additivo**: 13 migrations V001-V013, no breaking change
  fra release.

## 6. Soluzione tecnica adottata

| Layer | Tecnologia | Razionale |
|---|---|---|
| **Runtime app** | Tauri 2 | Single-binary nativo, ~10x più leggero di Electron, supporto cross-OS, IPC type-safe verso Rust |
| **UI framework** | Svelte 5 (runes) | Compile-time reactivity, no virtual DOM, bundle minimal. Runes (`$state`/`$effect`) per stato locale fine-grained |
| **Code editor** | CodeMirror 6 | Editing prompt body con 3 extension custom: `placeholder-highlight` (segnaposti `{{var}}`), `lint-markers` (issue inline), `import-tokens` (hover tooltip + Ctrl+click su `{{import "..."}}`) |
| **DB locale** | SQLite + SQLCipher AES-256 | Vault cifrato. Argon2id per derivazione chiave da password utente. `sqlite-vec` extension per ricerca vettoriale |
| **Embeddings** | ONNX Runtime + sentence-transformers MiniLM L12 v2 multilingua | Embeddings calcolati **client-side** (no upload). 384-dim. Auto-unload dopo idle (free RAM ~150 MB) e riload on-demand |
| **Ricerca** | FTS5 + vec0 fusion via Reciprocal Rank Fusion (RRF) | Lessicale + semantica fuse con `alpha` configurabile in Impostazioni |
| **Linting** | Regex-based, 11 regole (LEN/PH/PII/STY/IMP) | Eseguito on-demand al editing. Toggle per categoria in Impostazioni → Linter |
| **Provider AI** | Trait `AIProvider` con 5 impl (Anthropic, OpenAI, OpenAI-compat, Ollama, Gemini) | API key cifrate nel vault SQLCipher. UI Provider Config in Impostazioni |
| **Diff** | jsdiff (npm `diff` 9.x, BSD-3) | Side-by-side + inline, preserve segnaposti `{{...}}` come token unitari |
| **Build / packaging** | tauri-action su GitHub Actions | Build cross-OS automatica al push del tag `v*`. Asset: `.dmg` + `.app.tar.gz` (macOS aarch64), `.deb` + `.rpm` + `.AppImage` (Linux), `.msi` + `setup.exe` + portable `.zip` (Windows) |
| **Test** | cargo unit + vitest + cargo-llvm-cov coverage gate | 416 test backend, 17 frontend, gate CI 70% line |

**File chiave**:
- Frontend: `apps/client/src/lib/superfici/*.svelte` (18 file),
  `apps/client/src/lib/components/*.svelte` (primitives), `apps/client/src/lib/codemirror/*.ts` (extension)
- Backend: `apps/client/src-tauri/src/*.rs` (~30 moduli), `migrations/V*.sql`

## 7. Elenco feature utente

Raggruppate per macro-area. Ogni feature è funzionante in `v0.7.0`.

### 7.1 Vault & sicurezza
- Crea vault cifrato con password (Argon2id) o vault aperto (no password)
- Sblocco vault al lancio
- Cambio password, eliminazione vault, idle-unlock dopo timeout
- **Audit log** di ogni operazione (creazione/modifica/eliminazione/export/import) con filtri ed export CSV
- Sync opzionale verso server `papsync` (workspace team — Fase 5)

### 7.2 Libreria prompt
- Lista prompt con: viste **Recenti / Preferiti / Tutti / Privati / Team**
- **Cartelle nidificate** (drag-and-drop dei prompt fra cartelle)
- **Tag** con colori, suggerimenti automatici basati su embeddings
- **Filtro per modello target** (Claude, GPT, Gemini, Llama, ecc.)
- **Filtro per ricerca testuale** (sidebar)
- **Ordinamento**: Recenti / Popolari / Migliori (rating medio 90gg) / A-Z
- **Esporta singola cartella** (sotto-albero JSON)

### 7.3 Editor prompt
- CodeMirror 6 con highlight di:
  - `{{var}}` segnaposti (azzurro)
  - `{{import "path"}}` token (sottolineato + tooltip + Ctrl+click)
  - Issue linter inline (sottolineato wavy: errore rosso, warning giallo, info blu)
- **Autosave** (debounce 2s) + **versioning** (ogni save crea versione)
- Metadata: titolo, descrizione, body, **visibilità** (privato/team), **modello target** (combo preset + free-text), cartella, **tag**
- Pannelli collapsible: **Diagnosi** (issue linter), **Test golden**, **Cronologia versioni**
- **+ Variante** dall'editor (crea variante B/C/Z dello stesso intent)
- **+ Golden** (crea caso di test)

### 7.4 Compilazione & uso
- **CompilatorePrompt** modale: form variabili rilevate → preview body compilato
- Formato output: **Markdown / JSON / Plain text**
- **Token estimation** (~4 char per token)
- **Copia clipboard** + auto-incremento `UseCount` + `LastUsedAt`
- **Rating post-uso**: toast 👎/😐/👍, modale opzionale per nota su voto negativo/neutro
- **Espansione import**: toggle per espandere `{{import "x"}}` ricorsivamente

### 7.5 Versioning & confronto
- **Cronologia prompt**: lista versioni con metadata
- **Diff fra versioni**: side-by-side colorato + inline + Markdown export
- **Confronto multi-prompt**: Cmd/Ctrl+click su 2+ prompt in Libreria → vista N-colonne
- **Confronta tutte varianti**: 1-click per vedere principale + tutte le varianti side-by-side
- **Fork**: clona un prompt team nel proprio workspace privato (banner "Fork di X" cliccabile per navigare all'originale)

### 7.6 Ricerca
- **Ricerca lessicale** FTS5 (titolo + body + descrizione)
- **Ricerca semantica** MiniLM client-side (similarità coseno top-K)
- **Ricerca ibrida** RRF con `alpha` configurabile
- Auto-unload Session ONNX dopo idle (configurabile), riload on-demand al primo uso post-drop

### 7.7 Linting (11 regole)
- **LEN001**: body troppo lungo (> 4000 char)
- **LEN002**: body troppo corto (< 30 char)
- **PH001**: segnaposto malformato `{x}` invece di `{{x}}`
- **PH003**: caratteri speciali nel nome del segnaposto
- **PII001**: email rilevate
- **PII003**: numero carta di credito (Luhn check)
- **PII004**: API key sospette (`sk-`, `Bearer`, ecc.)
- **STY001**: ripetizione n-gram
- **IMP001**: import non risolto
- **IMP002**: ciclo di import
- **IMP003**: profondità eccessiva
- **IMP004**: "questo prompt è importato da N altri" (info, cross-prompt linting)
- **Toggle per categoria** in Impostazioni → Linter (LEN/PH/PII/STY/IMP)

### 7.8 Golden examples & regression testing
> Differenziatore strategico: nessun altro prompt manager ha questo.

- **Crea golden** su un prompt: `input_vars` JSON + `expected_output` + `similarity_fn` (`cosine` / `exact-match` / `regex` / `llm-judge`) + soglia
- **Esegui golden** contro un provider AI scelto (Ollama default, Anthropic/OpenAI/Gemini/OpenAI-compat con API key)
- **Esegui tutti i golden** in batch sequenziale con summary `passed/failed/error`
- **Vista Regressioni**: tabella drift per `(prompt × provider × model)`, periodo 7/30/90/365 giorni, export CSV
- 5 provider AI configurabili da Impostazioni → Provider AI

### 7.9 Insight / statistiche
- Totali (prompt attivi/eliminati, tag, creati/aggiornati ultimi 30g, versioni)
- Top usati (UseCount + LastUsedAt)
- Candidati a cleanup (inattivi > 90 giorni)
- Distribuzione per tag / modello target / visibilità
- **Top prompt più importati** (grafo inverso `PromptImports`)
- **Lint health %**: % prompt senza issue + breakdown top 5 categorie
- Tutto client-side, nessun dato esce dal vault

### 7.10 Import / export
- **Export JSON** intero workspace (schema v1)
- **Export JSON cartella** (sotto-albero, con prompt + tag associati)
- **Import JSON** con modalità conflict resolution: `skip` / `overwrite` / `rename`
- **Export Markdown singolo prompt** con YAML front-matter (compatibile Jekyll/Hugo, include `imports: [...]` parsati dal body)

### 7.11 Componibilità
- Sintassi `{{import "path"}}` per importare un altro prompt come blocco
- Risoluzione path: `cartella/titolo` (case-insensitive) o `titolo` solo per root
- Compilazione ricorsiva con detection cicli + max depth (10)
- **Hover tooltip** mostra titolo + snippet body del prompt importato
- **Ctrl/Cmd+click** apre il prompt importato

### 7.12 Sync workspace (opt-in)
- Login server `papsync` con email + password
- Push/pull delta automatico ogni N secondi (configurabile)
- Conflict resolution UI (`ConflittoSync.svelte`)
- Logout + cleanup credenziali

### 7.13 MCP server
- Trasporto **stdio** verso Claude Desktop / Cursor / agenti
- Tool: `pap_search`, `pap_get`, `pap_compile` (versioning Fase 5)
- Trasporto HTTP/SSE rinviato a Fase 5

### 7.14 Onboarding & utilità
- **OnboardingWizard** prima apertura: scelta cifrato/non, password, prompt esempio
- **Command palette** (Ctrl/Cmd+Shift+P) per aprire prompt rapido
- **Tray icon** con scorciatoie (ridotto in background)
- **Ricerca semantica** abilitabile da Impostazioni (richiede download modello ~150 MB al primo uso)
- **Tema dark/light/auto**, **tono accent** configurabile, **lingua** (oggi solo italiano)

## 8. Mappa schermate (Information Architecture)

> Screenshot delle superfici saranno forniti a parte (cartella separata,
> versione Windows). Qui l'albero strutturale e le relazioni di apertura.

### Albero superfici

```
ROOT (single window, Tauri)
│
├── OnboardingWizard          [primo avvio, no vault esistente]
├── AuthLogin                 [vault cifrato esistente]
├── AuthResetPassword
├── AuthRecuperaWorkspace
│
└── Libreria (root surface)   [post-unlock, sempre disponibile]
    │
    ├── Sidebar
    │   ├── Workspace switcher (Personale / sync server)
    │   ├── Viste: Recenti / Preferiti / Tutti / Privati / Team
    │   ├── Tree Cartelle (drag-and-drop, context actions: + / ✎ / ⬇ / 🗑)
    │   ├── Lista Tag con colore + count
    │   ├── Filtro Modello Target
    │   ├── Link → Insight (modale)
    │   └── Link → Regressioni (modale)
    │
    ├── Lista prompt (centrale)
    │   ├── Search bar
    │   ├── Sort dropdown (Recenti / Popolari / Migliori / A-Z)
    │   ├── Bottone "+ Nuovo"
    │   └── Cards prompt (titolo, descrizione, badge visibilità, tag, conteggio uso)
    │
    └── Detail pane (destra)
        ├── Titolo + azioni: ★ / Modifica / Cronologia / + Variante / Fork / Esporta MD / Compila
        ├── Descrizione
        ├── Banner "Fork di X" (se prompt è un fork)
        ├── Pillole varianti + bottone "Confronta tutte"
        ├── Body preview (renderizzato)
        ├── Lista segnaposti rilevati
        ├── Tags + autocomplete
        └── Metadata (modello target, cartella, ecc.)

Modali sopra Libreria (z-index overlay):
├── EditorPrompt              [Modifica / Nuovo]
│   ├── Header: titolo + bottoni "+ Variante" / ✕
│   ├── Errore variante (banner)
│   ├── Colonna editor (CodeMirror)
│   │   └── Pannelli collapsible:
│   │       ├── Diagnosi (lista issue linter)
│   │       └── Test golden (config Ollama + lista golden + bottone "Esegui tutti")
│   └── Colonna metadata (descrizione, segnaposti, target, cartella, tag, visibilità)
│
├── CompilatorePrompt
│   ├── Form variabili (rilevate dal body)
│   ├── Toggle "Espandi import"
│   ├── Preview body compilato
│   ├── Selettore formato (Markdown / JSON / Plain)
│   ├── Token count
│   ├── Bottoni "Annulla / Copia"
│   └── Toast rating post-copia (👎 / 😐 / 👍)
│       └── Modale "Aggiungi nota" (su voto negativo/neutro)
│
├── ConfrontoPrompt           [N-colonne side-by-side]
├── CronologiaPrompt          [versioni + diff side-by-side / inline]
├── Insight                   [stats card + bar chart]
├── Regressioni               [tabella drift + filtri periodo + export CSV]
├── ConflittoSync             [solo se sync server attivo + conflict]
└── Impostazioni              [10 sezioni in sidebar]
    ├── Account
    ├── Sincronizzazione
    ├── Scorciatoie (hotkey global)
    ├── Aspetto (tema, tono accent, lingua)
    ├── Vault (info, cambio password, idle-unload)
    ├── Ricerca semantica (download modello, alpha RRF)
    ├── Provider AI (5 card configurabili)
    ├── Linter (5 toggle categoria)
    ├── Registro attività (audit log)
    ├── Lingua
    └── Informazioni

Window separate (overlay finestra Tauri):
└── CommandPalette            [Ctrl/Cmd+Shift+P, hotkey global]
```

### Stato corrente: stratificazione

Le 18 superfici sono state aggiunte in **7 release minor successive**:

| Release | Superfici aggiunte | Pain point UX presunto |
|---|---|---|
| `v0.1` | OnboardingWizard, Libreria, EditorPrompt, CompilatorePrompt, AuthLogin | Base solida |
| `v0.2` | Impostazioni (account, sync, hotkey, vault, audit), ConflittoSync, AuthResetPassword | Sezioni Impostazioni accumulano |
| `v0.3` | CronologiaPrompt, ConfrontoPrompt, Regressioni, Insight, AuthRecuperaWorkspace, CommandPalette, ricerca semantica section in Impostazioni | Modali si moltiplicano |
| `v0.4` | Pannello Test golden in Editor, Sezione Provider AI in Impostazioni | Editor diventa denso |
| `v0.5` | Pannelli Provider Config UI, modale "+ Variante" in Editor, modale "Aggiungi nota" rating, batch golden runner, sort by quality, provider Gemini | Detail pane si riempie di bottoni |
| `v0.6` | Inline marker linter, statistiche Insight estese, vista Confronto varianti, sezione Linter in Impostazioni, riload Session on-demand | Diagnosi/Linter sovrapposti |
| `v0.7` | Bottone "+ Variante" + "Confronta tutte" + "Esporta MD" nel detail pane, bottone "Esporta cartella" in sidebar, datalist target model, hover/click import, regola IMP004 | Accumulo bottoni nel detail pane / actions row |

## 9. Differenziatori strategici (da comunicare nel design)

Il prodotto ha **4 cose che nessun competitor offre** o offre con
limitazioni evidenti. Oggi sono "nascoste" sotto pannelli o sezioni
secondarie. Il redesign dovrebbe alzare la loro visibilità:

1. **Regression testing con golden examples** — "Il tuo prompt è
   un contratto comportamentale verificabile". Trasforma un prompt
   in un test suite. Nessun altro prompt manager lo offre. Oggi vive
   in un pannello collapsible nell'editor + vista Regressioni.
2. **Embeddings client-side** (MiniLM L12 v2 multilingua, 384-dim) —
   ricerca semantica **senza upload**, no API key necessaria, ~150 MB
   download una volta. Oggi è una sezione tra le altre in Impostazioni.
3. **Vault SQLCipher locale + AGPL** — privacy by design, codice
   open-source verificabile, no lock-in cloud. Oggi non comunicato
   visivamente al di fuori dell'onboarding.
4. **Componibilità via `{{import "path"}}`** — i prompt sono
   compostabili come funzioni; cambi un import e si aggiornano tutti
   i derivati. Oggi visibile solo nei prompt che li usano (Ctrl+click
   navigation atterrato in v0.7).

## 10. Constraint tecnici e out-of-scope

### Constraint tecnici (NON modificabili senza ripianificazione)

- **Tauri 2 + Svelte 5**: window management diverso da web (overlay
  reali con `WebviewWindow`). CommandPalette è già una window separata.
  Modali sono in DOM, no `<dialog>`.
- **CodeMirror 6** per editing body: editor a riga aperta, no rich-text
  WYSIWYG. Decoration via plugin TS.
- **Schema dati additivo**: 13 migration V001-V013, breaking change
  vietati. Il redesign UI non deve richiedere cambi al modello dati.
- **Single-binary nativo**: bundle deve restare ~50 MB cross-OS,
  niente CDN runtime.
- **A11y baseline**: 41 svelte-check warning a11y residui (autofocus,
  role/tabindex su `<dialog>` div, label association). Da migliorare,
  non da regredire.
- **i18n**: oggi solo italiano. Strings sparse nei file Svelte (no
  i18n framework). Refactor i18n è scope futuro.

### Out-of-scope per il redesign

- **Schema dati e tabelle**: schema deciso, no rinegoziazione.
- **Hotkey esistenti** (`Ctrl/Cmd+Shift+P` palette, ecc.): l'utente
  ha memoria muscolare, non cambiare arbitrariamente.
- **Modello mentale "vault locale"**: il prodotto è locale-first.
  Non spingere flusso "crea account cloud first".
- **Ricerca semantica**: gate su download modello (150 MB) resta —
  proporre alternative grafiche al "scarica per attivare", ma il
  download esiste.
- **Auth team / workspace shared**: scope Fase 5, non disegnare per
  questo flusso ora (lo Step 6+7 di Fase 4 è rinviato).
- **Dark/light mode + tono accent**: il design system attuale ha già
  token CSS in `tokens.css`. Il redesign può proporre nuovi token,
  ma deve restare a token-based (no hardcoded colors).

## 11. Stato corrente: numeri rilevanti

| Metrica | Valore |
|---|---|
| Versione corrente | `v0.7.0` (2026-05-08) |
| Release pubbliche | 7 minor + 3 patch (`v0.1.x`, `v0.2.x`, ecc.) |
| Test backend | 416 unit (cargo) |
| Test frontend | 17 vitest |
| Coverage backend | 74.14% line / 77.69% function (gate CI 70%) |
| File Svelte (superfici) | 18 |
| File Svelte (primitives) | ~20 (`$lib/components/`) |
| Extension CodeMirror custom | 3 (placeholder, lint-markers, import-tokens) |
| Migrazioni DB | 13 (V001-V013) |
| Provider AI implementati | 5 (Anthropic, OpenAI, OpenAI-compat, Ollama, Gemini) |
| Regole lint | 11 attive (3 skippate per scope) |
| Bundle binario | ~50 MB Linux/macOS, ~30 MB Windows |
| Lingua UI | Italiano |
| License | AGPL-3.0-only |

## 12. Allegati / riferimenti

### Da fornire a parte (non in questo doc)

- **Screenshot** delle superfici principali (versione Windows).
  L'utente li passa al designer come bundle separato.

### Asset di riferimento già in repo

- `docs/architettura/design-handoff/` — bundle Fase 1 (HTML statici
  delle 8 schermate principali, design system con `tokens.css` e
  `tokens.json`, `app.css` con stili globali). **Stato Fase 1**:
  riflette la UI iniziale, NON il prodotto v0.7. Utile come baseline
  storica e come reference dei token CSS in uso.
- `docs/architettura/overview.md` — panoramica architetturale dei
  moduli backend e frontend.
- `docs/architettura/schema-dati.md` — schema SQLite del vault locale.
- `docs/architettura/design-system.md` — punto di accesso al bundle
  design Fase 1.
- `docs/utente/` — guide d'uso correnti (per capire mental model
  dell'utente esistente).

### Repo

- GitHub: `https://github.com/robertomarchioro/prompt-a-porter`
- Release: `https://github.com/robertomarchioro/prompt-a-porter/releases/tag/v0.7.0`
- Codice frontend: `apps/client/src/`
- Codice backend: `apps/client/src-tauri/src/`

---

**Compilato il 2026-05-08** in preparazione del redesign UX/UI
post-`v0.7.0`. Sezioni 3 e 4 da rifinire dall'utente prima della
consegna al designer.
