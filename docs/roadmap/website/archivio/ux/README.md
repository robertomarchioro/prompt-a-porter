# Handoff — Rifiniture UX/estetica landing Prompt-à-porter

## Overview
Set di 5 interventi mirati sulla landing page pubblica (`robertomarchioro.github.io/prompt-a-porter`) per alzare **usabilità, tenuta mobile e accessibilità** senza toccare la direzione artistica, che è già ottima. Non è un redesign: sono fix chirurgici sui componenti e sui token esistenti.

> Escluso dal handoff il punto sulla densità della metafora (era il n.04 dell'audit) — solo copy, gestito a parte.

## About the design files
Il file `Audit UX Prompt-a-Porter.html` in questo bundle è un **riferimento visivo**: mostra ogni problema con un confronto *Prima → Dopo* renderizzato negli stessi token del sito. **Non è codice da copiare.** Il compito è applicare i fix nel codebase reale (Vue 3 + VitePress) seguendo i pattern già presenti.

Il sito vive in `apps/site/.vitepress/`:
- Stili: `theme/styles/landing.css` (tutto scopato sotto `.pap-landing`)
- Componenti: `theme/components/landing/*.vue`
- Logica download/OS: `theme/components/landing/os.ts` e `download.ts`

## Fidelity
**Hi-fi.** Colori, tipografia e spaziature sono definitivi e coincidono con `landing.css`. Riusa i token CSS esistenti, non introdurre nuovi valori se non indicato.

## Design tokens (già in `landing.css`, `:root` di `.pap-landing`)
- Bg: `--bg #0b0b0c` · `--bg2 #121314` · `--panel #17171a` · `--panel2 #1d1d21`
- Testo: `--ink #efedf2` · `--muted #8a8792` · `--faint #837f70` · `--line #26262b`
- Brand/azione (viola): `--violet #8470d5` · `--violet-br #9b86f0`
- Segnaposti/template (ambra): `--amber #d8a463` · `--amber-soft #e8c79a`
- Stato: `--green #6fbf73`
- Font: display `"Newsreader"` (300/400, italic per enfasi) · label/tech `"JetBrains Mono"` · body `"Inter"`
- Regola cromatica da rispettare: **viola = marchio/azione, ambra = segnaposti/template.**

---

## Fix 1 — Mobile: ripensare la vetrina, non scalarla `[Priorità 1 · sforzo alto]`

**Problema.** I mockup app sono larghi 1180px con testo base 12px (dettagli 9–10px). Sotto certe soglie `landing.css` li rimpicciolisce con `zoom`, mantenendo il layout a 4 colonne:
```css
@media (max-width: 1280px) { .pap-landing .showcase { zoom: .9; } }
@media (max-width: 1140px) { .pap-landing .showcase { zoom: .78; } }
@media (max-width: 980px)  { .pap-landing .showcase { zoom: .62; } }
@media (max-width: 720px)  { .pap-landing .showcase { zoom: .48; } }
```
A `.48` il testo effettivo scende a ~5px: illeggibile. La griglia dell'app è `.a-body { grid-template-columns: 204px 250px 1fr 196px; }` — non pensata per reflow.

**Fix.**
- Eliminare le regole `zoom` sulla `.showcase`.
- Sotto **720px** (idealmente già a 900px per il full app window): non mostrare l'app 4-colonne. Mostrare **una feature per volta** con un componente ridisegnato in verticale (single-column), font ≥ 13px, tap target ≥ 44px.
- Il carosello (`ShowcaseCarousel.vue`) su mobile diventa uno swipe orizzontale di card leggibili; le frecce e i tab restano ma le scene sono le versioni mobile-first, non le desktop scalate.
- Riferimento del target mobile: pannello "Dopo" del Fix 1 nel file audit (card singola: titolo Newsreader, campi segnaposto impilati, CTA `⌃↵ Compila e incolla` a piena larghezza).

**File coinvolti:** `landing.css` (media query + nuove classi mobile), `ShowcaseCarousel.vue`, i componenti scene in `theme/components/landing/scenes/`, `HeroStage.vue`.

**Accettazione:** su iPhone/Android (≤414px) ogni scena è leggibile senza zoom del browser; nessun testo < 13px; nessuna colonna orizzontale che va in overflow.

---

## Fix 2 — Hero: problema in una riga + una sola CTA `[Priorità 2 · sforzo medio]`

**Problema.** Above-the-fold sovraccarico: `LaunchRibbon` + `TopBar` + eyebrow (`⌘⇧P`) + `h1.head` 104px + `.sub` + **tre** `.hero-cta .btn` + `.hero-meta`, poi subito `.showcase` alta 540px auto-rotante. La frase-dolore ("I tuoi prompt valgono. Ma sono in fondo al cassetto") sta in `ProblemSection.vue`, molto più in basso.

**Fix.**
- Portare **il problema in una riga dentro l'hero** (sopra o al posto del sottotitolo attuale). Es. sottotitolo: "Libreria locale di prompt AI: cerca, compila i campi, incolla dove stai scrivendo."
- Ridurre l'hero a **una CTA primaria** (vedi Fix 3); gli altri inviti diventano link testuali sotto.
- Nell'hero mostrare **un solo artefatto** (la command palette `CmdkPalette.vue`) invece dell'intera vetrina auto-rotante; la vetrina completa scorre più in basso.

**File coinvolti:** `HeroStage.vue`, `Landing.vue` (ordine sezioni), `landing.css` (`.stage`, `.hero-cta`).

**Accettazione:** su desktop 1440px l'hero comunica *cosa fa il prodotto* + una CTA chiara senza scroll; la vetrina completa non è più il primo elemento auto-animato.

---

## Fix 3 — CTA con rilevamento OS `[Priorità 3 · sforzo basso]`

**Problema.** Cinque inviti a peso quasi pari: hero "Scarica l'app" / "Vedi su GitHub" / "Altre piattaforme →", più "Scarica" in `TopBar` e "Scopri il debutto" nel ribbon. Il visitatore deve scegliere da sé la build.

**Fix.**
- CTA primaria che **rileva l'OS** e propone il file giusto: es. "↓ Scarica per macOS · .dmg Apple Silicon". La logica esiste già in `os.ts` / `download.ts` — collegarla al bottone primario.
- Piattaforme secondarie ("Windows / Linux") e "Vedi su GitHub" come link testuali `.alt` sotto la CTA, non bottoni pari peso.
- In `TopBar.vue` un solo "Scarica" (che punta alla stessa azione/ancora), non due percorsi paralleli.

**File coinvolti:** `HeroStage.vue`, `TopBar.vue`, `os.ts`, `download.ts`, `landing.css` (`.hero-cta`).

**Accettazione:** aprendo il sito da macOS/Windows/Linux il bottone primario mostra la build corretta; il fallback "altre piattaforme" resta a un click.

---

## Fix 4 — Contrasto e dimensioni minime del testo `[Priorità 4 · sforzo basso · A11y]`

**Problema.** Molte etichette mono girano a 9–10px nel colore `--faint #837f70` su fondo scuro → contrasto sotto WCAG AA (4.5:1) e sotto il minimo di leggibilità. Esempi in `landing.css`: `.sb .grp` (9px), `.lst .row .tm` (9.5px), `.a-status` (9.5px), `.mt .grp .h` (9px), vari `.imp-*`/`.cl-foot` a 9.5–10px in `--faint`.

**Fix.**
- Alzare il **minimo delle etichette/didascalie testuali a 11px**.
- Dove `--faint` porta testo (non pura decorazione), schiarirlo verso `--muted` o un nuovo `--faint-txt ≈ #a7a3b0` così da superare AA. Mantenere `--faint` solo per elementi decorativi/non informativi.
- Passare in rassegna ogni coppia testo/fondo con un contrast checker fino ad AA.

**File coinvolti:** `landing.css` (definizione token + i selettori elencati sopra).

**Accettazione:** nessun testo informativo < 11px; ogni coppia testo/fondo ≥ 4.5:1 (o 3:1 per testo ≥ ~19px bold).

---

## Fix 5 — Pausa carosello + hamburger mobile `[Priorità 5 · sforzo basso · 2 in 1]`

**Problema A — carosello auto-avanzante.** `.sc-progress i` avanza da solo. Su contenuti densi è un anti-pattern: la scena cambia mentre si legge. Ci sono frecce/tab, ma manca **pausa** e indicazione "dove sono".

**Fix A.**
- Aggiungere un controllo **pausa/play** accanto alle frecce (`.sc-controls`), un contatore "n/6" e puntini di posizione.
- **Fermare l'auto-avanzamento** al primo hover/focus/tocco (poi non ripartire, o ripartire solo su play esplicito).
- Rispettare già `prefers-reduced-motion` (oggi nasconde solo la progress bar): in reduced-motion l'auto-avanzamento va disattivato del tutto.

**File coinvolti:** `ShowcaseCarousel.vue`, `landing.css` (`.sc-controls`, `.sc-progress`).

**Problema B — nav che sparisce su mobile.** In `landing.css`: `@media (max-width: 720px) { .pap-landing .nav { display: none; } }` senza sostituto → si perde l'accesso alle sezioni.

**Fix B.**
- Aggiungere un **menu hamburger** in `TopBar.vue` che sotto 720px apre le stesse voci della nav ("La collezione", "Come funziona", "GitHub", "Scarica").

**File coinvolti:** `TopBar.vue`, `landing.css` (`.topbar`, `.nav`).

**Accettazione:** su mobile le voci di navigazione sono raggiungibili; il carosello non cambia scena da solo mentre l'utente interagisce; contatore e stato play/pausa visibili.

---

## Interazioni & comportamento (riassunto)
- Carosello: default in play su desktop non-reduced-motion; pausa su interazione; frecce + tab + dots sincronizzati; loop.
- CTA OS-aware: rileva a runtime (client), fallback a pagina releases per OS non riconosciuti.
- Hamburger: toggle open/close, chiude su selezione voce e su Esc; focus trap opzionale.
- Nessuna nuova animazione decorativa; conservare caret lampeggiante e ombre multilivello esistenti.

## Responsive — breakpoint di riferimento (già usati nel file)
1280 / 1140 / 980 / 720 px. Rimuovere l'approccio `zoom`; passare a layout mobile-first sui componenti densi (Fix 1).

## Assets
Nessun asset nuovo. Icona brand: `icons/icon-64.png` (già nel repo). Font da Google Fonts già caricati dal tema.

## Files in questo bundle
- `README.md` — questo documento.
- `Audit UX Prompt-a-Porter.html` — riferimento visivo Prima/Dopo per i 5 fix (il n.04 "metafora" è nel file ma **fuori scope** per questo handoff).

## Note
- Non spedire l'HTML dell'audit in produzione: è solo riferimento.
- Rispettare la regola cromatica viola/ambra in ogni nuovo elemento.
- Tutti i riferimenti a codice (`zoom`, `--faint`, `os.ts`, selettori) provengono da `landing.css` e dai componenti pubblici del repository.
