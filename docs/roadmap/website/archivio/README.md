# Handoff: Prompt-à-porter — Landing Page

## Overview
Landing page (marketing / product homepage) per **Prompt-à-porter**, un'app desktop *local-first* per gestire la propria collezione di prompt AI. Il concept creativo è **editoriale-fashion** ("prêt-à-porter" → "prompt-à-porter"): i prompt sono capi di un guardaroba, versionabili e cuciti su misura con i segnaposti, richiamabili ovunque con la **command palette `Ctrl+Shift+P`**.

Due varianti nel bundle:
- **`prompt-a-porter-landing.html`** — versione di lancio **v1.0 "Arioso Atelier"** (con nastro di lancio in cima + sezione "Debutto stagione" prima del footer). **Questa è la versione da implementare.**
- **`prompt-a-porter-landing-base-v0.8.html`** — la stessa pagina senza gli elementi di lancio (utile come riferimento della struttura base).

## About the Design Files
I file in questo bundle sono **riferimenti di design realizzati in HTML** — prototipi che mostrano l'aspetto e il comportamento voluti, **non codice di produzione da copiare così com'è**. Il compito è **ricreare questi design nell'ambiente del codebase di destinazione** (React, Vue, Svelte, Astro, ecc.) usando i pattern e le librerie già in uso. Se non esiste ancora un ambiente, scegliere il framework più adatto (per una landing statica, ottimi Astro / Next static / plain HTML+CSS) e implementarli lì. L'HTML è monolitico per comodità di prototipazione: in produzione va scomposto in componenti.

## Fidelity
**Alta fedeltà (hi-fi).** Colori, tipografia, spaziature, ombre e interazioni sono definitivi. Ricreare l'UI in modo pixel-perfect. Uniche eccezioni (placeholder da sostituire con asset reali):
- I **mockup dell'app** dentro il carosello (palette, finestra libreria, feature) sono ricostruzioni in HTML/CSS dell'app reale, utili come riferimento visivo. In produzione conviene sostituirli con **screenshot/registrazioni reali** dell'app o con lo stesso markup rifinito.
- Il **logo mark** è un placeholder tipografico: una "P" serif in un quadrato con gradiente viola. Sostituire con il logo reale se disponibile.

## Layout generale
- **Larghezza**: pagina fluida a piena larghezza (`body { width:100%; max-width:100% }`). I contenuti sono incolonnati con container centrati **max-width 1240px** (`.wrap`, `.topbar`, `.manifesto .in`, `.season .sin`, `.ft-in`) e padding orizzontale 80px.
- **Design width di riferimento**: ~1280–1440px. Breakpoint responsive definiti a 1280 / 1140 / 980 / 720px (vedi § Responsive).
- **Sfondo**: near-black caldo `#0B0B0C`, tema scuro per tutta la pagina.

Ordine verticale delle sezioni (versione di lancio):
1. **Ribbon di lancio** (`.ribbon`) — barra sottile in cima
2. **Topbar** (`.topbar`) — logo + nav + CTA
3. **Hero** (`.stage`) — titolo + sottotitolo + CTA + **carosello showcase** con 6 scene
4. **Striscia "si veste su"** — *nota: rimossa nella versione finale; vedi il manifesto sotto*
5. **Manifesto** (`.manifesto`) — slogan a 2 colonne con immagine palette (light) a destra
6. **Il problema** (`.sec .problem`)
7. **La collezione** (`.grid`) — 5 card "capi" con texture-tessuto
8. **Debutto stagione** (`.season`, solo versione lancio) — Arioso Atelier v1.0
9. **Footer** (`.footer`) — la maison

---

## Design Tokens

Definiti come CSS custom properties in `:root`:

| Token | Hex | Uso |
|---|---|---|
| `--bg` | `#0B0B0C` | sfondo pagina |
| `--bg2` | `#121314` | sfondo sezioni/fasce |
| `--panel` | `#17171A` | pannelli, card, pill |
| `--panel2` | `#1D1D21` | superfici rialzate, kbd |
| `--ink` | `#EFEDF2` | testo primario |
| `--muted` | `#8A8792` | testo secondario |
| `--faint` | `#837F70` | label/hint tenui (contrasto già alzato) |
| `--line` | `#26262B` | bordi, divisori |
| `--violet` | `#8470D5` | **accento brand** (gradiente, mark) |
| `--violet-br` | `#9B86F0` | **azione/enfasi** (link, active, titoli enfatici) |
| `--amber` | `#D8A463` | **segnaposti `{{…}}`**, badge "privato" |
| `--amber-soft` | `#E8C79A` | enfasi calda su testo |
| `--green` | `#6FBF73` | stati "salvato"/verde |

