# Handoff: Prompt a Porter — Redesign UX/UI

> **Versione**: redesign v0.8 · **Data**: 2026-05-08 · **Origine**: brief in `docs/architettura/design-handoff/2026-05-08-redesign-brief.md` · **Design system base**: `design_handoff_promptvault/` (Fase 1)

## Overview

Redesign completo dell'app desktop **Prompt a Porter** (libreria locale di prompt LLM, macOS, local-first). Il redesign affronta tutti i pain point della Sezione 3 del brief e riconfigura l'app intorno a un **layout 3-pannelli editor-first** invece dell'architettura attuale a 8 superfici disgiunte.

I cambiamenti principali rispetto alla Fase 1:

- **3 pannelli ridimensionabili e collassabili** (sidebar nav · lista prompt · detail/editor) — il prompt viene editato in-place, non più in modale.
- **Sidebar gerarchica** con gruppi collassabili: Viste · Visibilità · Cartelle · Tag · Modello target · workspace switcher in alto.
- **Lista a densità variabile**: Compatta / Comoda / Anteprima con N righe (slider).
- **Detail pane a tabs**: Editor · Anteprima · Diagnosi · Test golden · Cronologia · Import & Varianti — invece di renderer/test/regressioni come schermate separate.
- **Toolbar markdown nell'editor** (B, I, H1, H2, liste, citazione, codice, link, hr) + token `{{var}}` e `import` come pill cliccabili.
- **Right-rail metadati contestuali** (visibilità, target, cartella, tag, segnaposti rilevati, import composti, varianti A/B) con toggle Meta separato.
- **Status bar funzionale**: vault locale cifrato, prompt corrente, stato salvataggio, scorciatoia palette.
- **Modali solo dove servono**: Compila, Insight, Regressioni, Impostazioni, Command Palette.

---

## About the Design Files

I file in `prototype/` sono **design reference creati in HTML** — un prototipo cliccabile che mostra l'intent visivo e di interazione, **non production code da copiare 1:1**.

Il task è **ricreare questi design nell'ambiente del codebase target** (Tauri + React, secondo brief Sezione 5) usando i pattern e le librerie già stabilite, oppure scegliere il framework più appropriato se l'environment non è ancora consolidato.

Il prototipo è scritto come pochi file `.jsx` caricati via Babel standalone — è una scaffolding di lavoro, non un'architettura raccomandata.

## Fidelity

**High-fidelity**. Il prototipo usa i token finali (vedi `prototype/tokens.css`, copia di Fase 1), tipografia finale (Inter + JetBrains Mono), spacing finale, copy in italiano definitivo, stati interattivi (hover/active/dirty/saved), modali completamente popolate.

Da reinterpretare nel framework target con fedeltà pixel.

---

## Files in `prototype/`

| File | Ruolo |
|------|-------|
| `redesign.html` | Entry point — apri in browser per vedere il prototipo |
| `redesign.css` | Tutti gli stili del redesign (~1090 righe). Token via CSS variables da `tokens.css` |
| `tokens.css` | Design tokens (colori, spaziatura, tipografia) — copiato da `design_handoff_promptvault/` |
| `app.jsx` | Componente root, state, layout 3-pannelli, DetailPane, StatusBar, hook tweaks |
| `components.jsx` | TitleBar, Sidebar, NavGroup, NavItem, ListPane, PromptCard, DetailTabs, MockEditor, RightRail |
| `modals.jsx` | CompilaModal, InsightModal, RegressioniModal, ImpostazioniModal, Palette |
| `icons.jsx` | Set inline SVG (tutto disegnato a mano, no librerie esterne) |
| `data.jsx` | Mock data: prompts, tag, target, history, lint, goldens |
| `tweaks-panel.jsx` | Tweaks (toggle live di tema/densità/layout, persistenti) |

Per aprirlo: `open prototype/redesign.html` (richiede connessione a unpkg.com per React + Babel).

---

