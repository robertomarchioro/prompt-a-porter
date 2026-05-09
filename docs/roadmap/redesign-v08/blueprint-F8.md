# Blueprint F8 — 5 modali (Compila / Insight / Regressioni / Impostazioni / Palette)

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F8 · **Decisioni designer**: #6 (rating ±1 + nota in Compila), #7 (slider hybrid alpha in pannello "Filtri avanzati" Palette), #12 (Impostazioni 5 macro + 5 sub-sezioni accordion in Avanzate + ⌘K) · **Stima**: 11 giorni FT · **Bloccato da**: F4 ✅ (header bottoni placeholder)

Sostituisce i placeholder di click su bottoni Star/Fork/Esporta MD/Compila/⚙/⌃⇧P + footer Sidebar (Insight/Regressioni) + bottoni "+" (cartella/tag/golden/variante) con modali reali porting delle superfici legacy.

**Spezzato in 5 sub-PR autonome**:

| Sub-PR | Modale | Stima |
|---|---|---|
| **F8 PR-A** | Compila + primitive Modale + store globale | 2.5 gg |
| **F8 PR-B** | Insight (dashboard) | 1 gg |
| **F8 PR-C** | Regressioni (drift report) | 1.5 gg |
| **F8 PR-D** | Impostazioni (5 macro + 5 sub-sezioni accordion + ⌘K) | 5 gg |
| **F8 PR-E** | Palette (cmdk-like + ⌃⇧P + slider hybrid) | 2 gg |

Tutte contro `feat/redesign-v08`. Ogni sub-PR è autonoma.

## Indice

