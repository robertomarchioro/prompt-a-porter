# Blueprint F1 — Shell layout 3-pannelli

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F1 · **Decisioni designer**: nessuna bloccante per F1 (#5/#11/#14 drag-reorder vivono in F3, #4 contrast vive in F0 PR-A già mergiata) · **Stima**: 4 giorni FT · **Bloccato da**: F0 ✅ chiusa

Blueprint operativo autoportante per il layout shell 3-pannelli del redesign v0.8. Fornisce la base strutturale su cui F2 (Sidebar), F3 (List pane), F4 (DetailPane) e F7 (StatusBar) inseriranno i propri component.

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery (1 PR)](#2-strategia-di-delivery-1-pr)
3. [Scelta paneforge vs roll-our-own](#3-scelta-paneforge-vs-roll-our-own)
4. [Component Shell.svelte](#4-component-shellsvelte)
5. [Component TitleBar.svelte](#5-component-titlebarsvelte)
6. [Component StatusBar.svelte](#6-component-statusbarsvelte)
7. [Resizer + paneforge integrazione](#7-resizer--paneforge-integrazione)
8. [Wiring in App.svelte](#8-wiring-in-appsvelte)
9. [Token CSS necessari](#9-token-css-necessari)
10. [Edge case + decisioni di scope](#10-edge-case--decisioni-di-scope)
11. [Test attesi](#11-test-attesi)
12. [Exit criteria](#12-exit-criteria)
13. [Dipendenze su F2-F11](#13-dipendenze-su-f2-f11)

---

## 1. Obiettivo

Stabilire la **struttura layout root** del redesign v0.8: titlebar 36px (header bar applicativa) + body 3-pannelli ridimensionabili + statusbar 28px. Tutto il resto del redesign costruirà sopra questa shell.

**Output funzionale F1**:
- `Shell.svelte` renderizza titlebar (placeholder), body grid 3-pannelli (placeholder con classe diversa per dev visivo), statusbar (placeholder).
- Resizer drag horizontal funzionanti tra sidebar/list e tra list/detail con clamp 180-360 / 0-480.
- Right-rail nidificato dentro detail (gestito in F4/F6, F1 lo lascia libero come `flex: 1`).
- Theme toggle nella titlebar muta `statoTema.tema` via store F0 + cascade automatica via `$effect` di App.svelte.
- Renderizzata in modalità **showcase** dietro URL param `?redesign-shell` (analogo a `?demo` per `DemoComponenti`). Niente sostituzione del flusso libreria attuale.

**Out of scope F1**:
- Logica sidebar gerarchica (F2)
- Lista prompt + densità (F3)
- DetailPane editor (F4)
- Right-rail metadati (F6)
- Status bar funzionale con tooltip vault (F7)
- Workspace switcher placeholder (F2)

## 2. Strategia di delivery (1 PR)

- **Branch**: `feat/redesign-f1-shell-layout`
- **Target**: `feat/redesign-v08`
- **Effort**: 4 gg distribuiti come da §F1 piano:
  - 0.5 gg setup paneforge + dep
  - 1.0 gg Shell.svelte + grid CSS
  - 0.5 gg TitleBar.svelte (placeholder con theme toggle live)
  - 0.5 gg StatusBar.svelte (placeholder)
  - 0.5 gg Resizer.svelte wrapper paneforge
  - 0.5 gg App.svelte wiring + URL param
  - 0.5 gg test + smoke
- **Visibility**: solo dietro `?redesign-shell` URL param. `Libreria.svelte` legacy resta invariato.

## 3. Scelta paneforge vs roll-our-own

Decisione confermata da audit C (vedi `decisioni-designer.md` §5 / piano §5):

- **paneforge ^1.0.2** scelto per:
  - Stesso autore di `bits-ui` (huntabyte / svecosystem) → coerenza ecosistema
  - Peer `svelte ^5.29` runes nativo
  - API `<PaneGroup>`/`<Pane>`/`<PaneResizer>` con collapse, double-click reset, keyboard a11y
  - Bundle ~6 KB gzip
- **Scartato** roll-our-own: drag handler ~80 righe + gestione collapse persistente + a11y costano ~200 righe e perdiamo manutenzione esterna.

Versione: `paneforge ^1.0.2` come da piano §5.

## 4. Component `Shell.svelte`

### Path

`apps/client/src/lib/superfici/Shell.svelte` (NEW)

### Layout

```
┌──────────────── TitleBar (36px) ────────────────┐
├────────┬──┬──────────┬──┬──────────────────────┤
│        │  │          │  │                      │
│Sidebar │R1│List      │R2│Detail                │
│248px   │1 │320px     │1 │1fr (minmax(0,1fr))   │
│        │px│          │px│                      │
├────────┴──┴──────────┴──┴──────────────────────┤
└────────────── StatusBar (28px) ─────────────────┘
```

### Markup

```svelte
<script lang="ts">
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import TitleBar from "$lib/componenti/TitleBar.svelte";
  import StatusBar from "$lib/componenti/StatusBar.svelte";
  // F2/F3/F4/F6 useranno questi slot. Per F1 placeholder inline.
</script>

<div class="shell-root">
  <TitleBar />
  <main class="shell-body">
    <PaneGroup direction="horizontal" autoSaveId="redesign-shell-v08">
      <Pane defaultSize={20} minSize={14} maxSize={30}>
        <div class="placeholder-pane sidebar-placeholder">
          <p>Sidebar (F2)</p>
        </div>
      </Pane>
      <PaneResizer class="resizer" />
      <Pane defaultSize={26} minSize={0} maxSize={40}>
        <div class="placeholder-pane list-placeholder">
          <p>List pane (F3)</p>
        </div>
      </Pane>
      <PaneResizer class="resizer" />
      <Pane defaultSize={54}>
        <div class="placeholder-pane detail-placeholder">
          <p>Detail / Editor (F4) + Right-rail (F6)</p>
        </div>
      </Pane>
    </PaneGroup>
  </main>
  <StatusBar />
</div>

<style>
  .shell-root {
    display: grid;
    grid-template-rows: var(--titlebar-h) 1fr var(--statusbar-h);
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .shell-body {
    overflow: hidden;
  }

  .placeholder-pane {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-surface);
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .sidebar-placeholder { background: var(--bg-surface); }
  .list-placeholder { background: var(--bg-canvas); }
  .detail-placeholder { background: var(--bg-canvas); }

  :global(.resizer) {
    width: 1px;
    background: var(--border-subtle);
    position: relative;
    cursor: col-resize;
  }

  :global(.resizer::after) {
    content: "";
    position: absolute;
    inset: 0 -3px;
    /* hit-area 7px (1px visivo + 3px per lato), come da prototipo */
  }

  :global(.resizer:hover),
  :global(.resizer[data-resize-handle-active]) {
    background: var(--accent-team);
  }
</style>
```

### Note implementative

- **`autoSaveId`**: paneforge salva automaticamente le posizioni via localStorage con key `paneforge:redesign-shell-v08`. Si combina con lo store F0 in F2 per persistere su preferenze backend (rinviato per non scope-creep su F1).
- **`minSize`/`maxSize` in percentuale**: paneforge lavora con percentuali del container, non pixel. Calcolo: con viewport 1200px (default Tauri), 14% = 168px, 30% = 360px → vicino al clamp 180-360 del prototipo. Con viewport 1600px, 14% = 224px → ancora dentro range usabile. Acceptable in F1; raffinabile in F2 con `useContainerSize`.
- **`overflow: hidden`** su `.shell-body` previene scrollbar globale quando il contenuto interno overflow.
- **Right-rail nested**: F1 NON nidifica un secondo `<PaneGroup>` per dividere detail da right-rail. F4/F6 lo aggiungeranno dentro detail-placeholder.

## 5. Component `TitleBar.svelte`

### Path

`apps/client/src/lib/componenti/TitleBar.svelte` (NEW)

### Scope F1

Placeholder funzionale con theme toggle. NO workspace switcher (F2), NO search-as-palette (F8), NO traffic lights macOS (decorations native già attive).

### Markup

```svelte
<script lang="ts">
  import { statoTema, salvaTemaTono } from "$lib/stores/preferenze.svelte";
  import { Sun, Moon } from "lucide-svelte";

  function toggleTema(): void {
    const successivo = statoTema.tema === "dark" ? "light" : "dark";
    statoTema.tema = successivo; // cascade $effect in App.svelte applica automaticamente
    void salvaTemaTono(successivo, statoTema.tono); // persisti sul backend
  }
</script>

<header class="titlebar">
  <div class="brand">
    <span class="glyph">P</span>
    <span class="name">Prompt a Porter</span>
    <span class="version-tag">v0.8 redesign shell</span>
  </div>
  <div class="actions">
    <button
      type="button"
      class="icon-button"
      aria-label="Cambia tema"
      onclick={toggleTema}
    >
      {#if statoTema.tema === "dark"}
        <Sun size={16} />
      {:else}
        <Moon size={16} />
      {/if}
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--titlebar-h);
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
  }

  .glyph {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    background: var(--accent-team);
    color: var(--accent-team-on);
    font-weight: var(--fw-bold);
    font-size: var(--fs-xs);
  }

  .name {
    font-weight: var(--fw-semibold);
    color: var(--text-default);
  }

  .version-tag {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    padding: 2px var(--sp-2);
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
</style>
```

### Note

- Toggle tema chiama `salvaTemaTono` per persistere. La cascade `$effect` di App.svelte applica automaticamente via `applicaThemeTone`.
- Icone `Sun`/`Moon` da `lucide-svelte` (già in dep, decisione designer #3).
- Nessun `-webkit-app-region: drag` perché Tauri usa decorations native.

## 6. Component `StatusBar.svelte`

### Path

`apps/client/src/lib/componenti/StatusBar.svelte` (NEW)

### Scope F1

Placeholder funzionale con dot vault statico + nome prompt placeholder + kbd palette.
F7 lo riempie con tooltip live, dot dinamico, save status, ecc.

### Markup

```svelte
<script lang="ts">
  // F1: placeholder. F7 lo wire-uppa.
</script>

<footer class="statusbar">
  <div class="seg">
    <span class="dot dot-ok" aria-hidden="true"></span>
    <span>vault locale</span>
  </div>
  <span class="sep" aria-hidden="true"></span>
  <div class="seg">
    <span>(nessun prompt selezionato)</span>
  </div>
  <div class="right">
    <span class="seg">
      <kbd>⌃⇧P</kbd>
      <span class="muted">cerca</span>
    </span>
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: var(--statusbar-h);
    padding: 0 var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .seg {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--text-muted);
  }

  .dot-ok { background: var(--success); }

  .sep {
    width: 1px;
    height: 14px;
    background: var(--border-subtle);
  }

  .right {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-3);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px var(--sp-1);
    border-radius: var(--radius-sm);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    color: var(--text-default);
  }

  .muted {
    color: var(--text-subtle);
  }
</style>
```

## 7. Resizer + paneforge integrazione

### Install

```bash
pnpm add paneforge@^1.0.2
```

### Verifica peer

- paneforge richiede svelte ^5.29 (verificato).
- Nessuna config aggiuntiva richiesta in `vite.config.ts` o `svelte.config.js`.

### Pattern uso

I `<PaneResizer>` di paneforge hanno hit-area di default ~3px. Per matchare il prototipo (1px visivo + 7px hit) uso il pattern `:global(.resizer::after)` col `inset: 0 -3px`. Vedi §4 markup Shell.

### Stati interattivi

- Default: `background: var(--border-subtle)` (grigio chiaro)
- Hover: `background: var(--accent-team)` (viola)
- Active drag: `[data-resize-handle-active]` selector (paneforge lo aggiunge automaticamente)

### Persistenza posizioni

- `autoSaveId="redesign-shell-v08"` su `<PaneGroup>` salva su localStorage key `paneforge:redesign-shell-v08`
- F2/F3 può migrare a backend `preferenze` (campo `layout_panels: { sidebar, list }`) come iterazione futura

## 8. Wiring in `App.svelte`

### Diff

Aggiungo branch sopra `else if (mostraDemo)` per gestire URL param `?redesign-shell`:

```svelte
<script lang="ts">
  // ... import esistenti
  import Shell from "$lib/superfici/Shell.svelte";

  const mostraRedesignShell = new URLSearchParams(
    window.location.search,
  ).has("redesign-shell");
</script>

{#if mostraRedesignShell}
  <Shell />
{:else if mostraDemo}
  <DemoComponenti />
{:else if etichetta === "palette"}
  <CommandPalette />
  <!-- ... resto invariato ... -->
```

### Note

- `mostraRedesignShell` ha priorità sopra `mostraDemo` per coerenza con la convenzione "URL param modifica root".
- L'app esistente (`onboarding`, `libreria`) NON è toccata.
- L'utente lancia con `tauri dev` poi navigando a `tauri://localhost?redesign-shell` (o configurando window URL in tauri.conf.json per finestra di test).

## 9. Token CSS necessari

Tutti i token usati da F1 sono GIÀ in `apps/client/src/styles/tokens.css`:

- `--titlebar-h: 36px` ✅
- `--statusbar-h: 28px` ✅
- `--bg-canvas`, `--bg-surface`, `--bg-overlay` ✅
- `--text-default`, `--text-muted`, `--text-subtle` ✅
- `--border-subtle` ✅
- `--accent-team`, `--accent-team-on` ✅
- `--success` (dot OK) ✅
- `--font-ui`, `--font-mono` ✅ (caricati via fonts.css F0)
- `--sp-1` ... `--sp-3`, `--fs-xs`, `--fs-sm`, `--fw-bold`, `--fw-semibold`, `--radius-sm`, `--radius-full` ✅

**Nessun nuovo token**. F1 usa solo quelli già esistenti.

## 10. Edge case + decisioni di scope

| # | Caso | Comportamento atteso F1 |
|---|---|---|
| 1 | Viewport molto stretto (<800px) | Tauri `minWidth: 800` previene. Resizer continua a clampare in % |
| 2 | Drag-resize con mouse fuori finestra | paneforge gestisce con `mousemove` su document |
| 3 | Tastiera (Tab + Enter su resizer) | paneforge fornisce `aria-valuenow` + arrow keys nativi |
| 4 | Double-click su resizer | paneforge ripristina default size |
| 5 | List pane collapsed (size=0) | F3 lo gestirà con stato `data-list-collapsed`. F1 permette `minSize=0` ma non aggiunge UI per ripristinare |
| 6 | Right-rail | F1 NON nidifica secondo PaneGroup. F4/F6 lo aggiungono dentro detail-placeholder |
| 7 | Theme toggle race con load preferenze | toggle scrive direttamente lo store, salvaTemaTono in background. Race accettabile (l'utente vede il cambio immediato anche se save è ancora pending) |
| 8 | localStorage paneforge corrotto | paneforge ha try/catch interno, fallback ai defaultSize |
| 9 | Finestre multiple Tauri (libreria + palette) | Ogni finestra ha il proprio document → cascade tema/tono già funziona da F0. F1 Shell renderizzata solo nella finestra `libreria` con `?redesign-shell` |
| 10 | Riapertura app: prompt-a-porter ha sempre `?redesign-shell` se l'utente lo settava? | NO: il param vive nell'URL `index.html`, non persiste tra sessioni. Per attivare = lanciare con flag dedicato (in F8 spostiamo a feature flag via Impostazioni > Avanzate) |

## 11. Test attesi

### Unit/component test

`apps/client/src/lib/superfici/Shell.svelte.test.ts` — saltato in F1: i test dei component Svelte 5 con runes richiedono plugin svelte-vitest (vedi note F0 PR-B). Spostiamo a F11 quando aggiungeremo il plugin globalmente.

### Smoke test manuale (prima del merge)

- [ ] `npm run dev` avvia l'app
- [ ] Apertura URL con `?redesign-shell` mostra Shell, NON Libreria
- [ ] Apertura URL senza param mostra Libreria legacy (no regressione)
- [ ] TitleBar mostra brand + theme toggle
- [ ] Click su Sun/Moon switcha `data-theme` su `<html>` istantaneamente
- [ ] Drag resizer 1 (sidebar↔list): la sidebar resize correttamente, clamp ai limiti
- [ ] Drag resizer 2 (list↔detail): list resize correttamente
- [ ] Reload pagina: paneforge ripristina le posizioni dei pannelli
- [ ] Double-click resizer: ripristina defaultSize
- [ ] Tab keyboard nav arriva al theme toggle e al resizer
- [ ] Apertura DevTools: nessun errore in console

### Test type-check

- `npm run check`: 0 errors

## 12. Exit criteria

PR `feat/redesign-f1-shell-layout` può fare merge solo se:

- [ ] `paneforge ^1.0.2` aggiunto come dependency
- [ ] `Shell.svelte` con grid CSS + 3 placeholder pannelli + 2 PaneResizer
- [ ] `TitleBar.svelte` con theme toggle live (chiama `salvaTemaTono`)
- [ ] `StatusBar.svelte` placeholder con dot vault + kbd palette
- [ ] `App.svelte` branch `?redesign-shell` integrato sopra `mostraDemo`
- [ ] Smoke test §11 passato manualmente (anche se non in CI per ora)
- [ ] `npm run check` 0 errors
- [ ] CI lint-and-test verde
- [ ] Bundle CSS+JS aggiunto: ≤ 12 KB gzip (paneforge ~6 KB + 3 component minimal)

## 13. Dipendenze su F2-F11

F1 sblocca:

- **F2** (Sidebar): popola il primo `<Pane>` di Shell con NavGroup, workspace switcher, sidebar-mini. Sostituisce `.sidebar-placeholder`.
- **F3** (List pane): popola il secondo `<Pane>`. Sostituisce `.list-placeholder`.
- **F4** (DetailPane): popola il terzo `<Pane>`. Aggiunge un secondo `<PaneGroup>` nidificato per dividere detail da right-rail.
- **F6** (Right rail): vive dentro il PaneGroup nidificato di F4.
- **F7** (Status bar): estende `StatusBar.svelte` di F1 con dot vault dinamico, tooltip SQLCipher, save status reattivo.
- **F8 Palette**: la titlebar di F1 espone solo theme toggle. F8 aggiunge search-as-palette nel centro della titlebar.
- **F9** (Routing): `App.svelte` rimpiazza il branch `?redesign-shell` con flusso normale post-cutover.

**Interface contract** che F1 espone:

```typescript
// $lib/superfici/Shell.svelte — root del nuovo layout
// $lib/componenti/TitleBar.svelte — header bar applicativa
// $lib/componenti/StatusBar.svelte — footer applicativa
```

CSS classi:
- `.placeholder-pane` per identificare gli slot rimpiazzabili (verrà rimossa quando F2/F3/F4 forniscono i component reali).
- `.titlebar`, `.statusbar` come hook per stili di pagina.

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se durante implementazione paneforge mostra issue di compatibilità Svelte 5.29 o se decisioni designer impongono cambi.