## Tema, tipografia, token

Tutti i token sono già definiti in `prototype/tokens.css`. Punti chiave:

- **Tema scuro** default (`<html data-theme="dark">`), tema chiaro disponibile (`data-theme="light"`).
- **Tono palette**: `data-tone="zinc"` (default) — il design system supporta anche `slate` e `stone`. Le UI lo trattano come variazione cromatica neutra.
- **Font UI**: Inter (400/500/600/700)
- **Font mono**: JetBrains Mono — usato per: editor body, kbd, indicatori token, codice
- **Titlebar**: 36px · **Statusbar**: 28px · **Resizer**: 1px (hit-area 7px via `::after`)
- **Radius**: 6px (controlli) · 8px (card/modal) · 4px (chip/pill)

---

## Screens / Views

### 1. Layout shell

```
┌──────────────────────── Title bar (36px) ─────────────────────┐
│ [P] Prompt a Porter   [⌃⇧P search/palette]   [theme][⚙][- □ ×]│
├────────┬──┬────────────┬──┬──────────────────────────┬────────┤
│        │  │            │  │                          │        │
│ Side-  │  │ List pane  │  │ Detail / Editor          │ Right- │
│ bar    │  │            │  │                          │ rail   │
│ 248px  │  │ 360px      │  │ 1fr (minmax(0,1fr))      │ 280px  │
│        │  │            │  │                          │        │
├────────┴──┴────────────┴──┴──────────────────────────┴────────┤
│ Status bar (28px) · vault · prompt · salvato · ⌃⇧P            │
└────────────────────────────────────────────────────────────────┘
```

- Body è **CSS grid**: `grid-template-columns: var(--col-sidebar) 1px var(--col-list) 1px minmax(0,1fr)`
- `min-width: 0; overflow: hidden` solo su `.sidebar`, `.list-pane`, `.detail` (NON su tutti i grid item — i resizer sono 1px e non devono averlo).
- **Resizer**: drag horizontal (mousedown + mousemove). Sidebar 180–360px · Lista 0–480px · Right-rail 220–480px.
- **Stati collassati**:
  - Sidebar collassata → 44px, mostra solo le icone delle viste in colonna verticale (`.sidebar-mini`).
  - Lista collassata → 36px, mostra solo l'icona "≡" per ripristinare. La toggle `data-list-collapsed="true"` mette `display: none` ai figli interni e mostra `.list-restore`.
  - Right-rail collassata → grid del DetailPane diventa `1fr 0`, rail nascosta.

### 2. Title bar

- 3 colonne grid: `1fr auto 1fr` (left brand · center search-as-palette · right controls).
- Brand: bullet "P" su rounded-square + "Prompt a Porter" weight 600.
- Search-as-palette: pill 380px max, placeholder "Cerca prompt, tag o azioni…", kbd `⌃⇧P` a destra. Click → apre `Palette` modal.
- Right: theme toggle (eye icon), settings (⚙), traffic lights macOS-style (Min · Max · Close — close con hover rosso).

### 3. Sidebar (espansa)

- **Workspace switcher in alto**: avatar circolare "P" + "Personale" + chevron-down (placeholder per multi-workspace future). Pulsante collapse `«` accanto.
- **NavGroup**: 5 sezioni collassabili con header uppercase 10px, count opzionale a destra del nome, pulsante `+` per le sezioni con creazione (Cartelle, Tag).
  - **Viste**: Recenti · Preferiti · Tutti i prompt
  - **Visibilità**: Privati (dot ambra) · Team (dot blu)
  - **Cartelle**: gerarchia con indent. Click su una cartella filtra la lista.
  - **Tag**: ognuno con dot colorato (vedi `data.jsx#TAGS`).
  - **Modello target**: collassata di default. Tutti / Claude Sonnet / Claude Opus / GPT-4o / Local (Ollama).
- **Footer**: Insight · Regressioni (link rapidi alle modali).

### 4. Sidebar (collassata) `.sidebar-mini`

