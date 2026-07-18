# Handoff: Prompt-à-porter — Landing **Desktop** (concept "Scontrino cucito", opzione 4a)

## Overview
Versione **desktop** della landing di Prompt-à-porter (app desktop *local-first* per gestire una collezione di prompt AI). Stessa direzione creativa della versione mobile 3a — concept **"Scontrino cucito"** su palette **Cloud Dancer** (Pantone Color of the Year 2026, bianco caldo/freddo bilanciato) — riorganizzata per viewport largo.

Larghezza di riferimento **1280px**, contenuti incolonnati con container centrato **max-width 1080px** e padding orizzontale **48px**. Concept portante: la pagina è un **atelier** editoriale; lo **scontrino** (con le feature di collezione) diventa una **colonna laterale**, cucita al resto da **filo** e **spilli**.

Differenza chiave vs. la versione mobile: qui l'utente **è già al computer**, quindi la CTA di download è legittima — **"Scarica l'app"** con tripletta Windows / macOS / Linux (nel mobile era invece "Mandami il link"). La regola resta: **viola = brand & azione**, **ambra = segnaposti/template** (`{{…}}` sempre ambra).

## About the Design Files
Il file in questo bundle è un **riferimento di design realizzato in HTML** — un prototipo che mostra aspetto e comportamento voluti, **non codice di produzione da copiare così com'è**. Il compito è **ricreare questo design nell'ambiente del codebase di destinazione** (la landing reale vive in `apps/site`, VitePress/Vue) usando pattern e librerie già in uso; se manca un ambiente adatto, scegliere il framework più appropriato per una landing statica (Astro / VitePress / plain HTML+CSS).

⚠️ **Note tecniche sul file** (identiche al handoff mobile):
- Il prototipo è un "Design Component" (`.dc.html`) su un runtime di prototipazione (`support.js`): **ignorarlo**, servono solo markup e stili inline come riferimento visivo.
- I token letterali `{{nome}}`, `{{globale autore}}`, `{{import "ruolo"}}` sono scritti come `<i style="font-style:inherit">{</i>{nome}}` per aggirare il parser del runtime. Nel codice reale scrivere semplicemente `{{nome}}` (testo normale) nella pill ambra.
- Il file contiene più esplorazioni; **la schermata desktop da implementare è `#4a`** (sezione `#t4`). Gli altri turni sono mobile o scartati.

## Fidelity
**Alta fedeltà (hi-fi).** Colori, tipografia, spaziature, ombre e composizione sono definitivi. Ricreare pixel-perfect. Eccezioni (placeholder da sostituire con asset reali):
- Il **mockup della command palette** sopra il "documento" (hero) e la **palette light** del manifesto sono ricostruzioni HTML/CSS: in produzione sostituibili con screenshot/registrazioni reali dell'app.
- Il **logo mark** ("P" serif su quadrato viola) è un placeholder: usare l'icona ufficiale `{ P }`.

---

## Design Tokens
Identici alla versione mobile (palette Cloud Dancer + brand). Riepilogo:

### Colori — Cloud Dancer (chiara)
`#F1F0EC` pagina · `#FBFBF9` superficie · `#FFFFFF` bianco (palette/input) · `#26251F` ink · `#54524A` sub · `#8C8A80` muted · `#AEAB9F` faint · `#E2E0D8` linea · `#ECEAE2`/`#E7E5DC` linee tenui · `#D8D6CC` tratteggi/perforazioni.

### Colori — brand (invarianti)
Viola azione `#6E56CF` · viola 2 `#8470D5` · viola profondo `#5A45BE` · viola chiaro (su scuro) `#B9A7F0` · crema enfasi `#F0DFC0` · tint `rgba(132,112,213,.10)`.
**Ambra segnaposti**: testo `#A36A2E` su bg `#F3E9D6`, bordo `#E9DCC1`. Ambra flag `#E8C79A`, filo/becco `#C89A55`/`#D8A463`. Verde "incl." `#6FA07A`.
Gradiente brand bottoni: `linear-gradient(160deg,#8470D5,#6E56CF)`. Ribbon: `linear-gradient(100deg,#8470D5,#6E56CF 50%,#5A45BE)`. Banda download: `linear-gradient(165deg,#F4F1FC,#EDE9F7 60%,#F3EEE6)` bordo `#E1DBF2`.

