# Handoff: PromptVault — Fase 1 (design system + 8 superfici)

## Overview
PromptVault è un'app **macOS desktop** per gestire una libreria personale e di team di prompt riutilizzabili. Vault locale cifrato, sync opzionale per workspace team, evocazione globale via hotkey, parsing live dei segnaposti `{{nome}}` con anteprima e copia.

Questo bundle è la **fase 1 di design**: design system completo (tokens + componenti) e 8 superfici principali in alta fedeltà.

## About the Design Files
I file in `design/` sono **prototipi HTML statici** — riferimenti visivi e di interazione, **non** codice di produzione da copiare.

Il task è **ricreare questi design nell'ambiente target del progetto** (suggerito: **Tauri + React + TypeScript** se non esiste già un codebase, oppure Electron / SwiftUI nativo se preferito), riusando le sue librerie e i suoi pattern. Se non c'è ancora un codebase, scegli il framework più adatto e implementa lì.

I tokens (`design/tokens.css` e `design/tokens.json`) sono invece **direttamente consumabili** in qualsiasi setup CSS/Tailwind/Style-Dictionary.

## Fidelity
**High-fidelity (hifi).** Pixel, colori, tipografia, spaziature e stati sono tutti finali e da riprodurre fedelmente. La densità (liste 32–40px), gli accent semantici (ambra=privato, viola=team), il monospaced ovunque conta il parsing — sono tutte decisioni intenzionali da preservare.

---

## Stack consigliato (se nuovo codebase)
- **Tauri 2.x** (più leggero di Electron, binari piccoli, accesso nativo a tray + global hotkey + filesystem cifrato)
- **React 18 + TypeScript** per l'UI
- **CSS variables direttamente da `tokens.css`** (oppure Tailwind generato dai tokens)
- **SQLite + SQLCipher** per il vault cifrato
- **CodeMirror 6** per l'editor del body prompt (parsing live `{{...}}`, evidenziazione segnaposti)
- **cmdk** o equivalente per il Command Palette
- **Inter** (UI) e **JetBrains Mono** (mono) — entrambi self-hosted, niente Google Fonts CDN per app desktop

---

## Design Tokens
Sorgente di verità: **`design/tokens.css`** (variabili CSS, dark + light + 3 toni neutri) e **`design/tokens.json`** (mirror per build tools).

### Palette
- **Tono neutro scelto: `zinc`** (neutro caldo). I file `tokens.css/.json` contengono anche slate (freddo) e stone (carta) come alternative — non rimuoverli, possono servire per tematizzazione futura.
- **Accent privato:** ambra `oklch(0.78 0.14 78)` su dark, `oklch(0.62 0.13 65)` su light. Usato per: glifo lucchetto, vault, "privato", focus inputs.
- **Accent team:** viola `oklch(0.70 0.18 290)` su dark, `oklch(0.55 0.20 285)` su light. Usato per: workspace, sync, CTA condivisione, focus team.
- **Coppia `-on` per ogni accent**: testo/icona da metterci sopra (rispetta WCAG AA).
- **Coppia `-soft`**: versione a bassa opacità per badge, pill, sfondi sottili.
- **Semantici:** `--success`, `--warning`, `--danger` con variante `-soft`.

### Tipografia
| Token | Valore | Uso |
|---|---|---|
| `--font-ui` | Inter | UI generale |
| `--font-mono` | JetBrains Mono (no ligatures) | body prompt, codice, paths, hotkey, stati di sync, conteggi |
| `--fs-xs` 12 / `--fs-sm` 13 / `--fs-base` 14 / `--fs-lg` 16 / `--fs-xl` 20 / `--fs-2xl` 24 / `--fs-3xl` 30 | scale tipografica |
| `--lh-tight` 1.2 / `--lh-snug` 1.35 / `--lh-normal` 1.5 / `--lh-loose` 1.7 | line-height |
| `--tracking-tight` -0.01em / `--tracking-caps` 0.06em | letter-spacing |
| `--fw-regular` 400 / `--fw-medium` 500 / `--fw-semibold` 600 / `--fw-bold` 700 | pesi |