44px. Stack verticale di icon-button 32×32 con stesso ordine della sidebar espansa. Tooltip su hover. Click sul primo bottone (chevrons-right) riapre.

### 5. List pane

Header sticky (3 row):
1. **Title row**: viewTitle + count badge + sub "aggiornata ora" + collapse `«` a destra.
2. **Search row**: input filtro locale + bottone primario "+ Nuovo".
3. **Toolbar row**: chip densità (Compatta/Comoda/Anteprima) + filtri attivi · sort dropdown a destra (Recenti / Popolari / Migliori / A-Z).

**PromptCard** (3 densità):
- **Compatta**: 1 riga, solo title + meta-time + visibility-dot.
- **Comoda** (default): title + desc 1 riga + tags + use-count + favorite star.
- **Anteprima**: come Comoda + N righe di body in mono (slider 1–8 in Impostazioni).

Card = 12px padding orizzontale, 8px verticale. Active state: background `var(--bg-overlay)` + bordo sx 2px accent. Hover: bg leggero.

Visibility-dot a sinistra: 6px, colore `--accent-private` o `--accent-team`.

### 6. Detail / Editor pane

**Header** (no border bottom finché non si scrolla l'editor):
- **Title row**: title input grande (20px, no border, weight 600) + desc textarea (13px text-muted) — entrambi inline-editable. A destra: star · Fork · Esporta MD · **Compila** (primario) · Meta toggle (separato a fondo).
- **Meta row**: chip orizzontali — Privato/Team · cartella · target · var. (B · 3 totali) · fork di "…" · spazio · usato · aggiornato.
- **Tabs**: Editor · Anteprima · Diagnosi (badge warn) · Test golden (badge count) · Cronologia (badge count) · Import & Var. (badge count). Active = underline accent + colore strong.

**Editor body** (tab Editor):
- **Toolbar markdown**: B · I · | · H1 · H2 · | · UL · OL · quote · | · code · codeblock · link · hr · | · `+ {{var}}` · `+ import` · | · search · indicatore (saved/dirty + L/C + char + tok).
- **Mock editor** (`MockEditor`): gutter numerico + content area editable. Tokenizza `{{var}}` (highlight viola) e `{{import "x"}}` (highlight blu). La riga linted è sottolineata in giallo.

**Tab Anteprima**: rendering del body con sostituzione segnaposti dai default. Mono 13px, line-height 1.65, su `bg-surface`.

**Tab Diagnosi**: lista lint warnings/errors per riga, click per saltare.

**Tab Test golden**: tabella golden tests con drift score + last-run.

**Tab Cronologia**: lista versioni con autore (avatar + nome) · timestamp · delta `+/−`. Click apre diff side-by-side.

**Tab Import & Var.**: import composti (clic apre il prompt sorgente in nuovo tab) + varianti A/B con toggle confronta.

### 7. Right rail (Metadati)

Sezioni:
- **Metadati**: Visibilità segmented (Privato/Team) · Modello target select · Cartella select · Tag picker (pill rimovibili + input per aggiungerne).
- **Segnaposti rilevati**: lista auto-detected da `{{var}}` nel body, con tipo (testo/enum/multilinea), default, asterisco se obbligatorio.
- **Import composti**: lista con icona fork.
- **Varianti A/B**: pill orizzontali (A · B corrente · C) + bottone "+ Variante" + "Confronta tutte".

### 8. Status bar

Sinistra: dot status + "vault locale" (tooltip espande dettagli SQLCipher/embeddings).
Centro: nome prompt corrente (con icona visibility).
Destra: stato salvataggio (dot + "salvato 14s fa" / dirty) · kbd `⌃⇧P` cliccabile.

### 9. Modali

- **Palette** (`⌃⇧P`): input + sezioni Prompt recenti / Azioni. Footer: kbd hints (↑↓ naviga · ↵ apri · ⌃↵ compila & copia · esc).
- **Compila**: form coi segnaposti rilevati (ognuno con type-aware input — text/select/textarea), preview live a destra, footer "Compila e copia ⌃↵".
- **Insight**: dashboard con grafici uso · prompt più usati · token medi.
- **Regressioni**: tabella drift output golden tra modelli/versioni — width `min(1200px, 92vw)`.
- **Impostazioni**: sezioni Aspetto (tema/tono) · Vista lista (densità + righe preview) · Editor · Sicurezza · Avanzate.

Tutte le modali: backdrop `rgba(0,0,0,0.5)` + close ESC + click-outside-to-close. Header con title + sub + ✕. Footer con azioni primarie a destra.

---

## Interactions & Behavior

### Drag-resize colonne
`onMouseDown` su `.resizer` cattura `e.clientX`, attiva listener `mousemove` document, calcola delta, applica clamp (sidebar 180–360, lista 0–480, rail 220–480). Cleanup su `mouseup`. Cursor `col-resize`.

### Tweaks panel
Lista controlli tweakable (tema, tono, densità lista, righe preview, sidebar/rail collapsed). Stato persistito via `__edit_mode_set_keys` postMessage al parent. Vedi `tweaks-panel.jsx`.

### Stato dirty / autosave
`setDirty(true)` su qualunque input change. Mock di autosave a 1.2s di idle (in app.jsx). Aggiorna `saved` con timestamp friendly.

### Filtri lista
`q` (text local), `view` (recenti/preferiti/tutti/privati/team), `activeFolder`, `activeTag`, `modelTarget`. Tutti combinati AND. I chip filtro attivi appaiono nella terza row della list-toolbar con ✕ per rimuoverli.

### Densità lista
3-stati ciclico, `data-density` su `.list-pane`. CSS nasconde `desc`/`preview` come da modalità. `--preview-lines` controlla `-webkit-line-clamp` su `.preview`.

### Token highlighting nell'editor
Regex: `/(\{\{import\s+"[^"]+"\}\})|(\{\{[a-zA-Z_][\w]*\}\})/g`. Render inline con className `import-token` (blu) o `placeholder-token` (viola).

---

## State Management

In `app.jsx` (top-level):

```ts
// Tweaks (persistiti)
theme, tone, density, previewLines, sidebarCollapsed, rightRailCollapsed, view

// Layout (locale)
colSidebar, colList, rightRail

// Selezione
activeId, tab, modelTarget, activeFolder, activeTag, sort

// Editor
dirty, saved (timestamp friendly)

// UI
modal (null | 'compila' | 'insight' | 'regressioni' | 'impostazioni'), paletteOpen
```

Tutti gli `useEffect` che persistono i tweak sono nelle prime ~30 righe di `app.jsx`.

---

## Design Tokens

Vedi `prototype/tokens.css` (copia 1:1 della Fase 1 + nuove variabili usate dal redesign — segnalate con `/* added */` se ce ne sono di nuove). I principali:

```css
/* Colori */
--bg-canvas, --bg-surface, --bg-elev, --bg-overlay
--text-strong, --text-default, --text-muted, --text-subtle
--border-subtle, --border-default, --border-strong
--accent-private (ambra), --accent-team (blu)
--info, --success, --warning, --danger

/* Tipografia */
--font-ui: 'Inter', -apple-system, …
--font-mono: 'JetBrains Mono', 'SF Mono', …

/* Layout */
--titlebar-h: 36px
--statusbar-h: 28px

/* Spacing & radius — vedi tokens.css per la scala completa */
```

---

## Assets

- **Icone**: tutte SVG inline in `icons.jsx`. Stroke-based, 24×24 viewBox, scaled via prop `size`. Stile coerente: stroke-width 1.8, stroke-linecap round, stroke-linejoin round. **Sostituibile con Lucide**: i nomi sono già allineati (chevron, search, file, folder, lock, users, bar-chart, settings, etc.); le `md-*` sono custom per la toolbar markdown.
- **Font**: caricati via Google Fonts CDN (`Inter` + `JetBrains Mono`). In produzione: self-host secondo policy local-first (vedi brief Sezione 8 su privacy/network).
- **Nessun bitmap** — tutto SVG inline o CSS.

---

## Notes per l'implementazione (Tauri + React)

1. **Layout**: il grid del body è il pattern centrale. Mantenere CSS grid con `minmax(0, 1fr)` sull'ultima colonna per evitare overflow del DetailPane su prompt lunghi.
2. **Editor reale**: sostituire `MockEditor` con **CodeMirror 6** o **Monaco** in modalità markdown. Mantenere il tokenizer custom per `{{var}}` / `{{import "..."}}` come decoration plugin (CM6) o monarch tokenizer (Monaco).
3. **Persistenza tweaks**: il prototipo usa `postMessage` perché vive in iframe. Nell'app reale, persistere su `~/Library/Application Support/PromptVault/preferences.json` (Tauri).
4. **Drag-resize**: c'è già `useResizable`-style logic in `app.jsx`. In produzione consiglio `react-resizable-panels` per gestire keyboard a11y e double-click-to-reset.
5. **Modali**: usare il `<dialog>` nativo + `showModal()` per ESC + a11y free. Il prototipo le simula con div fissi.
6. **Sidebar mini state**: persistere lo stato collassato per pannello. Il prototipo lo fa già.
7. **Status bar**: i tooltip di vault/sync devono mostrare path reali, dimensione DB SQLCipher, ultima rotazione master-key.
8. **Cronologia con autore**: backend deve esporre `author_name` + `author_avatar_color` (deterministico dall'hash dell'email/username) per ogni revision.
9. **Test golden**: la tab è UI-only nel prototipo. Il backend deve fornire endpoint per esecuzione + diff con baseline.
10. **Command palette**: usare `cmdk` (pacchetto npm) per fuzzy match, con i nostri item types: prompt / tag / folder / action.

---

## Pain points del brief — checklist

| Pain point (Sez. 3) | Risolto da |
|---|---|
| Editor in modale piccola | Detail pane full-width, tabs di prima classe |
| Renderer separato dal prompt | Tab "Anteprima" inline |
| Test golden nascosti in modale 7 | Tab "Test golden" con badge sul detail |
| Regressioni isolate (schermo 8) | Modale + link sidebar, larghezza 1200px |
| Cronologia senza autore | Lista versioni con avatar+nome, diff side-by-side |
| Sidebar piatta | NavGroup collassabili, workspace switcher, gerarchia tag/cartella |
| Densità unica | 3 modalità (Compatta/Comoda/Anteprima) + slider righe |
| Niente differenziazione visibilità | Dot accent + chip nella card + filtro vista |
| Status bar muta | Vault/cifratura/sync/save/palette tutti vivi |
| Mancanza command palette | `⌃⇧P` ovunque + search-as-palette nella titlebar |
| Token `{{var}}` come testo | Highlight tipografico + chip nel rail |
| Import compositi invisibili | `{{import "x"}}` highlightato + sezione rail dedicata |
| Fork senza tracciabilità | Badge "fork di" cliccabile nel meta-row |
| Varianti A/B nascoste | Pill nel rail + tab "Import & Var." con confronta |

---

## Domande aperte / decisioni da confermare con design

- Confermare il segmented "Visibilità" (Privato/Team) come unico controllo (vs. dropdown con futuri ruoli).
- Decidere se il workspace switcher in alto a sinistra apre un menù o naviga direttamente.
- Iconografia toolbar markdown: confermare il set custom in `icons.jsx#md-*` vs. integrare Lucide/Phosphor.
- Tema chiaro: verificare contrast ratio sui chip colored e sull'editor mono — fatto su scuro, da rivedere su chiaro.
- Drag-reorder dei prompt nella lista (cartella → cartella) — non ancora implementato nel prototipo.
