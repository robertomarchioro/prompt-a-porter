# Blueprint — Menu contestuale (tasto destro context-aware)

> Stato: **design, no codice**. Approccio (a): menu **contestuale** che cambia
> in base alla superficie cliccata. La personalizzazione utente (quali voci
> mostrare) è **fuori scope** — vedi §11.
> Origine: discussione 2026-06-14. Oggi il tasto destro mostra il menu
> **nativo della webview** (Copia/Incolla/Ispeziona), uguale ovunque.

## 1. Obiettivo

Sostituire — solo dove ha senso — il menu nativo generico con un menu
contestuale che offre le azioni pertinenti all'elemento cliccato (card
prompt, cartella, tag, editor, variante…). Riusare gli handler/comandi già
esistenti e riempire i 3 stub azione di `DetailPane`.

Vincoli: **ibrido** (il nativo resta nei campi di testo), **una sola
istanza** di menu aperta, **accessibile da tastiera** (WCAG 2.1 — l'app è già
AA da M2/F10), **zero dipendenze nuove** (focus/posizionamento manuali, come
il focus-trap già adottato in F10).

## 2. Stato attuale

| Aspetto | Stato | Riferimento |
| --- | --- | --- |
| Menu custom | **assente** | nessun `oncontextmenu`/`ContextMenu` nel codebase |
| Menu nativo | **attivo** | `tauri.conf.json` non lo disabilita; nessun `preventDefault` su contextmenu |
| Primitivo popover | assente | esiste solo `Tooltip.svelte`; nessun portal/dropdown riusabile |
| Focus-trap | pattern esistente | usato in modali (`CompilaModal`, `NuovaCartellaModal`, `PaletteModal`) — manuale, no dep |
| Shortcut util | presente | `lib/util/shortcut.ts` `fmtShortcut("mod+d")` → `⌘D`/`Ctrl+D` |
| Selezione multipla | parziale | `ListPane.svelte:58` `selezioneMultipla?: Set<string>` (Cmd/Ctrl+click, già usata per Diff libero) |

### 2.1 Inventario azioni → backend (cosa è già pronto)

| Azione | Handler frontend | Comando backend | Stato |
| --- | --- | --- | --- |
| Apri prompt | `onSelezionaPrompt(id)` (ListPane) | — | ✅ |
| Apri in Compila | `apriCompila()` (DetailPane:370) | `prompt_compila` | ✅ |
| Duplica (fork) | **stub** `console.log("F8 fork")` (DetailPane:506) | `fork::prompt_fork` | ⚠️ backend pronto, da wirare |
| Preferito (toggle) | **stub** `console.log("F8 toggle fav")` (:494) | `libreria::libreria_toggle_preferito` | ⚠️ backend pronto, da wirare |
| Esporta Markdown | **stub** `console.log("F8 export MD")` (:515) | `import_export::prompt_export_markdown` | ⚠️ backend pronto, da wirare |
| Salva (snapshot) | `salvaManuale()` (:328) | `prompt_aggiorna` | ✅ |
| Elimina prompt | `eliminaPrompt()` (:341, soft-delete) | `editor::prompt_elimina` | ✅ |
| Sposta in cartella | — | `cartelle::prompt_sposta` | ⚠️ comando ok, manca UI submenu |
| Aggiungi/rimuovi tag | `aggiungiTag/rimuoviTag` (:431/440) | (via `prompt_aggiorna`) | ✅ |
| Nuova cartella | `creaNuovoPrompt`/modale | `folder_crea` | ✅ |
| Rinomina cartella | — | `cartelle::folder_rinomina` | ⚠️ comando ok, manca trigger menu |
| Sposta cartella | — | `cartelle::folder_sposta` | ✅ comando |
| Elimina cartella | — | `cartelle::folder_elimina` | ⚠️ comando ok, manca trigger menu |
| Crea variante | `apriModaleCreaVariante` (RightRail:148) | `varianti::prompt_crea_variante` | ✅ |
| Promuovi variante | `promuoviAPrincipale` (RightRail:111) | `varianti::prompt_promuovi_variante` | ✅ |
| Apri import | `apriImport(path)` (RightRail:239) | `prompt_resolve_import_preview` | ✅ |
| **Rinomina tag / unisci / colore / elimina tag** | — | **nessuno** | ❌ backend mancante |
| **Rinomina etichetta variante / elimina variante** | — | **nessuno dedicato** | ❌ gap (delete = `prompt_elimina`?) |

