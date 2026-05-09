# Blueprint F6 — Right-rail metadati

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F6 · **Decisioni designer**: #1 (visibilità dropdown estensibile a futuri ruoli) · **Stima**: 3 giorni FT · **Bloccato da**: F4 ✅ (integrazione DetailPane + secondo PaneGroup)

Aggiunge il pannello laterale destro al `DetailPane` con metadati interattivi. Sostituisce il bottone "Meta toggle" placeholder di F4 con uno toggle funzionale che mostra/nasconde il rail. Il rail contiene 4 sezioni: Metadati (visibilità + target + cartella + tag), Segnaposti rilevati, Import composti, Varianti A/B.

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery (1 PR)](#2-strategia-di-delivery-1-pr)
3. [Util client estrai-imports.ts](#3-util-client-estrai-importsts)
4. [Component RightRail.svelte](#4-component-rightrailsvelte)
5. [Sezione Metadati (decisione #1)](#5-sezione-metadati-decisione-1)
6. [Sezione Segnaposti rilevati](#6-sezione-segnaposti-rilevati)
7. [Sezione Import composti](#7-sezione-import-composti)
8. [Sezione Varianti A/B](#8-sezione-varianti-ab)
9. [Modifica DetailPane.svelte](#9-modifica-detailpanesvelte)
10. [Persistenza Meta collapsed](#10-persistenza-meta-collapsed)
11. [Edge case + scope](#11-edge-case--scope)
12. [Test attesi](#12-test-attesi)
13. [Exit criteria](#13-exit-criteria)
14. [Dipendenze su F5/F8](#14-dipendenze-su-f5f8)

---

## 1. Obiettivo

**Output funzionale F6**:
- Right-rail con 4 sezioni
- Visibilità come **dropdown estensibile** (decisione #1): voci attive Privato/Team + voci future disabilitate (Workspace condiviso, Pubblico/Marketplace) con tooltip "Disponibile in v0.9+"
- Target model select (datalist con modelli predefiniti + custom)
- Cartella select (lista da `folder_lista`)
- Tag picker (chip rimovibili + input + tag suggeriti semantici da `tags_suggest`)
- Lista segnaposti rilevati nel body (parser client `estraiSegnaposti` esistente)
- Lista import composti (parser client nuovo `estraiImports`)
- Varianti A/B come pill orizzontali da `varianti_lista` + bottone "+ Variante" + "Confronta tutte" (logged F5)
- Toggle Meta in DetailPane header diventa funzionale: mostra/nasconde il rail
- Stato collapsed persistito in localStorage

**Out of scope F6**:
- Bottone "+ Variante" attivo (richiede modale F8 o flusso Compila)
- Bottone "Confronta tutte" (apre tab Import & Var. di F5)
- Aggiunta nuovo target/cartella inline (riusa quelli esistenti, F8 modali per crearne nuovi)

## 2. Strategia di delivery (1 PR)

- **Branch**: `feat/redesign-f6-right-rail`
- **Target**: `feat/redesign-v08`
- **Effort**: 3 gg
  - 0.25 gg util `estrai-imports.ts` + test
  - 0.5 gg `RightRail.svelte` skeleton + sezioni
  - 0.75 gg sezione Metadati (visibilità dropdown + target + cartella + tag picker)
  - 0.5 gg sezione Segnaposti rilevati + Import composti
  - 0.5 gg sezione Varianti A/B (pill A/B/C + bottoni)
  - 0.5 gg DetailPane secondo PaneGroup + Meta toggle
  - 0.25 gg smoke + commit

## 3. Util client `estrai-imports.ts`

### Path

`apps/client/src/lib/util/estrai-imports.ts` (NEW)

### Implementazione

```typescript
/**
 * Estrae i path degli {{import "..."}} dal body di un prompt.
 * Pattern stesso usato lato backend (prompt_componibili::re_import) e
 * dal plugin CodeMirror import-tokens.ts.
 */
const RE = /\{\{\s*import\s+"([^"]+)"\s*\}\}/g;

export function estraiImports(body: string): string[] {
  const set = new Set<string>();
  let m: RegExpExecArray | null;
  RE.lastIndex = 0;
  while ((m = RE.exec(body)) !== null) {
    set.add(m[1]);
  }
  return Array.from(set);
}
```

### Test

`apps/client/src/lib/util/estrai-imports.test.ts`:
- Body senza import → []
- Body con 1 import → 1 path
- Body con import duplicati → dedup
- Body con whitespace nei pattern → matching corretto
- Body con import + altre `{{var}}` → solo import path

## 4. Component `RightRail.svelte`

### Path

`apps/client/src/lib/components/RightRail.svelte` (NEW)

### Props

```typescript
interface TagInfo {
  id: string;
  nome: string;
  colore: string;
}

interface Props {
  promptId: string;
  body: string;
  visibilita: string;
  targetModel: string;
  folderId: string | null;
  tags: TagInfo[];
  onCambiaVisibilita: (v: string) => void;
  onCambiaTarget: (t: string) => void;
  onCambiaFolder: (f: string | null) => void;
  onAggiungiTag: (nome: string) => void;
  onRimuoviTag: (id: string) => void;
  onApriTabImportVar?: () => void;
}
```

### Layout

Stack verticale scrollable con 4 sezioni separate da border-subtle. Ogni sezione ha header (titolo uppercase 10px stile NavGroup) + body. Per minimizzare scope creep, **F6 inlinea le 4 sezioni dentro RightRail.svelte** invece di creare 4 component separati.

## 5. Sezione Metadati (decisione #1)

### Visibilità dropdown estensibile

```svelte
<select
  class="select"
  value={visibilita}
  onchange={(e) => onCambiaVisibilita(e.currentTarget.value)}
>
  <option value="private">Privato</option>
  <option value="workspace">Team</option>
  <optgroup label="Disponibili in versioni future">
    <option value="" disabled>Workspace condiviso (v0.9+)</option>
    <option value="" disabled>Pubblico/Marketplace (v1.0+)</option>
  </optgroup>
</select>
```

NB: `<optgroup>` HTML standard rende disabled le opzioni elegantemente. Voci future hanno `value=""` per non interferire col bind.

### Target model

```svelte
<input
  list="target-models"
  class="input"
  value={targetModel}
  oninput={(e) => onCambiaTarget(e.currentTarget.value)}
  placeholder="Modello target"
/>
<datalist id="target-models">
  <option value="claude-sonnet-4-6">Claude Sonnet 4.6</option>
  <option value="claude-opus-4-5">Claude Opus 4.5</option>
  <option value="claude-haiku-4-5">Claude Haiku 4.5</option>
  <option value="gpt-4o">GPT-4o</option>
  <option value="ollama">Ollama (locale)</option>
</datalist>
```

### Cartella

```svelte
<select
  class="select"
  value={folderId ?? ""}
  onchange={(e) => onCambiaFolder(e.currentTarget.value || null)}
>
  <option value="">Nessuna cartella</option>
  {#each cartelle as c (c.id)}
    <option value={c.id}>{c.path}</option>
  {/each}
</select>
```

### Tag picker

- Chip rimovibili dei tag attuali
- Input per aggiungere (Enter conferma)
- Sotto: lista tag suggeriti (max 5) da `tags_suggest({ testo: titolo + " " + body, limit: 5 })`
- Click su tag suggerito → `onAggiungiTag(nome)`

Debounce 500ms su input body change prima di invocare `tags_suggest`.

## 6. Sezione Segnaposti rilevati

Usa util esistente `estraiSegnaposti(body)` da `lib/template.ts`.

```svelte
{#each estraiSegnaposti(body) as s (s.nome)}
  <div class="segnaposto">
    <code class="ph">{`{{${s.nome}}}`}</code>
    <span class="tipo">testo</span>
  </div>
{/each}
```

F6 mostra solo la lista con tipo "testo" default. F8 può aggiungere editor di tipo/default (modale Compila).

## 7. Sezione Import composti

Usa util nuovo `estraiImports(body)` (vedi §3).

```svelte
{#each estraiImports(body) as path (path)}
  <button class="import-row" onclick={() => apriImport(path)}>
    <GitFork size={12} />
    <code>{path}</code>
  </button>
{/each}
```

`apriImport(path)` invoca cmd `prompt_resolve_import_preview({ path })` → ritorna `{id, titolo, body}`. Se OK, dispatch evento `pap:apri-prompt` con id.

## 8. Sezione Varianti A/B

Usa `varianti_lista(parent_id)` cmd esistente.

```svelte
<div class="varianti">
  {#each varianti as v (v.id)}
    <button
      class="pill"
      class:active={v.id === promptId}
      onclick={() => apriVariante(v.id)}
    >
      {v.variant_label}
    </button>
  {/each}
  <button class="pill add" onclick={() => console.log("F8 modale crea variante")}>
    + Variante
  </button>
</div>
{#if varianti.length > 1}
  <button class="link" onclick={onApriTabImportVar}>Confronta tutte</button>
{/if}
```

`apriVariante` dispatch evento `pap:apri-prompt` con id (gestito da Shell).

## 9. Modifica DetailPane.svelte

### Secondo PaneGroup nidificato

```svelte
<article class="detail-pane">
  <header class="detail-header">...</header>

  <PaneGroup direction="horizontal" autoSaveId="detail-rail-v08">
    <Pane defaultSize={metaCollapsed ? 100 : 70} minSize={50}>
      <div class="detail-body">
        {#if tabAttivo === "editor"}
          <MarkdownToolbar/><EditorTab/><EditorIndicator/>
        {:else}
          placeholder
        {/if}
      </div>
    </Pane>
    {#if !metaCollapsed}
      <PaneResizer class="resizer" />
      <Pane defaultSize={30} minSize={20} maxSize={40}>
        <RightRail
          {promptId}
          body={body}
          visibilita={dettaglio.visibilita}
          targetModel={dettaglio.target_model}
          folderId={dettaglio.folder_id}
          tags={dettaglio.tags}
          onCambiaVisibilita={(v) => { dettaglio.visibilita = v; pianificaAutosave(); }}
          onCambiaTarget={(t) => { dettaglio.target_model = t; pianificaAutosave(); }}
          onCambiaFolder={(f) => { dettaglio.folder_id = f; pianificaAutosave(); }}
          onAggiungiTag={(nome) => aggiungiTag(nome)}
          onRimuoviTag={(id) => rimuoviTag(id)}
          onApriTabImportVar={() => (tabAttivo = "import-var")}
        />
      </Pane>
    {/if}
  </PaneGroup>
</article>
```

### Meta toggle attivo

```svelte
<button
  class="ico meta-toggle"
  type="button"
  onclick={() => (metaCollapsed = !metaCollapsed)}
  title={metaCollapsed ? "Mostra metadata" : "Nascondi metadata"}
>
  <PanelRight size={14} />
</button>
```

### Tag handlers

`prompt_aggiorna` esistente accetta `tag_nomi: string[]`. Aggiungere/rimuovere:

```typescript
function aggiungiTag(nome: string): void {
  if (!dettaglio || dettaglio.tags.some((t) => t.nome === nome)) return;
  dettaglio.tags = [...dettaglio.tags, { id: `tmp-${nome}`, nome, colore: "" }];
  pianificaAutosave();
}

function rimuoviTag(id: string): void {
  if (!dettaglio) return;
  dettaglio.tags = dettaglio.tags.filter((t) => t.id !== id);
  pianificaAutosave();
}
```

NB: il backend genera l'id reale al save successivo. UI usa nome come chiave logica.

## 10. Persistenza Meta collapsed

LocalStorage key `pap.detail.meta-collapsed` con valore boolean. Default `false`.

```typescript
const KEY = "pap.detail.meta-collapsed";

function caricaMetaCollapsed(): boolean {
  try {
    return localStorage.getItem(KEY) === "1";
  } catch {
    return false;
  }
}

function salvaMetaCollapsed(v: boolean): void {
  try {
    localStorage.setItem(KEY, v ? "1" : "0");
  } catch { /* ignore */ }
}
```

## 11. Edge case + scope

| # | Caso | Comportamento |
|---|---|---|
| 1 | Body vuoto | Sezione Segnaposti vuota, sezione Import vuota |
| 2 | Prompt senza varianti | Sezione Varianti mostra solo bottone "+ Variante" |
| 3 | Variante che è essa stessa parent | Backend gestisce — F6 visualizza con label dalla struct backend |
| 4 | tags_suggest backend offline | Fallback: nessun suggerimento, no crash |
| 5 | Click su import path inesistente | `prompt_resolve_import_preview` ritorna errore → console error, no toast |
| 6 | Switch prompt mentre rail aperto | Re-load varianti_lista + suggerimenti |
| 7 | Resize rail oltre 40% | clamp paneforge 40% |
| 8 | Visibilità "Workspace condiviso" cliccato | option disabled, no-op nativo HTML |
| 9 | Tag duplicato aggiunto | Skip silently |
| 10 | Cartella eliminata mentre selezionata | Reload `folder_lista`, dropdown si aggiorna; fallback "Nessuna cartella" |

## 12. Test attesi

### Unit test inline

`apps/client/src/lib/util/estrai-imports.test.ts`:
- Body senza import → []
- Body con import singolo
- Body con import multipli (dedup)
- Body con whitespace nei pattern
- Body misto import + segnaposti

### Smoke test manuale

- [ ] DetailPane apre con rail visibile (default)
- [ ] Meta toggle nasconde rail; click ripristina
- [ ] Stato collapsed persiste tra reload
- [ ] Cambio visibilità via dropdown → autosave
- [ ] Cambio target via datalist → autosave
- [ ] Cambio cartella via select → autosave
- [ ] Aggiungi tag via Enter → chip + autosave
- [ ] Rimuovi tag via X chip → autosave
- [ ] Suggerimenti tag aggiornati dopo edit body (debounced 500ms)
- [ ] Lista segnaposti sincronizzata col body editor
- [ ] Lista import composti sincronizzata
- [ ] Click su import row → console "F4/F5 apri prompt: <id>"
- [ ] Pill varianti mostrate, click su "+ Variante" → console "F8 modale"
- [ ] Click "Confronta tutte" → switcha tab a Import & Var.

### Type-check

- `npm run check`: 0 errors
- `npm test`: tutti pass

## 13. Exit criteria

PR `feat/redesign-f6-right-rail` può fare merge solo se:

- [ ] `lib/util/estrai-imports.ts` + test (≥ 4 casi)
- [ ] `lib/components/RightRail.svelte` con 4 sezioni
- [ ] Visibilità dropdown estensibile (decisione #1)
- [ ] Tag picker con suggerimenti `tags_suggest`
- [ ] Varianti A/B con `varianti_lista`
- [ ] DetailPane con secondo PaneGroup nidificato + Meta toggle funzionale + persistenza
- [ ] Smoke test §12 passato manualmente
- [ ] `npm run check` 0 errors
- [ ] `npm test` tutti pass
- [ ] CI lint-and-test verde
- [ ] Bundle aggiunto: ≤ 6 KB gzip

## 14. Dipendenze su F5/F8

F6 sblocca:

- **F5 tab Import & Var.**: callback `onApriTabImportVar` switcha tab — F5 popolerà il contenuto reale
- **F5 tab Diagnosi**: nessuna dipendenza diretta
- **F8 modale Compila**: leggerà sezione Segnaposti per type-aware input
- **F8 modale crea cartella/tag**: bottoni "+ cartella/tag" in F2 NavGroup wireuppano alle modali
- **F8 modale crea variante**: bottone "+ Variante" rail wireuppa alla modale

**Interface contract** che F6 espone:

```typescript
// $lib/components/RightRail.svelte — pannello laterale destro
// $lib/util/estrai-imports.ts — parser client {{import "x"}}
// localStorage key "pap.detail.meta-collapsed"
// CustomEvent "pap:apri-prompt" — detail: string id (per import row click + variante click)
```

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se durante implementazione `tags_suggest` mostra latenza eccessiva.
