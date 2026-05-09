# Blueprint F10 — Accessibility + tema chiaro pass

> Cumulativo per le 4 sub-PR autonome F10. Riferimenti:
> [Piano F10](../redesign-v08.md#f10--accessibility--tema-chiaro-pass) (3 gg FT).

## Obiettivo

Portare il redesign v0.8 a conformità **WCAG 2.1 AA** prima del cleanup finale F11:

- Contrast ≥ 4.5:1 per testo, 3:1 per UI
- Keyboard navigation completa (Tab, Shift+Tab, ←→, ESC)
- Focus indicator visibile su tutti gli interactive
- aria-label su icon-only button
- Screen reader smoke test (NVDA / VoiceOver)
- Reduced-motion rispettato in toto

## Stato baseline (verificato pre-blueprint)

- ✅ Token `--focus-ring` esistente in `tokens.css:109`
- ✅ Media query `prefers-reduced-motion: reduce` presente in `tokens.css:295`
- ✅ `tokens.light.css` esistente (override tema chiaro)
- ❌ `bits-ui` NON installato: useremo focus trap manuale in `Modale.svelte` (no nuova dep)
- ⚠️ Focus ring NON applicato uniformemente sui component
- ⚠️ aria-label mancanti su molti icon-only button

## Sub-PR

| Sub-PR | Scope | Stima |
|---|---|---|
| **F10 PR-A** | A11y baseline (focus-ring uniforme + aria-label icon-only button) | 1 gg |
| **F10 PR-B** | Keyboard nav (focus trap in Modale primitive + resizer ←→) | 0.5 gg |
| **F10 PR-C** | Tema chiaro contrast pass (tokens.light.css review + fix chip/mono) | 0.5-1 gg |
| **F10 PR-D** | Reduced-motion audit (verifica response animation/transition) | 0.5 gg |

---

## 1. F10 PR-A — A11y baseline

### Path

Cross-cutting. File target principali:

- `apps/client/src/lib/components/TitleBar.svelte`
- `apps/client/src/lib/components/Sidebar.svelte`
- `apps/client/src/lib/components/SidebarMini.svelte`
- `apps/client/src/lib/components/StatusBar.svelte`
- `apps/client/src/lib/components/Modale.svelte`
- `apps/client/src/lib/superfici/DetailPane.svelte` (toolbar markdown editor)
- `apps/client/src/lib/superfici/{Compila,Insight,Regressioni,Impostazioni,Palette}Modal.svelte`
- `apps/client/src/lib/superfici/Onboarding.svelte`
- `apps/client/src/styles/tokens.css` (utility class `.focus-ring`)

### Scope

1. **Aggiungere utility class `:focus-visible`** che applica `box-shadow: var(--focus-ring)` su tutti gli interactive (button, a, input, textarea, select, [role="button"]).
2. **Audit aria-label** su icon-only button (Settings ⚙ / Sun-Moon / X chiudi / collapse / drag handles / toolbar markdown). Già presenti in F8 dove esplicitati; serve grep e fix dei missing.
3. **role="dialog"** + **aria-modal="true"** + **aria-labelledby** sulla primitive Modale (già presente, verifica).

### Stima

1 gg

---

## 2. F10 PR-B — Keyboard nav (focus trap + resizer)

### Path

- `apps/client/src/lib/components/Modale.svelte` — focus trap
- `apps/client/src/lib/superfici/Shell.svelte` — keyboard handler resizer

### Scope focus trap (Modale.svelte)

Implementazione manuale (no `bits-ui` per evitare nuova dep ~50 KB):

1. `onMount`: query `[autofocus]` o primo focusable; `.focus()`
2. Listener `keydown` Tab:
   - Se `e.shiftKey` e focus su primo focusable → cycle a ultimo
   - Se !shift e focus su ultimo → cycle a primo
   - Altrimenti default behavior (browser)
3. `onDestroy`: rilascia focus al trigger originale (salvato in `previouslyFocused = document.activeElement`)

### Scope resizer keyboard (Shell.svelte)

`paneforge` espone i resizer come `<div role="separator">` ma senza keyboard nav by default.
Aggiungere:
- `tabindex="0"` sul resizer
- `onkeydown` ←/→: chiama `panel.resize(currentSize ± step)` con clamp min/max
- `aria-orientation="vertical"`
- `aria-valuenow/min/max` per screen reader

### Stima

0.5 gg

---

## 3. F10 PR-C — Tema chiaro contrast pass

### Path

- `apps/client/src/styles/tokens.light.css`
- `apps/client/src/styles/tokens.css` (token base se necessario)

### Scope

Audit ratio contrast tema chiaro per:
- `--text-default` su `--bg-canvas` (target ≥ 4.5:1)
- `--text-muted` su `--bg-canvas` / `--bg-input` (target ≥ 4.5:1 per testo, 3:1 per UI)
- Chip/badge colored: `--accent-team`, `--accent-private`, `--warning`, `--accent-success`, `--accent-danger` (target 3:1 per UI; testo bianco su accent ≥ 4.5:1)
- Mono editor: `--font-mono` color su `--bg-input`

### Tool

Ratio check via formula WebAIM:
- `(L1 + 0.05) / (L2 + 0.05)` con L = luminance relativa.
- In assenza di tool integrato, usare valori HEX dei token + calcolatore esterno (chrome devtools / contrast-ratio.com per spot-check).

### Stima

0.5-1 gg

---

## 4. F10 PR-D — Reduced-motion audit

### Path

- `apps/client/src/styles/tokens.css` (estende `@media (prefers-reduced-motion: reduce)`)
- Spot check in CSS dei component con `transition`/`animation` non globali

### Scope

1. Grep `transition\|animation` su tutti i `.svelte` in `apps/client/src/lib/`.
2. Verifica che ogni dichiarazione sia coperta dalla media query globale (`* { transition: none !important; animation: none !important }` in `@media reduce`) oppure ha fallback specifico.
3. Aggiungere fallback dove mancante (es. `pop-in` keyframe in Modale primitive).

### Stima

0.5 gg

---

## 5. Pattern comune di integrazione

Per ogni sub-PR:

1. Applicare modifica con minimal diff
2. Verifica `npm run check` 0 errors + `npm test` 88/88 dopo ogni edit
3. Spot check visivo (avvio dev server) per regressioni layout
4. Commit + push + PR autonoma + Monitor CI + auto-merge appena verde

### Out of scope F10

- E2E test screen reader automation (deferito a F11)
- Color-blind simulation testing (manuale; fuori scope automatizzazione)
- High-contrast Windows mode override (fallback nativo OS basta)
- Localizzazione testi a11y (i18n è fuori scope redesign)
- Window legacy `palette` Tauri (gestita in F11)

---

> **Stato blueprint**: 1.0 — pronto per esecuzione iterativa autonoma.