**Conseguenza**: il menu contestuale è quasi tutto "wiring" di backend
esistenti. Le sole voci che richiedono nuovo backend (azioni tag avanzate,
rinomina/elimina variante) si possono **rinviare** o marcare `disabilitato`
nel primo taglio.

## 3. Decisioni di design

1. **Un solo primitivo** `MenuContestuale.svelte` + un **singolo stato
   globale** (store) → al massimo un menu aperto; aprirne uno chiude l'altro.
2. **Descrittore per superficie**: ogni componente fornisce una funzione pura
   `vociMenu(ctx): VoceMenu[]`. Il primitivo è "dumb": riceve voci + posizione
   e renderizza. Nessuna logica di dominio dentro il primitivo.
3. **Ibrido col nativo**: il custom si attiva **solo** sulle superfici-app
   (preventDefault locale). Nei semplici `<input>`/`<textarea>` di testo si
   lascia il menu nativo (Copia/Incolla di sistema). Niente soppressione
   globale del contextmenu.
4. **Riuso handler esistenti**: le `azione` delle voci puntano agli handler
   già definiti (DetailPane/RightRail/ListPane), non si duplica logica.
5. **A11y first**: apertura anche da tastiera (tasto Menu / `Shift+F10`) sull'
   elemento focato; frecce per navigare, `Enter`/`Space` per attivare, `Esc`
   per chiudere, focus che ritorna all'elemento d'origine.
6. **Posizionamento manuale**: `position: fixed` al cursore, con clamp al
   viewport (flip se sborda a destra/in basso). Niente libreria di floating.

## 4. Primitivo `MenuContestuale.svelte`

### 4.1 API

```ts
interface Props {
  aperto: boolean;
  x: number;              // clientX (o rect dell'elemento se da tastiera)
  y: number;              // clientY
  voci: VoceMenu[];
  onChiudi: () => void;   // chiamato su Esc, click fuori, selezione voce
}
```

### 4.2 Comportamento

- Render in overlay `position: fixed; z-index` sopra tutto; backdrop
  trasparente che cattura il click-fuori (`onpointerdown` → `onChiudi`).
- **Clamp viewport**: dopo il mount misura `getBoundingClientRect()`; se
  `x + w > innerWidth` flippa a sinistra; idem verticale. (`$effect` post-render.)
- **Tastiera**: `↑/↓` scorrono le voci abilitate (skip separatori e
  disabilitate, wrap-around); `→` apre un submenu, `←` lo chiude; `Enter`/`Space`
  attiva; `Esc` chiude e **restituisce il focus** all'origine.
- **Focus**: al mount focalizza la prima voce abilitata; trap entro il menu
  (riusa il pattern manuale dei modali esistenti).
- **Submenu**: una voce con `figli` apre un secondo `MenuContestuale`
  affiancato (stesso componente, ricorsivo). Hover + tastiera.
- **Pericolo**: voci `pericolo: true` con stile `--danger` e, se
  `confermaRichiesta`, passano per il dialog di conferma esistente (come
  `eliminaPrompt`).

## 5. Modello dati

```ts
type VoceMenu =
  | {
      id: string;
      label: string;
      icona?: Component;            // lucide-svelte, opzionale
      scorciatoia?: string;         // "mod+d" → reso con fmtShortcut()
      pericolo?: boolean;
      disabilitato?: boolean;       // backend mancante → grigio + tooltip
      figli?: VoceMenu[];           // submenu (es. lista cartelle/tag)
      azione?: () => void | Promise<void>;
    }
  | { separatore: true };
```

### 5.1 Stato globale (store)

```ts
// lib/stores/menu-contestuale.ts
interface StatoMenu { aperto: boolean; x: number; y: number; voci: VoceMenu[]; }
export const menuContestuale = writable<StatoMenu>({ aperto: false, x: 0, y: 0, voci: [] });
export function apriMenu(x: number, y: number, voci: VoceMenu[]): void { … }
export function chiudiMenu(): void { … }
```

Il primitivo è montato **una volta** nello `Shell.svelte` (root), legge lo
store. Ogni superficie chiama `apriMenu(e.clientX, e.clientY, vociMenu(ctx))`
nel proprio handler `oncontextmenu`.

### 5.2 Aggancio per superficie (esempio)