**Regola cromatica fondamentale (rispettarla):** **viola = brand & azione**, **ambra = segnaposti/template**. I token `{{…}}` sono SEMPRE ambra JetBrains Mono su tint ambra (`background: rgba(216,164,99,.12); color: var(--amber)`), mai del colore d'accento della card.

Gradiente brand (mark, bottoni primari, active): `linear-gradient(150deg, #8470D5, #6E56CF)` (150° per il mark, 160° per i bottoni).

### Spaziatura
Scala usata (px): 3, 5, 7, 9, 12, 14, 18, 22, 26, 30, 38, 46, 62, 80, 104, 108. Padding sezioni verticale tipico: 104–108px. Gap griglie: 18px (card), 46–80px (colonne).

### Border radius
- Pill / tondi: `100px`
- Card grandi / finestre: `14–18px`
- Bottoni: `8–10px`
- Pillole piccole / kbd / code: `4–7px`
- Icone quadrate (mark, feature): `8px`

### Ombre
- Card in hover: `0 30px 60px -32px rgba(0,0,0,.75)`
- Palette (cmdk): `0 50px 110px -30px rgba(0,0,0,.85), 0 0 0 1px rgba(132,112,213,.18), 0 0 70px -20px rgba(132,112,213,.4)`
- Finestra app / card lancio: `0 50px 100px -40px rgba(0,0,0,.7)`

### Glow viola (riutilizzato)
`radial-gradient(ellipse … , rgba(132,112,213,.32→.13), transparent)` — usato dietro hero, manifesto e sezione stagione per dare profondità.

---

## Typography