**Regola:** mono è obbligatorio ovunque conti il parsing — body prompt, `{{segnaposti}}`, hotkey, paths, conteggi. Inter ovunque sia testo umano.

### Spaziatura, radii, ombre, motion
- Scala spaziatura `--sp-1`..`--sp-8` su base 4px (4, 8, 12, 16, 20, 24, 32, 48).
- Radii: `--radius-sm` 4 / `--radius-md` 6 / `--radius-lg` 10 / `--radius-xl` 14 / `--radius-full`.
- Ombre: `--shadow-sm/md/lg` — definite con tinta neutra, mai blur eccessivo.
- Motion: `--motion-fast` 120ms / `--motion-normal` 180ms / `--motion-slow` 280ms con curve cubic-bezier. **Niente animazioni gratuite**: solo entrate di modali, cambio tema, hover sottili.

### Backgrounds (4 livelli)
`--bg-canvas` (sfondo finestra) → `--bg-surface` (sidebar, panel) → `--bg-overlay` (hover, item attivo) → `--bg-raised` (modali, popover) → `--bg-input` (input fields). I bordi seguono la stessa progressione: `--border-subtle` < `--border-default` < `--border-strong`.

---

## Surfaces

### 01 — Command Palette (`design/01 Command Palette.html`)
**3 varianti** per scegliere direzione (poi ne va implementata UNA):
- **Variante A · Linear severo** — minimale, mono, denso
- **Variante B · Raycast prosumer** — bilanciato, gerarchia chiara, scelta consigliata
- **Variante C · Warp terminale** — drammatico, full-screen, terminale-like

**Comportamento:**
- Hotkey globale **⌃⇧P** (configurabile in Impostazioni)
- Modale centrato, larghezza ~640px, animazione di entrata 180ms
- Fuzzy search su titolo + tag + body
- Risultati raggruppati: "Recenti" → "Risultati" → "Azioni"
- Frecce ↑↓ per navigare, Tab per anteprima, ↵ apre/compila, Esc chiude
- Footer mostra le hotkey contestuali

### 02 — Libreria (`design/02 Libreria.html`)
Finestra principale **3 pannelli**: 240px sidebar / 360px lista / resto detail.

**Sidebar:** workspace switcher in alto, gruppi "Viste" (Recenti/Preferiti/Tutti), "Visibilità" (Privati/Team), "Tag" (con dot colorati), Impostazioni in basso.
**Lista centrale:** header con titolo vista + count + sort, search bar inline, card prompt (titolo, descrizione 2-line, tag, timestamp, indicatore visibilità).
**Detail:** titolo grande, descrizione, meta-row (visibilità, autore, timestamp, uso), body in mono dentro un riquadro, tabella segnaposti rilevati (nome / tipo / default / obbligatorio), cronologia.

**Status bar in fondo** sempre visibile: stato sync, path vault, versione, hint hotkey.

### 03 — Editor Prompt (`design/03 Editor Prompt.html`)
Modale scrim 960×720 max. Layout 2 colonne.
**Sinistra:** titolo, descrizione, body editor in mono con highlighting `{{...}}` live.
**Destra:** segnaposti rilevati (parser deve aggiornarli al volo), tag picker, switch visibilità privato/team, anteprima rendering con default applicati.
Footer: autosave indicator, anteprima full ⌘P, Annulla esc, Salva ⌘S.

**Parsing segnaposti:** regex `/\{\{(\w+)\}\}/g`, deduplica per nome, ogni segnaposto ha tipo (testo / multilinea / enum), default opzionale, flag obbligatorio.

