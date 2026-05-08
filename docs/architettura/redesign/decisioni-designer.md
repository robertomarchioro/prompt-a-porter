# Decisioni designer — Redesign UX/UI v0.8

> **Versione**: 1.0 · **Data apertura**: 2026-05-08 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` · **Sorgente design**: `docs/architettura/redesign/`

Tabella delle 14 decisioni che richiedono input del designer prima del cutover v0.8.0. Bundle handoff unico (vedi nota in calce).

## Legenda stato

| Stato | Significato |
|---|---|
| 🟡 open | Decisione ancora da prendere — designer deve scegliere |
| 🔵 in-attesa | Designer ha ricevuto, in revisione |
| 🟢 risolta | Decisione presa, riga aggiornata col risultato |
| 🚫 dropped | Decisione caduta (cambio scope) |

## Bloccanti

- **F0 (Foundation)** è bloccato dalla decisione **#4** (contrast review tema chiaro). Tutte le altre possono essere risolte in parallelo o entro la fase che le consuma.

---

## Tabella decisioni

| # | Stato | Area | Decisione richiesta | Default proposto | Bloccante per |
|---|---|---|---|---|---|
| 1 | 🟡 open | Right-rail Metadati | Visibilità: dropdown estensibile a futuri ruoli (lavoro condiviso, public marketplace) o segmented Privato/Team? | **Dropdown** (utente conferma estensibile) | F6 |
| 2 | 🟡 open | Sidebar | Workspace switcher: oggi 1 vault/utente — placeholder UI o bottone disabilitato? | Placeholder visivo (avatar "P" + nome workspace + chevron disabilitato) | F2 |
| 3 | 🟡 open | Toolbar markdown | Iconografia: set custom `md-*` del prototipo o consolidamento su `lucide-svelte` (già in dep)? | **`lucide-svelte`** (utente conferma consolidamento) | F4 |
| 4 | 🟡 open | **Tema chiaro — bloccante F0** | Contrast review chip colored e mono editor su sfondo chiaro: token override o riusiamo i valori in `tokens.css`? | Designer fornisce eventuali override — token aggiornati in PR dedicata pre-F0 | **F0** |
| 5 | 🟡 open | List pane | Drag-reorder prompt: linea bordo 2px (Linear-like) o card-shift fisico durante drag? | Linea 2px `accent-team` | F3 |
| 6 | 🟡 open | Modale Compila | Rating ±1 + nota su voto neg/neutro (feature attuale): preservare o ridurre a thumbs up/down semplice? | **Preservato** (riduzione = regressione UX, telemetria mostra uso reale) | F8 |
| 7 | 🟡 open | Palette | Slider alpha ricerca ibrida (vector vs keyword): preservare o nascondere in "filtri avanzati"? | Preservato in pannello collassato "Filtri avanzati" della Palette | F8 |
| 8 | 🟡 open | Tab Confronto | Riduzione N-way arbitrario (oggi 2-3 prompt qualunque) → solo A/B/C dello stesso parent: confermare? | Riduzione confermata (decisione redesign README) — N-way migrato in tab dedicata | F5 |
| 9 | 🟡 open | Tab Cronologia | Diff: side-by-side default + toggle unified, o solo unified? | **Side-by-side** (`diff2html`) — utente conferma | F5 |
| 10 | 🟡 open | Sidebar | Workspace switcher click → menù multi-vault o naviga? Oggi disabilitato (1 vault/utente) — design futuro? | Decisione designer per UX target post-multi-vault (no impatto v0.8) | nessuno |
| 11 | 🟡 open | Drag-reorder | Cross-cartella (drag prompt da lista → cartella in sidebar): conferma UX? | Sì, con visual highlight cartella di destinazione | F3 |
| 12 | 🟡 open | Modale Impostazioni | 8 sezioni attuali (account/sync/hotkey/aspetto/vault/ricerca/provider/audit) → 5 prototipo (Aspetto/Vista lista/Editor/Sicurezza/Avanzate). Dove finiscono `audit log AI`, `provider config`, `ricerca + embedding status`? | Sub-sezioni di "Avanzate" | F8 |
| 13 | 🟡 open | Tab Cronologia | Avatar autore: hash deterministico colorato (gravatar-like) o iniziali su sfondo accent? | Hash deterministico colorato | F5 |
| 14 | 🟡 open | List pane | Drag-reorder visual cue per riordino dentro stessa cartella: linea bordo o card-shift? | Linea 2px `accent-team` | F3 |

---

## Allegati per il bundle al designer

Inviare al designer in unico handoff:

1. **Brief sorgente**: `docs/architettura/design-handoff/2026-05-08-redesign-brief.md`
2. **Redesign README**: `docs/architettura/redesign/README.md`
3. **Prototipo**: aprire `docs/architettura/redesign/prototype/redesign.html` in browser
4. **Piano operativo**: `docs/roadmap/redesign-v08.md` (per contesto fasi/effort/scope)
5. **Questo file**: `docs/architettura/redesign/decisioni-designer.md` (le 14 decisioni in tabella)
6. **Screenshots Windows**: forniti separatamente da utente — riferiti dal brief §5 (Mappa schermate)

## Workflow di aggiornamento

Quando il designer risponde:

1. Cambiare stato della riga da 🟡 open → 🔵 in-attesa (alla ricezione) → 🟢 risolta (alla decisione finale).
2. In colonna "Default proposto" sostituire con la decisione effettiva.
3. Aggiungere riga "Note designer" inline se la decisione richiede contesto.
4. Commit con messaggio `docs(redesign): risolta decisione #N - <area>`.

---

> **Stato corrente**: 14/14 decisioni aperte (🟡). Bloccante F0: #4 (tema chiaro). Tutte le altre sbloccabili in parallelo durante backend V014 + F0.