### Tipografia
Newsreader (titoli, 300/400, `<em>` italic per enfasi) · Inter (400/500/600, corpo/UI) · JetBrains Mono (label/eyebrow uppercase, token, dettagli).

Scala (desktop, px):
| Elemento | Font | Size | Weight | Note |
|---|---|---|---|---|
| H1 hero | Newsreader | 98 | 300 | line-height .92, `-.04em`; `à` italic viola |
| Sub hero | Newsreader | 25 | 300 | max 600px |
| Manifesto | Newsreader | 56 | 300 | `-.025em`; `<em>` viola |
| H2 sezioni ("La collezione è tua", "Come provare un abito") | Newsreader | 44–50 | 300 | `-.02/-.03em` |
| Header scontrino | Newsreader | 24 | 400 | |
| Totale "gratis" | Newsreader italic | 28 | 400 | viola |
| Card servizi (titolo) | Newsreader | 19 | 400 | |
| Voci scontrino | Inter | 13 (titolo 600) / 10.5 (desc) | | |
| Corpo | Inter | 14.5 | 400 | line-height 1.65 |
| Nav / link footer | Inter | 13.5 | 400/500 | |
| Eyebrow / label / meta | JetBrains Mono | 9–12 | 400/700 | uppercase, letter-spacing .18–.3em |

### Spaziatura, raggi, ombre
- Container: max-width **1080–1180px**, padding orizzontale **48px**. Padding verticale sezioni **56–80px**; hero **42px**; manifesto band **70px**.
- Raggi: card grande pagina **16px**; mockup/card **13–20px**; **scontrino 4px**; pill **100px**; bottoni **11px**; chip **5px**.
- Ombre: pagina `0 60px 120px -50px rgba(90,88,78,.5)`; palette hero `0 40px 70px -26px rgba(90,88,78,.6)` + `0 0 0 1px rgba(132,112,213,.12)`; scontrino `0 30px 60px -34px rgba(90,88,78,.5)`; bottone `0 16px 34px -14px rgba(132,112,213,.6)`; spilli `drop-shadow(0 4px 5px rgba(60,50,30,.35))`.

---

## Screens / Views

### Schermata unica: Landing desktop "Scontrino cucito" (`#4a`), 1280px
Ordine verticale dei blocchi:

1. **Ribbon di lancio** — full-bleed, gradiente viola `linear-gradient(100deg,#8470D5,#6E56CF 50%,#5A45BE)` + righe diagonali `repeating-linear-gradient(115deg,rgba(255,255,255,.08) 0 2px,transparent 2px 24px)`. Container 1180px, padding `11px 48px`, flex a 3: flag ambra "✦ Nuova stagione" · messaggio "Debutta *Arioso Atelier* · … · Autunno-Inverno 2026 · v1.0 · codice `Cloud Dancer 11-4201`" (Inter 13 `#E4DEFA`, codename/em crema) · link "Scopri il debutto →".

2. **Topbar** — container 1180px, padding `26px 48px`, flex space-between. Brand (mark 30×30 gradiente + "Prompt a Porter" Newsreader 19). Nav: "La collezione", "I servizi", "Come funziona", "Guida" (Inter 13.5 `#54524A`) + bottone "Scarica" (bordo `#E2E0D8`, bg `#FBFBF9`, radius 9). Hover link → viola.

3. **Hero** — centrato, glow radiale viola dietro. Eyebrow pill kbd "Ctrl + Shift + P" + "richiama la tua collezione, in qualsiasi app". H1 "Prompt-*à*-porter" (98px). Sub (Newsreader 25). CTA: primary "↓ Scarica l'app" (gradiente brand) + ghost "Vedi su GitHub". Riga piattaforme: **icona + nome** Windows (4 quadrati) / macOS (mela) / Linux (Tux, becco+zampe ambra), monocrome `#54524A`, + meta "gratis · local-first · v1.0".

