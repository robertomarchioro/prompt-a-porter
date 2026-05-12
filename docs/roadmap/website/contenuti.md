# Contenuti landing page — Handoff a Claude Design

> **Destinatario**: Claude Design (o designer umano) per la fase di ideazione grafica.
> **Obiettivo**: fornire tutto il copy, struttura, audience e CTA necessari per progettare la landing senza dover ri-chiedere.

## Frame

**Cosa promuoviamo**: PaP **Personale** (single user, local-first). Niente teaser Enterprise — quando v2.0 esisterà, faremo refresh della landing.

**Cosa NON promettere**:
- Multi-utente / workspace team
- SSO / E2E encryption
- Web app / Browser extension
- Sync server obbligatorio
- API pubblica
- Approval workflow / RBAC

Tutto quanto sopra è v2.0, gate domanda-driven. Non va comunicato come "in arrivo" perché non ha data.

## Lingua

**Solo italiano** in v1 della landing. EN traduzione futura quando il progetto cresce. Tutto il copy qui sotto è già in IT pronto da usare.

## Audience target

**Primaria**:
- Professionisti che lavorano quotidianamente con LLM (Claude, GPT, Gemini) e accumulano prompt sparsi tra appunti, note, screenshot.
- Sviluppatori che usano AI in workflow (code review, documentazione, debugging) e vogliono trattare i prompt come asset versionabili.
- Creatori di contenuti (copywriter, marketer, formatori) che riutilizzano template di prompt con variazioni.

**Secondaria**:
- Power user di Notion/Obsidian che cercano un'alternativa specializzata per i prompt.
- Curiosi del movimento "prompt engineering" che vogliono andare oltre il prompt-singolo.

**Non target**:
- Aziende/team (audience v2.0).
- Utenti casuali che usano ChatGPT 1 volta a settimana.

## Brand identity (riferimenti)

L'applicazione usa già un design system maturo. La landing dovrebbe ereditare il **tono visivo** del client desktop per coerenza:

- **Palette**: 3 toni neutri configurabili (`zinc`, `slate`, `stone`). Default `zinc`. Accento "team" (blu intenso) per CTA primari.
- **Tipografia**: Inter (UI) + JetBrains Mono (codice/snippet).
- **Tono**: tecnico ma accessibile. Mai "magico" (no "rivoluziona la tua produttività", no "potenzia con l'AI"). Mai marketing-speak gonfio.
- **Voce**: italiano corretto, registro medio, no anglismi gratuiti ("workflow" sì, "use case" sì; "leverage" no, "skill set" no).

## Struttura proposta (8 sezioni)

### 1. Hero

**Titolo (h1)**: Prompt a Porter
**Sottotitolo**: La tua libreria di prompt AI, sul tuo computer. Open source.
**Pitch breve (1 frase)**: Smetti di perdere i prompt nei file di testo. Versionali, falli evolvere, riusali con i segnaposti — tutto in locale, niente cloud obbligatorio.

