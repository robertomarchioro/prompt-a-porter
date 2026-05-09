# Blueprint F11 — Test + perf + cleanup finale

> Cumulativo per le 4 sub-PR autonome F11. Ultima fase del redesign v0.8
> prima della release v0.8.0.
> Riferimenti: [Piano F11](../redesign-v08.md#f11--test--perf) (4 gg + 8 buffer = 12 gg FT).

## Obiettivo

Chiudere il redesign v0.8 con:
- **Cleanup finale**: rimuovere 8 superfici legacy ancora orfane (~7283 righe)
- **Test coverage**: vitest + @testing-library/svelte sui nuovi component (target ≥ 70% gate CI)
- **Bundle size**: target ≤ +100 KB gzip vs v0.7.0; se sforiamo, code-split `diff2html`
- **Perf**: drag-resize ≤ 16ms/frame, apertura prompt DetailPane ≤ 300ms first-paint

## Stato baseline (verificato pre-blueprint)

- ✅ 88/88 vitest pass, no regressioni dopo F10
- ✅ Branch `feat/redesign-v08` aggiornato (PR #122-#125 mergiate)
- ✅ Conformità WCAG 2.1 AA + 2.3.3 raggiunta in F10
- ⚠️ 8 superfici legacy orfane (verificate 0 importers reali)
- ⚠️ 2 superfici legacy LIVE (NON cancellabili in PR-A):
  - `OnboardingWizard.svelte` (wrappato da `Onboarding.svelte:27`)
  - `CommandPalette.svelte` (window separata routata da `App.svelte:22` per hotkey OS-level)
- ⚠️ Coverage backend Rust (74.14% in v0.7.0); coverage frontend non ancora misurata

## Sub-PR

| Sub-PR | Scope | Stima |
|---|---|---|
| **F11 PR-A** | Cleanup 8 superfici legacy orfane (~7283 righe DELETE) | 0.5 gg |
| **F11 PR-B** | Test coverage vitest sui nuovi component (Modale, store modale.svelte, Onboarding) | 2 gg |
| **F11 PR-C** | Bundle size check + eventuale code-split `diff2html` | 1 gg |
| **F11 PR-D** | Perf profiling drag-resize + first-paint DetailPane | 1 gg |

Buffer 8 gg per regressioni emergenti già contabilizzato nel piano.

---

## 1. F11 PR-A — Cleanup legacy finale

### File DELETED

| File | Righe | Sostituito da |
|---|---|---|
| `CompilatorePrompt.svelte` | 927 | CompilaModal (F8 PR-A) |
| `ConfrontoPrompt.svelte` | 430 | DetailPane tab Confronta + DiffLibero (F5) |
| `CronologiaPrompt.svelte` | 671 | DetailPane tab Cronologia (F5) |
| `EditorPrompt.svelte` | 1869 | DetailPane editor inline (F4) |
| `Impostazioni.svelte` | 2223 | ImpostazioniModal (F8 PR-D1+D2) |
| `Insight.svelte` | 573 | InsightModal (F8 PR-B) |
| `Regressioni.svelte` | 368 | RegressioniModal (F8 PR-C) |
| `ConflittoSync.svelte` | 222 | Orfano (no event-driven attivo, non più importato) |

**Totale: -7283 righe** (verificato `wc -l` pre-blueprint).

### File NON cancellabili (mantenuti)

- `OnboardingWizard.svelte`: importato da `Onboarding.svelte:27` (Onboarding F9 lo wrappa come step "setup")
- `CommandPalette.svelte`: importato da `App.svelte:22` (window separata legacy per hotkey OS-level)

Cancellazione di questi 2 deferita a refactor futuro che assorba il wizard direttamente in `Onboarding.svelte` e/o rimuova la window OS-level.

### Verifica pre-DELETE

Già eseguita in pianificazione: `grep -rln "from.*\$lib/superfici/<File>"` e check string match diretto → 0 importers per gli 8 file.

### Stima

0.5 gg

---

## 2. F11 PR-B — Test coverage nuovi component

### Path

- `apps/client/src/lib/components/Modale.test.ts` (NEW)
- `apps/client/src/lib/stores/modale.svelte.test.ts` (NEW)
- `apps/client/src/lib/superfici/Onboarding.test.ts` (NEW)
- Eventuali ulteriori component con logic non triviale

### Scope

Vitest + `@testing-library/svelte` su:

1. **Modale primitive** (componente F8 critico):
   - Render con props minimi
   - Click backdrop → onChiudi
   - ESC keydown → onChiudi
   - Tab cycling (focus trap)
   - Return focus al trigger su unmount
2. **Store modale.svelte.ts** (singleton state):
   - apriModale → statoModale.attiva updated
   - chiudiModale → statoModale.attiva = null
   - Discriminated union type-safety (compile-time, no runtime test)
3. **Onboarding.svelte** state machine:
   - Caricamento → setup (vault non esiste)
   - Caricamento → sblocco (vault cifrato)
   - Caricamento → oncompletato (vault già aperto)
   - Sblocco → password sbagliata → errore
   - Sblocco → password corretta → oncompletato

Mock `invoke` Tauri via `vi.mock("@tauri-apps/api/core")`.

### Target

Coverage frontend ≥ 70% (gate CI da impostare; backend già a 74.14%).

### Stima

2 gg

---

## 3. F11 PR-C — Bundle size check + code-split

### Scope

1. Build production con `npm run build` (vite)
2. Misura bundle size `dist/` gzip (script `gzip -c | wc -c`)
3. Confronta con v0.7.0 baseline (target ≤ +100 KB gzip)
4. Se sforiamo: code-split `diff2html` con dynamic import in DetailPane tab Cronologia (è il dep più grosso, ~50 KB)
5. Verifica che il PaletteModal (F8 PR-E) non porti tutto il modulo `@tauri-apps/api/event` a meno che serva

### Tool

- `vite-bundle-visualizer` (dep dev)
- `rollup-plugin-visualizer` come alternativa

### Stima

1 gg

---

## 4. F11 PR-D — Perf profiling

### Scope

1. **Drag-resize 60fps**: aprire Shell, drag PaneResizer, profilo Chrome DevTools Performance
   - Target: ≤ 16ms/frame
   - Se sforiamo: aggiungere `will-change: width` al pane container, debounce listener, o requestAnimationFrame
2. **First-paint DetailPane** ≤ 300ms:
   - Profile click PromptCard → DetailPane visible
   - Se sforiamo: lazy-load `EditorPrompt` content via dynamic import, ottimizzare `prompt_get` cmd backend

### Tool

- Chrome DevTools Performance tab
- `console.time` / `console.timeEnd` per misure ad-hoc se serve

### Stima

1 gg

---

## 5. Pattern comune di integrazione

Per ogni sub-PR:

1. Applicare modifica con minimal diff
2. Verifica `npm run check` 0 errors + `npm test` 88/88+ dopo ogni edit
3. Per PR-B: nuovi test dovrebbero portare test count > 88
4. Commit + push + PR autonoma + Monitor CI + auto-merge appena verde

### Out of scope F11 (rinviati a release/sprint successivi)

- DELETE OnboardingWizard.svelte (richiede assorbimento del wizard direttamente in Onboarding.svelte; refactor non triviale)
- DELETE CommandPalette.svelte (richiede rimozione window separata Tauri + global-shortcut migration; refactor architetturale)
- E2E test Playwright (suite non esiste; pianificazione separata)
- Screen reader smoke test automation (NVDA/VoiceOver; manuale)
- Color-blind simulation testing (manuale)

---

> **Stato blueprint**: 1.0 — pronto per esecuzione iterativa autonoma.
> Dopo F11 → release v0.8.0.