4. **Palette in evidenza (gesto à-porter)** — container max 1080, radius 20. Sfondo "documento" `linear-gradient(160deg,#E7E5DE,#D9D7CE)` con barra finestra (3 pallini + "documento · qualsiasi editor") e 3 righe grigie. Sopra, **command palette** bianca `position:absolute` centrata in basso (width 640): search (lente + "Cerca prompt, tag o azione…" + caret viola lampeggiante + kbd "esc"), label "Recenti", 3 item (1° attivo con barra viola + badge "privato" ambra: "Riassumi articolo in N punti", "Code review strutturata" con `<em>engineer</em>`, "Email professionale parametrica"), footer hint ("esc chiudi · ↑↓ naviga · ↵ seleziona · ⌃↵ compila e incolla", l'ultimo viola). Caption: "FUNZIONE IN EVIDENZA · LA PALETTE — premi ⌃⇧P ovunque".

5. **Manifesto** — banda full-bleed `#EDEBF6`, bordi top/bottom `#E2E0D8`, alone viola. Grid 2 col (1.02fr / .98fr) max 1080. Sinistra: kicker "— manifesto —", slogan "Se ci puoi scrivere, ci puoi *incollare il tuo prompt.*" (Newsreader 56, `<em>` viola), riga tecnica mono con bordo-top. Destra: **palette light** (bianca, tint ambra sull'attivo) inclinata `rotate(-2deg)` su alone viola sfocato, 2 item ("Brainstorm con criteri", "Traduzione con glossario").

6. **Collezione (banda a 2 colonne)** — max 1080, padding `80px 48px 40px`, grid `400px / 1fr`, gap 56, align start.
   - **Sinistra — scontrino come colonna** (bg `#FBFBF9`, bordo `#E2E0D8`, radius 4), retto da **2 spilli** SVG agli angoli (testa viola sx, ambra dx). Header "La collezione · scontrino" + "SEI CAPI · AW26 · Nº 4A-001". Intestazione colonne "CAPO / PREZZO". **6 voci** (feature reali): Ogni modifica in vetrina / Capi su misura (`{{nome}}` · `{{globale autore}}`) / Cerca al buio dell'armadio / Prova prima di comprare / Un tono, dieci tagli (`{{import "ruolo"}}`) / Il taglio da terminale (`pap`), ognuna con "incl." verde. **Totale** con bordo-top ink + "*gratis*" (Newsreader italic 28 viola). **Dentellatura** inferiore (doppio radial-gradient `#D8D6CC`/`#F1F0EC`, `background-size:20px 20px`, repeat-x, h 20px).
   - **Destra — I servizi dell'atelier**: kicker mono viola, H2 "La collezione è tua. L'atelier *la tiene in forma.*" (Newsreader 44), paragrafo di raccordo, **grid 2×2** di card sobrie (bg `#FBFBF9`, bordo, radius 13): **Ritocco** (✦), **Diagnosi** (◉), **CLI `pap`** (›_), **MCP** (⊕) — icona in quadrato tint viola + titolo Newsreader 19 + descrizione Inter 12. Sotto, **filo** SVG (punto-filza ambra `dasharray 7 6` + ago).

7. **Clienti tipo** — max 1080, label "A CHI STA BENE PROMPT A PORTER" centrata; riga di **3 cartellini appesi** (bg `#FBFBF9`, radius `4px 4px 9px 9px`, foro in alto, leggermente ruotati): **Lo sviluppatore** / **Il copywriter** (`email_v3_FINAL` mono ambra) / **Il ricercatore**, una descrizione ciascuno (Inter 12.5, centrata).

8. **Banda download** — bg `linear-gradient(165deg,#F4F1FC,#EDE9F7 60%,#F3EEE6)`, bordo-top `#E1DBF2`, alone viola. Grid 2 col (1.1fr / .9fr). Sinistra: kicker "IN DUE MINUTI", H2 "Come provare *un abito.*" (Newsreader 50), paragrafo (installer firmato / `.dmg` universale / `.deb`-AppImage / auto-update), CTA "↓ Scarica la 1.0" + meta, riga piattaforme con badge **"universale"** ambra accanto a macOS. Destra: **mini care-ticket spillato** (spillo viola centrale, card ruotata `1.5deg`) con "ETICHETTA DI MANUTENZIONE" e 5 righe pallino-ambra: 100% locale / no cloud / nessun account / lavabile in git / AGPL 3.0.

9. **Footer** — bg `#FBFBF9`, bordo-top. Grid 4 col (1.4fr/1fr/1fr/1fr): brand + tagline Newsreader; colonne link "La collezione" / "Atelier" / "La casa" (Inter 13.5, hover viola). Riga bottom con bordo-top: **rocchetto di filo** SVG + **barcode** (repeating-linear-gradient) a sinistra; a destra "© 2026 Prompt-à-porter · Edizione locale · Stagione AW26" + firma "Disegnato da *Roberto* · cucito da ✦ Claude" ("Claude" in clay `#D97757`).

---

## Interactions & Behavior
Prototipo prevalentemente statico. Comportamenti da implementare:
- **Nav** con ancore alle sezioni (`#collezione`, `#servizi`, `#come-funziona`) + link esterni (Guida, GitHub).
- **CTA "Scarica"/"Scarica l'app"/"Scarica la 1.0"**: rilevamento OS se fattibile (Windows → installer, macOS → `.dmg` universale, Linux → `.deb`/AppImage), fallback alla release Latest; le icone-piattaforma linkano ai rispettivi artefatti.
- **Caret palette**: `@keyframes blink` (opacity 0 al 50%), 1.1s steps(1) infinite — decorativa; disattivare con `prefers-reduced-motion`.
- **Hover**: bottone primario `filter:brightness(1.08)`; link/nav/chip transizione colore ~.18–.2s verso viola; card servizi lift/ombra opzionale.
- Opzionale (non nel prototipo): rendere la palette hero un **mini-carosello** con più scene reali (Ritocco a diff, Test Golden).

## State Management
Minimo. Nessun data-fetching di pagina. Eventuale rilevamento OS per la CTA (stringa `platform`).

## Responsive
Progettato a **1280px**. Breakpoint suggeriti nel rebuild:
- ≤1080px: ridurre padding a 32–40px; hero H1 con `clamp()`.
- ≤900px: le griglie a 2 colonne (manifesto, collezione, download) diventano 1 colonna; lo scontrino torna a larghezza piena sopra i servizi; footer 2 colonne.
- ≤680px: **collassare nella versione mobile 3a** (vedi handoff mobile), non comprimere il desktop. Servizi 1 colonna, clienti-tag in colonna.

## Assets
Nessuna immagine esterna: palette, mockup, tessuti, spilli, ago, filo, rocchetto, icone OS (Windows/Apple/Tux), barcode sono **HTML/CSS/SVG inline**. Font: Google Fonts (Newsreader, Inter, JetBrains Mono) — self-host in produzione. Logo: placeholder "P" → icona ufficiale `{ P }`. Sostituire i mockup app con screenshot reali quando disponibili.

## Files
- `Prompt-a-porter Mobile.dc.html` — prototipo (implementare **solo `#4a`**, sezione `#t4`, per il desktop; `#3a` per il mobile).
- `4a-desktop.png` — render di riferimento della schermata desktop.
- Vedi anche il handoff mobile (`design_handoff_mobile_landing/`) per la variante 392px: **stesso design system**, due layout della stessa direzione.

## Contesto prodotto
App desktop **local-first** (Tauri, Win/macOS/Linux); command palette globale `Ctrl+Shift+P`; segnaposti `{{…}}`; versioning git-style; ricerca semantica ONNX MiniLM on-device; varianti A/B + golden test; import composti; servizi: Ritocco (AI a diff), Diagnosi (linter), CLI `pap` (read-only), server MCP; vault SQLite cifrato AES-256; auto-update firmato; AGPL 3.0. Release v1.0 codename **«Arioso Atelier»**, stagione **Autunno-Inverno 2026**. Repo: `github.com/robertomarchioro/prompt-a-porter`. Landing di produzione: `apps/site` (VitePress).