```svelte
<!-- PromptCard.svelte -->
<div
  oncontextmenu={(e) => { e.preventDefault(); apriMenu(e.clientX, e.clientY, vociPrompt(p)); }}
  onkeydown={(e) => { if (e.shiftKey && e.key === "F10") { /* apri da rect */ } }}
>
```

## 6. Menu per area (taglio iniziale)

Legenda: ✅ wirabile subito · ⚠️ backend ok, manca UI · ❌ backend mancante → `disabilitato` nel primo taglio.

### 6.1 Card prompt (`PromptCard.svelte` → `vociPrompt`)
```
Apri                       ↵     ✅
Apri in Compila            ⌘↵    ✅
Duplica (fork)             ⌘D    ⚠️ prompt_fork
Preferito ⭐ (toggle)            ⚠️ libreria_toggle_preferito
Rinomina                   F2    ✅ (prompt_aggiorna su Title)
──────────
Sposta in cartella      ▸        ⚠️ submenu folder_lista → prompt_sposta
Gestisci tag            ▸        ✅ submenu add/remove (libreria_tag_lista)
Crea variante                    ✅ prompt_crea_variante
──────────
Copia contenuto                  ✅ clipboard
Copia come {{import}}            ✅ clipboard (reference)
Esporta come Markdown            ⚠️ prompt_export_markdown
──────────
Elimina                    ⌫     ✅ prompt_elimina (conferma)
```

### 6.2 Cartella (`Sidebar`/`NavItem` → `vociCartella`)
```
Nuovo prompt qui                 ✅ (preset folderId)
Nuova sottocartella              ✅ folder_crea
Rinomina                   F2    ⚠️ folder_rinomina (inline edit)
──────────
Elimina cartella                 ⚠️ folder_elimina (conferma + cosa-fare-dei-prompt)
```

### 6.3 Tag (`Sidebar`/`NavItem` → `vociTag`)
```
Filtra per questo tag            ✅ (= click)
Rinomina tag                     ❌ backend mancante (disabilitato)
Cambia colore           ▸        ❌ backend mancante (disabilitato)
Unisci con…                      ❌ backend mancante (disabilitato)
──────────
Elimina tag                      ❌ backend mancante (disabilitato)
```
> Tutte le azioni tag avanzate sono **disabilitate** finché non si aggiunge
> un modulo `tags.rs` backend. Decisione separata (vedi §10 PR opzionale).

### 6.4 Editor CodeMirror (`EditorTab.svelte` → `vociEditor`)
```
Inserisci segnaposto    ▸        ✅ submenu: segnaposti rilevati + globali → inserisciVariabile()
Inserisci import        ▸        ✅ inserisciImport()
Inserisci {{global …}} ▸        ✅ globale_placeholder_lista
──────────
Taglia / Copia / Incolla         ✅ comandi edit standard (document.execCommand o API CM6)
Seleziona tutto
──────────
Vai alla riga…                   ✅ riusa pap:goto-line
```
> Nell'editor il custom **integra** gli item di edit, non li rimuove: l'utente
> mantiene Copia/Incolla. Su selezione attiva, mostra anche "Cerca selezione".

### 6.5 Pillole varianti (`RightRail.svelte` → `vociVariante`)
```
Passa a questa variante          ✅ apriVariante(id)
Rinomina etichetta               ❌ backend mancante (disabilitato)
Promuovi a principale            ✅ prompt_promuovi_variante
Duplica variante                 ⚠️ prompt_crea_variante (da preset)
──────────
Elimina variante                 ❌ gap (valutare prompt_elimina sulla variante)
```

### 6.6 Chip tag sul prompt (`RightRail` → `vociChipTag`)
```
Rimuovi dal prompt               ✅ onRimuoviTag(id)
Filtra libreria per questo tag   ✅
Vai alla gestione tag            ✅ (apre sezione)
```

### 6.7 Pannelli segnaposti/import (`RightRail` → `vociSegnaposto`/`vociImport`)
```
[segnaposto] Copia nome · Inserisci nell'editor        ✅
[import]     Apri prompt importato · Vai alla definizione  ✅ apriImport()
```

### 6.8 Selezione multipla (≥2 in `ListPane`, `selezioneMultipla`)
```
Confronta (Diff)                 ✅ onConfronta (già esiste)
Sposta N in cartella    ▸        ⚠️ prompt_sposta in loop
Aggiungi tag a N        ▸        ✅
Esporta N come Markdown          ⚠️ prompt_export_markdown in loop / zip
──────────
Elimina N                        ✅ prompt_elimina in loop (conferma unica)
```

