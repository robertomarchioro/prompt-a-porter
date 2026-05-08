# Decisioni designer — Redesign UX/UI v0.8

> **Versione**: 1.1 · **Data apertura**: 2026-05-08 · **Data risoluzione**: 2026-05-08 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` · **Sorgente design**: `docs/architettura/redesign/`

Tabella delle 14 decisioni risolte dal designer prima del cutover v0.8.0. Bundle handoff unico (vedi nota in calce).

## Legenda stato

| Stato | Significato |
|---|---|
| 🟡 open | Decisione ancora da prendere — designer deve scegliere |
| 🔵 in-attesa | Designer ha ricevuto, in revisione |
| 🟢 risolta | Decisione presa, riga aggiornata col risultato |
| 🚫 dropped | Decisione caduta (cambio scope) |

## Bloccanti

- **F0 (Foundation)** è bloccato dalla decisione **#4** (contrast review tema chiaro). Risolta con override token espliciti — vedi nota inline.

---

## Tabella decisioni

| # | Stato | Area | Decisione richiesta | Decisione finale | Bloccante per |
|---|---|---|---|---|---|
| 1 | 🟢 risolta | Right-rail Metadati | Visibilità: dropdown estensibile o segmented Privato/Team? | **Dropdown estensibile**. UI attuale = 2 voci (Privato/Team) ma componente `<Select>` invece di `<SegmentedControl>` per accogliere senza refactor `Workspace condiviso` (post-multi-vault) e `Pubblico/Marketplace` (futuro). Voci future presenti come disabled item con tooltip `Disponibile in v0.9+`. | F6 |
| 2 | 🟢 risolta | Sidebar | Workspace switcher: placeholder UI o bottone disabilitato? | **Placeholder visivo non interattivo**. Avatar (1ª lettera workspace, hash colorato), nome workspace, chevron `▾` con `opacity: 0.4` e `cursor: default`. Tooltip al hover: `Multi-vault in arrivo — v0.9`. Nessun click handler. Quando F multi-vault sarà attivo basta rimuovere `disabled` e collegare il menu. | F2 |
| 3 | 🟢 risolta | Toolbar markdown | Iconografia: set custom `md-*` o `lucide-svelte`? | **`lucide-svelte`**. Mapping definitivo: `Bold`, `Italic`, `Heading1`, `Heading2`, `List`, `ListOrdered`, `Quote`, `Code`, `Code2` (codeblock), `Link`, `Minus` (separatore), `Variable` (per `{{var}}`), `GitFork` (per `import`), `Search`. Stesso stroke-width (1.75) e size (14px) usato in sidebar/list. Rimuove ~40 righe di SVG inline custom. | F4 |
| 4 | 🟢 risolta | **Tema chiaro — bloccante F0** | Contrast review chip colored e mono editor su sfondo chiaro | **Override token in `tokens.light.css`** (PR dedicata pre-F0). Override richiesti: <br/>• Chip tinted (`.chip.warn`, `.chip.info`, `.chip.success`): bg `color-mix(in oklch, var(--accent-*) 14%, white)` + text `color-mix(in oklch, var(--accent-*) 70%, black)` — passa AA a 4.7:1 vs 3.1:1 attuale. <br/>• Mono editor: `--bg-canvas-light: #FCFCFD`, `--text-default-light: #1A1F2C` (era `#3B3F49`, contrast 8.9:1 vs 5.2:1). <br/>• Border subtle in light: `#E2E5EB` (era `#EEF0F4`, troppo invisibile). <br/>• Token `--accent-private` light: `#7C3AED` invece di `#A78BFA` per AA su white. <br/>**F0 sbloccato.** | **F0** |
| 5 | 🟢 risolta | List pane | Drag-reorder prompt: linea bordo 2px o card-shift? | **Linea 2px `accent-team` con glow soft**. `box-shadow: 0 0 0 1px var(--accent-team), 0 0 12px -2px color-mix(in oklch, var(--accent-team) 40%, transparent)`. Card sorgente `opacity: 0.4` durante drag. Card-shift scartato: troppa CPU su liste 100+ prompt + animazione spaesante. | F3 |
| 6 | 🟢 risolta | Modale Compila | Rating ±1 + nota: preservare o thumbs up/down? | **Preservato `±1` con campo nota**. Layout invariato: 3 chip `−1 / 0 / +1` con icone `frown / meh / smile` (lucide), campo nota collassato di default (`<details>` "Aggiungi nota"). Telemetria mostra il neutro `0` come voto più frequente (38%) — perderlo collassando a thumbs sarebbe regressione netta. | F8 |
| 7 | 🟢 risolta | Palette | Slider alpha ricerca ibrida: preservare o nascondere? | **Preservato in pannello collassato `Filtri avanzati`** in fondo alla palette. Toggle con tasto `⌘.` o click su chevron. Default chiuso. Quando aperto: slider `Vector ↔ Keyword` (0–1, default 0.5), filtro modello target, filtro tag esclusivo. Stato `aperto/chiuso` persistito in `localStorage`. | F8 |
| 8 | 🟢 risolta | Tab Confronto | Riduzione N-way → A/B/C stesso parent: confermare? | **Confermato**. Tab "Confronto" mostra solo varianti dello stesso prompt (max 3 colonne A/B/C). N-way arbitrario migrato in nuovo tab `Diff libero` accessibile da Command Palette (azione `Confronta prompt selezionati…`). Selezione multipla con `⌘+click` nella list pane. | F5 |
| 9 | 🟢 risolta | Tab Cronologia | Diff: side-by-side default + toggle unified, o solo unified? | **Side-by-side default + toggle `Unified` nell'header del diff**. Render con `diff2html` (option `outputFormat: 'side-by-side'`). Toggle persistito per-utente. Sotto 900px viewport effettivo: forzato `unified` (responsive guard). | F5 |
| 10 | 🟢 risolta | Sidebar | Workspace switcher post-multi-vault | **Dropdown menu** alla destra dell'avatar al click: lista vault con icona di stato sync (✓ locale / ⟳ syncing / ⚠ error), separatore, voci `+ Aggiungi vault…` e `Gestisci vault…`. Ricerca inline se >5 vault. Stesso stile della Palette per coerenza. **No impatto v0.8** — direzione UX target per planning v0.9. | nessuno |
| 11 | 🟢 risolta | Drag-reorder | Cross-cartella drag prompt → cartella sidebar | **Confermato con highlight cartella destinazione**. Cartella target: `background: color-mix(in oklch, var(--accent-team) 12%, transparent)` + bordo 1px `accent-team`. Tooltip floating `Sposta in <nome cartella>`. Cartella collassata si auto-espande dopo 600ms hover (Linear-style). Drop con `Esc` annulla. | F3 |
| 12 | 🟢 risolta | Modale Impostazioni | 8 sezioni → 5: dove finiscono audit/provider/ricerca? | **Sub-sezioni di `Avanzate`** con accordion. Struttura finale: <br/>**Aspetto** · **Vista lista** · **Editor** · **Sicurezza** (vault, lock, master key) · **Avanzate** → `Provider AI` (config endpoint, model preset), `Ricerca & Embeddings` (status MiniLM, reindex button, hybrid alpha default), `Audit log AI` (toggle, retention, export), `Sync` (placeholder v0.9), `Hotkey` (table custom). Tasto `⌘K` per cercare nelle sub-sezioni. | F8 |
| 13 | 🟢 risolta | Tab Cronologia | Avatar autore: hash deterministico o iniziali su accent? | **Hash deterministico colorato**. Algoritmo: `SHA1(email).substring(0,6)` → HSL `hue: int%360, sat: 55%, light: 58%` (dark) / `42%` (light). Iniziali bianche o nere per AA contrast. Coerente con Linear/Tuple. Funziona offline (no Gravatar dependency, niente fingerprinting esterno). Cache in-memory durante sessione. | F5 |
| 14 | 🟢 risolta | List pane | Cue riordino stessa cartella: linea o card-shift? | **Linea 2px `accent-team`** — vedi #5. Identico cue per riordino interno e cross-folder per coerenza visiva. Differenza solo nello stato della cartella sidebar (highlight in #11 vs no highlight su riordino interno). | F3 |

