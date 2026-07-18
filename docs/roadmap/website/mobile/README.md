# Handoff: Prompt-à-porter — Landing **Mobile** (concept "Scontrino cucito", opzione 3a)

## Overview
Landing page **mobile-first** per **Prompt-à-porter**, app desktop *local-first* per gestire una collezione di prompt AI. Concept creativo editoriale-fashion ("prêt-à-porter" → "prompt-à-porter"): i prompt sono capi di un guardaroba, versionabili e cuciti su misura con i segnaposti, richiamabili ovunque con la **command palette `Ctrl+Shift+P`**.

Questa consegna riguarda **una sola schermata**: il layout mobile verticale, larghezza di riferimento **392px** (viewport telefono). Il concept si chiama **"Scontrino cucito"**: la pagina è costruita come uno **scontrino d'atelier** (carta chiara, dentellature, spilli e filo da cucito) su palette **Cloud Dancer** (Pantone Color of the Year 2026, un bianco caldo/freddo bilanciato).

Differenza chiave rispetto a una landing desktop: **niente pulsante "Scarica per Windows"**. L'utente è già sul telefono, l'app vive sul desktop → la CTA è **"Mandami il link"** (campo email; funzione backend da realizzare: invia all'indirizzo il link di download).

## About the Design Files
Il file in questo bundle è un **riferimento di design realizzato in HTML** — un prototipo che mostra aspetto e comportamento voluti, **non codice di produzione da copiare così com'è**. Il compito è **ricreare questo design nell'ambiente del codebase di destinazione** (la landing reale vive in `apps/site`, VitePress/Vue) usando i pattern e le librerie già in uso; se non esiste ancora un ambiente adatto, scegliere il framework più appropriato (per una landing statica: Astro / VitePress / plain HTML+CSS).

⚠️ **Nota tecnica sul file**: il prototipo è un "Design Component" (`.dc.html`) che gira su un runtime di prototipazione (`support.js`). **Ignorare quel runtime**: servono solo il markup e gli stili inline come riferimento visivo. Due dettagli d'implementazione del prototipo da NON riprodurre letteralmente:
- I token letterali `{{nome}}`, `{{globale autore}}`, `{{import "ruolo"}}` sono scritti come `<i style="font-style:inherit">{</i>{nome}}` per aggirare il parser del runtime. Nel codice reale scrivere semplicemente `{{nome}}` (testo normale), rispettando lo stile della pill ambra.
- Il file contiene anche vecchie esplorazioni (turni `#t1` e `#t2`, opzioni 1a–1d, 2a–2d): **sono scartate**. L'unica schermata da implementare è **`#3a`** (turno `#t3`).

## Fidelity
**Alta fedeltà (hi-fi).** Colori, tipografia, spaziature, ombre e composizione sono definitivi per il mobile. Ricreare l'UI in modo pixel-perfect. Uniche eccezioni (placeholder da sostituire con asset reali in produzione):
- Il **mockup della command palette** sopra il "documento" (l'immagine del gesto à-porter) è una ricostruzione HTML/CSS: in produzione sostituibile con screenshot/registrazione reale dell'app.
- Il **logo mark** è un placeholder tipografico ("P" serif in quadrato con gradiente viola). Usare l'icona ufficiale `{ P }` se disponibile.

---

## Design Tokens

### Colori — palette "Cloud Dancer" (chiara)
| Ruolo | Hex | Uso |
|---|---|---|
| Pagina / carta | `#F1F0EC` | sfondo del telefono (Pantone 11-4201 Cloud Dancer, approx) |
| Superficie | `#FBFBF9` | card, scontrino, pill chiare |
| Bianco | `#FFFFFF` | palette cmdk, input |
| Ink | `#26251F` | testo primario, header scontrino |
| Sub | `#54524A` | testo secondario forte |
| Muted | `#8C8A80` | descrizioni, note |
| Faint | `#AEAB9F` | eyebrow/label mono tenui |
| Hairline testo | `#9C9A90` | battute ".txt" (dove usato) |
| Linea | `#E2E0D8` | bordi, divisori pieni |
| Linea soft | `#ECEAE2` / `#E7E5DC` | bordi tenui / tratteggi voci |
| Linea tratteggiata | `#D8D6CC` | perforazioni, separatori scontrino |

### Colori — brand (invarianti, dal design system)
| Ruolo | Hex |
|---|---|
| Viola brand/azione | `#6E56CF` |
| Viola 2 (gradiente) | `#8470D5` |
| Viola profondo (gradiente) | `#5A45BE` |
| Viola chiaro (su scuro) | `#B9A7F0` |
| Crema enfasi (su viola) | `#F0DFC0` |
| Tint viola | `rgba(132,112,213,.10)` |
| **Ambra — segnaposti `{{…}}`** | testo `#A36A2E` su bg `#F3E9D6`, bordo `#E9DCC1` |
| Ambra flag/accento | `#E8C79A` (flag), `#D8A463` / `#C89A55` (filo, becco pinguino) |
| Verde "incl."/ok | `#6FA07A` |

**Regola cromatica fondamentale (vincolo, non suggerimento):** **viola = brand & azione**, **ambra = segnaposti/template**. I token `{{…}}` sono SEMPRE ambra JetBrains Mono su tint ambra, mai viola.

Gradiente brand: `linear-gradient(160deg,#8470D5,#6E56CF)` (bottoni), `linear-gradient(135deg,#8470D5,#6E56CF 52%,#5A45BE)` (ribbon), `linear-gradient(150deg,#8470D5,#6E56CF)` (logo mark).

### Tipografia
Tre famiglie (Google Fonts): **Newsreader** (serif editoriale, titoli, weight 300/400, spesso `<em>` italic per enfasi), **Inter** (400/500/600, corpo/UI), **JetBrains Mono** (400/500/700, label/eyebrow uppercase, token, dettagli tecnici).

Scala usata (mobile, px):
| Elemento | Font | Size | Weight | Note |
|---|---|---|---|---|
| H1 "Prompt-à-porter" | Newsreader | 54 | 300 | line-height .9, `-.04em`; `à` italic viola |
| Sottotitolo hero | Newsreader | 19 | 300 | line-height 1.4; `<em>` ink |
| Ribbon titolo | Newsreader | 26 | 300 | bianco; `<em>` crema `#F0DFC0` |
| Manifesto | Newsreader | 29 | 300 | `-.02em`; `<em>` viola |
| Header scontrino | Newsreader | 22 | 400 | |
| Titolo desktop CTA | Newsreader | 27 | 300 | `<em>` viola |
| Totale "gratis" | Newsreader italic | 26 | 400 | viola |
| Voci scontrino (titolo) | Inter | 12.5 | 600 | ink |
| Voci scontrino (desc) | Inter | 10 | 400 | muted |
| Corpo / paragrafi | Inter | 11.5–12.5 | 400 | line-height 1.5–1.6 |
| Eyebrow / label | JetBrains Mono | 8–9 | 400/700 | uppercase, letter-spacing .16–.28em |
| Token `{{…}}`, kbd, meta | JetBrains Mono | 9–10 | 400 | |

### Spaziatura, raggi, ombre
- Spaziatura orizzontale schermo: **16–24px** dai bordi.
- Raggi: telefono **38px**; card/mockup **12–22px**; **scontrino 4px** (angolo carta); pill **100px**; bottoni **11–12px**; chip piccole **3–5px**; icone quadrate **8px**.
- Ombre: telefono `0 46px 92px -46px rgba(90,88,78,.45)`; card chiare `0 20–26px 44–50px -22..-30px rgba(90,88,78,.5)`; ribbon `0 18px 38px -18px rgba(110,86,207,.65)`; bottone primario `0 12px 26px -12px rgba(132,112,213,.7)`.
- Spilli (SVG): `drop-shadow(0 4px 5px rgba(60,50,30,.35))`.

---

## Screens / Views

### Schermata unica: Landing mobile "Scontrino cucito" (`#3a`)
Colonna verticale, larghezza **392px**, sfondo `#F1F0EC`. Ordine dei blocchi dall'alto:

1. **Status bar finta** — "9:41" + tre puntini/batteria (`JetBrains Mono` 12px, ink). Solo cornice mobile; in produzione si può omettere.

2. **Header sito** — flex space-between, padding `16px 24px 2px`.
   - Brand: quadrato 26×26 radius 8 gradiente `linear-gradient(150deg,#8470D5,#6E56CF)`, "P" Newsreader 15px bianca + "Prompt a Porter" Newsreader 16px.
   - Hamburger: glifo `☰` 19px, colore `#AEAB9F`. (Apre menu di navigazione — da implementare.)

3. **Ribbon di lancio** ("strillo") — card full-width margin `12px 14px 0`, radius 16, gradiente viola `135deg,#8470D5,#6E56CF 52%,#5A45BE`, ombra viola. Overlay decorativi: righe diagonali `repeating-linear-gradient(115deg, rgba(255,255,255,.09) 0 2px, transparent 2px 22px)` + alone ambra radiale in alto a dx.
   - Flag pill: "✦ Nuova stagione", mono 8px uppercase, testo `#3A2A12` su `#E8C79A`, radius 100px.
   - A destra: "AW 2026 · v1.0" mono 8.5px `#E4DEFA`.
   - Titolo: "Debutta *Arioso Atelier*" (Newsreader 26, `<em>` crema `#F0DFC0`).
   - Riga: "La prima stagione esce dalla beta · codice `Cloud Dancer 11-4201`" (Inter 11 `#E4DEFA`, codename mono crema) + pill "Scopri il debutto →" (Inter 11 600, bg `rgba(255,255,255,.16)`, bordo `rgba(255,255,255,.28)`).

4. **Hero** — centrato, padding `24px 24px 20px`.
   - Eyebrow pill: kbd `⌃⇧P` (mono 9px, bg bianco, bordo, `#6E56CF`) + "in qualsiasi app" (Inter 10.5 `#6E6C60`).
   - H1 "Prompt-*à*-porter" (Newsreader 54, `à` italic `#6E56CF`).
   - Sub: "La tua collezione di prompt AI. *Pronti da indossare.*" (Newsreader 19).
   - Pitch: "Un tasto magico e il prompt giusto è tra le dita: lo compili e lo incolli dove stai scrivendo." (Inter 12.5 `#8C8A80`).

5. **Palette in evidenza (il gesto à-porter)** — mockup: un riquadro (radius 22, bordo `#E2E0D8`) con sfondo "documento" pallido `linear-gradient(160deg,#E7E5DE,#D9D7CE)` (tre righe finte grigie) e, in `position:absolute` in basso, la **command palette** bianca che ci galleggia sopra:
   - Search row: lente SVG (`stroke #AEAB9F`) + "Cerca prompt…" (`#AEAB9F`) + caret viola lampeggiante (`@keyframes blink` opacity 0 a 50%, 1.1s steps(1) infinite).
   - Item attivo: bg `linear-gradient(90deg,rgba(132,112,213,.1),transparent)`, barra sx `inset 2px 0 0 #6E56CF`, titolo "Riassumi articolo in N punti" (Inter 12.5 500), desc con `<em>N</em>` viola, badge "privato" ambra.
   - Secondo item: "Email professionale parametrica" + badge "privato".
   - Footer hint: "↵ seleziona" · "⌃↵ compila e incolla" (mono 9.5, il secondo viola).
   - Caption sotto: "FUNZIONE IN EVIDENZA · LA PALETTE" (mono 9 uppercase `#AEAB9F`).

6. **Manifesto in un fiato** — banda edge-to-edge, padding `30px 24px`, bg `#EDEBF6`, bordi top/bottom `#E2E0D8`, alone viola radiale tenue. Kicker "— manifesto —" (mono 9 uppercase `#6E56CF`); frase "Se ci puoi scrivere, ci puoi *incollare il tuo prompt.*" (Newsreader 29, `<em>` viola); sotto "in ogni editor, applicazione o sito web." (mono 10 `#8C8A80`).

7. **Divisore filo da cucito** — SVG punto-filza: `path` ondulato `stroke #C89A55` `stroke-dasharray 7 6`, con un **ago** (triangolo `#B9B6AC` + cruna) all'estremità destra.

8. **Scontrino spillato** — la card centrale. Contenitore bg `#FBFBF9`, bordo `#E2E0D8`, radius 4, ombra soft. Due **spilli** SVG in `position:absolute` sugli angoli superiori (testa viola `#6E56CF` a sx, ambra `#D8A463` a dx; ago `#C7C4BA`), con `drop-shadow`.
   - **Header**: "Atelier · scontrino" (Newsreader 22) centrato + "COLLEZIONE AW26 · Nº 3A-001" (mono 9 uppercase `#AEAB9F`), bordo inferiore tratteggiato.
   - **Intestazione colonne**: "CAPO" (sx) — "PREZZO" (dx), mono 8.5 uppercase `#AEAB9F`, bordo inferiore pieno ink. (Nessuna colonna SKU.)
   - **6 voci** (le feature reali in produzione), ognuna riga flex space-between con titolo Inter 12.5 600 + descrizione Inter 10 `#8C8A80`, e a destra "incl." (`#6FA07A` 10px). Bordo inferiore tratteggiato tra le righe:
     1. **Ogni modifica, in vetrina** — "save = versione · diff git · ripristino"
     2. **Capi su misura** — "`{{nome}}` ad-hoc · `{{globale autore}}`" (token ambra)
     3. **Cerca al buio dell'armadio** — "significato, non parola · ONNX MiniLM"
     4. **Prova prima di comprare** — "varianti A/B · golden test · costo stimato"
     5. **Un tono, dieci tagli** — "`{{import "ruolo"}}` · sartoria modulare" (token ambra)
     6. **Il taglio da terminale** — "`pap` render in pipe · sola lettura" (`pap` in viola)
   - **Servizi dell'atelier · inclusi**: label mono + 3 chip pill (`Ritocco ✦`, `Diagnosi`, `MCP`) su `#F1F0EC` bordo `#E2E0D8`, tra due tratteggi.
   - **Totale**: "TOTALE" (mono uppercase ink) — "*gratis*" (Newsreader italic 26 viola) + sotto "local-first · open source · AGPL 3.0" (Inter 9.5 `#AEAB9F`).
   - **Dentellatura inferiore**: bordo perforato a semicerchi (doppio `radial-gradient` che alterna `#D8D6CC` per l'ombra e `#F1F0EC` per il colore pagina, `background-size:18px 19px`, `repeat-x`), altezza 19px. **Renderla ben visibile.**

9. **Clienti tipo = etichette di manutenzione** — label "A CHI STA BENE" centrata; riga flex di **3 cartellini** (tag appesi) `flex:1`, bg `#FBFBF9` bordo `#E2E0D8`, radius `4px 4px 7px 7px`, leggermente ruotati (`-1.5deg / 1deg / -1deg`), con **foro** in alto (cerchio 8px bordo `#C6C3B9`). Ciascuno: label mono 8 uppercase `#6E56CF` + una riga Inter 10:
   - **Sviluppatore** — "Trenta chiamate al giorno. Template versionabili come il codice."
   - **Copywriter** — "Cinquanta `email_FINAL`. Un brief, mille varianti." (`email_FINAL` mono ambra)
   - **Ricercatore** — "Tre versioni a confronto. Coi golden test, non a occhio."

   > _Nota: nel prototipo, tra questo blocco e la sezione desktop era previsto un cartellino "PaP vs .txt" (confronto a 4 righe); è stato rimosso in fase di revisione. Non implementarlo salvo richiesta._

10. **Sezione desktop "Te lo mandiamo"** — card margin `22px 16px 0`, radius 20, bg `linear-gradient(165deg,#F4F1FC,#EDE9F7 62%,#F3EEE6)`, bordo `#E1DBF2`, alone viola radiale in alto a dx.
    - Eyebrow "L'ATELIER È SUL DESKTOP" (mono 8.5 `#6E56CF`).
    - Titolo "Questo capo si prova al computer. *Te lo mandiamo.*" (Newsreader 27, `<em>` viola).
    - Paragrafo (Inter 11.5 `#6E6C60`): "Prompt a Porter vive su Windows, macOS e Linux. Lasciaci l'indirizzo: il link ti aspetta quando ti siedi al computer. Niente da inquadrare, niente app da cercare."
    - **Form**: input placeholder "la-tua@email.it" (bianco, bordo `#E1DBF2`, radius 12) + bottone "Mandami il link →" (gradiente brand, bianco, radius 12, ombra viola). **Nessun QR** (scelta deliberata: l'utente è già sul telefono).
    - Nota con pallino verde: "un solo link. Niente newsletter, niente account, niente scuse."
    - Riga piattaforme `space-between`, bordo superiore `#E1DBF2`: **icona + nome** per ciascuna — Windows (4 quadrati), macOS (mela), Linux (pinguino Tux con becco/zampe ambra); tutte icone monocrome `#54524A`, testo Inter 11.5 `#54524A`. (Nessun link GitHub: inutile da mobile.)

11. **Footer** — centrato, padding `22px 20px 26px`. **Rocchetto di filo** SVG (corpo `#EDEAE1`, avvolgimenti viola `#8470D5`, filo che esce). **Barcode** decorativo (`repeating-linear-gradient` di barre ink su chiaro, altezza 32px). Care label mono 9 `#8C8A80`: "100% LOCALE · NO CLOUD · NESSUN ACCOUNT · AGPL 3.0" + "Disegnato da Roberto · cucito da ✦ Claude" (`#AEAB9F`).

---

## Interactions & Behavior
Il prototipo è in gran parte **statico**. Comportamenti previsti da implementare:
- **Hamburger** (`☰`): apre menu navigazione (La collezione / Come funziona / Guida / GitHub) — non presente nel prototipo mobile, da definire (drawer o overlay).
- **Caret palette**: animazione `blink` (opacity 0 al 50%), `1.1s steps(1) infinite` — puramente decorativa.
- **"Mandami il link"** (CTA principale): **funzione backend da realizzare** — l'utente inserisce l'email e riceve il link di download dell'app desktop. Validazione email base; stato di successo/errore da progettare (es. "Fatto, controlla la posta."). Nessun account, nessuna newsletter.
- **Icone piattaforma**: link alle rispettive release (Windows / macOS / Linux) su GitHub Releases.
- **Ribbon "Scopri il debutto →"**: ancora alla sezione stagione/debutto (se presente nella landing completa) o alla release.
- **Hover/active**: bottone primario `filter:brightness(1.08)`; transizioni colore ~.18–.2s verso viola su link/chip.
- **`prefers-reduced-motion`**: disattivare il blink del caret.

## State Management
Minimo. Solo lo stato del form "Mandami il link": `email` (string), `status` (`idle | sending | sent | error`). Nessun data-fetching lato pagina oltre l'invio email.

## Responsive
Design **mobile-first**, canvas 392px. Il contenuto è a colonna singola fluida: usare `max-width` ~420–460px e centrare su schermi più larghi, oppure sviluppare in seguito una variante desktop dedicata (non inclusa in questa consegna). Hit target minimi 44px per bottone/CTA.

## Assets
- **Nessuna immagine esterna.** Palette, mockup, tessuti, spilli, ago, filo, rocchetto, icone OS (Windows/Apple/Tux), barcode e QR sono **HTML/CSS/SVG inline**. Le lenti di ricerca sono SVG inline.
- **Font**: Google Fonts (Newsreader, Inter, JetBrains Mono). In produzione: self-host per performance.
- **Logo**: placeholder "P" serif — sostituire con l'icona ufficiale `{ P }` viola.
- In produzione sostituire il mockup della palette con **screenshot/registrazione reale** dell'app.

## Files
- `Prompt-a-porter Mobile.dc.html` — prototipo (contiene più esplorazioni; **implementare solo l'opzione `#3a`**, dentro la sezione `#t3`).
- `3a-mobile.png` — (se incluso) render di riferimento della schermata 3a.

## Contesto prodotto (dai docs del repo)
App desktop **local-first** (Tauri, Windows/macOS/Linux), single-user; command palette globale `Ctrl+Shift+P`; segnaposti `{{…}}`; versioning git-style; ricerca semantica ONNX MiniLM on-device; varianti A/B + golden test; import composti; CLI `pap` (read-only) + server MCP; vault SQLite cifrato AES-256; auto-update firmato; licenza **AGPL 3.0**. Release cardine v1.0 codename **«Arioso Atelier»**, stagione **Autunno-Inverno 2026**. Repo: `github.com/robertomarchioro/prompt-a-porter`. Landing di produzione: `apps/site` (VitePress).