Tre famiglie da Google Fonts (già linkate nell'`<head>`):

```
Newsreader — display serif (opsz 6..72; pesi 300/400/500; italic)
Inter — UI sans (400/500/600)
JetBrains Mono — mono (400/500/700)
```

Ruoli:
- **Newsreader** (`.serif`, weight 300, spesso `<em>` italic per l'enfasi): tutti i titoli editoriali — H1 hero, H2 sezioni, titoli card, slogan, "Arioso Atelier". Letter-spacing negativo (da -.02 a -.04em) sui titoli grandi.
- **Inter**: corpo testo, nav, bottoni, descrizioni.
- **JetBrains Mono** (`.mono`, `.label`, `.kbd`, `code`): label/eyebrow (uppercase, letter-spacing .18–.3em), tasti tastiera, contenuto prompt, token `{{…}}`, dettagli tecnici.

Scala tipografica principale (px):
| Elemento | Font | Size | Weight | Note |
|---|---|---|---|---|
| H1 hero (`h1.head`) | Newsreader | 104 | 300 | line-height .92, `-.04em`; `<em>` corsivo viola |
| Sottotitolo hero (`.sub`) | Newsreader | 26 | 300 | line-height 1.4, colore `#CFCBD8` |
| H2 sezioni | Newsreader | 52 | 300 | `-.02em` |
| Slogan manifesto | Newsreader | 58 | 300 | `<em>` viola |
| Frase tecnica manifesto (`.tech`) | JetBrains Mono | 22 | 400 | colore `--amber-soft`, letter-spacing .04em |
| H2 stagione (`.season h2`) | Newsreader | 82 | 300 | `-.03em` |
| Titolo card (`.card h3`) | Newsreader | 25 | 400 | vira a `--ac` in hover |
| Eyebrow/label | JetBrains Mono | 11–12 | 400/500 | uppercase, letter-spacing .18–.3em |
| Corpo | Inter | 14–17 | 400 | line-height 1.65–1.8 |
| Kicker feature (`.feat .k`) | JetBrains Mono | 11 | — | uppercase, letter-spacing .18em, colore viola |
| Stat value (`.row2 .v`) | Newsreader | 30 | — | viola, `white-space:nowrap` |

---

## Screens / Views (sezioni)

### 1. Ribbon di lancio (`.ribbon`) — solo versione v1.0
- **Scopo**: annunciare la release 1.0.
- **Layout**: barra full-bleed, container 1240px, padding `11px 80px`, flex a 3 elementi con gap 18px.
- **Sfondo**: `linear-gradient(90deg,#17151F,#221C3A 42%,#2A2140 58%,#17151F)`, bordo inferiore `--line`.
- **Contenuto**:
  - `.flag` — pill "Nuova stagione", mono 9.5px uppercase, testo scuro `#15121C` su `--amber-soft`, radius 100px.
  - `.msg` — "**Arioso Atelier** è in negozio: la **prima stagione** di Prompt-à-porter esce dalla beta · Autunno-Inverno 2026 · v1.0". "Arioso Atelier" in Newsreader italic viola (`.code`).
  - `.go` — link "Scopri il debutto →" (ancora `#stagione`), colore `--amber-soft`, hover bianco.

### 2. Topbar (`.topbar`)
- **Layout**: flex space-between, container 1240px, padding `30px 80px`, `z-index:5`.
- **Brand**: `.mk` quadrato 30×30, radius 8, gradiente brand, "P" Newsreader 18px bianca, glow `0 0 24px rgba(132,112,213,.5)`. Accanto `.nm` "Prompt a Porter" Newsreader 19px.
- **Nav** (`.nav`): link "La collezione" (`#collezione`), "Come funziona" (`#come-funziona`), "GitHub" (repo), e CTA "Scarica" (`.dl`, bordo `--line`, radius 8, hover bordo+testo viola). Link mono/inter 13px, colore `--muted`, hover `--violet-br`.

### 3. Hero + Carosello Showcase (`.stage` → `.showcase`)
- **Scopo**: presentare il prodotto e mostrarne le funzioni a rotazione.
- **Stage**: padding `38px 0 80px`, `text-align:center`, glow radiale viola `.spot` in assoluto dietro (z-index 0), contenuto z-index 2.
- **Eyebrow** (`.eyebrow`): pill con `kbd` "⌃ ⇧ P" + testo "richiama la tua collezione, in qualsiasi app".
- **H1**: "Prompt-**à**-porter" (à in corsivo viola).
- **Sub**: claim; poi **hero CTA** (`.hero-cta`): bottone primario "⊞ Scarica per Windows" (gradiente brand), bottone ghost "Vedi su GitHub", link testuale "Altre piattaforme →" (`.alt`, sottolineato). Sotto, meta mono "gratis · local-first · v1.0".
- **Showcase** (`.showcase`, width 1180, margin-top 38): 
  - `.viewport` height **540px**, `position:relative`. Contiene 6 `.scene` assolute sovrapposte; **la scena attiva ha `.on` → `display:flex`, le altre `display:none`** (toggle via `display`, non opacity — vedi § Interazioni per il motivo).
  - **Le 6 scene**:
    1. **La palette** — command palette `.cmdk` (width 660): search bar con lente, caret lampeggiante, "esc"; label "Recenti"; 4 item (titolo + descrizione + badge "privato" ambra); primo item `.active` (fondo viola tint + barra viola sx); footer con hint tasti (esc/↑↓/↵/⌃↵ "compila e incolla").
    2. **La libreria** — finestra app `.appwin` (1180×500): title bar (mark + "Prompt a Porter" + pill versione + controlli); body a 4 colonne (sidebar viste/visibilità/cartelle/tag; lista prompt con "+ Nuovo"; editor con titolo, meta pill, tab, toolbar, codice con token `{{numero_punti}}`/`{{articolo}}`; pannello metadati con segnaposti rilevati/varianti); status bar mono con "Ctrl+Shift+P".
    3. **Compila e incolla** (`.feat` 2 col) — copy a sx (kicker "Feature 01 · Segnaposti", H3, testo, 2 stat) + `.segcard` a dx (form segnaposti con anteprima compilata, footer "⌃↵ compila e incolla").
    4. **Cronologia** (`.feat`) — copy + `.vercard`: timeline versioni (v7 attuale…v1) + diff git-style (righe `.add` verdi / `.del` rosse / `.ctx`) con token ambra.
    5. **Import** (`.feat`) — copy + `.impcard`: 3 chip sorgente (`ruolo-engineer` v4, `stile-conciso` v2, `formato-md` v1) che "cuciono" nel prompt composto.
    6. **Giorno · Sera** (`.feat`) — copy + `.wardrobe`: due mini-palette affiancate (`.mini.day` chiara "Abito da giorno" / `.mini.night` scura "Abito da sera") con toggle ☀/☾ al centro.
  - **Controlli** (`.sc-controls`): freccia `‹` (`#scPrev`), `.tabline` con 6 `.tab` (La palette / La libreria / Compila e incolla / Cronologia / Import / Giorno · Sera; tab `.on` = gradiente brand), freccia `›` (`#scNext`).
  - **Progress bar** (`.sc-progress`): barra 240×2px sotto le tab, riempita da gradiente viola (`i` con width animata). Nascosta con `prefers-reduced-motion`.
  - **Caption** (`.sc-cap` `#cap`): frase che cambia con la scena, `--amber-soft` sul `<b>`.

### 4. Manifesto (`.manifesto`)
- **Scopo**: lo slogan-bomba + la parte tecnica.
- **Sfondo**: `#15151A` (leggermente più chiaro della pagina) + `::before` glow viola a piena larghezza → la fascia si legge come banda edge-to-edge. Bordi top/bottom `--line`.
- **Layout**: `.in` grid 2 col (`1.02fr .98fr`), gap 64, container 1240, padding `78px 80px`, `align-items:center`.
- **Sinistra**: `.slogan` "Se ci puoi scrivere, ci puoi **incollare il tuo prompt**." (Newsreader 58, `<em>` viola); sotto `.tech` (mono 22, ambra-soft, bordo-top): "Si abbina con qualsiasi campo di testo, in ogni editor, applicazione o sito web."
- **Destra** (`.m-art`): **palette in versione LIGHT** (carta avorio, ricerca con bordo viola, testo scuro, badge "privato" ambra), inclinata `rotate(-2deg)`, width 520, su alone viola sfocato (`::before`). ⚠️ Questa istanza di `.cmdk` è tematizzata light localmente — vedi le regole `.manifesto .m-art .cmdk*` nel file.

### 5. Il problema (`.sec .problem`)
- **Layout**: `.prob-head` grid 2 col (`1fr 1.12fr`), gap 80.
- **Sinistra**: H2 "I tuoi prompt valgono. Ma sono in fondo al cassetto." (Newsreader 52).
- **Destra**: paragrafo descrittivo + `.lead` (Newsreader 21) "Prompt a Porter è il tuo guardaroba: … *pronto da indossare* con un tasto." (`<em>` viola).

### 6. La collezione (`.grid`)
- **Scopo**: 5 feature presentate come **capi con tessuti diversi**.
- **Layout**: grid 6 col, gap 18. Card grandi (`.big`, span 3) le prime due; card piccole (`.sm`, span 2) le altre tre.
- **Card** (`.card`): radius 16, bordo `--line`, sfondo `linear-gradient(180deg, var(--tint), var(--bg2) 62%)`; **hover** → translateY(-5px) + ombra + bordo `--ac` + titolo `--ac`. Struttura: `.swatch` (fascia tessuto in alto, con `.hang` gruccia e `.sku` cartellino) + `.body` (h3 + p).
- **5 varianti colore/tessuto** (ogni card ha classe `.cc-*` che ridefinisce `--ac/--sw1/--sw2/--tint` e la texture dello `.swatch`):
  - `.cc-violet` — **velluto** viola (`--ac #9B86F0`), SKU "VER", "Ogni modifica, in vetrina"
  - `.cc-amber` — **tweed di lana** ocra (`--ac #E0B380`), SKU "TPL", "Capi su misura"
  - `.cc-rose` — **lino** terracotta (`--ac #E1998A`), SKU "SRCH", "Cerca al buio dell'armadio"
  - `.cc-sage` — **twill di cotone** salvia (`--ac #93C3AE`), SKU "A/B", "Prova prima di comprare"
  - `.cc-blue` — **seta** blu (`--ac #9CB2E6`), SKU "IMP", "Un tono, dieci tagli"
  - Le texture sono ottenute con `repeating-linear-gradient`/`radial-gradient` su `.swatch` e `.swatch::after` (vedi commenti "tessuti reali" nel CSS). L'armocromia è "stagione gioiello/autunno".

### 7. Debutto stagione (`.season`) — solo versione v1.0
- **Scopo**: lancio narrativo della v1.0 "Arioso Atelier".
- **Sfondo**: glow viola in alto a dx + `--bg`. Bordo top `--line`.
- **Layout**: `.sin` grid 2 col (`.94fr 1.06fr`), gap 72, padding `104px 80px`.
- **Sinistra**: `.plate` "Autunno-Inverno 2026 · Debutto" (mono viola con lineetta); H2 "Arioso *Atelier*" (Newsreader 82); `.verline` pill "v**1.0**" + "subentra a *«Ago e Filo»*, il codename di laboratorio"; `.lede` (Newsreader 22) "Il capo era ancora *in cucitura*. Con la 1.0 l'ago si posa…"; `.scta` bottone "Scarica la 1.0 per Windows" + nota mono.
- **Destra** (`.card-launch`): header "Nella collezione di debutto" + stamp "v1.0 · AI 2026"; 4 righe `.cl-row` (icona quadrata viola-tint + titolo + descrizione) con le promesse della release; footer "Penelope ha finito la tela…" con dot verde. Il token `{{import "x" with k=v}}` nella 3ª riga è stilizzato ambra.

### 8. Footer (`.footer`)
- **Layout**: `.ft-in` container 1240, padding-top 78. `.ft-top` grid 4 col (`1.3fr 1fr 1fr 1fr`), gap 48.
- **Brand**: mark + "Prompt-à-porter" + tagline Newsreader "La tua collezione di prompt AI. *Pronti da indossare.*" + CTA (Scarica/GitHub).
- **3 colonne link**: "La collezione", "Atelier", "La casa" (link a ancore interne e URL GitHub reali: releases, docs, LICENSE, issues).
- **Care label** (`.ft-label`): "Etichetta di manutenzione" + chip mono ambra "100% locale · no cloud · nessun account · lavabile in git · AGPL 3.0".
- **Bottom** (`.ft-bottom`): "© 2026 Prompt-à-porter · Edizione locale · Stagione AW26" + firma **"Disegnato da Roberto · cucito da ✦ Claude"** — "Roberto" link GitHub (`.name`, Newsreader italic), "Claude" in clay `#D97757` con glyph sparkle SVG generico (`.name.claude`). ⚠️ Il glyph è una stella generica, **non** il marchio Anthropic.

---

## Interactions & Behavior

### Carosello (JS in fondo al file)
- **Autoplay** via `setInterval` a step di 100ms (NON `requestAnimationFrame`: in iframe/tab in background rAF si ferma; `setInterval` no). `elapsed` accumula, `prog` width = `elapsed/DUR[i]*100%`; a fine durata → `go(i+1)`.
- **Durate per scena** (`DUR`, ms): `[4200, 5400, 5400, 5400, 5400, 5400]`.
- **Switch scena via `display`**: `paint(n)` toggla la classe `.on` (che fa `display:flex` vs `display:none`). ⚠️ **Non** usare fade in opacity/transition/animation: in questo ambiente le animazioni CSS possono restare congelate al frame 0 lasciando la scena invisibile. Il toggle `display` è la scelta robusta.
- **Caption**: fade rapido (opacity 0→1 con setTimeout 200ms) al cambio scena.
- **Tab** (`.tab[data-go]`): click → `go(n)`. **Frecce** `#scPrev`/`#scNext`: `go(i∓1)` con wraparound modulo 6.
- **Pausa su hover**: `mouseenter` sul `.viewport` → `paused=true`, `mouseleave` → `paused=false` (l'interval continua ma non incrementa `elapsed`).
- **`prefers-reduced-motion: reduce`**: niente autoplay (return in `restart()`), progress bar nascosta via CSS. La navigazione manuale (tab/frecce) resta attiva.
- **Caret lampeggiante**: `@keyframes blink` (opacity 0 a 50%), 1.1s steps(1) infinite — decorativo.

### Hover / stati
- Nav link, footer link, tab, frecce, bottoni: transizione colore/bordo ~.18–.2s verso viola.
- Card collezione: lift + ombra + accento colore su hover (transition .22s).
- Bottone primario: `filter:brightness(1.08)` su hover; ghost/secondari virano a viola.

### Navigazione
Tutte ancore interne (`#collezione`, `#come-funziona`, `#stagione`) + URL esterni al repo GitHub (`/releases`, `/issues`, `/tree/main/docs`, `/blob/main/LICENSE`). Nessun link "#" morto.

---

## State Management
Poco stato, tutto locale al carosello:
- `i` — indice scena corrente (0–5).
- `elapsed` — ms trascorsi nella scena corrente.
- `paused` — bool (hover).
- `tickTimer` — handle setInterval.
- `reduce` — bool da `matchMedia('(prefers-reduced-motion: reduce)')`.

In un framework: gestire come stato del componente Carousel (`activeIndex`, `elapsed`, `paused`), con un timer in `useEffect`/`onMount` che si pulisce in unmount. Rispettare reduced-motion. Nessun data-fetching.

## Responsive
Breakpoint (media query nel file):
- **≤1280px**: `.showcase { zoom:.9 }`
- **≤1140px**: `.showcase { zoom:.78 }`, topbar/wrap padding 48px
- **≤980px**: `.showcase { zoom:.62 }`; `.prob-head`, `.manifesto .in`, `.season .sin` diventano 1 colonna; `.ft-top` 2 col; H1/H2 con `clamp()`
- **≤720px**: `.showcase { zoom:.48 }`; nav nascosta; footer 1 col
- `zoom` è usato per scalare il carosello a larghezza fissa (1180px). In un rebuild pulito, valutare `transform: scale()` con `transform-origin` + wrapper a altezza calcolata, oppure rendere il carosello fluido. `zoom` funziona ovunque tranne Firefox datati.

## Assets
- **Nessuna immagine esterna.** Tutti i mockup (palette, finestra app, feature, tessuti) sono HTML/CSS puro. Le lenti di ricerca sono SVG inline. Le icone tessuto/feature sono glyph Unicode (⊞ ✒ ⊕ ◉ ↓ ☀ ☾ ⌕).
- **Font**: Google Fonts (Newsreader, Inter, JetBrains Mono) via `<link>`. In produzione: self-host o `next/font` per performance.
- **Logo**: placeholder "P" serif — sostituire con asset reale.
- **In produzione** sostituire i mockup dell'app con screenshot/registrazioni reali quando disponibili.

## Files
- `prompt-a-porter-landing.html` — **versione da implementare** (v1.0 "Arioso Atelier", con ribbon + sezione stagione).
- `prompt-a-porter-landing-base-v0.8.html` — variante senza elementi di lancio (riferimento struttura base).
- `screenshots/` — riferimenti visivi renderizzati:
  - `01-full-page.png` — vista in cima (ribbon, topbar, hero, carosello)
  - `02-carosello-libreria.png` — scena "La libreria" (finestra app completa)
  - `03-carosello-compila.png` — scena "Compila e incolla" (segnaposti)
  - `04-carosello-giorno-sera.png` — scena tema chiaro/scuro
  - `05-manifesto.png` — slogan + palette light
  - `06-collezione.png` — le 5 card "capi" con tessuti
  - `07-debutto-stagione.png` — sezione Arioso Atelier v1.0
  - `08-footer.png` — footer "la maison"

Nel progetto originale, altre esplorazioni/varianti scartate vivono in `models/` (non necessarie per l'implementazione).

## Note sul contesto prodotto (dai docs del repo)
- **Arioso Atelier** = codename della **v1.0.0**, prima release stabile della linea "Personale", esce dalla beta; subentra al codename di laboratorio **«Ago e Filo»**. Etichetta stagionale **Autunno-Inverno 2026** (release H2 2026).
- Prodotto: app desktop **local-first** (Windows/Tauri), single-user, ricerca semantica con modello **ONNX MiniLM** embedded (nessun cloud), versioning git-style, segnaposti `{{…}}`, import composti, varianti A/B + golden test, licenza **AGPL 3.0**. Repo: `github.com/robertomarchioro/prompt-a-porter`.
