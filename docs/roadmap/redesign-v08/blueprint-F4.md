# Blueprint F4 — DetailPane + Editor tab

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F4 · **Decisioni designer**: #3 (mapping lucide-svelte definitivo per toolbar markdown), #1 (visibilità dropdown estensibile — applicato in right-rail F6, qui solo chip statico) · **Stima**: 6 giorni FT · **Bloccato da**: F1 ✅, F3 PR-A ✅

Blueprint operativo autoportante per il pannello detail del redesign v0.8. Sostituisce `.detail-placeholder` di F1 con `DetailPane` completa: header (title + desc inline-editable + toolbar azioni + meta-row chip + tab strip 6 tab) + tab Editor funzionante con CodeMirror 6 + autosave.

**Le 5 altre tab (Anteprima · Diagnosi · Test golden · Cronologia · Import & Var.) sono placeholder in F4** — popolate in F5.

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery (1 PR)](#2-strategia-di-delivery-1-pr)
3. [Component EditorIndicator.svelte](#3-component-editorindicatorsvelte)
4. [Component MarkdownToolbar.svelte](#4-component-markdowntoolbarsvelte)
5. [Component EditorTab.svelte](#5-component-editortabsvelte)
6. [Component DetailTabs.svelte](#6-component-detailtabssvelte)
7. [Component DetailPane.svelte](#7-component-detailpanesvelte)
8. [Autosave 2s idle](#8-autosave-2s-idle)
9. [Wiring in Shell.svelte](#9-wiring-in-shellsvelte)
10. [Edge case + scope](#10-edge-case--scope)
11. [Test attesi](#11-test-attesi)
12. [Exit criteria](#12-exit-criteria)
13. [Dipendenze su F5-F8](#13-dipendenze-su-f5-f8)

---

## 1. Obiettivo

Pannello detail con header completa + tab Editor funzionante. Le altre 5 tab sono visibili nel tab strip ma il loro contenuto è placeholder per F5.

**Output funzionale F4**:
- `DetailPane.svelte` montato quando `promptSelezionato != null` (stato in Shell)
- Header con title input grande inline-editable + descrizione textarea inline-editable
- Toolbar destra: Star (preferito) · Fork · Esporta MD · Compila · Meta toggle. **F4 logga in console** — F8 li wireuppa.
- Meta-row chip statici: Privato/Team · Cartella · Target model · Varianti · Fork-of · Uso · Aggiornato. **F6** li renderà interattivi col right-rail.
- Tab strip 6 tab: Editor (default attivo) · Anteprima · Diagnosi · Test golden · Cronologia · Import & Var.
- Tab Editor: CodeMirror 6 con `@codemirror/lang-markdown` + plugin esistenti (`import-tokens`, `placeholder-highlight`, `lint-markers`)
- MarkdownToolbar: 18 azioni con icone lucide-svelte (decisione #3 mapping definitivo)
- EditorIndicator: saved/dirty + L/C + char + tok (proxy `Math.ceil(chars/4)`)
- Autosave 2s idle via `prompt_aggiorna` (cmd esistente)

**Out of scope F4**:
- Le 5 tab Anteprima/Diagnosi/Golden/Cronologia/ImportVar → contenuto in F5
- Right-rail metadati interattivi → F6
- Modali (Compila/Insight/Regressioni/Impostazioni) → F8
- Bottoni header attivi (Star/Fork/Esporta MD) → F8 (per ora console.log)

## 2. Strategia di delivery (1 PR)

- **Branch**: `feat/redesign-f4-detail-pane`
- **Target**: `feat/redesign-v08`
- **Effort**: 6 gg
  - 0.25 gg install `@codemirror/lang-markdown` + check
  - 0.5 gg `EditorIndicator.svelte` (state visivo)
  - 1.0 gg `MarkdownToolbar.svelte` (18 azioni con icone + commands CodeMirror)
  - 2.0 gg `EditorTab.svelte` (CodeMirror 6 setup + decoration plugins + autosave glue)
  - 0.5 gg `DetailTabs.svelte` (tab strip + active state)
  - 1.0 gg `DetailPane.svelte` (header + meta-row + slot tab + load detail)
  - 0.25 gg wire-up Shell.svelte
  - 0.5 gg test + smoke

## 3. Component EditorIndicator.svelte

### Path

`apps/client/src/lib/components/EditorIndicator.svelte` (NEW)

### Props

```typescript
interface Props {
  statoSalvataggio: "salvato" | "dirty" | "salvando" | "errore";
  righe: number;
  colonna: number;
  chars: number;
}
```

### Comportamento

Mostra inline `salvato 14s fa` / `salvando…` / `modifiche non salvate` / `errore salvataggio` + `L:N` + `C:N` + `chars` + `~tok`. Tok stimato `Math.ceil(chars/4)` — proxy coerente con backend `Statistiche.token_medi`.

### Note

- Usa `var(--font-mono)` per i numeri.
- Saved usa `dot dot-ok` (verde), dirty usa giallo, errore rosso.

## 4. Component MarkdownToolbar.svelte

### Path

`apps/client/src/lib/components/MarkdownToolbar.svelte` (NEW)

### Props

```typescript
interface Props {
  view: EditorView | null;
  onInserisciVariabile?: () => void;
  onInserisciImport?: () => void;
}
```

### 18 azioni (decisione #3 mapping)

| Bottone | Icona lucide | Comando |
|---|---|---|
| Bold | `Bold` | wraps selection con `**…**` |
| Italic | `Italic` | wraps con `*…*` |
| separatore | `Minus` | (visivo) |
| Heading 1 | `Heading1` | prefisso `# ` riga corrente |
| Heading 2 | `Heading2` | prefisso `## ` |
| separatore | | |
| List unordered | `List` | prefisso `- ` riga corrente |
| List ordered | `ListOrdered` | prefisso `1. ` |
| Quote | `Quote` | prefisso `> ` |
| separatore | | |
| Code inline | `Code` | wraps con `` ` `` |
| Code block | `Code2` | wraps con triple backtick newline |
| Link | `Link` | wraps con `[…](url)` |
| HR | `Minus` | inserisce `\n---\n` |
| separatore | | |
| `+ {{var}}` | `Variable` | callback prop `onInserisciVariabile` |
| `+ import` | `GitFork` | callback prop `onInserisciImport` |
| separatore | | |
| Search | `Search` | apre CM6 search panel |

Stroke-width 1.75, size 14px, coerente con sidebar/list (decisione #3).

### Comandi CodeMirror

Usano `view.dispatch({ changes: ... })` con commands da `@codemirror/commands` per inserzioni/wrapping.

## 5. Component EditorTab.svelte

### Path

`apps/client/src/lib/components/EditorTab.svelte` (NEW)

### Props

```typescript
interface Props {
  body: string;
  onChangeBody: (newBody: string) => void;
  promptId: string | null;
  onApriPrompt?: (id: string) => void;
  editorView?: EditorView | null; // bindable
}
```

### Setup CodeMirror

```typescript
import { EditorView, lineNumbers, keymap } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { searchKeymap } from "@codemirror/search";
import { markdown } from "@codemirror/lang-markdown";
import { importTokens } from "$lib/codemirror/import-tokens";
import { placeholderHighlight } from "$lib/codemirror/placeholder-highlight";

const state = EditorState.create({
  doc: body,
  extensions: [
    lineNumbers(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
    markdown(),
    importTokens({ onapri: onApriPrompt }),
    placeholderHighlight(),
    EditorView.updateListener.of((u) => {
      if (u.docChanged) onChangeBody(u.state.doc.toString());
    }),
  ],
});
```

### Note

- View ricreata se `promptId` cambia (smount/mount con `$effect`).
- Plugin esistenti riusati 1:1.

## 6. Component DetailTabs.svelte

### Path

`apps/client/src/lib/components/DetailTabs.svelte` (NEW)

### Props

```typescript
type TabId = "editor" | "anteprima" | "diagnosi" | "golden" | "cronologia" | "import-var";

interface Props {
  tabAttivo: TabId;
  badge?: { diagnosi?: number; golden?: number; cronologia?: number; importVar?: number };
  onSeleziona: (tab: TabId) => void;
}
```

### Markup

Strip orizzontale con 6 button. Tab attivo: underline 2px `var(--accent-team)` + colore `var(--text-default)`. Badge count opzionale a destra del label (Diagnosi, Test golden, Cronologia, Import & Var.). F5 popolerà; F4 li accetta come prop ma renderizza 0 se non forniti.

## 7. Component DetailPane.svelte

### Path

`apps/client/src/lib/superfici/DetailPane.svelte` (NEW)

### Props

```typescript
interface Props {
  promptId: string;
  onChiudi?: () => void;
}
```

### Stato interno

```typescript
let dettaglio = $state<PromptDettaglio | null>(null);
let body = $state("");
let titolo = $state("");
let descrizione = $state("");
let statoSalvataggio = $state<"salvato" | "dirty" | "salvando" | "errore">("salvato");
let tabAttivo = $state<TabId>("editor");
let editorView = $state<EditorView | null>(null);
let righe = $state(1);
let colonna = $state(1);
let chars = $state(0);
```

### Comportamento

- onMount + `$effect` su promptId → `libreria_dettaglio({ id })` invoca cmd Tauri esistente
- Popola titolo/descrizione/body
- Autosave 2s idle (vedi §8)

### Header layout

- title-row: `<input>` titolo grande + actions destra (Star/Fork/Export/Compila/Meta)
- `<textarea>` descrizione inline (auto-grow)
- meta-row: chip statici (Privato/Team con dot accent, Cartella, Target, Uso, Aggiornato)
- `<DetailTabs>`

### Body

- Se `tabAttivo === "editor"`: `<MarkdownToolbar>` + `<EditorTab>` + `<EditorIndicator>`
- Altrimenti: placeholder F5 con label tab

## 8. Autosave 2s idle

```typescript
let timerAutosave: ReturnType<typeof setTimeout> | undefined;

function pianificaAutosave(): void {
  statoSalvataggio = "dirty";
  if (timerAutosave) clearTimeout(timerAutosave);
  timerAutosave = setTimeout(salvaBackground, 2000);
}

async function salvaBackground(): Promise<void> {
  if (!titolo.trim() || !body.trim() || !dettaglio) return;
  statoSalvataggio = "salvando";
  try {
    await invoke("prompt_aggiorna", {
      dati: {
        id: promptId,
        titolo: titolo.trim(),
        descrizione: descrizione.trim(),
        body: body.trim(),
        visibilita: dettaglio.visibilita,
        tag_nomi: dettaglio.tags.map((t) => t.nome),
        target_model: dettaglio.target_model || null,
        folder_id: dettaglio.folder_id,
      },
    });
    statoSalvataggio = "salvato";
    window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
  } catch (e) {
    console.error("[detail] save", e);
    statoSalvataggio = "errore";
  }
}
```

Pattern coerente con `EditorPrompt.svelte` legacy (timer 2000ms idle).

## 9. Wiring in Shell.svelte

### Diff

```svelte
<Pane defaultSize={54}>
  {#if promptSelezionato}
    <DetailPane
      promptId={promptSelezionato}
      onChiudi={() => (promptSelezionato = null)}
    />
  {:else}
    <div class="placeholder-pane detail-placeholder">
      <p>Seleziona un prompt dalla lista</p>
    </div>
  {/if}
</Pane>
```

## 10. Edge case + scope

| # | Caso | Comportamento |
|---|---|---|
| 1 | Prompt selezionato cancellato (race) | `libreria_dettaglio` ritorna errore → mostra "Prompt non trovato" |
| 2 | Save fallisce (backend offline) | statoSalvataggio = "errore" + console.error |
| 3 | Body vuoto + autosave | salvaBackground early return (no save di prompt vuoto) |
| 4 | Switch prompt durante save in corso | Annulla timer, vecchio save completa async, lista refresh post-evento |
| 5 | Tab cambia mentre autosave pending | Il timer continua, save avviene, tab già cambiato |
| 6 | Click bottone Star/Fork/Export/Compila | console.log "F8 ..." (placeholder) |
| 7 | CodeMirror con doc molto grande (>10k righe) | CM6 nativo gestisce, no degradation |
| 8 | Hot-update da pap:lista-mutata mentre tab Editor attivo | NON ricaricare il dettaglio (wipe modifiche locali). Listener selettivo: solo quando promptId cambia |

## 11. Test attesi

### Smoke test manuale

- [ ] Click su PromptCard in lista → DetailPane montata con titolo+descrizione+body live
- [ ] Tab strip mostra 6 tab, Editor attivo di default
- [ ] Click su altre tab cambia placeholder F5
- [ ] Modifica titolo → autosave 2s → indicator "salvato"
- [ ] Modifica body → autosave + indicator dirty → salvato
- [ ] Click su Bold/Italic/Heading1 ecc. → markup inserito a position cursor
- [ ] `+ {{var}}` inserisce `{{nome}}` con cursor selezionato
- [ ] `+ import` inserisce `{{import "path"}}`
- [ ] Token `{{var}}` highlighted (placeholder-highlight)
- [ ] Token `{{import "x"}}` con underline + Ctrl+click chiama callback
- [ ] Bottoni header (Star/Fork/Export/Compila/Meta) loggano in console
- [ ] EditorIndicator aggiorna L/C/chars
- [ ] Switch prompt da lista → DetailPane aggiorna senza memory leak

### Type-check + test client

- `npm run check`: 0 errors
- `npm test`: tutti pass (no nuovi test obbligatori — i component CM6 vanno testati manualmente; F11 aggiungerà plugin svelte-vitest)

## 12. Exit criteria

PR `feat/redesign-f4-detail-pane` può fare merge solo se:

- [ ] `@codemirror/lang-markdown` aggiunto come dep
- [ ] `EditorIndicator.svelte` con stato saved/dirty/salvando/errore + L/C/chars/tok
- [ ] `MarkdownToolbar.svelte` con 18 azioni lucide (mapping #3)
- [ ] `EditorTab.svelte` con CodeMirror 6 setup + plugin esistenti
- [ ] `DetailTabs.svelte` con 6 tab + badge count
- [ ] `DetailPane.svelte` con header + meta-row + slot tab + autosave
- [ ] `Shell.svelte` swap detail-placeholder → DetailPane (condizionale su promptSelezionato)
- [ ] Smoke test §11 passato manualmente
- [ ] `npm run check` 0 errors
- [ ] `npm test` tutti pass
- [ ] CI lint-and-test verde
- [ ] Bundle aggiunto: ≤ 30 KB gzip (lang-markdown ~15 KB + 5 component minimal)

## 13. Dipendenze su F5-F8

F4 sblocca:

- **F5** (Tab detail): popola le 5 tab placeholder con contenuto reale.
- **F6** (Right-rail): aggiunge secondo `<PaneGroup>` nidificato dentro DetailPane. Meta toggle in header diventa funzionale.
- **F7** (StatusBar): nessuna dipendenza diretta. F7 aggiorna `StatusBar` con prompt corrente (legge `promptSelezionato` da Shell).
- **F8 modali**: bottoni header (Star/Fork/Export/Compila) wireuppati alle modali.

**Interface contract** che F4 espone:

```typescript
// $lib/superfici/DetailPane.svelte — pannello detail
// $lib/components/DetailTabs.svelte — tab strip riusabile
// $lib/components/EditorTab.svelte — wrapper CodeMirror
// $lib/components/MarkdownToolbar.svelte — toolbar 18 azioni
// $lib/components/EditorIndicator.svelte — indicator state
```

CustomEvent emessi:
- `pap:lista-mutata` dopo ogni save (refresh sidebar conteggi)

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se durante implementazione `@codemirror/lang-markdown` mostra issue di compatibilità o se l'autosave race condition richiede stato globale invece di locale.