### 04 — Renderer / Compilatore (`design/04 Renderer.html`)
Finestra dedicata, 2 colonne.
**Sinistra:** form con un campo per ogni segnaposto, progress bar in alto (N di M compilati), Tab per avanzare. Il campo attivo ha un anello sottile ambra **solo** sul code-pill del nome (NON sul box intero — questo era un feedback esplicito, vedi commit più recente).
**Destra:** anteprima live in mono, valori sostituiti evidenziati con sfondo `accent-private-soft`, valori vuoti con sfondo `bg-overlay` corsivo. Toggle Markdown / Plain / JSON in alto. Footer con stato pronto, conteggio token, **Copia ⌘↵** primario, dropdown send-to per integrazioni future.

### 05 — Impostazioni (`design/05 Impostazioni.html`)
Layout sidebar (220px) + content. Sezioni: Account, Sync, Hotkey, Aspetto, Vault, Lingua, Info.
Pattern righe: `1fr auto` con label/descrizione a sinistra e controllo a destra. Gruppi con titoletti mono uppercase.
**Hotkey input** con stato "registrazione" (ring viola attorno + bordo team).
**Vault** include: path con button "Mostra in Finder", cambio password, esporta/importa, **elimina vault** in `btn--danger`.

### 06 — Onboarding (`design/06 Onboarding.html`)
Wizard **3 step**, modale centrale 720px.
1. **Profilo:** Personale (ambra) vs Team (viola), card grandi cliccabili.
2. **Password vault:** input password + conferma, strength meter (4 livelli), checkbox "salta cifratura" come opzione sconsigliata.
3. **Hotkey:** registrazione combinazione, Esc reset, switch "crea prompt di esempio".
Progress bar 4px in alto. Switcher "Step 1/2/3" in basso solo per la review (rimuovere in produzione).

### 07 — Auth (`design/07 Auth.html`)
**3 schermate** tutte 440px centrate:
- **Login** — email + password, link "Password dimenticata", checkbox "Ricorda 30g", divider "oppure", Continua con SSO.
- **Reset password** — solo email, alert giallo "i prompt cifrati con la vecchia password andranno persi".
- **Recupera workspace** — dato un email mostra lista workspace associati con avatar + nome + URL sync, oppure "Crea nuovo workspace".

Workspace pill in alto fa da breadcrumb. Footer con versione.

### 08 — Tray icon (`design/08 Tray.html`)
Glifo monocromatico **vault con notch a destra** + 5 line indicator (rappresentano i prompt). Disegnato a 16×16 e 32×32 pixel-perfect. Stroke 1.25px @16, 2px @32, square caps, miter joins, `currentColor` per essere Apple template image.
**Dot di stato:** opzionale, top-right del glifo. Ambra = ha aggiornamenti / errori non critici, rosso = errore sync.

**Menu contestuale macOS** stile vibrancy:
- Header con nome app + meta riga (count prompt + last sync)
- Apri Command Palette ⌃⇧P, Nuovo prompt ⌘N, Mostra libreria ⌘L
- Sezione Workspace con check accanto al corrente, click su altro = switch
- Sincronizza ora ⌘R, Impostazioni ⌘,
- Esci ⌘Q

---

## Interactions & Behavior

### Global hotkey
| Combo | Azione |
|---|---|
| ⌃⇧P | Apri Command Palette (globale, anche con app in background) |
| ⌘N | Nuovo prompt |
| ⌘K | Apri palette dentro l'app |
| ⌘L | Mostra libreria |
| ⌘S | Salva (editor) |
| ⌘↵ | Compila & copia |
| ⌘R | Reset form / sync ora |
| ⌘, | Impostazioni |
| Esc | Chiudi modale / annulla |
| Tab | Avanza segnaposto / autocompleta |

### Stati e feedback sempre visibili
- **Sync** in status bar libreria + dot opzionale sul tray.
- **Autosave** dell'editor con timestamp ("ultima 12s fa").
- **Modifiche non salvate** come pill warning ("3 modifiche non salvate").
- **Conteggi** ovunque (12 prompt, 4 segnaposti, ~847 token).

