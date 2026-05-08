# Blueprint F0 — Foundation

> **Versione**: 1.0 · **Data**: 2026-05-08 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F0 · **Decisioni designer**: `docs/architettura/redesign/decisioni-designer.md` (#4) · **Stima**: 2 giorni FT (di cui ~0.5 PR `tokens.light.css` dedicata pre-F0)

Documento operativo autoportante per la fase Foundation. Chi lo legge non deve saltare ad altri file per implementare F0.

## Indice

1. [Obiettivo F0](#1-obiettivo-f0)
2. [Strategia di delivery (2 PR)](#2-strategia-di-delivery-2-pr)
3. [PR-A — `feat/redesign-tokens-light` (override #4)](#3-pr-a--featredesign-tokens-light-override-4)
4. [PR-B — `feat/redesign-f0-foundation`](#4-pr-b--featredesign-f0-foundation)
5. [Font self-hosting](#5-font-self-hosting)
6. [Store preferenze persistite](#6-store-preferenze-persistite)
7. [Cascade tema/tono in `App.svelte`](#7-cascade-temato-no-in-appsvelte)
8. [Edge case](#8-edge-case)
9. [Test attesi](#9-test-attesi)
10. [Exit criteria](#10-exit-criteria)
11. [Rischi specifici F0](#11-rischi-specifici-f0)
12. [Dipendenze su F1+](#12-dipendenze-su-f1)

---

## 1. Obiettivo F0

Stabilire il **layer di base** (design tokens, font, tema/tono, store preferenze) sopra cui tutte le fasi successive (F1-F11) costruiranno l'UI redesign. Senza F0 nulla del resto del redesign può partire.

**Output funzionale**:
- L'app legge la preferenza utente (`theme: dark|light`, `tone: zinc|slate|stone`) all'avvio.
- Applica gli attributi `data-theme` e `data-tone` su `<html>`.
- `tokens.css` + `tokens.light.css` (override) producono i 4 schemi colore (`dark/zinc`, `dark/slate`, `dark/stone`, `light/*`) con contrast AA verificato sul tema chiaro.
- Font Inter + JetBrains Mono caricati offline (no CDN).
- Toggle tema (placeholder per ora, F1 lo wireup nella titlebar) muta il valore nello store, persiste su disco, e l'effetto si propaga.

## 2. Strategia di delivery (2 PR)

### PR-A: `feat/redesign-tokens-light`

- **Target**: branch `feat/redesign-v08`
- **Scope**: solo `apps/client/src/styles/tokens.light.css` (override designer #4) + linking in `app.css`
- **Effort**: ~0.5 gg
- **Rationale separazione**: la PR è piccola e auto-contenuta. Permette al designer di riverificare i valori in CI prima che vengano consumati dai component F1+. Se gli override vanno aggiustati, è un solo file da toccare senza ripple sul resto della Foundation.
- **Exit**: `npm run check` verde, contrast script (vedi §9) passa AA su tutti i pair text/bg + chip + border.

### PR-B: `feat/redesign-f0-foundation`

- **Target**: branch `feat/redesign-v08`
- **Bloccato da**: PR-A merged
- **Scope**: tokens.css importato in `app.css`, store preferences, $effect su `<html>`, font self-hosting, fallback `prefers-color-scheme`.
- **Effort**: ~1.5 gg
- **Exit**: 4 combinazioni tema×tono renderate dimostrabili (anche se senza UI altra che placeholder), bundle ≤ +30 KB font, no console errors.

## 3. PR-A — `feat/redesign-tokens-light` (override #4)

### File: `apps/client/src/styles/tokens.light.css` (NEW)

Override esatti dal designer (decisione #4 in `decisioni-designer.md`). Tutti i valori sono definitivi e non vanno modificati senza nuovo handoff designer.

```css
/* tokens.light.css — Override designer #4 per contrast AA tema chiaro
   Caricato dopo tokens.css, attivato da [data-theme="light"] */

[data-theme="light"] {
  /* Mono editor body — contrast 8.9:1 (era 5.2:1 con #3B3F49) */
  --bg-canvas: #FCFCFD;
  --text-default: #1A1F2C;

  /* Border subtle — meno invisibile (era #EEF0F4) */
  --border-subtle: #E2E5EB;

  /* Accent privato — passa AA su white (era #A78BFA) */
  --accent-private: #7C3AED;
}

/* Chip tinted — passa AA a 4.7:1 (era 3.1:1)
   Pattern color-mix richiede CSS Color Module Level 5 (Chrome 111+, Safari 16.4+, Firefox 113+).
   Tauri 2 usa WebView2 (Edge Chromium) e WKWebView macOS che lo supportano nativi. */
[data-theme="light"] .chip.warn {
  background: color-mix(in oklch, var(--warning) 14%, white);
  color: color-mix(in oklch, var(--warning) 70%, black);
}
[data-theme="light"] .chip.info {
  background: color-mix(in oklch, var(--info) 14%, white);
  color: color-mix(in oklch, var(--info) 70%, black);
}
[data-theme="light"] .chip.success {
  background: color-mix(in oklch, var(--success) 14%, white);
  color: color-mix(in oklch, var(--success) 70%, black);
}
```

**Note importanti**:
- `color-mix(in oklch, ...)` è supportato da WebView2 (Chrome 111+) e WKWebView (Safari 16.4+) — il design del prototipo usa la stessa funzione, quindi compatibile con ambiente Tauri 2.
- Nomi classi `.chip.warn`, `.chip.info`, `.chip.success`: questi sono nomi *futuri* — non esistono component oggi. Saranno usati da PromptCard meta-row (F3), DetailPane meta-row (F4), Status bar (F7), modali F8.
- Il file resta dormiente fino a quando le classi `.chip.*` saranno renderate: zero impatto visivo finché non c'è UI. Niente regressioni.

### File: `apps/client/src/app.css` (MODIFY)

Aggiungere `@import` di `tokens.css` (esistente nel prototipo) e `tokens.light.css`:

```css
/* Top of app.css */
@import url('./styles/tokens.css');
@import url('./styles/tokens.light.css');

/* ... resto invariato ... */
```

### Copia `tokens.css` nel codebase

Il file `docs/architettura/redesign/prototype/tokens.css` (302 righe) va copiato 1:1 in `apps/client/src/styles/tokens.css`. NON modificarlo: gli override designer vanno solo in `tokens.light.css`. Coerenza garantita: `tokens.css` definisce i 3 tone × 2 theme + accent + semantici + motion + reduced-motion media query.

### Path completi PR-A

```
apps/client/src/styles/tokens.css         (NEW, copy 1:1 dal prototipo)
apps/client/src/styles/tokens.light.css   (NEW, override designer)
apps/client/src/app.css                   (MODIFY, aggiungere @import)
```

### Smoke test PR-A

```bash
cd apps/client
npm run check        # svelte-check verde
npm run dev          # avvio app, ispezione DevTools
# Console DevTools:
document.documentElement.dataset.theme = 'light'
# Verifica visiva: chip placeholder hardcoded (creato ad hoc per smoke), bg/text/border, mono editor area
```

---

## 4. PR-B — `feat/redesign-f0-foundation`

### Cascade `<html>` con `data-theme` + `data-tone`

In `apps/client/src/App.svelte`, prima del root component, aggiungere `$effect` che sincronizza store → DOM.

```svelte
<script lang="ts">
  import { preferences } from '$lib/stores/preferences';
  import { onMount } from 'svelte';

  // Auto-detect prefers-color-scheme se utente non ha mai settato il tema
  onMount(() => {
    if (preferences.theme === 'auto') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      preferences.theme = mq.matches ? 'dark' : 'light';

      // Listener per cambio sistema (utente passa da dark a light o viceversa)
      mq.addEventListener('change', (e) => {
        if (preferences.theme === 'auto') {
          preferences.theme = e.matches ? 'dark' : 'light';
        }
      });
    }
  });

  $effect(() => {
    document.documentElement.dataset.theme = preferences.theme;
    document.documentElement.dataset.tone = preferences.tone;
  });
</script>

<!-- ... resto App.svelte ... -->
```

**Note**:
- Lo store usa `$state` runes Svelte 5 (vedi §6).
- `dataset.theme = 'auto'` è transitorio — viene risolto a `'dark'` o `'light'` al mount.
- L'`$effect` riapplica gli attributi a ogni cambio store senza richiedere un setter manuale.

---

## 5. Font self-hosting

### Scelta del subset

**Latin only** (sufficiente per IT, no CJK/cirillico/greek):
- `latin` (U+0000-007F, U+00A0-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD)
- `latin-ext` (U+0100-024F, U+0259, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20CF, U+2113, U+2C60-2C7F, U+A720-A7FF) — opzionale per nomi UI (es. caratteri rumeni/turchi se workspace nominati così).

**Decisione**: solo `latin` (subset minimale). `latin-ext` rinviato. Risparmio bundle ~50% per famiglia.

### File da scaricare

Da Google Fonts via `google-webfonts-helper` (https://gwfh.mranftl.com/fonts) o tools equivalenti che forniscono i font come `.woff2` con subset specifico.

#### Inter (4 weights, latin only)

```
apps/client/static/fonts/inter-400-latin.woff2     (~14 KB)
apps/client/static/fonts/inter-500-latin.woff2     (~14 KB)
apps/client/static/fonts/inter-600-latin.woff2     (~14 KB)
apps/client/static/fonts/inter-700-latin.woff2     (~14 KB)
```

#### JetBrains Mono (2 weights, latin only)

```
apps/client/static/fonts/jetbrains-mono-400-latin.woff2  (~17 KB)
apps/client/static/fonts/jetbrains-mono-500-latin.woff2  (~17 KB)
```

**Totale**: 6 file, ~90 KB on-disk, ~30 KB gzip stimati (woff2 è già pre-compresso, lo riduciamo poco col gzip CDN).

### `@font-face` declarations

In `apps/client/src/styles/fonts.css` (NEW):

```css
/* Inter */
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/fonts/inter-400-latin.woff2') format('woff2');
}
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 500;
  font-display: swap;
  src: url('/fonts/inter-500-latin.woff2') format('woff2');
}
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 600;
  font-display: swap;
  src: url('/fonts/inter-600-latin.woff2') format('woff2');
}
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('/fonts/inter-700-latin.woff2') format('woff2');
}

/* JetBrains Mono */
@font-face {
  font-family: 'JetBrains Mono';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/fonts/jetbrains-mono-400-latin.woff2') format('woff2');
}
@font-face {
  font-family: 'JetBrains Mono';
  font-style: normal;
  font-weight: 500;
  font-display: swap;
  src: url('/fonts/jetbrains-mono-500-latin.woff2') format('woff2');
}
```

Importare in `app.css`:

```css
@import url('./styles/tokens.css');
@import url('./styles/tokens.light.css');
@import url('./styles/fonts.css');
```

### Verifica privacy local-first

- Controllare che `tokens.css` (copiato dal prototipo) NON contenga `@import url('https://fonts.googleapis.com/...')` — se sì, rimuoverlo. La privacy dell'app dipende dal NON fare network requests verso CDN font (vedi brief §8).
- L'unica fonte font deve essere il file `static/fonts/*.woff2` locale.

---

## 6. Store preferenze persistite

### File: `apps/client/src/lib/stores/preferences.ts` (NEW)

Schema dello store con `$state` rune di Svelte 5 + persistenza Tauri filesystem.

```typescript
import { invoke } from '@tauri-apps/api/core';
import { readTextFile, writeTextFile, BaseDirectory, exists } from '@tauri-apps/plugin-fs';

const PREFS_FILE = 'preferences.json';

// Schema versionato — migrations future via campo version.
type PreferencesSchema = {
  version: 1;
  theme: 'dark' | 'light' | 'auto';
  tone: 'zinc' | 'slate' | 'stone';
  // F2+ aggiungerà: sidebarCollapsed, rightRailCollapsed, density, previewLines, ecc.
};

const DEFAULTS: PreferencesSchema = {
  version: 1,
  theme: 'auto',  // Risolto a dark/light da App.svelte mount
  tone: 'zinc',
};

// $state runes — reattivo automaticamente in tutti i $effect dipendenti
export const preferences = $state<PreferencesSchema>({ ...DEFAULTS });

// Carica da disco al boot. Chiamato una volta da App.svelte.
export async function loadPreferences(): Promise<void> {
  try {
    const filePresent = await exists(PREFS_FILE, { baseDir: BaseDirectory.AppLocalData });
    if (!filePresent) {
      // Prima esecuzione: persisti i defaults
      await savePreferences();
      return;
    }
    const raw = await readTextFile(PREFS_FILE, { baseDir: BaseDirectory.AppLocalData });
    const loaded = JSON.parse(raw) as Partial<PreferencesSchema>;

    // Validation difensiva: se qualcosa è corrotto, usa default per quel campo
    if (loaded.version === 1) {
      preferences.theme = (['dark', 'light', 'auto'] as const).includes(loaded.theme as never)
        ? (loaded.theme as PreferencesSchema['theme'])
        : DEFAULTS.theme;
      preferences.tone = (['zinc', 'slate', 'stone'] as const).includes(loaded.tone as never)
        ? (loaded.tone as PreferencesSchema['tone'])
        : DEFAULTS.tone;
    } else {
      // Versione futura non gestita: fallback defaults + log
      console.warn('[preferences] unknown version, using defaults', loaded.version);
    }
  } catch (err) {
    console.error('[preferences] load failed, using defaults', err);
  }
}

// Salva su disco. Chiamato da $effect su mutazione preferences.
export async function savePreferences(): Promise<void> {
  try {
    const payload: PreferencesSchema = {
      version: 1,
      theme: preferences.theme,
      tone: preferences.tone,
    };
    await writeTextFile(PREFS_FILE, JSON.stringify(payload, null, 2), {
      baseDir: BaseDirectory.AppLocalData,
    });
  } catch (err) {
    console.error('[preferences] save failed', err);
  }
}
```

### Hookup in `App.svelte`

```svelte
<script lang="ts">
  import { preferences, loadPreferences, savePreferences } from '$lib/stores/preferences';
  import { onMount } from 'svelte';

  let loaded = $state(false);

  onMount(async () => {
    await loadPreferences();
    loaded = true;
  });

  // Persisti automaticamente ogni cambio (debounced 250ms per evitare flush troppo frequenti)
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    // Trigger di reattività: leggi entrambi i campi
    const _ = `${preferences.theme}|${preferences.tone}`;
    if (!loaded) return;  // Skip durante boot

    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void savePreferences();
    }, 250);
  });

  // Applica attributi data-theme/tone su <html>
  $effect(() => {
    if (!loaded) return;

    // Risolvi 'auto' al volo per evitare di scrivere 'auto' come data-theme (CSS non sa cos'è)
    const resolvedTheme =
      preferences.theme === 'auto'
        ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
        : preferences.theme;

    document.documentElement.dataset.theme = resolvedTheme;
    document.documentElement.dataset.tone = preferences.tone;
  });
</script>

{#if loaded}
  <!-- Resto dell'app (Shell di F1+) -->
{:else}
  <!-- Splash temporaneo durante load preferences (~10ms in pratica) -->
  <div class="boot-loader">…</div>
{/if}
```

### Permission Tauri 2

Verificare `apps/client/src-tauri/capabilities/main.json` o `tauri.conf.json` — il plugin `fs` deve avere permessi su `$APPLOCALDATA/preferences.json`:

```json
{
  "permissions": [
    "fs:default",
    "fs:allow-app-local-data-read",
    "fs:allow-app-local-data-write",
    {
      "identifier": "fs:scope-app-local-data",
      "allow": [{ "path": "$APPLOCALDATA/preferences.json" }]
    }
  ]
}
```

**Path effettivo per OS**:
- macOS: `~/Library/Application Support/<bundle-id>/preferences.json`
- Linux: `~/.local/share/<bundle-id>/preferences.json`
- Windows: `C:\Users\<user>\AppData\Roaming\<bundle-id>\preferences.json`

(Tauri risolve `BaseDirectory.AppLocalData` correttamente per ciascuna piattaforma.)

---

## 7. Cascade tema/tono in `App.svelte`

Vedi §6 per la wireup completo. Punti chiave:

- `data-theme="dark|light"` su `<html>` (risolto da `'auto'`).
- `data-tone="zinc|slate|stone"` su `<html>`.
- `tokens.css` riconosce le combinazioni grazie ai selettori già definiti (ad es. `[data-tone="slate"][data-theme="dark"]`).
- `tokens.light.css` aggiunge override applicati esclusivamente quando `[data-theme="light"]` matcha.

### Verifica visiva combinazioni

Con DevTools della Tauri WebView, smoke test:

```js
// Iterare tutte le combinazioni
['dark', 'light'].forEach(t =>
  ['zinc', 'slate', 'stone'].forEach(o => {
    document.documentElement.dataset.theme = t;
    document.documentElement.dataset.tone = o;
    console.log(t, o, getComputedStyle(document.documentElement).getPropertyValue('--bg-canvas'));
  })
);
```

Output atteso: 6 valori `--bg-canvas` distinti (3 tone × 2 theme), nessuno vuoto.

---

## 8. Edge case

| # | Edge case | Comportamento atteso | Test |
|---|---|---|---|
| 1 | Prima esecuzione (no `preferences.json` su disco) | Carica defaults (`theme: 'auto'`, `tone: 'zinc'`), persiste su disco subito | E2E smoke test su istanza pulita |
| 2 | `preferences.json` corrotto (JSON invalido) | Catch error, log, fallback defaults, sovrascrivi al prossimo save | Unit test su `loadPreferences` con file rotto |
| 3 | `preferences.json` con campo sconosciuto (forward-compat) | Ignora il campo, mantieni quelli noti | Unit test |
| 4 | `preferences.json` con `version: 2` (futura) | Log warning, usa defaults — NON sovrascrivere il file (rispetto della versione user) | Unit test |
| 5 | Utente cambia tema sistema operativo (light → dark) mentre l'app è aperta E `theme === 'auto'` | App si adegua live tramite listener `matchMedia` | Test manuale (DevTools simulate) |
| 6 | Utente seleziona esplicitamente `theme: 'light'` poi tema sistema cambia | App NON cambia (override esplicito vince) | Test manuale |
| 7 | Tauri `fs` plugin manca permessi (config errata) | Log error sul write, app continua a funzionare con valori in-memory | Manual test rimuovendo permission |
| 8 | Utente cambia tema 50 volte in 1 secondo (stress test) | Debounce 250ms → 1 sola scrittura su disco | Test manuale + verifica file mtime |
| 9 | Font woff2 mancante a runtime (file corrotto/cancellato) | `font-display: swap` mostra fallback `-apple-system` finché non risolve, no crash | Test manuale rimuovendo file |
| 10 | App eseguita su Windows 7 / macOS 10.13 (vecchio WebView) | `color-mix()` non supportato → chip in tema chiaro fallback ai colori `--warning` ecc. senza color-mix | Tauri 2 minimum req è già Win10 1809+ / macOS 10.15+, dove tutti gli engine moderni supportano color-mix |

---

## 9. Test attesi

### Unit test — `apps/client/src/lib/stores/preferences.test.ts`

Vitest + mock `@tauri-apps/plugin-fs`. Coprire:

- ✓ Default loaded quando file non esiste
- ✓ Validation campi unknown ignorati
- ✓ Validation campi corrotti → fallback default
- ✓ `version: 2` futuro → defaults, no sovrascrittura
- ✓ Save serializza correttamente (`version: 1` + `theme` + `tone`)
- ✓ Save catch error non lancia exception

### Contrast script — `scripts/check-contrast.mjs` (NEW)

Headless contrast check usando wcag-contrast (npm) o roll-our-own:

```js
// Pseudocodice
import { hex } from 'wcag-contrast';

const PAIRS = [
  // [bg, fg, label, min-ratio]
  ['#FCFCFD', '#1A1F2C', 'mono editor on canvas', 4.5],
  ['mix(--warning 14% white)', 'mix(--warning 70% black)', 'chip warn', 4.5],
  // ... ecc per chip info, success
  ['#FCFCFD', '#7C3AED', 'accent-private on canvas', 4.5],
  ['#E2E5EB', '#FCFCFD', 'border subtle on canvas', 3.0], // UI elements
];

for (const [bg, fg, label, min] of PAIRS) {
  const ratio = hex(bg, fg);
  if (ratio < min) {
    console.error(`FAIL: ${label} ${ratio.toFixed(2)} < ${min}`);
    process.exit(1);
  }
}
```

NB: per i pair con `color-mix()` serve risoluzione manuale (l'OkLCH va convertito a sRGB esplicitamente). In alternativa, semplificare lo script per testare solo i valori statici (`#FCFCFD`, `#1A1F2C`, ecc.) e lasciare il check chip a F10 quando avremo component reali da screenshot.

### Smoke test E2E — `apps/client/tests/e2e/foundation.spec.ts` (placeholder)

```ts
test('toggling theme persists across reload', async ({ page }) => {
  await page.goto('tauri://localhost');
  await page.evaluate(() => {
    // Manuale set, in F8 sarà via Impostazioni UI
    window.localStorage.setItem('test-set-theme', 'light');
  });
  await page.reload();
  const theme = await page.evaluate(() => document.documentElement.dataset.theme);
  expect(theme).toBe('light');
});
```

(Test placeholder — completare quando F8 ci dà UI di toggle.)

---

## 10. Exit criteria

PR-A `feat/redesign-tokens-light` può fare merge solo se:

- [ ] `tokens.css` copiato 1:1 dal prototipo (no modifiche) in `apps/client/src/styles/`
- [ ] `tokens.light.css` contiene esattamente i 4 override #4 (chip warn/info/success, mono colors, border, accent-private)
- [ ] `app.css` importa entrambi i file
- [ ] `npm run check` verde (svelte-check)
- [ ] Bundle CSS aggiunto: ≤ 5 KB gzip

PR-B `feat/redesign-f0-foundation` può fare merge solo se:

- [ ] PR-A merged
- [ ] `preferences.ts` con $state rune + load/save Tauri fs
- [ ] `App.svelte` con onMount → loadPreferences + 2 $effect (data attrs + autosave debounced)
- [ ] `fonts.css` con 6 @font-face Inter + JetBrains Mono
- [ ] 6 file woff2 in `static/fonts/`
- [ ] Capability Tauri permessi `$APPLOCALDATA` aggiornati
- [ ] Test unit `preferences.test.ts` con 6 casi (vedi §9)
- [ ] Smoke manuale: toggle DevTools `dataset.theme` mostra cambio CSS visibile
- [ ] Smoke manuale: 4 combinazioni dark/light × zinc/slate/stone tutte renderizzano valori `--bg-canvas` distinti
- [ ] Bundle aggiunto: ≤ 30 KB font + ≤ 5 KB JS preferences
- [ ] `prefers-color-scheme` listener funziona quando `theme === 'auto'`
- [ ] `npm run check` verde
- [ ] Coverage gate CI 70% mantenuto (può salire grazie a `preferences.test.ts`)

---

## 11. Rischi specifici F0

| # | Rischio | Probabilità | Mitigazione |
|---|---|---|---|
| 1 | `color-mix()` non supportato nell'engine WebView target | LOW | Tauri 2 minimum req Win10 1809+/macOS 10.15+: tutti i WebView2/WKWebView su queste versioni supportano color-mix Chrome 111+ / Safari 16.4+. Verifica con `CSS.supports('color', 'color-mix(in oklch, red, blue)')` al boot, log warning se false |
| 2 | Permission Tauri `fs` non concessi (config dev vs prod divergenti) | MED | Test su build prod prima di mergere PR-B; se fallisce, fallback in-memory + toast warning utente |
| 3 | Self-hosted font con subset sbagliato (caratteri italiani mancanti per nomi workspace) | LOW | Test smoke manuale con stringhe `àèéìòù` + `«»…—` + `áñçü`. Latin subset Google Fonts copre tutti i caratteri italiani standard |
| 4 | `prefers-color-scheme: dark` listener non triggera su Tauri WebView su tutte le piattaforme | MED | Test manuale su 3 OS. Se fallisce su Linux GTK WebView, accettare degradation: lo store mantiene 'auto' ma resolve a 'light' fisso fino al prossimo restart |
| 5 | Debounce 250ms troppo aggressivo (perde l'ultimo save se app crasha entro 250ms) | LOW | Accettabile: la perdita massima è 1 click su tema. F11 può aggiungere `flush()` su window beforeunload |
| 6 | Designer cambia idea su override #4 dopo PR-A merged | LOW | PR dedicata isolata = revert facile. Note di sintesi designer §3 piano stabilizzano gli override |
| 7 | `data-theme="auto"` accidentalmente leakato su `<html>` (CSS non lo riconosce) | MED | $effect risolve sempre 'auto' a 'dark'/'light' prima di scrivere dataset (vedi §6 codice). Test unit specifico |

---

## 12. Dipendenze su F1+

F0 sblocca:

- **F1** (Shell layout): consuma `tokens.css` per `--col-sidebar`, `--titlebar-h`, `--statusbar-h`. Senza F0 non rendere niente.
- **F2** (Sidebar): consuma `--bg-surface`, `--text-default`, `--accent-team`, `--accent-private` (per dot visibilità).
- **F3** (List pane drag-reorder): consuma `--accent-team` per glow box-shadow.
- **F4** (DetailPane Editor): consuma `--font-mono` per CodeMirror.
- **F8** (Modale Impostazioni): legge/scrive `preferences` per UI tema/tono — F0 deve avere lo store esposto.
- **F10** (A11y tema chiaro): contrast pass usa esattamente i valori in `tokens.light.css`. Se gli override #4 cambiano in F0, F10 va riallineata.

**Interface contract** che F0 espone:

```typescript
// $lib/stores/preferences.ts
export const preferences: PreferencesSchema;  // $state mutabile
export async function loadPreferences(): Promise<void>;
export async function savePreferences(): Promise<void>;
```

```text
// CSS variables disponibili dopo F0
// (nei browser DevTools accessibili come getComputedStyle del documentElement)
--bg-canvas, --bg-surface, --bg-elev, --bg-overlay, --bg-input, --bg-scrim, --bg-raised
--text-strong, --text-default, --text-muted, --text-subtle, --text-disabled
--border-subtle, --border-default, --border-strong
--accent-private, --accent-private-strong, --accent-private-soft, --accent-private-on
--accent-team, --accent-team-strong, --accent-team-soft, --accent-team-on
--success, --warning, --danger, --info (+ varianti -soft)
--font-ui, --font-mono
--titlebar-h, --statusbar-h (NB: questi SONO da aggiungere a tokens.css se mancano nel prototipo originale — vedi F1 blueprint)
--motion-fast, --motion-normal, --motion-slow
--easing-standard, --easing-emphasis
--radius-sm, --radius-md, --radius-lg, --radius-full
--sp-1 ... --sp-8
--fs-xs ... --fs-3xl, --lh-tight ... --lh-loose, --fw-regular ... --fw-bold, --tracking-*
--focus-ring, --focus-ring-private
--shadow-sm, --shadow-md, --shadow-lg
```

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se durante PR-A o PR-B emergono cambi di scope (es. designer modifica override token, edge case nuovi).