1. [F8 PR-A — Modale primitive + Compila](#1-f8-pr-a--modale-primitive--compila)
2. [F8 PR-B — Insight](#2-f8-pr-b--insight)
3. [F8 PR-C — Regressioni](#3-f8-pr-c--regressioni)
4. [F8 PR-D — Impostazioni](#4-f8-pr-d--impostazioni)
5. [F8 PR-E — Palette](#5-f8-pr-e--palette)
6. [Pattern comune di integrazione](#6-pattern-comune-di-integrazione)

---

## 1. F8 PR-A — Modale primitive + Compila

### Path

- `apps/client/src/lib/components/Modale.svelte` (NEW — primitive riusabile)
- `apps/client/src/lib/stores/modale.svelte.ts` (NEW — store globale)
- `apps/client/src/lib/superfici/CompilaModal.svelte` (NEW — porting Compilatore)

### Primitive Modale

```svelte
<!-- backdrop + container + ESC + click-outside + a11y -->
<script lang="ts">
  interface Props {
    titolo: string;
    sottotitolo?: string;
    onChiudi: () => void;
    larghezza?: "sm" | "md" | "lg" | "xl";
    children: Snippet;
    footer?: Snippet;
  }
</script>
<div role="dialog" aria-modal="true" class="backdrop" onclick={onChiudi}>
  <div class="container" data-w={larghezza} onclick={(e) => e.stopPropagation()}>
    <header>{titolo} <button onclick={onChiudi}><X/></button></header>
    {@render children()}
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>
```

ESC handler globale via `window.addEventListener("keydown", ...)` con cleanup onDestroy.

### Store globale `modale.svelte.ts`

```typescript
type ModaleAttiva =
  | { tipo: "compila"; promptId: string }
  | { tipo: "insight" }
  | { tipo: "regressioni" }
  | { tipo: "impostazioni"; sezione?: string }
  | { tipo: "palette" }
  | null;

class StatoModale {
  attiva = $state<ModaleAttiva>(null);
}

export const statoModale = new StatoModale();
export function apriModale(modale: NonNullable<ModaleAttiva>): void {
  statoModale.attiva = modale;
}
export function chiudiModale(): void {
  statoModale.attiva = null;
}
```

Shell.svelte mounta la modale appropriata in fondo al template.

### CompilaModal.svelte

Porting da `CompilatorePrompt.svelte` legacy (927 righe → ridotto). Funzioni:
- Form segnaposti type-aware (testo/multilinea/enum)
- Preview live a destra del form
- Output 3 formati (testo / markdown / json)
- **Rating ±1 con icone Frown/Meh/Smile** (decisione #6) + campo nota collassato in `<details>`
- Bottone "Compila & copia" (`⌃↵`)
- Cmd Tauri: `prompt_dettaglio` per body/segnaposti, `rating_aggiungi` post-compilazione

### Bottoni che apriranno la modale (post F8 PR-A)

- `DetailPane.svelte`: bottone "Compila" header → `apriModale({ tipo: "compila", promptId })`
- `F8 PR-E Palette`: azione "Compila & copia (⌃↵)"

### Stima

2.5 gg (di cui 0.5 primitive + store)

---

## 2. F8 PR-B — Insight

### Path

`apps/client/src/lib/superfici/InsightModal.svelte` (NEW)

### Scope

Porting `Insight.svelte` legacy (573 righe) come modale. Mostra:
- Totali (prompt attivi/eliminati, tag, ultimi 30g, versioni totali)
- Top 10 prompt più usati (lista)
- Prompt non usati (lista)
- Distribuzione per tag / target model / visibilità
- Lint health % + breakdown categorie
- **`token_medi`** (V014 backend)

Cmd: `statistiche_query` esistente.

Aperto da Sidebar footer "Insight" + SidebarMini icon BarChart3.

### Stima

1 gg

---

## 3. F8 PR-C — Regressioni

### Path

`apps/client/src/lib/superfici/RegressioniModal.svelte` (NEW)

### Scope

Porting `Regressioni.svelte` legacy (368 righe). Larghezza fissa `min(1200px, 92vw)`.

Mostra:
- Tabella drift score per (prompt × provider × model) ultimi N giorni
- Slider giorni 1-90 (default 30)
- Bottone "Esporta CSV" (preservato — feature scope-drift designer)
- Cmd: `regression_report({ giorni })`, `regression_report_csv({ giorni })`

Aperto da Sidebar footer "Regressioni" + SidebarMini icon AlertTriangle.

### Stima

1.5 gg

---

## 4. F8 PR-D — Impostazioni

### Path

`apps/client/src/lib/superfici/ImpostazioniModal.svelte` (NEW, ~700-1000 righe stimate)

### Struttura (decisione #12)

5 macro-sezioni navigabili a sinistra + dettaglio a destra:

1. **Aspetto** — tema dropdown (auto/dark/light) + tono palette (zinc/slate/stone)
2. **Vista lista** — densità default + righe preview
3. **Editor** — autosave delay, line wrapping, ecc.
4. **Sicurezza** — vault path, lock now, cambia master key
5. **Avanzate** (5 sub-sezioni accordion):
   - **Provider AI** (config endpoint + model preset per Anthropic/OpenAI/OpenAI-compat/Ollama/Gemini)
   - **Ricerca & Embeddings** (status MiniLM, reindex button, hybrid alpha default)
   - **Audit log AI** (toggle, retention, export)
   - **Sync** (placeholder v0.9)
   - **Hotkey** (table custom)

### ⌘K cerca sub-sezioni

Input search in alto (⌘K focus) filtra le voci accordion in Avanzate.

### Cmd Tauri

- `preferenze_carica` / `preferenze_salva` esistenti
- `provider_config_carica` / `provider_config_salva` esistenti
- `vault_cambia_password`, `vault_lock`
- `embeddings_status`, `embeddings_reindex` (verifica)

### Aperto da

- TitleBar `⚙` icon (da aggiungere)
- Palette `⌘,` shortcut

### Stima

5 gg (la più grossa di F8 — possibile split in PR-D1 macro Aspetto/Vista/Editor/Sicurezza + PR-D2 Avanzate sub-sezioni)

---

## 5. F8 PR-E — Palette

### Path

`apps/client/src/lib/superfici/PaletteModal.svelte` (NEW)

### Scope

Porting `CommandPalette.svelte` legacy (628 righe). Sostituisce window separata Tauri esistente con modale interna.

- Input text con focus on open
- Sezioni: Prompt recenti / Azioni
- Item types: prompt / tag / cartella / azione
- Keyboard nav ↑↓ ↵ ESC
- ⌃↵ Compila & copia (apre CompilaModal)
- **Pannello "Filtri avanzati" collassato** (decisione #7): toggle `⌘.` o chevron, default chiuso, persistito localStorage
  - Slider Vector ↔ Keyword (0-1, default 0.5)
  - Filtro modello target
  - Filtro tag esclusivo
- Cmd: `ricerca_ibrida({ query, alpha, target_model?, tag_id? })` esistente

### Aperto da

- Hotkey globale ⌃⇧P (`global-shortcut` plugin Tauri)
- TitleBar search-as-palette
- StatusBar kbd `⌃⇧P`
- F5 PR-F bottone "Confronta selezionati" → azione palette "Confronta prompt selezionati…"

### Stima

2 gg

---

## 6. Pattern comune di integrazione

Per ogni sub-PR:

1. Creare component `lib/superfici/<Nome>Modal.svelte` che usa `<Modale>` primitive (creata in PR-A)
2. Estendere store `modale.svelte.ts` con il tipo nuovo se necessario
3. Aggiornare Shell.svelte: mount modale condizionale basato su `statoModale.attiva.tipo`
4. Wire-up trigger bottoni nelle superfici esistenti (DetailPane, Sidebar, TitleBar, StatusBar)

Tutte le modali seguono:
- Backdrop `var(--bg-scrim)` con click-outside-to-close
- Container `var(--bg-raised)` border `var(--border-subtle)` border-radius `var(--radius-md)`
- Header con titolo + sottotitolo + ✕
- Footer azioni a destra (primary/secondary)
- ESC chiude (gestito da primitive)

### Interface contract cumulativo

```typescript
// $lib/components/Modale.svelte — primitive backdrop+container
// $lib/stores/modale.svelte.ts — statoModale + apriModale + chiudiModale
// $lib/superfici/CompilaModal.svelte — F8 PR-A
// $lib/superfici/InsightModal.svelte — F8 PR-B
// $lib/superfici/RegressioniModal.svelte — F8 PR-C
// $lib/superfici/ImpostazioniModal.svelte — F8 PR-D
// $lib/superfici/PaletteModal.svelte — F8 PR-E
```

---

> **Stato blueprint**: 1.0 — pronto per esecuzione iterativa.