**CTA primario**: `Scarica per Windows` (link diretto all'asset portable .zip Latest release)
**CTA secondario**: `Vedi su GitHub` (link al repo)

**Visual**: screenshot del client (Libreria + DetailPane + un prompt visibile con segnaposti evidenziati) o mockup pulito.

### 2. Il problema

**Titolo**: I tuoi prompt valgono. Ma probabilmente sono ovunque.
**Body**:
> File di testo sparsi. Note in app diverse. Screenshot di conversazioni. Prompt copiati e incollati che ogni volta vanno adattati a mano. Quando trovi quello buono, finisce nel buco nero degli appunti.
>
> Prompt a Porter è la tua libreria personale: prompts come asset di prima classe, non come testo casuale.

### 3. Le capability (4-5 card)

Ogni card ha icona, titolo, 1-2 righe di descrizione.

**Card 1 — Versiona ogni modifica**
Ogni save crea una versione. Diff stile git. Torna indietro quando l'ultimo "miglioramento" ha rotto tutto.

**Card 2 — Segnaposti riutilizzabili**
`{{nome}}` per variabili ad-hoc, `{{globale autore}}` per valori che vivono nel vault. Il prompt diventa template.

**Card 3 — Ricerca semantica locale**
Cerca per significato, non solo per parola. Modello ONNX MiniLM embedded — niente API, niente cloud, niente quote.

**Card 4 — Varianti e A/B test**
Crea variazioni dello stesso prompt, confrontale fianco a fianco, scegli la migliore con i golden test (regression testing sui prompt).

**Card 5 — Prompt componibili**
`{{import "altro-prompt"}}` per riusare blocchi. Definisci una volta il tono, importalo in 10 prompt diversi.

### 4. Locale, davvero

**Titolo**: I dati restano sul tuo computer.
**Body**:
> Vault SQLite cifrato AES-256 con master password tua. Modello AI eseguito on-device. Sync server è **opzionale** — se non lo configuri, PaP non parla con internet.
>
> Open source AGPL 3.0: chiunque può ispezionare il codice. La privacy non è una feature opt-in, è il default.

Visual: schema architettura semplificato (cassaforte locale + cervello on-device + freccia sbarrata verso cloud).

### 5. Per chi è (3 persone tipo)

Mini-card con foto-style illustrazione + scenario in 2 frasi:

**Lo sviluppatore**: usa Claude/GPT 30 volte al giorno per code review, refactor, documentazione. Vuole template ripetibili e versionabili come il codice.

**Il copywriter**: ha 50 prompt per email marketing, brief creativi, social. Ogni cliente è una variante. Vuole non riscriverli da zero ogni volta.

**Il ricercatore**: prepara prompt sperimentali per i suoi paper. Vuole confrontare 3 versioni dello stesso prompt e capire quale dà output migliori.

### 6. Differenziatori (vs alternative)

**Titolo**: Cosa lo rende diverso da Notion / un file Markdown / un altro tool.

| | PaP | Notion | File .md | Tool cloud |
|---|---|---|---|---|
| Locale-first | ✅ | ❌ | ✅ | ❌ |
| Cifratura vault | ✅ | ❌ | ❌ | varia |
| Versionamento auto | ✅ | git su .md | git | varia |
| Segnaposti compilabili | ✅ | ❌ | ❌ | ✅ |
| Ricerca semantica | ✅ on-device | cloud | grep | cloud |
| Regression testing | ✅ | ❌ | ❌ | varia |
| Open source AGPL | ✅ | ❌ | N/A | varia |
| Zero costo continuativo | ✅ | freemium | ✅ | abbon. |

### 7. Installazione

**Titolo**: Inizia in 2 minuti.
**Sotto-sezione**: Windows
1. Scarica `Prompt-a-Porter-portable-windows-x64-vX.Y.Z.zip` dalla release Latest.
2. Estrai dove vuoi.
3. Esegui `Prompt-a-Porter.exe`. Niente installazione, niente UAC.

**Sotto-sezione**: Linux / macOS
Build automatiche disponibili dalla [pagina release GitHub](https://github.com/robertomarchioro/prompt-a-porter/releases/latest).

**Box informativo**: Auto-update silenzioso in arrivo con v1.0 (firmato Authenticode).

### 8. Open source

**Titolo**: Codice aperto. Roadmap pubblica. Issue aperte.
**Body**:
> PaP è open source con licenza AGPL 3.0. Codice su GitHub: contribuire è semplice, le issue documentano lo stato del progetto, la roadmap è pubblica.
>
> Niente telemetria. Niente account obbligatorio. Niente dark pattern.

**CTA**: `⭐ Vedi su GitHub` + link.

### 9. Footer

- Link: GitHub | Issue tracker | License (AGPL 3.0) | CHANGELOG
- Copyright: Roberto Marchioro · {anno}
- Nessun "Privacy Policy" complesso perché non raccogliamo dati — solo un disclaimer breve: "Questa pagina usa Matomo self-hosted per analytics anonimizzati. Nessun cookie di terze parti. Nessun dato condiviso."

## Tono ed esempi di copy da NON usare

❌ "Rivoluziona il tuo modo di lavorare con l'AI"
❌ "Sblocca la produttività"
❌ "Sfrutta la potenza dei Large Language Models"
❌ "Powered by AI" come tag generico
❌ "Easy to use" generico — sostituire con esempio concreto

✅ "Versiona, confronta, riusa. Localmente."
✅ "I tuoi prompt restano sul tuo disco."
✅ "Vault cifrato AES-256, modello ONNX on-device."
✅ "Codice aperto. Nessun account."

## Asset richiesti a Claude Design

L'handoff a Claude Design deve produrre:

1. **Mockup high-fidelity** per le 8 sezioni sopra (desktop 1440px + mobile 375px).
2. **Componenti riutilizzabili**: card capability, card persona, tabella confronto, hero, CTA button (primario + secondario).
3. **Palette tonale** scelta (zinc/slate/stone) + variazioni dark/light.
4. **Tipografia** definita (sizes h1-h6, body, mono).
5. **Screenshot del prodotto** preparati: Libreria con prompt visibili, DetailPane con segnaposti, vista Confronto varianti, Diagnosi linting.
6. **Icona/logo** della pagina (può riusare l'icona esistente del client `Prompt-a-Porter.exe`).
7. **Favicon** + `apple-touch-icon` 180×180.
8. **Meta OG image** 1200×630 per anteprime social.

## Tracciamento progetto

Quando l'agente parallelo riceve i mockup da Claude Design, apre PR sul branch `feat/website-vN`. Il PR template include:
- [ ] Mockup desktop applicati
- [ ] Mockup mobile applicati
- [ ] Copy IT verificato
- [ ] Screenshot prodotto aggiornati alla release Latest
- [ ] Build statico verde
- [ ] Lighthouse score ≥ 95 in Performance/Accessibility/SEO
- [ ] Matomo integrato e tracking verificato in staging

## Manutenzione di questo documento

- Quando v1.0.0 si avvicina (M3-M4 di v1.0 chiusi): refresh sezione "Le capability" perché nuove feature sono visibili.
- Quando un differenziatore emerge nei feedback utenti: aggiungere a §6.
- Quando il copy attuale si rivela non funzionante (es. bounce rate alto su una sezione tracciato da Matomo): iterare.