## 7. Soppressione del nativo (ibrido)

- **Default**: non si tocca il contextmenu globale → il nativo resta dove non
  agganciamo nulla (input di testo, aree vuote).
- **Superfici-app**: `e.preventDefault()` **solo** negli handler dedicati.
- **Editor**: preventDefault sul wrapper CM6, ma il menu custom include gli
  item di edit per non regredire Copia/Incolla.
- Niente flag globale in `tauri.conf.json`: la granularità per-superficie è
  più sicura e reversibile.

## 8. Edge case e a11y

- **Click fuori / blur / scroll / resize** → chiudi il menu.
- **Apertura su elemento non focato**: dal contextmenu mouse l'elemento
  diventa il target; da tastiera (`Shift+F10`) usa l'elemento focato e
  posiziona il menu sul suo `getBoundingClientRect()`.
- **Viewport overflow**: flip orizzontale/verticale; se troppo alto, scroll
  interno con `max-height`.
- **Submenu vicino al bordo**: apre a sinistra del genitore.
- **Voci disabilitate**: non focalizzabili da frecce, `aria-disabled`, tooltip
  "Disponibile prossimamente" per i gap backend.
- **ARIA**: `role="menu"`, voci `role="menuitem"`, submenu
  `aria-haspopup`/`aria-expanded`; etichette descrittive.
- **Una sola istanza**: aprire un menu mentre un altro è aperto chiude il
  precedente (gestito dallo store).
- **Pericolo**: conferma per Elimina (riusa il dialog esistente), nessuna
  azione distruttiva immediata.

## 9. Test plan

- **Unit (vitest)**: `vociPrompt/vociCartella/...` ritornano le voci attese
  per un contesto dato (incluse le disabilitate per gap backend); helper di
  posizionamento (clamp/flip) puro e testato.
- **Componente**: `MenuContestuale` — navigazione frecce salta separatori e
  disabilitate, `Esc` chiude e ripristina focus, submenu apre/chiude con
  `→`/`←`.
- **Integrazione handler**: i wiring dei 3 stub (fork/preferito/export)
  invocano i comandi giusti (mock `invoke`).
- **A11y**: snapshot ruoli ARIA; apertura da `Shift+F10`.
- Niente nuovi test backend (i comandi esistono già e sono coperti).

## 10. Breakdown PR proposto

| PR | Scope | Rischio |
| --- | --- | --- |
| **PR-1** | Primitivo `MenuContestuale.svelte` + store `menu-contestuale.ts` + mount in `Shell` + a11y/tastiera/posizionamento + test componente | basso-medio |
| **PR-2** | Aggancio **card prompt** (`vociPrompt`) + **wiring dei 3 stub** (fork/preferito/export) + submenu "Sposta in cartella" + "Gestisci tag" | basso |
| **PR-3** | Aggancio **cartelle** (`vociCartella`, rinomina inline + elimina) | basso |
| **PR-4** | Aggancio **editor** (`vociEditor`: inserimenti + edit standard + vai-alla-riga) | medio (CM6) |
| **PR-5** | Aggancio **varianti** + **chip tag** + **pannelli segnaposti/import** | basso |
| **PR-6** | Aggancio **selezione multipla** (azioni bulk) | basso-medio |
| **PR-7** *(opzionale)* | Backend `tags.rs` (rinomina/unisci/colore/elimina) → abilita le voci tag oggi disabilitate | medio |

Ogni PR verde-su-CI e mergeabile da sola. PR-1 è il prerequisito; PR-2…6
sono indipendenti tra loro. Le voci con backend mancante restano
`disabilitato` finché non arriva PR-7.

## 11. Fuori scope

- **Personalizzazione utente** del menu (scegliere quali voci mostrare /
  riordinarle) — possibile Fase 2 col **medesimo pattern del linter**
  (catalogo + toggle), ma non ora: prima il menu deve esistere.
- **Backend azioni tag avanzate** e **rinomina/elimina variante**: in PR-7
  opzionale o rinviati; nel primo taglio sono voci disabilitate, non assenti
  (così la struttura del menu è già completa e leggibile).

## 12. Doc utente da aggiornare

- `docs/utente/scorciatoie.md` (o equivalente): aggiungere le scorciatoie
  introdotte (F2 rinomina, ⌘D duplica, ⌫ elimina) e l'apertura del menu da
  tastiera (`Shift+F10`).
