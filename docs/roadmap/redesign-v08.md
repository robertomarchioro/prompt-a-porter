# Redesign UX/UI v0.8 — Piano di esecuzione

> **Versione documento**: 1.0 · **Data**: 2026-05-08 · **Autore piano**: Roberto Marchioro · **Sorgente design**: `docs/architettura/redesign/` · **Brief sorgente**: `docs/architettura/design-handoff/2026-05-08-redesign-brief.md`

Questo documento è il piano operativo per portare il redesign del prototipo in `docs/architettura/redesign/` dentro l'app `prompt-a-porter` (Tauri 2 + Svelte 5). Copre strategia di merge, roadmap fasi, decisioni aperte, gap backend, dipendenze, mapping superfici esistenti, mockup di migrazione, rischi e numeri.

## Indice

1. [Strategia di merge](#1-strategia-di-merge)
2. [Roadmap F0–F11](#2-roadmap-f0f11)
3. [Decisioni designer (14) — handoff in un colpo solo](#3-decisioni-designer-14--handoff-in-un-colpo-solo)
4. [Gap backend + migrazione V014](#4-gap-backend--migrazione-v014)
5. [Dipendenze npm + bundle impact](#5-dipendenze-npm--bundle-impact)
6. [Allegato A — Mapping 15 superfici Svelte → redesign](#6-allegato-a--mapping-15-superfici-svelte--redesign)
7. [Allegato E — Mockup migrazione EditorPrompt.svelte → DetailPane](#7-allegato-e--mockup-migrazione-editorpromptsvelte--detailpane)
8. [Rischi e mitigazioni](#8-rischi-e-mitigazioni)
9. [Numeri](#9-numeri)

---

## 1. Strategia di merge

### Pattern: long-running branch + sub-PR + feature flag

```
main ──────────────────────────────────────────────► (PR cutover finale)
  └─ feat/redesign-v08 ────────────────────────────► PR #N (squash)
       ├─ feat/redesign-f0-foundation       → PR contro feat/redesign-v08
       ├─ feat/redesign-f1-shell-layout     → PR contro feat/redesign-v08
       ├─ feat/redesign-f2-sidebar          → PR contro feat/redesign-v08
       ├─ ...
       └─ feat/redesign-f11-test-perf       → PR contro feat/redesign-v08
```

### Regole

- **Branch lungo `feat/redesign-v08`** parte da `main` e ci resta 4-6 sprint (12-13 settimane FT, 6-7 sprint a pace v0.7).
- **Ogni F0-F11 = 1 PR** contro `feat/redesign-v08` (NON contro main). Review piccola (~10-15 file/PR), CI verde, merge incrementale interno.
- **Feature flag `__experimentalRedesign`** in `Impostazioni > Avanzate`, **nascosto/debug-only** finché F11 non chiude. UI vecchia resta default.
- **Rebase periodico** di `feat/redesign-v08` da main ogni ~2 settimane per evitare drift conflittuali.
- **Cutover finale**: PR `feat/redesign-v08 → main`. Nello stesso commit si rimuovono le 8 superfici legacy (Editor/Compilatore/Confronto/Cronologia/Insight/Regressioni/Libreria/Impostazioni nelle loro forme attuali) e il flag.

### Conta PR

| Target | Numero |
|---|---|
| PR contro `feat/redesign-v08` (sub-fase) | ~14 (12 fasi + 2 hotfix) |
| PR contro `main` (cutover + eventuali patch urgenti) | ~2 |
| **Totale PR contro main** | **~2** |

### Vantaggi

- `main` sempre stabile durante 4-6 sprint di refactor.
- Sviluppo v0.8.x parallelo possibile su main (signing macOS, fix urgenti) senza conflitto.
- Rollback = `git revert` sul commit cutover.
- UI vecchia funzionante per early-adopter durante il refactor (toggle flag → live test).

### Costo

- Doppio mantenimento UI per ~12 settimane.
- Rebase chirurgico ogni ~2 settimane.

### Rilascio

- Tag `v0.8.0` sul cutover finale.
- Changelog ricco con sezione "redesign" che linka questo piano.

---

## 2. Roadmap F0–F11

### Dipendenze tra fasi

```
F0 ──┐
     ├─► F1 ──┬─► F2 ──┐
     │        ├─► F3 ──┼─► F9 ──► F10 ──► F11
     │        │        │
     │        └─► F4 ──┼─► F5
     │                 │
     │                 └─► F6
     │
     └─► F7 ──────────────► (parallelo a F2-F6)
              F8 ──────────► (parallelo, dipende da F4)
```

### F0 — Foundation

**Bloccato da**: contrast review designer su tema chiaro (decisione #4).

**Output**:
- Importazione `tokens.css` in `apps/client/src/app.css` con cascade su `[data-theme]` + `[data-tone]`.
- Self-hosting font Inter + JetBrains Mono in `apps/client/static/fonts/` con `@font-face` (fuori CDN per privacy local-first).
- Store globale prefs (`apps/client/src/lib/stores/preferences.ts`) — `$state` runes, persistito su `~/Library/Application Support/PromptVault/preferences.json` via Tauri `fs` plugin.
- Attributi `data-theme="dark|light"` + `data-tone="zinc|slate|stone"` applicati su `<html>` da `App.svelte` con `$effect`.
- Riguardo `prefers-color-scheme`: rispetto fallback automatico se l'utente non ha mai settato il tema.

**Files target**:
- `apps/client/src/app.css` (riscritto)
- `apps/client/src/lib/stores/preferences.ts` (NEW)
- `apps/client/src/App.svelte` (effect tema/tono)
- `apps/client/static/fonts/*.woff2` (NEW × 8 file font)

**Exit criteria**: 4 combinazioni tema×tono funzionanti (dark/light × zinc/slate); `npm run check` verde; bundle aggiunge ≤ 30 KB font (subset latin).

**Stima**: 2 giorni FT.

---

### F1 — Shell layout 3-pannelli

**Output**:
- `Shell.svelte` (NEW) — root del nuovo layout: title bar 36px + body grid + status bar 28px.
- `TitleBar.svelte` (NEW) — 3 colonne: brand · search-as-palette · controls (theme toggle + ⚙ + window controls).
- `StatusBar.svelte` (NEW) — dot vault + nome prompt + dot saved + kbd `⌃⇧P` cliccabile.
- Body CSS grid: `grid-template-columns: var(--col-sidebar) 1px var(--col-list) 1px minmax(0,1fr)` + right-rail nidificato dentro detail.
- Resizer drag horizontal con clamp 180-360 (sidebar), 0-480 (lista), 220-480 (rail), 1px hit-area 7px via `::after`.
- `paneforge` integrazione (già scelto in §5) — usa `<PaneGroup>`/`<Pane>`/`<PaneResizer>`.

**Files target**:
- `apps/client/src/lib/superfici/Shell.svelte` (NEW)
- `apps/client/src/lib/componenti/TitleBar.svelte` (NEW)
- `apps/client/src/lib/componenti/StatusBar.svelte` (NEW)
- `apps/client/src/lib/componenti/Resizer.svelte` (NEW — wrapper paneforge)

**Exit criteria**: Shell vuoto renderizza con drag-resize funzionante, status bar e title bar mostrano placeholder, theme toggle in title bar applica `data-theme` su `<html>`.

**Stima**: 4 giorni FT.

---

### F2 — Sidebar (espansa + mini)

**Output**:
- `Sidebar.svelte` (NEW) — workspace switcher + 5 NavGroup collassabili (Viste · Visibilità · Cartelle · Tag · Modello target) + footer Insight/Regressioni.
- `NavGroup.svelte` (NEW) — header uppercase 10px + count + bottone `+` opzionale + slot children collapsibile.
- `NavItem.svelte` (NEW) — icona + label + count + dot opzionale (visibilità o tag color).
- `SidebarMini.svelte` (NEW) — variante 44px collapsed: stack icone 32×32 con tooltip hover.
- Stato collassato persistito in store prefs F0.
- Workspace switcher: oggi 1 vault/utente (decisione #2 utente), quindi placeholder UI (avatar "P" + "Personale" + chevron disabilitato) — wire-up multi-vault rinviato a v0.9+.

**Files target**: `apps/client/src/lib/componenti/Sidebar*.svelte` (NEW × 4-5 file)

**Exit criteria**: cliccare un Tag/Cartella filtra la lista (anche se la lista è ancora vuota in F2); collapse `«` riduce sidebar a 44px e ripristina; tutti i count badge sono live (zero hardcoded).

**Stima**: 3 giorni FT.

---

### F3 — List pane

**Output**:
- `ListPane.svelte` (NEW) — header sticky 3-row (title+collapse · search+nuovo · toolbar densità+filtri+sort) + body scrollable di `PromptCard`.
- `PromptCard.svelte` (NEW) — 3 modalità densità (Compatta · Comoda · Anteprima) con `data-density`, slider righe preview 1-8 collegato a `--preview-lines`.
- Filtri attivi → chip con ✕ in toolbar; combinati AND con `q`/`view`/`activeFolder`/`activeTag`/`modelTarget`.
- Sort dropdown: Recenti / Popolari / Migliori (per rating) / A-Z.
- **Drag & drop riordino** (decisione #5 utente): `dnd-kit` pure-Svelte non esiste — uso `svelte-dnd-action` o roll-our-own con HTML5 dragstart/dragover. Riordino dentro la stessa cartella + spostamento cross-cartella verso sidebar (handler condiviso F2).

**Files target**: `apps/client/src/lib/componenti/ListPane*.svelte`, `PromptCard.svelte` (NEW × 3-4 file)

**Exit criteria**: 3 densità funzionano; sort applica ordine corretto; chip filtro visualizzati con ✕; drag-reorder persiste l'ordine (richiede V014 backend, vedi §4).

**Stima**: 4 giorni FT (+1 giorno per drag&drop integrato).

---

### F4 — Detail shell + Editor tab

**Output**:
- `DetailPane.svelte` (NEW) — header (title input + desc textarea inline-editable + toolbar Star/Fork/Esporta MD/Compila/Meta toggle) + meta-row chip + tab strip + slot tab attivo.
- `DetailTabs.svelte` (NEW) — 6 tab: Editor · Anteprima · Diagnosi · Test golden · Cronologia · Import & Var. Underline accent-team. Badge count per tab Diagnosi/Golden/Cronologia/Import.
- `EditorTab.svelte` (NEW) — wrapper CodeMirror 6 con `@codemirror/lang-markdown` + decoration plugin esistente (`apps/client/src/lib/codemirror/import-tokens.ts` già pronto, riusabile 1:1).
- `MarkdownToolbar.svelte` (NEW) — 18 azioni: B · I · | · H1 · H2 · | · UL · OL · quote · | · code · codeblock · link · hr · | · `+ {{var}}` · `+ import` · | · search · indicatore (saved/dirty + L/C + char + tok). Icone da `lucide-svelte` (decisione #3 utente).
- `EditorIndicator.svelte` (NEW) — saved/dirty + L/C + char + tok in coda alla toolbar.
- Logica autosave: porting da `EditorPrompt.svelte:pianificaAutosave` (1.2s idle, già implementata).

**Files target**: `apps/client/src/lib/componenti/Detail*.svelte`, `Editor*.svelte`, `MarkdownToolbar.svelte` (NEW × 5-6 file)

**Exit criteria**: aprire un prompt nel detail mostra header completo + tabs; tab Editor è 1:1 funzionale rispetto a `EditorPrompt.svelte` attuale (autosave incluso); toolbar 18 azioni inseriscono il markup giusto.

**Stima**: 6 giorni FT.

---

### F5 — Tabs detail (Anteprima · Diagnosi · Test golden · Cronologia · Import & Var.)

**Output per tab**:

| Tab | Logica | Files target | Stima |
|---|---|---|---|
| **Anteprima** | Render body con sostituzione default segnaposti, mono 13px, line-height 1.65 su `bg-surface` | `AnteprimaTab.svelte` (NEW) | 1 gg |
| **Diagnosi** | Lista lint warning/error per riga, click jump alla riga (riusa `lintIssues` esistente da `EditorPrompt.svelte`) | `DiagnosiTab.svelte` (NEW) | 1.5 gg |
| **Test golden** | Tabella golden con drift score + last-run, integra cmd `golden_lista`/`esegui` esistenti (gap zero, vedi §4) | `GoldenTab.svelte` (NEW) | 2 gg |
| **Cronologia** | Lista versioni con avatar autore (deterministico da hash) + timestamp + delta `+/-`. Click apre diff side-by-side via `diff2html`. Toggle unified ↔ side-by-side. **Richiede V014 backend** (vedi §4). | `CronologiaTab.svelte` (NEW), `DiffViewer.svelte` (NEW) | 4 gg |
| **Import & Var.** | Import composti + varianti A/B con toggle "Confronta tutte" (porting `ConfrontoPrompt.svelte` ridotto ad A/B/C dello stesso parent) | `ImportVarTab.svelte` (NEW) | 3 gg |

**Stima totale F5**: 11.5 giorni FT.

---

### F6 — Right rail (Metadati)

**Output**:
- `RightRail.svelte` (NEW) — toggle Meta separato (può collassare/espandere indipendentemente dalla detail width).
- Sezioni:
  - **Metadati**: Visibilità **dropdown** (decisione #1 utente — estensibile a futuri ruoli) · target select · cartella select · tag picker (chip rimovibili + input). Tag suggeriti semantici (porting da `EditorPrompt.svelte` — vector+frequenza score).
  - **Segnaposti rilevati**: lista auto-detected da `{{var}}` con tipo (testo/enum/multilinea), default, asterisco se obbligatorio.
  - **Import composti**: lista con icona fork.
  - **Varianti A/B**: pill orizzontali (A · B corrente · C) + bottone "+ Variante" + "Confronta tutte".

**Files target**: `apps/client/src/lib/componenti/RightRail*.svelte` (NEW × 4-5 file)

**Exit criteria**: tutti gli edit nel rail propagano dirty=true sull'editor; tag suggeriti semantici rispondono; varianti A/B passano al "Confronta tutte" che apre la tab Import & Var.

**Stima**: 3 giorni FT.

---

### F7 — Status bar funzionale

**Output**:
- `StatusBar.svelte` da F1 esteso con:
  - Dot vault (verde/giallo/rosso a seconda stato sync) + tooltip espande dettagli SQLCipher (path DB, dimensione, ultima rotazione master-key).
  - Centro: nome prompt corrente con icona visibility.
  - Destra: dot saved + "salvato 14s fa" (timestamp friendly), oppure "in modifica…" se dirty; kbd `⌃⇧P` cliccabile apre modale Palette.

**Files target**: `apps/client/src/lib/componenti/StatusBar.svelte` (extended)

**Exit criteria**: tooltip vault mostra path reale, dimensione DB, timestamp ultima rotazione; dot saved si aggiorna entro 1.2s da modifica.

**Stima**: 1.5 giorni FT (parallelo a F2-F6).

---

### F8 — Modali residue (Compila · Insight · Regressioni · Impostazioni · Palette)

**Output per modale**:

| Modale | Porting da | Note | Stima |
|---|---|---|---|
| **Compila** | `CompilatorePrompt.svelte` (927 righe) | Form segnaposti type-aware + preview live + rating ±1 con nota (decisione designer #6 — preservato) + copia 3 formati. Migra a `<dialog>` nativo via `bits-ui` Dialog. | 2 gg |
| **Insight** | `Insight.svelte` (573 righe) | Dashboard porting leggero. Aggiungere `token_medi` (extend backend §4). | 1 gg |
| **Regressioni** | `Regressioni.svelte` (368 righe) | Larghezza fissa `min(1200px, 92vw)`. Preservare bottone esporta CSV (decisione designer #2). | 1.5 gg |
| **Impostazioni** | `Impostazioni.svelte` (2223 righe — 8 sezioni attuali) | **Rimappa 8 sezioni in 5 del prototipo** (Aspetto · Vista lista · Editor · Sicurezza · Avanzate) + preserva `audit log`/`provider config`/`ricerca embedding status` come sub-sezioni di Avanzate (decisione designer #3, #4, #5). Flag `__experimentalRedesign` qui in Avanzate (debug-only). | 4 gg |
| **Palette** | `CommandPalette.svelte` (628 righe) | `bits-ui` Command (decisione §5) — supersede `cmdk-sv` deprecato. Search-as-palette nella titlebar. Preservare slider alpha ricerca ibrida (decisione designer #7). Listener `Ctrl+Shift+P` via `@tauri-apps/plugin-global-shortcut` esistente. | 2 gg |

**Files target**: `apps/client/src/lib/superfici/{Compila,Insight,Regressioni,Impostazioni,Palette}.svelte` (riscritti con nuovo design system)

**Stima totale F8**: 10.5 giorni FT.

---

### F9 — Routing/cleanup

**Output**:
- App root rimpiazza il routing attuale (8 superfici disgiunte → Shell unico).
- `App.svelte` decide flusso:
  - Non autenticato → flusso **Onboarding** (assorbe `AuthLogin` + `AuthRecuperaWorkspace` + `AuthResetPassword` + wizard esistente, decisione #6 utente).
  - Autenticato → `Shell.svelte` con stato iniziale ultimo prompt aperto.
- Modali (Compila/Insight/Regressioni/Impostazioni/Palette) gestite tramite store globale `modalState` con type union.
- Eliminazione `DemoComponenti.svelte` (KILL — solo dev-tool).
- `ConflittoSync.svelte` resta come modale aux (evocata da event sync, fuori scope redesign primario).
- Aggiornamento test E2E per il nuovo routing.

**Files target**:
- `apps/client/src/App.svelte` (riscritto)
- `apps/client/src/lib/superfici/Onboarding.svelte` (NEW — consolida 4 superfici)
- `apps/client/src/lib/superfici/{AuthLogin,AuthRecuperaWorkspace,AuthResetPassword,DemoComponenti}.svelte` (DELETED)
- `apps/client/src/lib/stores/modalState.ts` (NEW)

**Exit criteria**: tutti i flussi attuali funzionano nel nuovo routing; nessuna superficie legacy referenziata; test smoke E2E verde.

**Stima**: 5 giorni FT (3 cleanup routing + 2 onboarding consolidation).

---

### F10 — Accessibility + tema chiaro pass

**Output**:
- A11y audit: `<dialog>` nativo via `bits-ui` Dialog (ESC + click-outside + focus trap free); resizer keyboard nav (← → con clamp); focus ring `--focus-ring` su tutti gli interactive.
- Tema chiaro contrast pass: review post-feedback designer (#4), eventuali aggiustamenti chip colored e mono editor.
- Reduced-motion: già in `tokens.css` (gestito da media query) — verifica che tutti gli `transition` e `animation` rispondano.
- aria-label su tutti gli icon-only button (sidebar mini, toolbar markdown, status bar).

**Files target**: cross-cutting (capitolo tema-chiaro su ogni component)

**Exit criteria**: WCAG 2.1 AA (contrast ≥ 4.5:1 per text, 3:1 per UI), keyboard-nav-completa, screen reader smoke test (NVDA Windows / VoiceOver macOS).

**Stima**: 3 giorni FT.

---

### F11 — Test + perf

**Output**:
- Vitest + `@testing-library/svelte` su tutti i nuovi component (target: ≥ 70% coverage gate CI mantenuto).
- Bundle size check post-redesign: target ≤ +100 KB gzip rispetto v0.7.0 (4 dep nuove pesano ~73 KB, +font self-hosted ~30 KB = ~103 KB). Se sforiamo, code-split `diff2html` solo quando si apre tab Cronologia.
- Drag-resize a 60fps (Chrome DevTools Performance profiling): target ≤ 16ms/frame.
- Apertura prompt in DetailPane: target ≤ 300ms first-paint dopo click su PromptCard.
- Buffer integrazione + regressioni emergenti: 8 giorni (15% del totale, già contabilizzati).

**Exit criteria**: tutti i test verdi, gate coverage 70% mantenuto, perf budget rispettato, regressioni note risolte.

**Stima**: 4 giorni FT.

---

### Tabella riepilogo fasi

| Fase | Output | Bloccato da | Stima FT |
|---|---|---|---|
| F0 | Foundation tokens + font + tema/tono store | Contrast review designer (#4) | 2 gg |
| F1 | Shell 3-pannelli + title/status bar + resize | F0 | 4 gg |
| F2 | Sidebar espansa + mini | F1 | 3 gg |
| F3 | List pane + 3 densità + drag-reorder | F1, V014 | 4 gg |
| F4 | DetailPane shell + Editor tab + toolbar MD | F1 | 6 gg |
| F5 | 5 tab detail (Anteprima/Diagnosi/Golden/Cronologia/ImportVar) | F4, V014 | 11.5 gg |
| F6 | Right rail Metadati | F4 | 3 gg |
| F7 | Status bar funzionale | F1 | 1.5 gg |
| F8 | 5 Modali (Compila/Insight/Regressioni/Impostazioni/Palette) | F4 | 10.5 gg |
| F9 | Routing + cleanup + Onboarding consolidato | F2-F8 | 5 gg |
| F10 | A11y + tema chiaro pass | F1-F9 | 3 gg |
| F11 | Test + perf + buffer 15% | F9 | 4 + 8 buffer = 12 gg |
| **TOTALE UI** | | | **63.5 gg FT** |
| Backend (parallelo) | V014 + extend autore + extend stats | — | 1.5 gg |
| **TOTALE PROGETTO** | | | **~65 gg FT** |

A pace v0.7 (sprint compresso ~1 giorno per quick win) si stima 6-7 sprint v0.8.x.

---

## 3. Decisioni designer (14) — handoff in un colpo solo

Tabella da inviare al designer in singolo handoff. Bloccante per F0: solo decisione #4 (contrast tema chiaro).

| # | Area | Decisione richiesta | Default proposto |
|---|---|---|---|
| 1 | Right-rail Metadati | Visibilità dropdown estensibile a futuri ruoli? | Dropdown (utente conferma estensibile) |
| 2 | Sidebar | Workspace switcher: menù multi-vault o naviga? | Placeholder disabilitato (1 vault/utente oggi) |
| 3 | Toolbar markdown | Icone custom `md-*` o `lucide-svelte`? | `lucide-svelte` (utente conferma consolidamento) |
| 4 | **Tema chiaro** | **Contrast review chip + editor mono — bloccante F0** | Designer fornisce override tokens se necessario |
| 5 | Lista | Drag-reorder prompt in scope v0.8? | Sì (utente conferma feature importante) |
| 6 | Modale Compila | Rating ±1 + nota su voto neg/neutro: preservare? | Preservato (riduzione = regressione UX) |
| 7 | Palette | Slider alpha ricerca ibrida (vector vs keyword): preservare? | Preservato in panel "filtri avanzati" della Palette |
| 8 | Tab Confronto | Riduzione N-way arbitrario → A/B/C dello stesso parent: confermare? | Riduzione confermata (decisione redesign README) |
| 9 | Cronologia | Diff side-by-side default + toggle unified | Side-by-side (`diff2html`) — utente conferma |
| 10 | Sidebar | Workspace switcher click → menù o naviga? | Decisione designer (oggi disabilitato) |
| 11 | Drag-reorder | Cross-cartella (lista → sidebar): conferma UX? | Sì, con visual highlight cartella di destinazione |
| 12 | Modale Impostazioni | 8 sezioni attuali → 5 prototipo: dove finiscono `audit`/`provider`/`ricerca`? | Sub-sezioni di "Avanzate" |
| 13 | Tab Cronologia | Avatar autore: hash deterministico (gravatar-like) o iniziali? | Hash deterministico colorato (gravatar-like) |
| 14 | List pane | Drag-reorder visual cue: linea bordo o card-shift? | Linea 2px accent-team (Linear-like) |

**Note operative**:
- Bundle al designer in PDF/Notion con: brief + redesign README + screenshots Windows (forniti separatamente da utente) + tabella sopra + prototipo HTML aperto.
- Mantenere conta delle 14 decisioni in un file `docs/architettura/redesign/decisioni-designer.md` con stato (open/risolta/in-attesa).

---

## 4. Gap backend + migrazione V014

### Stato per feature

| Feature | Backend | Azione |
|---|---|---|
| Tab Cronologia con autore | Parziale | Extend `prompt_get_history` con JOIN `Users.DisplayName` |
| Tab Test golden | Presente | Nessuna |
| Modale Insight (`token_medi`) | Parziale | Extend `Statistiche` struct con campo `token_medi` |
| Modale Regressioni | Presente | Nessuna |
| Right-rail varianti A/B | Presente | Nessuna |
| Drag-reorder cartella/prompt | **Missing** | **Migrazione V014 + 2 cmd nuovi** |

### Migrazione V014

**File**: `apps/client/src-tauri/migrations/V014__sort_order.sql`

```sql
-- V014: SortOrder per drag-reorder cartelle e prompt
ALTER TABLE Folders ADD COLUMN SortOrder INTEGER NOT NULL DEFAULT 0;
ALTER TABLE Prompts ADD COLUMN SortOrder INTEGER NOT NULL DEFAULT 0;

-- Backfill: ROW_NUMBER per workspace + parent
UPDATE Folders SET SortOrder = (
  SELECT COUNT(*) FROM Folders f2
  WHERE f2.WorkspaceId = Folders.WorkspaceId
    AND COALESCE(f2.ParentFolderId, '') = COALESCE(Folders.ParentFolderId, '')
    AND f2.CreatedAt < Folders.CreatedAt
);

UPDATE Prompts SET SortOrder = (
  SELECT COUNT(*) FROM Prompts p2
  WHERE p2.WorkspaceId = Prompts.WorkspaceId
    AND COALESCE(p2.FolderId, '') = COALESCE(Prompts.FolderId, '')
    AND p2.CreatedAt < Prompts.CreatedAt
);

CREATE INDEX idx_folders_sort ON Folders(WorkspaceId, ParentFolderId, SortOrder);
CREATE INDEX idx_prompts_sort ON Prompts(WorkspaceId, FolderId, SortOrder);
```

### Comandi Tauri nuovi

- `folder_riordina(id, new_sort: i32)` in `apps/client/src-tauri/src/folder.rs` (extend) — re-pack siblings con transazione.
- `prompt_riordina(id, new_sort: i32)` in `apps/client/src-tauri/src/prompt.rs` (extend) — idem.

### Effort backend stimato

| Item | Effort |
|---|---|
| V014 SQL + test migrazione | 0.25 gg |
| `folder_riordina` + `prompt_riordina` + test unit | 0.5 gg |
| Extend `prompt_get_history` con JOIN Users | 0.25 gg (~2h) |
| Extend `Statistiche` con `token_medi` (proxy char-count) | 0.25 gg (~3h) |
| Buffer | 0.25 gg |
| **Totale backend** | **1.5 gg FT** |

Backend è parallelo a F0 (può partire prima di F1).

---

## 5. Dipendenze npm + bundle impact

### Da installare

| Pacchetto | Versione | Ruolo | Bundle (gzip) |
|---|---|---|---|
| `paneforge` | ^1.0.2 | Resizable 3-pannelli | ~6 KB |
| `bits-ui` | ^2.18.1 | Command palette + Dialog/Tabs/Select/Tooltip headless | ~12 KB (Command) + tree-shake il resto |
| `@codemirror/lang-markdown` | ^6.5.0 | Markdown lang per editor | ~15 KB |
| `diff2html` | ^3.4.56 | Diff side-by-side render in tab Cronologia | ~40 KB (CSS+JS) |
| **Totale** | | | **~73 KB** |

### Note

- `paneforge` e `bits-ui` sono entrambi mantenuti da huntabyte (svecosystem) — coerenza ecosistema, peer `svelte ^5.x`.
- `cmdk-sv` **scartato**: deprecato (fermo a 0.0.19), assorbito in `bits-ui` 2.x.
- `lucide-svelte` ^0.460.0 già presente: serve per toolbar markdown (decisione #3) + icone sidebar/header.
- `diff` 9.x già presente: usato come engine per `diff2html` (lib di rendering separata).
- `@codemirror/{view,state,language,commands,autocomplete}` ^6.0.0 già presenti.
- Self-hosting font Inter + JetBrains Mono: ~30 KB gzip aggiuntivi (subset latin only) in `apps/client/static/fonts/`.

### Bundle impact totale

- Dep npm: +73 KB gzip
- Font self-hosted: +30 KB gzip
- **Totale**: **~103 KB gzip aggiunti** rispetto v0.7.0

Se sforiamo budget Tauri (target ≤ +100 KB), code-split `diff2html` con dynamic import solo all'apertura tab Cronologia (-40 KB dal bundle iniziale).

---

## 6. Allegato A — Mapping 15 superfici Svelte → redesign

| File · LOC | Ruolo attuale | Destinazione redesign | Δ | Feature uniche a rischio regressione |
|---|---|---|---|---|
| **Libreria.svelte** · 2418 | Lista prompt + sidebar viste/cartelle/tag, drag&drop, fork, export MD/JSON | → **List pane + Sidebar** (split) | LARGE | Drag&drop prompt↔cartella, esporta cartella JSON, esporta singolo prompt MD, fork con `fork_of`, dragoverFolder highlight |
| **EditorPrompt.svelte** · 1869 | Modale editor con CodeMirror, lint, golden inline, autosave, segnaposto/import token, tag suggeriti semantici | → **DetailPane** (header + tab Editor/Diagnosi/Golden + RightRail Metadati) | LARGE | Tag suggeriti semantici (vector+frequenza), pianificaAutosave 1.2s idle, golden inline editing/exec, salvaInBackground, lintIssues click-to-jump |
| **Impostazioni.svelte** · 2223 | Modale 8 sezioni (account/sync/hotkey/aspetto/vault/ricerca/provider/audit) | → **Modale Impostazioni** (5 sezioni con sub-sezioni in Avanzate) | MED | Sezione audit (log AI), provider config (5 provider), ricerca embedding status |
| **CompilatorePrompt.svelte** · 927 | Modale compila form segnaposti type-aware, preview live, rating ±1+nota, output 3 formati | → **Modale Compila** | MED | Rating thumbs ±1 con modale nota, output testo/markdown/json, toast copia |
| **CronologiaPrompt.svelte** · 671 | Modale cronologia: rollback, diff body/unified/side-by-side | → **DetailPane tab Cronologia** | LARGE | 3 modi vista diff, conferma rollback, autore mancante (V014) |
| **CommandPalette.svelte** · 628 | Palette `⌃⇧P` con ricerca ibrida vector+keyword | → **Modale Palette** + search-as-palette in title bar | SMALL | Slider alpha ricerca ibrida, `qualcheMatchSem` indicator |
| **Insight.svelte** · 573 | Dashboard uso: prompt più usati, token medi, conteggi | → **Modale Insight** | SMALL | Nessuna feature unica |
| **OnboardingWizard.svelte** · 533 | Wizard 5 step: profilo, password vault, hotkey, prompt esempio, tema | → **Onboarding** (assorbe AuthLogin/Recupera/Reset) | MED | Step skip cifratura, scelta tema |
| **ConfrontoPrompt.svelte** · 430 | Side-by-side fra 2-3 prompt arbitrari, diff testuale | → **DetailPane tab Import & Var.** (ridotto A/B/C) | MED | Confronto N-way arbitrario (oggi 2-3, redesign A/B/C) |
| **AuthRecuperaWorkspace.svelte** · 399 | Lookup workspace per email | → **Onboarding** | SMALL | Lookup workspace per email |
| **Regressioni.svelte** · 368 | Tabella drift score per modello/versione | → **Modale Regressioni** 1200px | SMALL | Esporta CSV, slider giorni 1-90 |
| **AuthResetPassword.svelte** · 355 | Reset password flow | → **Onboarding** | SMALL | Standard reset flow |
| **AuthLogin.svelte** · 294 | Login workspace remoto | → **Onboarding** | SMALL | Server URL custom (self-hosted) |
| **DemoComponenti.svelte** · 273 | Showcase primitivi UI | → **rimossa** | KILL | Solo dev-tool |
| **ConflittoSync.svelte** · 222 | Risoluzione conflitti sync per-entità | → **mantenuta** as-is | SMALL | Risoluzione per-entità |

**Distribuzione**: 3 LARGE · 4 MED · 7 SMALL · 1 KILL.

---

## 7. Allegato E — Mockup migrazione `EditorPrompt.svelte` → DetailPane

`EditorPrompt.svelte` (1869 LOC) è la superficie più complessa. Ecco dove finisce ogni pezzo.

### Mappa pezzi → destinazione

```
EditorPrompt.svelte (modale full-screen oggi)
│
├─ Header
│   ├─ Title input ────────────────────► DetailPane.header (top row, inline)
│   ├─ Description textarea ────────────► DetailPane.header (sotto title, inline)
│   ├─ Star button ─────────────────────► DetailPane.header (toolbar destra)
│   ├─ Fork button ─────────────────────► DetailPane.header (toolbar destra)
│   ├─ Esporta MD button ───────────────► DetailPane.header (toolbar destra)
│   ├─ Compila button (primario) ───────► DetailPane.header (toolbar destra)
│   ├─ Meta toggle button ──────────────► DetailPane.header (separato a fondo)
│   └─ Save status indicator ───────────► EditorIndicator (in MarkdownToolbar)
│
├─ Body (oggi tutto verticale stratificato)
│   ├─ CodeMirror editor markdown ──────► EditorTab.svelte (riusa import-tokens.ts)
│   ├─ Toolbar markdown (parz. attuale) ► MarkdownToolbar.svelte (ESPANSA a 18 azioni)
│   ├─ Lint diagnosi inline ─────────────► DiagnosiTab.svelte (separata in tab)
│   ├─ Test golden inline ───────────────► GoldenTab.svelte (separata in tab)
│   └─ Anteprima rendering inline ───────► AnteprimaTab.svelte (separata in tab)
│
├─ Sidebar destra (oggi ~30% width, integrata)
│   ├─ Visibilità segmented ─────────────► RightRail Metadati > Visibilità DROPDOWN (decisione #1)
│   ├─ Target model select ──────────────► RightRail Metadati > Target
│   ├─ Folder select ────────────────────► RightRail Metadati > Cartella
│   ├─ Tag picker (semantici) ───────────► RightRail Metadati > Tag (preserva logica vector)
│   ├─ Segnaposti rilevati ──────────────► RightRail > Segnaposti rilevati
│   ├─ Import composti ──────────────────► RightRail > Import composti
│   └─ Varianti A/B (limitato) ──────────► RightRail > Varianti A/B (con "+ Variante")
│
└─ Logica
    ├─ pianificaAutosave (1.2s idle) ────► store editorState + $effect in DetailPane
    ├─ salvaInBackground ────────────────► invariata (cmd Tauri)
    ├─ lintIssues click-to-jump ─────────► passata via prop a DiagnosiTab
    ├─ tag suggeriti semantici ──────────► fetch al mount RightRail Metadati
    ├─ golden inline editing/exec ───────► tutto dentro GoldenTab.svelte
    └─ import-tokens decoration ─────────► riusato 1:1 in EditorTab.svelte
```

### Cosa SI ROMPE (e va testato dopo migrazione)

1. **Autosave durante switch tab**: oggi siamo sempre nello stesso modale, l'editor è in DOM. Nel redesign cambiare tab smonta CodeMirror. → Mantenere store editorState con buffer + reidratazione su switch back; testare che dirty=true non venga perso.
2. **Lint click-to-jump cross-tab**: oggi click su lint salta nella stessa view. Nel redesign è in DiagnosiTab → click deve switchare a EditorTab + scrollare a riga + selezionare. Implementare `editorBus.gotoLine(n)`.
3. **Golden editing concorrente con editor**: oggi side-by-side. Nel redesign sono tab disgiunte → save dell'editor prima di switch a Golden tab (autosave forza flush).
4. **Anteprima live**: oggi rerender ad ogni tasto. Nel redesign è tab → debounced 500ms + invalidazione cache su switch.

### Cosa GUADAGNAMO

- Editor full-width non più strettamente confinato in modale → spazio reale per CodeMirror.
- Cronologia con autore + diff side-by-side accessibili in 1 click invece di 3.
- Right rail sempre visibile per metadati (oggi sparsi tra header + sidebar collassabile).
- Toolbar markdown di prima classe (oggi parziale).

---

## 8. Rischi e mitigazioni

| # | Rischio | Severità | Mitigazione |
|---|---|---|---|
| 1 | Drift `main` durante 12 settimane di branch lungo | ALTA | Rebase ogni 2 settimane + frozen window per merge non-essenziali su main durante F4-F5 |
| 2 | Tag semantici (feature unica vector+frequenza) persa nel porting | ALTA | Test e2e dedicato + check con fixture noti pre/post porting |
| 3 | Drag&drop cartelle: paneforge non lo offre, va integrato con `svelte-dnd-action` | MED | Spike F3 con prototipo isolato prima di integrare in ListPane |
| 4 | Bundle size eccede budget Tauri | MED | Code-split `diff2html` dynamic import; subset font esteso solo se necessario |
| 5 | Designer non risponde in tempi rapidi alle 14 decisioni | ALTA | F0 può comunque partire con tema scuro only; tema chiaro slittato a F10 se necessario |
| 6 | Migrazione V014 rompe vault esistenti utenti early-adopter | ALTA | V014 additiva (DEFAULT 0), test su DB v0.7.0 reale; rollback test |
| 7 | bits-ui Command api change durante refactor (lib in 2.x rapida evoluzione) | LOW | Pin patch version, weekly check changelog |
| 8 | Doppio mantenimento UI (vecchia+nuova) per 12 settimane | MED | Flag `__experimentalRedesign` debug-only riduce surface; nessun code-path duplicato in cmd Tauri (backend agnostico) |
| 9 | Test E2E esistenti rotti per cambio routing | MED | F11 dedica 2 gg a aggiornamento test E2E + smoke test critical path |
| 10 | Regressione UX su feature scope-drift (8 elementi) | MED | Tabella decisioni designer #6, #7, #8, #12 chiusa prima di F4-F5-F8 |

---

## 9. Numeri

### Effort

| Asse | Valore |
|---|---|
| Effort UI Svelte (FT) | 63.5 gg-uomo |
| Effort backend Rust (FT) | 1.5 gg-uomo |
| **TOTALE** | **~65 gg-uomo** |
| A pace v0.7 (sprint compresso) | **6-7 sprint v0.8.x** |

### Codebase

| Asse | Valore |
|---|---|
| Superfici Svelte attuali | 15 (12.183 LOC totali) |
| Superfici riscritte/portate | 14 (1 KILL) |
| Component primitives nuovi stimati | ~25 |
| Migrazioni schema | 1 (V014) |
| Comandi Tauri nuovi | 2 (`folder_riordina`, `prompt_riordina`) |
| Comandi Tauri estesi | 2 (`prompt_get_history`, `Statistiche`) |

### Dipendenze

| Asse | Valore |
|---|---|
| Dep npm nuove | 4 (paneforge, bits-ui, lang-markdown, diff2html) |
| Bundle impact npm | ~73 KB gzip |
| Font self-hosted | ~30 KB gzip |
| **Bundle totale aggiunto** | **~103 KB gzip** |

### Pull Request

| Asse | Valore |
|---|---|
| PR contro `feat/redesign-v08` | ~14 (12 fasi + 2 hotfix) |
| PR contro `main` | ~2 (cutover + eventuali patch urgenti) |

### Decisioni aperte

| Asse | Valore |
|---|---|
| Decisioni designer in handoff | 14 |
| Decisioni utente già chiuse | 7 (visibilità dropdown · 1 vault · lucide · contrast bloccante F0 · drag-reorder yes · onboarding consolidato · strategia merge) |

---

## Appendice — File deliverable di questo piano

| File | Stato |
|---|---|
| `docs/architettura/redesign/README.md` | Esistente (sorgente design) |
| `docs/architettura/redesign/prototype/*` | Esistente (9 file high-fidelity) |
| `docs/architettura/design-handoff/2026-05-08-redesign-brief.md` | Esistente (brief sorgente) |
| `docs/roadmap/redesign-v08.md` | **Questo file** |
| `docs/architettura/redesign/decisioni-designer.md` | DA CREARE in F-1 (tabella 14 decisioni con stato) |
| `apps/client/src-tauri/migrations/V014__sort_order.sql` | DA CREARE in backend pre-F3 |
| `apps/client/src-tauri/migrations/V014.md` | DA CREARE (changelog migrazione) |

---

> **Prossimo passo operativo**: aprire `docs/architettura/redesign/decisioni-designer.md` con le 14 righe della §3, allegare al brief, mandare al designer. Mentre attendiamo contrast review (#4 bloccante F0), il backend può partire su V014 + extend autore (1.5 gg in parallelo).
