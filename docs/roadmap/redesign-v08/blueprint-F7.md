# Blueprint F7 — Status bar funzionale

> **Versione**: 1.0 · **Data**: 2026-05-09 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §F7 · **Decisioni designer**: nessuna bloccante per F7 · **Stima**: 1.5 giorni FT · **Bloccato da**: F1 ✅, F4 ✅ (per stato salvataggio)

Estende `StatusBar.svelte` placeholder di F1 con stato live: dot vault con tooltip path, nome prompt corrente con icona visibilità, indicatore salvataggio reattivo, kbd palette cliccabile.

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery (1 PR)](#2-strategia-di-delivery-1-pr)
3. [Custom event di sincronizzazione](#3-custom-event-di-sincronizzazione)
4. [Estensione StatusBar.svelte](#4-estensione-statusbarsvelte)
5. [Modifica DetailPane.svelte](#5-modifica-detailpanesvelte)
6. [Edge case + scope](#6-edge-case--scope)
7. [Test attesi](#7-test-attesi)
8. [Exit criteria](#8-exit-criteria)
9. [Dipendenze su F8](#9-dipendenze-su-f8)

---

## 1. Obiettivo

**Output funzionale F7**:
- **Dot vault sinistra**: verde se `vault_aperto() === true`, rosso altrimenti. Tooltip native `title` mostra path da `vault_percorso()` ("Vault locale: /path/to/data_dir")
- **Centro**: nome prompt corrente con icona visibility (Lock per private, Users per workspace). Vuoto se nessun prompt selezionato → "(nessun prompt selezionato)"
- **Destra**: dot saved (verde) / dirty (giallo) / salvando (blu pulsante) / errore (rosso) + label `salvato 14s fa` / `modifiche non salvate` / `salvando…` / `errore salvataggio`
- **Far right**: kbd `⌃⇧P` cliccabile (apre Palette quando F8 wireuppa, per ora `console.log("F8 palette")`)

**Out of scope F7**:
- Tooltip vault con dettagli SQLCipher (dimensione DB / rotazione master-key) → richiederebbe nuovo cmd backend, scope-out F7.x
- Bottone palette attivo → F8 (modale)
- Animazione tooltip avanzata → F10 a11y/polish

## 2. Strategia di delivery (1 PR)

- **Branch**: `feat/redesign-f7-statusbar`
- **Target**: `feat/redesign-v08`
- **Effort**: 1.5 gg
  - 0.25 gg modifica DetailPane (dispatch 2 custom event)
  - 0.5 gg estensione StatusBar.svelte (3 segmenti reattivi + listener event + load vault info)
  - 0.5 gg testing + refinement visivo
  - 0.25 gg smoke + commit

## 3. Custom event di sincronizzazione

Pattern coerente con `pap:lista-mutata` di F3 PR-A.

### `pap:prompt-corrente`

**Dispatched da**: `DetailPane.svelte` quando carica un prompt o lo cambia.

**Detail**:
```typescript
{ id: string; titolo: string; visibilita: string } | null
```

`null` quando Shell rileva `promptSelezionato === null` (prompt deselezionato).

### `pap:save-stato`

**Dispatched da**: `DetailPane.svelte` ad ogni cambio di `statoSalvataggio`.

**Detail**:
```typescript
{ stato: "salvato" | "dirty" | "salvando" | "errore"; salvatoA?: string }
```

`salvatoA` è ISO timestamp dell'ultimo save riuscito (per il "salvato 14s fa" friendly).

## 4. Estensione StatusBar.svelte

### Segmenti

1. **Sx — vault**: dot OK/err + label "vault locale" / "vault chiuso" + tooltip path
2. **Centro — prompt corrente**: icona visibilità + titolo prompt
3. **Dx — save status**: dot stato + label time-relative
4. **Far Dx — kbd palette**: `⌃⇧P cerca` cliccabile

### Comportamento

- onMount: `Promise.all([vault_percorso, vault_aperto])` per metadata
- Listener `pap:prompt-corrente` aggiorna centro
- Listener `pap:save-stato` aggiorna destra
- `setInterval 10s` per re-render label "X s fa"

### Stato dot vault

- `dot-ok` (verde): `vaultAperto === true`
- `dot-err` (rosso): `vaultAperto === false` (vault chiuso o errore)

### Stato dot save

| stato | colore | label |
|---|---|---|
| `salvato` | verde | `salvato {N}s fa` (oppure "salvato ora") |
| `dirty` | giallo | `modifiche non salvate` |
| `salvando` | blu pulse | `salvando…` |
| `errore` | rosso | `errore salvataggio` |

## 5. Modifica DetailPane.svelte

Dopo `caricaDettaglio` riuscito:

```typescript
async function caricaDettaglio(id: string): Promise<void> {
  // ... existing code ...
  if (d) {
    window.dispatchEvent(
      new CustomEvent("pap:prompt-corrente", {
        detail: { id: d.id, titolo: d.titolo, visibilita: d.visibilita },
      }),
    );
  }
}
```

`$effect` su `statoSalvataggio` per dispatch:

```typescript
let salvatoTs = $state<string | null>(null);

$effect(() => {
  if (statoSalvataggio === "salvato") {
    salvatoTs = new Date().toISOString();
  }
  window.dispatchEvent(
    new CustomEvent("pap:save-stato", {
      detail: { stato: statoSalvataggio, salvatoA: salvatoTs },
    }),
  );
});
```

### Modifica Shell.svelte (clear)

Quando `promptSelezionato` torna null, dispatch clear:

```typescript
$effect(() => {
  if (promptSelezionato === null) {
    window.dispatchEvent(new CustomEvent("pap:prompt-corrente", { detail: null }));
  }
});
```

## 6. Edge case + scope

| # | Caso | Comportamento |
|---|---|---|
| 1 | StatusBar montata prima che Tauri risponda a vault_percorso | Path empty inizialmente, tooltip placeholder. Update appena async risolve |
| 2 | Vault chiuso (vault_aperto false) | Dot rosso + label "vault chiuso" + tooltip "Vault chiuso" |
| 3 | Save status null (nessun prompt aperto) | Sezione save status nascosta, mostra solo "vault" + "(nessun prompt selezionato)" + kbd |
| 4 | Cambio prompt mentre status "salvato 14s fa" attivo | Detail dispatch nuovo `pap:prompt-corrente`, status save NON resetta (resta finché DetailPane decide) |
| 5 | App in background per ore | tickRelative continua a girare ma label diventa "Xh fa" |
| 6 | Click su kbd palette | console.log "F8 palette" — F8 wireuppa |
| 7 | Vault info reload necessario (rotazione master-key) | F7 minimal: `caricaVaultInfo` solo onMount. F7.x può aggiungere listener `pap:vault-mutato` |

## 7. Test attesi

### Smoke test manuale

- [ ] App boot mostra StatusBar con dot vault verde + path tooltip + "(nessun prompt selezionato)" + kbd
- [ ] Click PromptCard → centro mostra nome prompt + icona visibilità
- [ ] Modifica titolo/body → status save mostra "modifiche non salvate" giallo
- [ ] Dopo 2s → "salvando…" blu pulse → "salvato ora" verde
- [ ] Dopo 30s → "salvato 30s fa"
- [ ] Switch prompt → centro si aggiorna
- [ ] Click ⌃⇧P kbd → console "F8 palette"
- [ ] Hover dot vault → tooltip path

### Type-check

- `npm run check`: 0 errors
- `npm test`: tutti pass

## 8. Exit criteria

PR `feat/redesign-f7-statusbar` può fare merge solo se:

- [ ] `StatusBar.svelte` esteso con vault info, prompt corrente, save status, kbd palette
- [ ] `DetailPane.svelte` dispatch `pap:prompt-corrente` post-load + `pap:save-stato` con effect
- [ ] `Shell.svelte` dispatch clear `pap:prompt-corrente` quando promptSelezionato → null
- [ ] Smoke test §7 passato manualmente
- [ ] `npm run check` 0 errors
- [ ] `npm test` tutti pass
- [ ] CI lint-and-test verde
- [ ] Bundle aggiunto: ≤ 2 KB gzip

## 9. Dipendenze su F8

F7 sblocca:

- **F8 Palette modale**: kbd `⌃⇧P` clickable → apre `Palette.svelte` quando F8 la rifa.
- **F8 modale Compila**: nessuna dipendenza diretta.

**Interface contract** (custom event):

```typescript
// CustomEvent "pap:prompt-corrente" — detail: { id, titolo, visibilita } | null
// CustomEvent "pap:save-stato" — detail: { stato, salvatoA? }
```

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se F7.x richiede metadata vault avanzati (dimensione DB, rotazione master-key) che richiedono nuovo cmd backend.