### Tema
Toggle Auto / Light / Dark in Impostazioni. Tutti i token hanno mappatura per entrambi. Toggle in tutti i prototipi in alto a destra (rimuovere in produzione, sostituire con preferenza utente).

### Animazioni
- Modali: scale-from-0.96 + fade, 180ms ease-out
- Hover card: nessun transform, solo cambio bg/border
- Theme switch: 120ms transition su bg/colors
- **Niente** animazioni decorative o "AI-style" (gradienti animati, pulse continui, ring brillanti). Feedback minimal e funzionale.

---

## State Management (alto livello)
- `vault`: lista prompt, scrittura/lettura SQLite cifrato
- `workspaces`: locale + N team workspace, switch attivo
- `sync`: stato (idle/syncing/error/offline), last-sync timestamp, polling configurabile
- `palette`: aperta/chiusa, query, risultati pre-fetched
- `editor`: prompt corrente, dirty flag, autosave timer, segnaposti parsati
- `renderer`: prompt scelto, valori form, output compilato
- `prefs`: tema, densità, lingua, hotkey, path vault

---

## Sicurezza (importante)
- Vault **SQLCipher AES-256**, password mai salvata, mai trasmessa
- Prompt **privati** non lasciano mai il dispositivo, nemmeno per workspace team
- Sync usa TLS verso endpoint configurabile (default suggerito: server self-hosted)
- Reset password = perdita prompt cifrati (avvisare nell'UI, vedi 07 Auth)

---

## Files

```
design/
├── tokens.css              ← variabili CSS (dark + light + 3 toni neutri)
├── tokens.json             ← mirror JSON dei tokens
├── app.css                 ← componenti base (button, input, etc.) — riferimento
├── Design System.html      ← documento esplorativo con razionali in italiano
├── index.html              ← hub navigabile delle 8 superfici
├── 01 Command Palette.html
├── 02 Libreria.html
├── 03 Editor Prompt.html
├── 04 Renderer.html
├── 05 Impostazioni.html
├── 06 Onboarding.html
├── 07 Auth.html
└── 08 Tray.html
```

Apri `design/index.html` per partire dall'hub navigabile.

---

## Principi di design (da preservare in implementazione)
1. **Densità da pro tool, non SaaS** — liste 32–40px, mai 56.
2. **Mono ovunque conta il parsing** — JetBrains Mono senza ligature.
3. **Privato ≠ Team** — due accent semantici, mai confondere col CTA generico.
4. **Tastiera-first** — ogni azione ha hotkey visibile (kbd nei pulsanti).
5. **Stato sempre on-screen** — sync, autosave, conteggi, mai sorprese silenziose.
6. **Dark first, light parità** — stessi tokens, due mappature, AA in entrambi.
7. **Niente effetti AI-pattern** — evitare ring brillanti, gradienti animati, pulse continui. Feedback minimal, funzionale, statico dove possibile.

---

## Redesign post-v0.7 (in corso)

Dopo 7 release minor (`v0.1` → `v0.7`) il prodotto si è stratificato e
viene avviato un nuovo round di design organico. Brief per il designer:

- [`2026-05-08-redesign-brief.md`](./2026-05-08-redesign-brief.md) —
  sintesi prodotto, persona target, architettura, soluzione tecnica,
  elenco feature v0.7 (14 macro-aree), mappa schermate (IA),
  differenziatori strategici, constraint, numeri rilevanti. Sezioni
  "pain points" e "goal del redesign" da rifinire dall'utente prima
  della consegna al designer.

Gli asset Fase 1 sopra (HTML statici, `tokens.css`, `app.css`) restano
**reference storica**: utili per capire la baseline visuale ma NON
riflettono il prodotto v0.7.