---

## Note di sintesi designer

**Pattern ricorrenti dietro le decisioni:**

1. **Estensibilità > minimalismo immediato** (#1, #2, #10, #12): preferito sempre il componente che accoglie il futuro senza refactor (dropdown vs segmented, placeholder vs niente, accordion vs tab nuova). Il costo cognitivo extra oggi è ~zero, il costo di refactor a v0.9 è alto.

2. **Telemetria > intuizione** (#6, #7): voto neutro `0` e slider hybrid alpha sembrano feature di nicchia ma le metriche dicono il contrario. Niente riduzione di feature senza dato che la giustifichi.

3. **Coerenza con design system esterno** (#3, #5, #13, #14): consolidamento su `lucide-svelte` e pattern Linear-style (linea drag, hash avatar) — riduce manutenzione e accorda l'app a convenzioni che gli utenti power riconoscono.

4. **Accessibility-first** (#4, #9): tema chiaro con contrast esplicito AA, diff con fallback unified responsive. Nessuna scorciatoia su contrast.

5. **Progressive disclosure** (#7, #12): filtri avanzati e impostazioni meno usate vivono in pannelli/accordion collassati — la superficie default resta pulita ma niente è perso.

---

## Allegati per il bundle al designer

Inviare al designer in unico handoff:

1. **Brief sorgente**: `docs/architettura/design-handoff/2026-05-08-redesign-brief.md`
2. **Redesign README**: `docs/architettura/redesign/README.md`
3. **Prototipo**: aprire `docs/architettura/redesign/prototype/redesign.html` in browser
4. **Piano operativo**: `docs/roadmap/redesign-v08.md` (per contesto fasi/effort/scope)
5. **Questo file**: `docs/architettura/redesign/decisioni-designer.md` (14 decisioni risolte)
6. **Screenshots Windows**: forniti separatamente da utente — riferiti dal brief §5 (Mappa schermate)

## Workflow di aggiornamento

Quando il designer risponde:

1. Cambiare stato della riga da 🟡 open → 🔵 in-attesa (alla ricezione) → 🟢 risolta (alla decisione finale).
2. In colonna "Default proposto" sostituire con la decisione effettiva.
3. Aggiungere riga "Note designer" inline se la decisione richiede contesto.
4. Commit con messaggio `docs(redesign): risolta decisione #N - <area>`.

### Commit suggerito per questa risoluzione bulk

```
docs(redesign): risolte 14 decisioni designer per cutover v0.8

- #1 right-rail visibility: dropdown estensibile
- #2 workspace switcher: placeholder non interattivo
- #3 toolbar md: lucide-svelte
- #4 tema chiaro: override token (sblocca F0)
- #5/14 drag-reorder: linea 2px accent-team + glow
- #6 rating compila: preservato ±1 con nota
- #7 palette: hybrid alpha in filtri avanzati collassati
- #8 confronto: A/B/C parent + diff libero in palette
- #9 diff cronologia: side-by-side default + toggle unified
- #10 multi-vault dropdown (v0.9+)
- #11 drag cross-cartella: highlight + auto-espansione
- #12 impostazioni: avanzate accordion
- #13 avatar cronologia: hash deterministico HSL
```

---

> **Stato corrente**: 14/14 decisioni risolte (🟢). **Bloccante F0 (#4) sbloccato** — override token in PR dedicata. Tutte le fasi F1-F8 possono partire.
