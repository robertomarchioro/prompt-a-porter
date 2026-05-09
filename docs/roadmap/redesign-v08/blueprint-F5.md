# Blueprint F5 — Tab detail (5 tab + Diff libero scope drift)

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F5 · **Decisioni designer**: #8 (riduzione N-way + nuovo Diff libero scope drift), #9 (diff side-by-side default + toggle unified, fallback unified <900px), #13 (avatar autore SHA1 deterministico) · **Stima**: 13 giorni FT (era 11.5 + 1.5 scope drift) · **Bloccato da**: F4 ✅, F6 ✅, V014 backend autore ✅

Popola le 5 tab placeholder del DetailPane con contenuto reale + aggiunge "Diff libero" come azione separata in Palette (scope drift decisione #8).

**Spezzato in 6 sub-PR** per evitare PR monolitiche:

| Sub-PR | Tab | Stima |
|---|---|---|
| **F5 PR-A** | Anteprima | 1 gg |
| **F5 PR-B** | Diagnosi | 1.5 gg |
| **F5 PR-C** | Test golden | 2 gg |
| **F5 PR-D** | Cronologia (con autore + diff side-by-side) | 4 gg |
| **F5 PR-E** | Import & Var. (porting ConfrontoPrompt → A/B/C) | 3 gg |
| **F5 PR-F** | Diff libero (scope drift #8 + multi-select ListPane) | 1.5 gg |

Tutte contro `feat/redesign-v08`. Ogni sub-PR è autonoma.

## Indice

1. [F5 PR-A — Anteprima](#1-f5-pr-a--anteprima)
2. [F5 PR-B — Diagnosi](#2-f5-pr-b--diagnosi)
3. [F5 PR-C — Test golden](#3-f5-pr-c--test-golden)
4. [F5 PR-D — Cronologia](#4-f5-pr-d--cronologia-con-autore--diff-side-by-side)
5. [F5 PR-E — Import & Var.](#5-f5-pr-e--import--var-porting-confronto)
6. [F5 PR-F — Diff libero](#6-f5-pr-f--diff-libero-scope-drift-8)
7. [Pattern comune di integrazione](#7-pattern-comune-di-integrazione)
8. [Dipendenze su F8](#8-dipendenze-su-f8)

---

## 1. F5 PR-A — Anteprima

### Path

`apps/client/src/lib/components/AnteprimaTab.svelte` (NEW)

### Scope F5 PR-A

- Render del body in `<pre>` mono 13px line-height 1.65 su `bg-surface`
- Segnaposti `{{nome}}` evidenziati con `accent-private` soft (placeholder visivi)
- Import `{{import "x"}}` evidenziati con `info` soft (richiamo visivo)
- **NO risoluzione default segnaposti** (F8 modale Compila introdurrà storage default → F5 PR-A.x potrà sostituire)
- **NO risoluzione import composti** (costoso N invoke; F5 PR-A.x può aggiungerlo se richiesto)

### Markup chiave

```svelte
<script lang="ts">
  interface Props {
    body: string;
  }
  let { body }: Props = $props();

  type Segmento =
    | { tipo: "testo"; testo: string }
    | { tipo: "segnaposto"; nome: string }
    | { tipo: "import"; path: string };

  // Regex unica: import OR segnaposto. Capture group decide il tipo.
  const RE = /(\{\{\s*import\s+"([^"]+)"\s*\}\})|(\{\{\s*(\w+)\s*\}\})/g;

  function parsa(testo: string): Segmento[] {
    const acc: Segmento[] = [];
    let last = 0;
    let m: RegExpExecArray | null;
    RE.lastIndex = 0;
    while ((m = RE.exec(testo)) !== null) {
      if (m.index > last) acc.push({ tipo: "testo", testo: testo.slice(last, m.index) });
      if (m[2]) acc.push({ tipo: "import", path: m[2] });
      else if (m[4]) acc.push({ tipo: "segnaposto", nome: m[4] });
      last = m.index + m[0].length;
    }
    if (last < testo.length) acc.push({ tipo: "testo", testo: testo.slice(last) });
    return acc;
  }

  const segmenti = $derived(parsa(body));
</script>
```

### Layout CSS

```css
.anteprima {
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.65;
  background: var(--bg-surface);
  color: var(--text-default);
  padding: var(--sp-3);
  white-space: pre-wrap;
  margin: 0;
  height: 100%;
  overflow-y: auto;
}

.ph {
  background: var(--accent-private-soft);
  color: var(--accent-private);
  border-radius: var(--radius-sm);
  padding: 1px 4px;
}

.imp {
  background: var(--info-soft);
  color: var(--info);
  border-radius: var(--radius-sm);
  padding: 1px 4px;
  text-decoration: underline dotted;
}
```

### Stima

1 gg.

## 2. F5 PR-B — Diagnosi

### Path

`apps/client/src/lib/components/DiagnosiTab.svelte` (NEW)

### Scope

- Lista warning/error per riga del body
- Riusa cmd `prompt_lint` esistente
- Click su riga → switcha a tab Editor + scroll to line + selezione (custom event `pap:goto-line` con detail line number)
- Categorie LEN/PH/PII/STY/IMP con icona + tooltip

### Stima

1.5 gg.

## 3. F5 PR-C — Test golden

### Path

`apps/client/src/lib/components/GoldenTab.svelte` (NEW)

### Scope

- Tabella golden con drift score + last-run
- Cmd esistenti: `golden_lista`, `golden_esegui`, `golden_crea`, `golden_aggiorna`, `golden_elimina`
- Bottoni: "+ Golden", "Esegui tutti", per riga "Esegui" / "Modifica" / "Elimina"

### Stima

2 gg.

## 4. F5 PR-D — Cronologia con autore + diff side-by-side

### Path

- `apps/client/src/lib/components/CronologiaTab.svelte` (NEW)
- `apps/client/src/lib/components/DiffViewer.svelte` (NEW)
- `apps/client/src/lib/util/avatar-hash.ts` (NEW — SHA1+HSL conformi #13)

### Scope

- Lista versioni da `prompt_get_history` (esteso V014 con autore)
- Avatar autore deterministico SHA1+HSL (decisione #13)
- Click su versione → diff side-by-side default (decisione #9) via `diff2html`
- Toggle unified ↔ side-by-side persistito per-utente
- Fallback automatico unified sotto 900px viewport (responsive guard)
- Bottone rollback per versione

### Dep nuova

- `diff2html ^3.4.56` (~40 KB gzip)

### Avatar hash (SHA1)

```typescript
async function sha1(s: string): Promise<string> {
  const enc = new TextEncoder().encode(s);
  const hash = await crypto.subtle.digest("SHA-1", enc);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export async function avatarColorePerEmail(email: string): Promise<{
  background: string;
  foreground: string;
}> {
  const h = await sha1(email);
  const hex = h.slice(0, 6);
  const hue = parseInt(hex, 16) % 360;
  return {
    background: `hsl(${hue} 55% 58%)`,
    foreground: "#fff",
  };
}
```

NB: SHA1 async via `crypto.subtle.digest`. F2 usa djb2 sync per workspace, F5 usa SHA1 async per email autore (decisione #13).

### Stima

4 gg.

## 5. F5 PR-E — Import & Var. (porting Confronto)

### Path

`apps/client/src/lib/components/ImportVarTab.svelte` (NEW)

### Scope

- Sezione **Import composti**: lista risolta (chiama `prompt_resolve_import_preview` per ogni path) con titolo + body snippet
- Sezione **Varianti A/B/C**: pill orizzontali da `varianti_lista` con tab attivo evidenziato
- Toggle "Confronta tutte" → grid 3-colonne con titolo+body delle varianti affiancate
- Porting da `ConfrontoPrompt.svelte` esistente ridotto ad A/B/C (decisione #8)

### Stima

3 gg.

## 6. F5 PR-F — Diff libero (scope drift #8)

### Path

- `apps/client/src/lib/superfici/DiffLibero.svelte` (NEW)
- Estensione `ListPane.svelte` con `⌘+click` multi-select
- Estensione `Palette.svelte` (F8) con azione "Confronta prompt selezionati…"

### Scope

- Tab/azione separata in Command Palette F8
- Selezione multipla con `⌘+click` in list pane → apre vista N-way arbitrario fino a 4 colonne
- NON inserito come tab del DetailPane (è cross-prompt)
- Render side-by-side via `diff2html`

### Stima

1.5 gg.

## 7. Pattern comune di integrazione

Per ogni sub-PR:

1. Crea component `<NomeTab>.svelte` in `lib/components/`
2. Modifica `DetailPane.svelte`: `{#if tabAttivo === "X"}<XTab .../>...{/if}` sostituisce il placeholder
3. Passa `body`, `promptId`, callback necessari
4. Per `DetailTabs.svelte` badge count: passa via prop (es. `badge.diagnosi = lintIssues.length`)

Tutte le sub-PR usano cmd Tauri **esistenti** — no backend nuovo richiesto (a parte F5 PR-D che usa il nuovo `prompt_get_history` con autore già live in V014).

## 8. Dipendenze su F8

F5 sblocca:

- **F8 Palette modale**: F5 PR-F aggiunge azione "Confronta prompt selezionati…" che la Palette deve esporre
- **F8 modale Compila**: F5 PR-A può leggere default values quando F8 li introduce
- **F8 modale Crea Variante**: F5 PR-E sezione Varianti aggiunge bottone "+ Variante" che apre la modale F8

**Interface contract** che F5 espone (cumulativo dopo tutte le sub-PR):

```typescript
// $lib/components/AnteprimaTab.svelte
// $lib/components/DiagnosiTab.svelte
// $lib/components/GoldenTab.svelte
// $lib/components/CronologiaTab.svelte
// $lib/components/DiffViewer.svelte
// $lib/components/ImportVarTab.svelte
// $lib/superfici/DiffLibero.svelte
// $lib/util/avatar-hash.ts
// CustomEvent "pap:goto-line" — detail: number — F5 Diagnosi → Editor scroll
```

---

> **Stato blueprint**: 1.0 — pronto per esecuzione iterativa. Ogni sub-PR aggiorna questo blueprint con dettagli post-merge.
