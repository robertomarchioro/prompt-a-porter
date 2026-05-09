# Blueprint F3 — List pane + drag-reorder

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F3 · **Decisioni designer**: #5 (linea 2px accent-team con glow), #11 (cross-cartella + auto-espansione 600ms hover + ESC annulla), #14 (cue identico riordino interno e cross-cartella) · **Stima**: 4 giorni FT · **Bloccato da**: F1 ✅, F2 ✅, V014 backend ✅

Blueprint operativo autoportante per il pannello centrale lista del redesign v0.8. Sostituisce `.list-placeholder` di F1 con `ListPane` completa: header sticky 3-row, PromptCard a 2 densità (Compatta/Comoda), drag-reorder atomico con cmd `prompt_riordina` (V014), cross-cartella drop con auto-espansione, integrazione filtri Sidebar (F2).

**Anteprima (3a densità + slider righe 1-8) è rinviata a F3 PR-B** che estende `PromptCard` backend con `body_preview: String`.

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery](#2-strategia-di-delivery)
3. [Store densità](#3-store-densità)
4. [Component PromptCard.svelte](#4-component-promptcardsvelte)
5. [Component ListPane.svelte](#5-component-listpanesvelte)
6. [Drag-reorder + cross-cartella](#6-drag-reorder--cross-cartella)
7. [Wiring in Shell.svelte](#7-wiring-in-shellsvelte)
8. [Token + valori esatti decisioni designer](#8-token--valori-esatti-decisioni-designer)
9. [Edge case + scope](#9-edge-case--scope)
10. [Test attesi](#10-test-attesi)
11. [Exit criteria](#11-exit-criteria)
12. [F3 PR-B (Anteprima)](#12-f3-pr-b-anteprima)
13. [Dipendenze su F4-F8](#13-dipendenze-su-f4-f8)

---

## 1. Obiettivo

Pannello centrale `ListPane` con dati live filtrati dalla Sidebar, drag-reorder dentro la stessa cartella e drag cross-cartella verso sidebar.

**Output funzionale F3 PR-A**:
- `ListPane.svelte` con header sticky 3-row (title + count · search + Nuovo · toolbar densità + chip filtri + sort)
- `PromptCard.svelte` con 2 densità: Compatta (1 riga) e Comoda (default: title + desc + tags + use-count + favorite)
- Sort dropdown: Recenti / Popolari / Migliori / A-Z (mapping a `recente`/`popolare`/`qualita`/`alfabetico`)
- Filtri attivi → chip `data-attivo` con ✕ rimuove, combinati AND
- Drag-reorder dentro lista con linea 2px accent-team + glow soft box-shadow (decisione #5/#14)
- Drag cross-cartella: highlight cartella destinazione + auto-espansione collassata 600ms hover (decisione #11)
- ESC annulla drag in corso (decisione #11)
- Densità + sort persistiti in localStorage

**Out of scope F3 PR-A**:
- Densità "Anteprima" (3a) → F3 PR-B
- Slider righe preview 1-8 → F3 PR-B
- Bottone "+ Nuovo" attivo (apre modale crea prompt) → F8
- Click su PromptCard apre Detail (F4 lo wireuppa)

## 2. Strategia di delivery

### F3 PR-A (questo blueprint)

- **Branch**: `feat/redesign-f3-list-pane`
- **Target**: `feat/redesign-v08`
- **Effort**: 3 gg
  - 0.25 gg store `densita.ts` + test
  - 0.5 gg `PromptCard.svelte` (2 densità)
  - 1.0 gg `ListPane.svelte` (header + body + filtri + sort + load lista)
  - 0.75 gg drag-reorder + cross-cartella (handlers HTML5 + auto-espansione)
  - 0.25 gg wire-up Shell.svelte (filtri ↔ Sidebar)
  - 0.25 gg test + smoke

### F3 PR-B (futuro, separato)

- **Branch**: `feat/redesign-f3-anteprima`
- **Target**: `main` per backend extension, poi `feat/redesign-v08` per client
- **Scope**: estende backend `PromptCard` con `body_preview: String` (substring 500 chars), aggiunge densità "Anteprima" + slider righe 1-8
- Stima: 1 gg

## 3. Store densità

### Path

`apps/client/src/lib/stores/densita.ts` (NEW, plain TS — pattern come `sidebar-collapsed.ts`)

### Schema localStorage

Key: `pap.lista.densita`. Valore JSON sintetico:

```json
{
  "densita": "comoda",
  "righePreview": 3,
  "ordine": "recente"
}
```

### API

```typescript
export type Densita = "compatta" | "comoda" | "anteprima";
export type Ordine = "recente" | "popolare" | "qualita" | "alfabetico";

export interface StatoLista {
  densita: Densita;
  righePreview: number; // 1-8, usato solo da densità "anteprima"
  ordine: Ordine;
}

export const DEFAULTS: StatoLista;
export function caricaStato(): StatoLista;
export function salvaStato(stato: StatoLista): void;
```

### Default

- `densita: "comoda"` (più informativa di compatta, no scroll preview)
- `righePreview: 3`
- `ordine: "recente"` (coerente backend default)

## 4. Component PromptCard.svelte

### Path

`apps/client/src/lib/components/PromptCard.svelte` (NEW)

### Props

```typescript
interface Props {
  id: string;
  titolo: string;
  descrizione: string;
  visibilita: string; // "private" | "workspace"
  preferito: boolean;
  usoCount: number;
  aggiornatoA: string;
  tags: { id: string; nome: string; colore: string }[];
  attivo?: boolean;
  densita: "compatta" | "comoda" | "anteprima";
  righePreview?: number; // F3 PR-B
  bodyPreview?: string;  // F3 PR-B
  onclick?: () => void;
  onToggleFavorite?: () => void;
}
```

### Layout

- Visibility-dot 6px a sinistra (`var(--accent-private)` o `var(--accent-team)`)
- Centro: title + descrizione (Comoda/Anteprima) + tags chip max 3 + "+N" overflow
- Right: meta-time (Compatta) o uso-count (Comoda) + star toggle
- Compatta = 1 riga; Comoda = title + desc + tags + use-count + favorite
- Anteprima (F3 PR-B) = aggiunta `<pre class="preview" style:--preview-lines={N}>` con `-webkit-line-clamp`

### Stati

- `:hover` → `background: var(--bg-overlay)`
- `[data-attivo]` → `background: var(--bg-overlay) + border-left: 2px solid var(--accent-team)`

## 5. Component ListPane.svelte

### Path

`apps/client/src/lib/components/ListPane.svelte` (NEW)

### Props

```typescript
interface Props {
  /** Filtri provenienti da Sidebar (F2) */
  vistaCorrente: string;
  folderSelezionato: string | null;
  tagSelezionato: string | null;
  modelTargetSelezionato: string;

  /** Callback per rimuovere un filtro (chip ✕) */
  onRimuoviFolder: () => void;
  onRimuoviTag: () => void;
  onRimuoviModelTarget: () => void;

  /** Callback selezione card */
  promptSelezionato: string | null;
  onSelezionaPrompt: (id: string) => void;

  /** Drag verso una cartella della Sidebar gestito a livello Shell */
  onDragSuFolder?: (promptId: string, folderId: string | null) => void;
}
```

### Comportamento

- onMount + `$effect` reattivo su filtri → invoca `libreria_lista({ vista, tag_id, cerca, ordine, target_model, folder_id })`
- Cerca testo: input con debounce 300ms (timer)
- Sort: dropdown 4 opzioni → mappa a `recente|popolare|qualita|alfabetico`, persiste via store
- Densità chip toggle → store densità

### Header sticky 3-row

1. **Title row**: `vistaCorrente` mappato a label IT ("Recenti", "Preferiti", "Privati", "Team", "Tutti i prompt") + count `{prompts.length}`
2. **Search row**: input "Cerca prompt..." + bottone primario "+ Nuovo"
3. **Toolbar row**: chip densità (Compatta/Comoda) + chip filtri attivi (folder/tag/modelTarget) con ✕ + sort dropdown

### Body scrollable

`{#each prompts as p (p.id)}` → `<PromptCard ...>` con `draggable=true` + `ondragstart/over/drop`.

### Schema dati TypeScript

```typescript
interface PromptCardData {
  id: string;
  titolo: string;
  descrizione: string;
  visibilita: string;
  preferito: boolean;
  uso_count: number;
  aggiornato_a: string;
  tags: { id: string; nome: string; colore: string }[];
}
```

## 6. Drag-reorder + cross-cartella

### Pattern HTML5 nativo (no svelte-dnd-action)

Coerente con `Libreria.svelte` legacy. Handler:
- `ondragstart`: salva `promptId` in `dataTransfer.setData("text/x-pap-prompt", id)` + applica `opacity: 0.4` alla sorgente
- `ondragover`: preventDefault per consentire drop, calcola target index (sopra/sotto card target via Y position)
- `ondrop`: legge `promptId`, calcola nuovo `SortOrder`, invoca `prompt_riordina({ prompt_id, new_sort })`
- `ondragend`: cleanup stati visivi (opacity, indicator)

### Cue visivo (decisione #5/#14)

Linea 2px `var(--accent-team)` con glow soft. Box-shadow esatto:

```css
.drop-indicator {
  position: relative;
}

.drop-indicator::before {
  content: "";
  position: absolute;
  left: 0; right: 0;
  top: 0;
  height: 2px;
  background: var(--accent-team);
  box-shadow:
    0 0 0 1px var(--accent-team),
    0 0 12px -2px color-mix(in oklch, var(--accent-team) 40%, transparent);
  pointer-events: none;
  z-index: 1;
}

.drop-indicator-bottom::before {
  top: auto;
  bottom: 0;
}
```

Card sorgente durante drag: `opacity: 0.4`.

### Cross-cartella drop (decisione #11)

Sidebar (F2) accetta drop:
- `ondragover` sulla cartella highlight con `background: color-mix(in oklch, var(--accent-team) 12%, transparent)` + bordo 1px `accent-team`
- Tooltip floating "Sposta in <nome cartella>"
- **Auto-espansione cartella collassata dopo 600ms hover**: timer su `ondragenter`, cancella su `ondragleave`. Quando timer scatta, se la cartella è collassata, espande (al momento le cartelle non sono nidificate in F2 — quindi auto-espansione non ha effetto pratico in F3 PR-A; mantengo la logica per F2.5/F3.x quando saranno nidificate)
- `ondrop` invoca `prompt_sposta({ prompt_id, folder_id })`

Per F3 PR-A, l'auto-espansione è implementata ma no-op (cartelle piatte). Quando F2 supporterà cartelle nidificate, l'auto-espansione si attiverà naturalmente. Documentato nel codice per future estensioni.

### ESC annulla (decisione #11)

```typescript
function ondragstart(e: DragEvent, id: string): void {
  e.dataTransfer?.setData("text/x-pap-prompt", id);
  document.addEventListener("keydown", onEscDuringDrag);
}

function onEscDuringDrag(e: KeyboardEvent): void {
  if (e.key === "Escape") {
    annullaDrag();
  }
}
```

Cleanup su `dragend`: rimuovi listener keydown, resetta opacity, rimuovi indicator.

### Cmd Tauri usate

- `prompt_riordina({ prompt_id, new_sort })` — V014 ✅ già live (re-pack atomico siblings)
- `prompt_sposta({ prompt_id, folder_id })` — esistente da Fase 4

### Aggiornamento conteggi sidebar post-azione

Dopo `prompt_riordina`/`prompt_sposta`:

```typescript
window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
```

Sidebar ascolta:

```typescript
onMount(() => {
  window.addEventListener("pap:lista-mutata", caricaContiViste);
  return () => window.removeEventListener("pap:lista-mutata", caricaContiViste);
});
```

(Sidebar refactor in F3 PR-A: aggiunge listener event in onMount.)

## 7. Wiring in Shell.svelte

### Diff

Sostituisce `.list-placeholder` con `<ListPane ...>`. I filtri sono già `$state` in Shell da F2.

```svelte
<Pane defaultSize={26} minSize={0} maxSize={40}>
  <ListPane
    {vistaCorrente}
    {folderSelezionato}
    {tagSelezionato}
    {modelTargetSelezionato}
    onRimuoviFolder={() => (folderSelezionato = null)}
    onRimuoviTag={() => (tagSelezionato = null)}
    onRimuoviModelTarget={() => (modelTargetSelezionato = "")}
    promptSelezionato={null}
    onSelezionaPrompt={(id) => console.log("F4 detail", id)}
    onDragSuFolder={(promptId, folderId) => spostaPrompt(promptId, folderId)}
  />
</Pane>
```

### Helper Shell

```typescript
async function spostaPrompt(promptId: string, folderId: string | null): Promise<void> {
  try {
    await invoke("prompt_sposta", { dati: { prompt_id: promptId, folder_id: folderId } });
    window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
  } catch (e) {
    console.error("[shell] spostaPrompt", e);
  }
}
```

## 8. Token + valori esatti decisioni designer

| Token | Valore | Decisione |
|---|---|---|
| Linea drag-indicator | `2px solid var(--accent-team)` | #5 |
| Glow box-shadow | `0 0 0 1px var(--accent-team), 0 0 12px -2px color-mix(in oklch, var(--accent-team) 40%, transparent)` | #5 |
| Card sorgente durante drag | `opacity: 0.4` | #5 |
| Cartella highlight drop | `background: color-mix(in oklch, var(--accent-team) 12%, transparent); border: 1px solid var(--accent-team)` | #11 |
| Auto-espansione delay | 600ms | #11 |
| Tooltip drop | "Sposta in <nome cartella>" | #11 |
| ESC annulla | sì | #11 |
| Cue identico riordino interno e cross-cartella | sì | #14 |

## 9. Edge case + scope

| # | Caso | Comportamento |
|---|---|---|
| 1 | Lista vuota (0 prompts dopo filtri) | EmptyState "Nessun prompt trovato" + bottone "Reset filtri" |
| 2 | Backend offline | Lista vuota + console.error, no crash |
| 3 | Drag su se stesso | No-op (newSort == currentSort, no-op early return già in `prompt_riordina` Rust V014) |
| 4 | Drag durante caricamento async | Drag ignora dati stale, usa snapshot al `dragstart` |
| 5 | Cerca testo + filtri attivi | Combinati AND lato backend (FiltroLista) |
| 6 | Sort cambia mentre drag in corso | Drag completa con snapshot pre-cambio; refresh dopo `prompt_riordina` |
| 7 | dataTransfer null (browser strict) | Fallback variabile module-level `let dragId: string | null = null` |
| 8 | Densità "anteprima" selezionata in F3 PR-A | Fallback a "comoda" + console.warn (F3 PR-B abilita) |
| 9 | Tag con `colore` empty | Fallback `var(--text-subtle)` |
| 10 | localStorage corrotto | DEFAULTS |

## 10. Test attesi

### Unit test inline

`apps/client/src/lib/stores/densita.test.ts`:
- `caricaStato` ritorna DEFAULTS su empty
- `caricaStato` parsing valido
- `caricaStato` corrupt → DEFAULTS
- `salvaStato` non lancia
- `righePreview` clampato 1-8

### Smoke test manuale

- [ ] `?redesign-shell` mostra ListPane con prompts live
- [ ] Click su Vista Sidebar filtra lista
- [ ] Click su Cartella filtra lista, chip "Cartella: <nome>" appare con ✕
- [ ] Click ✕ su chip rimuove filtro
- [ ] Sort dropdown cambia ordering
- [ ] Cerca testo filtra (debounce 300ms)
- [ ] Densità chip switch tra Compatta/Comoda
- [ ] Drag prompt sopra/sotto altro prompt → linea accent-team + glow
- [ ] Drop riordina via prompt_riordina, refresh lista
- [ ] Drag prompt su cartella Sidebar → highlight cartella
- [ ] Drop sposta via prompt_sposta, refresh lista + sidebar conteggi
- [ ] ESC durante drag → annulla, no-op
- [ ] Densità + sort persistono tra reload

### Type-check + test client

- `npm run check`: 0 errors
- `npm test`: tutti pass

## 11. Exit criteria

PR `feat/redesign-f3-list-pane` può fare merge solo se:

- [ ] `lib/stores/densita.ts` + test (≥ 5 casi)
- [ ] `lib/components/PromptCard.svelte` con 2 densità (compatta/comoda)
- [ ] `lib/components/ListPane.svelte` con header 3-row + body + drag handlers
- [ ] Drag-reorder linea 2px + glow box-shadow + opacity 0.4 sorgente
- [ ] Cross-cartella drop con highlight + auto-espansione 600ms (no-op se cartelle piatte)
- [ ] ESC annulla drag
- [ ] `Shell.svelte` swap `list-placeholder` → ListPane + spostaPrompt handler
- [ ] Evento `pap:lista-mutata` per refresh sidebar conteggi
- [ ] Smoke test §10 passato manualmente
- [ ] `npm run check` 0 errors
- [ ] `npm test` tutti pass
- [ ] CI lint-and-test verde
- [ ] Bundle aggiunto: ≤ 8 KB gzip

## 12. F3 PR-B (Anteprima)

PR separata futura per la 3a densità "Anteprima":

### Backend (PR contro main)

- Estendi `PromptCard` Rust con `body_preview: String`
- Modifica query `libreria_lista` per `SUBSTR(p.Body, 1, 500) AS BodyPreview`
- 1 test inline
- Stima: 0.5 gg

### Client (PR contro feat/redesign-v08)

- Aggiungi tipo client `body_preview?: string`
- Abilita densità "anteprima" in PromptCard.svelte (markup già pronto, solo unstub)
- Slider righe 1-8 in ListPane toolbar
- 2 test inline (clamp, persist)
- Stima: 0.5 gg

## 13. Dipendenze su F4-F8

F3 sblocca:

- **F4** (DetailPane): consuma `onSelezionaPrompt` di ListPane per aprire la card cliccata. F3 lo logga in console.
- **F8 modali**: bottone "+ Nuovo" della header invoca modale crea prompt (F8 la rifa).
- **F8 Palette**: indirettamente, refresh conteggi Sidebar quando palette crea/elimina via custom event `pap:lista-mutata`.

**Interface contract** che F3 espone:

```typescript
// $lib/components/ListPane.svelte — pannello lista
// $lib/components/PromptCard.svelte — card primitive (riusabile in F8 Palette)
// $lib/stores/densita.ts — solo ListPane
// CustomEvent "pap:lista-mutata" — segnala che la lista è cambiata, sidebar deve ricaricare conteggi
```

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione F3 PR-A. F3 PR-B (Anteprima + body_preview backend) sarà blueprint separato post-PR-A merge.
