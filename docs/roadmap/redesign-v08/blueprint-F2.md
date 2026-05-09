# Blueprint F2 — Sidebar (espansa + mini)

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F2 · **Decisioni designer**: #2 (workspace switcher placeholder), #10 (UX target post-multi-vault: nessun impatto v0.8) · **Stima**: 3 giorni FT · **Bloccato da**: F1 ✅ chiusa

Blueprint operativo autoportante per la Sidebar gerarchica del redesign v0.8. Sostituisce `.sidebar-placeholder` di F1 con component completi (workspace switcher, NavGroup collassabili, sidebar-mini 44px).

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery (1 PR)](#2-strategia-di-delivery-1-pr)
3. [Util `avatar-color.ts`](#3-util-avatar-colorts)
4. [Component `NavGroup.svelte`](#4-component-navgroupsvelte)
5. [Component `WorkspaceSwitcher.svelte`](#5-component-workspaceswitchersvelte)
6. [Component `Sidebar.svelte`](#6-component-sidebarsvelte)
7. [Component `SidebarMini.svelte`](#7-component-sidebarminisvelte)
8. [Persistenza stato collapsed](#8-persistenza-stato-collapsed)
9. [Wiring in `Shell.svelte`](#9-wiring-in-shellsvelte)
10. [Edge case + decisioni di scope](#10-edge-case--decisioni-di-scope)
11. [Test attesi](#11-test-attesi)
12. [Exit criteria](#12-exit-criteria)
13. [Dipendenze su F3-F8](#13-dipendenze-su-f3-f8)

---

## 1. Obiettivo

Implementare la sidebar gerarchica del redesign v0.8 con dati live dal backend e supporto collapsed/expanded.

**Output funzionale F2**:
- `Sidebar.svelte` espansa: workspace switcher in cima + 5 NavGroup collassabili + footer con link Insight/Regressioni
- 5 NavGroup: **Viste** (Recenti/Preferiti/Tutti) · **Visibilità** (Privati/Team) · **Cartelle** (gerarchia con indent) · **Tag** (con dot colorato) · **Modello target** (collassato di default, lista distinct)
- Click su un NavItem filtra la lista (gestione filtro live in F3, F2 espone solo `onSelect` via prop/event)
- `SidebarMini.svelte` 44px: stack icone 32×32 con tooltip, click sul primo bottone (chevrons-right) ripristina expanded
- Stato collapsed sidebar + collapsed dei singoli NavGroup persistito in localStorage
- Workspace switcher placeholder (decisione #2): avatar hash colorato + nome "Personale" + chevron disabilitato + tooltip "Multi-vault — v0.9"

**Out of scope F2**:
- Drag-reorder cartelle (F3 — usa V014 `folder_riordina`)
- Logica filtro lista (F3 consuma il `folderId`/`tagId` selezionato dalla sidebar)
- Navigazione a Insight/Regressioni (F8 modali)
- Workspace switcher attivo / multi-vault (rinviato a v0.9 secondo decisione #10)

## 2. Strategia di delivery (1 PR)

- **Branch**: `feat/redesign-f2-sidebar`
- **Target**: `feat/redesign-v08`
- **Effort**: 3 gg distribuiti:
  - 0.25 gg util `avatar-color.ts` + test
  - 0.5 gg `NavGroup.svelte` (header collassabile)
  - 0.25 gg `WorkspaceSwitcher.svelte` (placeholder)
  - 1.0 gg `Sidebar.svelte` (orchestrator + dati live)
  - 0.5 gg `SidebarMini.svelte` (variante 44px)
  - 0.25 gg store sidebar collapsed (localStorage)
  - 0.25 gg wire-up Shell.svelte + smoke

## 3. Util `avatar-color.ts`

### Path

`apps/client/src/lib/util/avatar-color.ts` (NEW)

### Scope

Hash deterministico per generare colore HSL stabile da una stringa qualsiasi (workspace name, ecc.). **F5 Cronologia** userà SHA1 dedicato per email autore (decisione #13). F2 usa djb2 per workspace name (no email, hash sync veloce).

### Implementazione

```typescript
/**
 * Hash djb2 per generare colore HSL deterministico da una stringa.
 * Usato per avatar workspace nella sidebar (F2).
 *
 * NB: F5 Cronologia usa SHA1 separato (decisione #13) — non riusare
 * questa funzione per email utente.
 */
export function hashDjb2(input: string): number {
  let hash = 5381;
  for (let i = 0; i < input.length; i++) {
    hash = ((hash << 5) + hash + input.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

/**
 * Genera coppia { background, foreground } HSL deterministica.
 * - hue derivato dall'hash (0-359)
 * - sat 55%, light 58% (coerente con palette dark/light tokens)
 * - foreground bianco se light < 60, nero altrimenti (AA contrast euristica)
 */
export function coloreAvatar(input: string): {
  background: string;
  foreground: string;
} {
  const hash = hashDjb2(input);
  const hue = hash % 360;
  const sat = 55;
  const light = 58;
  return {
    background: `hsl(${hue} ${sat}% ${light}%)`,
    foreground: light < 60 ? "#fff" : "#000",
  };
}
```

### Test (`apps/client/src/lib/util/avatar-color.test.ts`)

- Hash deterministico: stessa stringa → stesso hash, sempre
- Hash diverso per stringhe diverse (no collisione su un campione di 100)
- `coloreAvatar` ritorna HSL formato corretto
- Foreground bianco/nero corretto secondo soglia 60
- Stringa vuota non lancia (hash su empty = 5381, hue = 5381 % 360)

## 4. Component `NavGroup.svelte`

### Path

`apps/client/src/lib/components/NavGroup.svelte` (NEW)

### Props

```typescript
interface Props {
  titolo: string;          // "VISTE", "VISIBILITÀ", ecc. (uppercase)
  conteggio?: number;      // opzionale, mostrato a destra header
  bottonAggiungi?: boolean; // mostra "+" header (per Cartelle/Tag)
  onAggiungi?: () => void;
  collapsed?: boolean;     // stato esterno via $bindable
  children: Snippet;
}
```

### Markup chiave

- Header con `<button>` toggle (chevron-right collassato / chevron-down espanso) + titolo uppercase 10px + count opzionale a destra + bottone "+" opzionale
- Slot `children` renderizzato solo se `!collapsed`
- `$bindable` per permettere a Sidebar di gestire stato centralmente
- Icone `ChevronRight`/`ChevronDown`/`Plus` da `lucide-svelte` (decisione #3)

## 5. Component `WorkspaceSwitcher.svelte`

### Path

`apps/client/src/lib/components/WorkspaceSwitcher.svelte` (NEW)

### Props

```typescript
interface Props {
  nome: string; // "Personale"
}
```

### Comportamento (decisione #2)

- Avatar 22×22 con prima lettera maiuscola + bg HSL deterministico via `coloreAvatar(nome)`
- Nome workspace
- Chevron-down a destra con `opacity: 0.4` e `cursor: default`
- Tooltip `title="Multi-vault in arrivo — v0.9"` HTML nativo (F8 può sostituire con bits-ui Tooltip)
- Nessun click handler (placeholder visivo non interattivo)

## 6. Component `Sidebar.svelte`

### Path

`apps/client/src/lib/components/Sidebar.svelte` (NEW)

### Props

```typescript
interface Props {
  /** ID vista corrente (recenti/preferiti/tutti/privati/team) */
  vistaCorrente: string;
  /** ID cartella selezionata (null = nessun filtro) */
  folderSelezionato?: string | null;
  /** ID tag selezionato (null = nessun filtro) */
  tagSelezionato?: string | null;
  /** Modello target selezionato (string vuota = tutti) */
  modelTargetSelezionato?: string;

  /** Callback al cambio filtro */
  onSelezionaVista: (id: string) => void;
  onSelezionaFolder: (id: string | null) => void;
  onSelezionaTag: (id: string | null) => void;
  onSelezionaModelTarget: (model: string) => void;

  /** Footer link */
  onApriInsight?: () => void;
  onApriRegressioni?: () => void;
  onApriCollapse?: () => void;

  /** Stato collassamento gruppi (esterno, persistito in Shell) */
  gruppi: {
    viste: boolean;
    visibilita: boolean;
    cartelle: boolean;
    tag: boolean;
    modelTarget: boolean;
  };
}
```

### Caricamento dati al mount

```typescript
onMount(async () => {
  try {
    const [c, t, f] = await Promise.all([
      invoke<ConteggiViste>("libreria_conteggi"),
      invoke<TagInfo[]>("libreria_tag_lista"),
      invoke<Cartella[]>("folder_lista"),
    ]);
    conteggi = c;
    tags = t;
    cartelle = f;
  } catch (e) {
    console.error("[sidebar] caricamento dati", e);
  }
});
```

### Layout sezioni

1. **`.sidebar-top`**: WorkspaceSwitcher + bottone collapse `«`
2. **`.nav-list`** scrollable: 5 NavGroup
3. **`.sidebar-footer`**: 2 link Insight + Regressioni

### NavGroup contenuti

- **VISTE**: 3 NavItem (Recenti, Preferiti, Tutti)
- **VISIBILITÀ**: 2 NavItem con dot accent-private/accent-team (Privati, Team)
- **CARTELLE** (count, +button): each `cartelle` come NavItem
- **TAG** (count, +button): each `tags` come NavItem con dot colorato dal `tag.colore`
- **MODELLO TARGET** (collassato default): per F2 solo NavItem "Tutti"; F3 popolerà lista distinct via libreria

### Schema dati TypeScript

```typescript
interface ConteggiViste {
  tutti: number;
  preferiti: number;
  privati: number;
  team: number;
}

interface TagInfo {
  id: string;
  nome: string;
  colore: string;
}

interface Cartella {
  id: string;
  nome: string;
  path: string;
  parent_folder_id: string | null;
  conteggio_prompt: number;
  // ... altri campi backend non usati da F2
}
```

## 7. Component `SidebarMini.svelte`

### Path

`apps/client/src/lib/components/SidebarMini.svelte` (NEW)

### Props

```typescript
interface Props {
  onApriExpand: () => void;
  onApriInsight?: () => void;
  onApriRegressioni?: () => void;
}
```

### Layout

Stack verticale 44px width, padding `var(--sp-2) 0`. Icone:
1. **ChevronsRight** (espandi) — solo bottone funzionale F2
2. Separatore
3. Clock (Recenti) · Star (Preferiti) · LayoutList (Tutti)
4. Separatore
5. Lock (Privati) · Users (Team)
6. Separatore
7. Folder · Tag · Cpu (Modello target)
8. Footer spacer (push-down)
9. BarChart3 (Insight) · AlertTriangle (Regressioni)

### Note F2

- Solo "Espandi" e Insight/Regressioni hanno callback. Le altre icone sono visual-only con `title` tooltip.
- F3 può aggiungere callback per saltare al filtro corrispondente.

## 8. Persistenza stato collapsed

### File

`apps/client/src/lib/stores/sidebar-collapsed.ts` (NEW, plain TS)

### Schema localStorage

Key: `pap.sidebar.collapsed`. Valore JSON:

```json
{
  "sidebarCollapsed": false,
  "gruppi": {
    "viste": false,
    "visibilita": false,
    "cartelle": false,
    "tag": false,
    "modelTarget": true
  }
}
```

### API

```typescript
export interface StatoSidebar {
  sidebarCollapsed: boolean;
  gruppi: {
    viste: boolean;
    visibilita: boolean;
    cartelle: boolean;
    tag: boolean;
    modelTarget: boolean;
  };
}

export const DEFAULTS: StatoSidebar; // export per test
export function caricaStato(): StatoSidebar;  // try/catch fallback DEFAULTS
export function salvaStato(stato: StatoSidebar): void;  // try/catch silent
```

### Default

`modelTarget: true` (collassato) per coerenza con prototipo. Tutti gli altri `false`.

### Reattività con Shell.svelte

Shell mantiene `let stato = $state<StatoSidebar>(caricaStato())`. `$effect` debounced 200ms chiama `salvaStato(stato)` su ogni cambio.

## 9. Wiring in `Shell.svelte`

### Diff

- Importa Sidebar/SidebarMini + caricaStato/salvaStato
- Sostituisce `.sidebar-placeholder` con `{#if stato.sidebarCollapsed}<SidebarMini/>{:else}<Sidebar/>{/if}`
- Quando collapsed, il primo Pane usa `defaultSize={4} minSize={4} maxSize={4}` (≈44px su viewport 1100px)
- Quando expanded, mantiene il `defaultSize={20}` di F1
- Mantiene state filtri stub (vistaCorrente/folderSelezionato/tagSelezionato/modelTargetSelezionato) per F2; F3 li promuoverà a livello superiore o store dedicato

### State filtri F2

Stub per ora — i 4 filtri sono `$state` locali in Shell che si aggiornano sui callback Sidebar. F3 li userà per popolare la lista.

## 10. Edge case + decisioni di scope

| # | Caso | Comportamento atteso |
|---|---|---|
| 1 | localStorage corrotto | `caricaStato` catch → DEFAULTS |
| 2 | Backend offline (cmd Tauri fallisce) | `onMount` catch → conteggi/tag/cartelle vuoti, sidebar funzionale ma senza dati |
| 3 | Click su NavItem inesistente (race con cancellazione cartella) | F3 lo gestirà; F2 chiama callback, no crash |
| 4 | Sidebar-mini bottoni cliccati senza callback | No-op (no error). F3 può wire-up |
| 5 | Workspace switcher cliccato | No-op (cursor: default). Tooltip mostra messaggio v0.9 |
| 6 | NavGroup vuoto (es. 0 cartelle) | Gruppo si renderizza vuoto, `conteggio={0}` mostrato. UX OK |
| 7 | Tag senza colore (`colore` vuoto) | dot fallback `var(--text-subtle)` (gestito in style) |
| 8 | Sidebar > 360px | maxSize 30% in F1 limita; clamp dinamico in F2 ulteriore |

## 11. Test attesi

### Unit test inline (`apps/client/src/lib/util/avatar-color.test.ts`)

- `hashDjb2` deterministico (stessa stringa = stesso hash)
- `hashDjb2("")` non lancia
- `coloreAvatar` ritorna formato HSL string parseable
- Foreground bianco/nero coerente con soglia
- Hash diverso per input diversi (campione)

### Smoke test manuale

- [ ] Apertura `?redesign-shell` mostra Sidebar con dati live
- [ ] Click su Recenti/Preferiti/Tutti switcha vista
- [ ] Click su una Cartella la evidenzia attiva
- [ ] Click su un Tag idem
- [ ] Click su NavGroup header collapse/expand correttamente
- [ ] Stato collapsed persiste tra reload (localStorage)
- [ ] Click su collapse `«` riduce sidebar a 44px (mostra SidebarMini)
- [ ] Click su `»` di SidebarMini espande di nuovo
- [ ] Workspace switcher mostra avatar "P" colorato + tooltip al hover

### Type-check + test client

- `npm run check`: 0 errors
- `npm test`: tutti i test passano (incluso avatar-color)

## 12. Exit criteria

PR `feat/redesign-f2-sidebar` può fare merge solo se:

- [ ] `lib/util/avatar-color.ts` + `.test.ts` (test inline)
- [ ] `NavGroup.svelte` con header collassabile + slot
- [ ] `WorkspaceSwitcher.svelte` placeholder
- [ ] `Sidebar.svelte` con 5 NavGroup + dati live + footer
- [ ] `SidebarMini.svelte` con stack icone 32×32
- [ ] `lib/stores/sidebar-collapsed.ts` (localStorage)
- [ ] `Shell.svelte` swap Sidebar/SidebarMini basato su stato
- [ ] Smoke test §11 passato manualmente
- [ ] `npm run check` 0 errors
- [ ] `npm test` tutti pass
- [ ] CI lint-and-test verde
- [ ] Bundle aggiunto: ≤ 8 KB gzip

## 13. Dipendenze su F3-F8

F2 sblocca:

- **F3** (List pane): consuma i 4 callback di filtro Sidebar (vista/folder/tag/modelTarget) per popolare la lista. F3 sostituisce gli stub `console.log` di F2 in Shell.
- **F8 Modali**: bottone "+ Aggiungi cartella/tag" della NavGroup chiama un handler che apre la modale (F8 le rifa).
- **F8 Modali Insight/Regressioni**: footer Insight/Regressioni links → modali quando F8 le wireuppa.

**Interface contract** che F2 espone:

```typescript
// $lib/components/Sidebar.svelte — orchestrator espanso
// $lib/components/SidebarMini.svelte — variante 44px
// $lib/components/NavGroup.svelte — primitive riusabile da F8 Impostazioni accordion
// $lib/components/WorkspaceSwitcher.svelte — solo Sidebar lo usa per ora
// $lib/util/avatar-color.ts — riusato in F8 Compila / F5 (F5 ha SHA1 dedicato)
// $lib/stores/sidebar-collapsed.ts — solo Shell lo consuma
```

CSS classi globali esposte: nessuna (tutto scoped Svelte).

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se durante implementazione si scopre che i bottoni "+" delle NavGroup richiedono integrazione complessa con modali esistenti (in tal caso, scope-out a F8).
