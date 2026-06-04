# Contenuti landing page — Handoff a Claude Design

> **Destinatario**: Claude Design (o designer umano) per la fase di ideazione grafica.
> **Obiettivo**: fornire tutto il copy, struttura, audience e CTA necessari per progettare la landing senza dover ri-chiedere.
> **Nome del prodotto**: Prompt-a-porter

## Frame

**Cosa promuoviamo**: versione di Prompt-a-porter **Personale** (single user, local-first). Niente teaser Enterprise — quando v2.0 esisterà, faremo refresh della landing.

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
- Appassionati che iniziano a usare più sitemi LLM per passione, divertimento o studio

**Secondaria**:
- Power user di Notion/Obsidian che cercano un'alternativa specializzata per i prompt.
- Curiosi del movimento "prompt engineering" che vogliono andare oltre il prompt-singolo.

**Non target**:
- Aziende/team (audience v2.0).
- Utenti casuali che usano ChatGPT 1 volta a settimana.

## Brand identity (riferimenti)

Il nome "Prompt a Porter" è un omaggio dichiarato al **prêt-à-porter**: prompt pronti da indossare, cuciti su misura. La landing deve **onorare il gioco di parole** con un mood editoriale che strizza l'occhio al mondo della moda — non come parodia, ma come tono.

### Il doppio senso del nome (concept portante — leggere prima di tutto)

Il nome non è solo un gioco di parole carino: contiene **due significati**, e la landing deve farli sentire entrambi. Sono il cuore concettuale da cui far nascere lo stile grafico.

**1. "à porter" — pronti da portare con sé, indossabili al volo.**
Premi la **combinazione di tasti magici** (hotkey globale, configurabile) da qualsiasi applicazione: si apre una palette al volo, **selezioni** il prompt, lo **compili** riempiendo i segnaposti, lo **copi** e lo **incolli dove vuoi** — chat AI, editor, email. Niente "apri l'app, cerca, copia, torna indietro". Il prompt giusto è addosso a te, a un tasto di distanza, ovunque tu stia scrivendo. _Questo è un comportamento reale del prodotto (Command Palette via global shortcut), non una metafora — va comunicato come feature concreta._

**2. "prêt-à-porter" — il passaggio dalla sartoria all'industrializzazione.**
Nella moda, il prêt-à-porter è stato la rivoluzione che ha portato l'abito **dalla sartoria su misura** (un capo, un cliente, cucito a mano) **alla produzione industriale** (collezioni, taglie, riproducibilità) — senza perdere il gusto. Prompt a Porter fa lo stesso con i prompt: si parte dall'**artigianato** del prompt salvato in un file di testo, copiato e riadattato a mano ogni volta, e si arriva all'**industrializzazione** — versioni, segnaposti, varianti A/B, ricerca semantica, riuso modulare, golden test. Lo stesso prompt smette di essere un capo unico e diventa una **collezione riproducibile**. La landing deve raccontare questo arco: _da fondo del cassetto a guardaroba curato; da `prompt-FINAL-davvero.txt` a collezione._

> Regola per Claude Design: ogni scelta visiva forte dovrebbe poggiare su almeno uno di questi due assi — la **velocità/portabilità** ("à porter", il gesto del tasto magico) o la **trasformazione artigianato→industria** (prêt-à-porter, sartoria che diventa collezione). Sono il punto di partenza, non un dettaglio.

**Mood**: rivista di moda contemporanea (think Wallpaper, COS Magazine, Aesop website). Pulito, ariosa, white space generoso, dettagli tipografici raffinati. Il prodotto è tecnico ma si presenta come una **collezione curata**, non come un tool.

**Vocabolario suggerito** (usare con misura, mai forzato): _collezione · guardaroba · cassetto · armadio · sartoria · atelier · capi · vetrina · su misura · cuciti · tagli · stagione · indossare · fondo del cassetto_.

**Tipografia**: indicazione direzionale a Claude Design — serif editoriale per i titoli (es. Fraunces, Tiempos, GT Sectra, Newsreader) abbinato a sans-serif pulito per il body (Inter, IBM Plex Sans). Monospace per snippet (JetBrains Mono). La scelta finale è di Claude Design.

**Palette**: **aperta** — decide Claude Design senza vincoli di coerenza con il client desktop. Il mood di riferimento è "fashion brand minimalista" (nero profondo + neutri caldi + 1 accent), non "SaaS techno". La landing è la **vetrina**, può permettersi una voce visiva propria.

**Tono**: **ironico e auto-consapevole**. Diciamo cose serie con leggerezza. Mai marketing-speak gonfio ("rivoluziona la tua produttività"), mai techno-speak rigido ("leverage your AI stack"). Riferimento: come una rivista di moda parla di vestiti — con autorevolezza ma con humor.

**Voce**: italiano corretto, registro medio-alto editoriale. Anglismi solo dove inevitabili nel dominio tecnico (prompt, software, file). Niente "leverage", "skill set", "use case" tradotti goffi. Niente esclamativi gratuiti.

## Struttura proposta (8 sezioni)

Tutte le sezioni applicano il tone-of-voice editoriale fashion definito sopra. Il messaggio funzionale resta invariato — quello che cambia è la cornice.

### 1. Hero

**Titolo (h1)**: Prompt a Porter
**Sottotitolo**: La tua collezione di prompt AI. Pronti da indossare.
**Pitch breve (1 frase)**: Versionali, riusali, cuciscili su misura con i segnaposti. Tutto in locale, niente cloud obbligatorio.
**Riga d'aggancio (sotto il pitch, opzionale ma consigliata)**: Un tasto magico e il prompt giusto è già tra le dita: lo compili e lo incolli dove stai scrivendo.

> Nota a Claude Design: l'hero è il posto naturale per **mostrare il gesto "à porter"** — l'hotkey globale che apre la palette sopra qualsiasi app. Un'animazione/illustrazione del tasto magico → palette → prompt compilato → incollato altrove racconta in un colpo solo il primo significato del nome. Valutare se renderlo il visual principale dell'hero.

**CTA primario**: `Scarica per Windows` (link diretto all'asset portable .zip Latest release)
**CTA secondario**: `Vedi su GitHub` (link al repo)

**Visual**: screenshot del client (Libreria + DetailPane + un prompt visibile con segnaposti evidenziati) o mockup pulito, presentato in stile editoriale (es. lookbook con white space generoso, non screenshot product-marketing standard).

### 2. Il problema

**Titolo**: I tuoi prompt valgono. Ma sono nel fondo del cassetto.
**Body**:
> File di testo sparsi. Note in app diverse. Screenshot di conversazioni. Prompt copiati e incollati che ogni volta vanno adattati a mano. Quando trovi quello buono, finisce nel buco nero degli appunti.
>
> Prompt a Porter è il tuo guardaroba: ogni prompt al suo posto, pronto da indossare quando serve. Senza ricuciture, senza ricordarsi dove l'avevi lasciato.

### 3. La collezione (5-6 card capability)

Le **capacità del prodotto** presentate come capi di una collezione. Ogni card ha icona, titolo (con flair fashion), 1-2 righe di descrizione funzionale (chiara, tecnica, niente forzature).

**Card 0 — Un tasto magico e lo indossi**
Premi l'hotkey globale da qualsiasi app: si apre la palette, cerchi il prompt, lo compili al volo e lo incolli dove stai scrivendo. Nessun salto di finestra. _(Questa è la card che incarna il "à porter": metterla per prima o subito dopo l'hero.)_

**Card 1 — Ogni modifica, in vetrina**
Ogni save crea una versione. Diff stile git. Torna indietro quando l'ultima modifica ha rovinato tutto.

**Card 2 — Capi su misura**
`{{nome}}` per variabili ad-hoc, `{{globale autore}}` per valori che vivono nel vault. Il prompt diventa template, il template diventa abito su misura.

**Card 3 — Trova quello giusto. Anche al buio dell'armadio.**
Cerca per significato, non solo per parola. Modello ONNX MiniLM embedded — niente API, niente cloud, niente quote.

**Card 4 — Prova prima di comprare**
Crea varianti dello stesso prompt, confrontale fianco a fianco, scegli la migliore con i golden test (regression testing sui prompt).

**Card 5 — Lo stesso tono, in tagli diversi**
`{{import "altro-prompt"}}` per riusare blocchi. Definisci una volta lo stile, riusalo in dieci prompt diversi. Sartoria modulare.

### 4. Il guardaroba resta a casa tua.

**Titolo**: Il guardaroba resta a casa tua.
**Body**:
> Vault SQLite cifrato AES-256 con master password tua. Modello AI eseguito on-device. Sync server è **opzionale** — se non lo configuri, PaP non parla con internet.
>
> Open source AGPL 3.0: chiunque può ispezionare il codice. La privacy non è una feature opt-in, è il taglio di partenza.

Visual: schema architettura semplificato (cassaforte locale + cervello on-device + freccia sbarrata verso cloud), trattato come illustrazione editoriale, non come "diagramma tecnico SaaS".

### 5. Tre clienti tipo

**Titolo**: A chi sta bene Prompt a Porter.

Mini-card con illustrazione editoriale (no foto stock) + scenario in 2 frasi. Tono leggero, riconoscibile.

**Lo sviluppatore**: usa Claude/GPT trenta volte al giorno per code review, refactor, documentazione. Vuole template ripetibili e versionabili come il codice. Si rifiuta di chiamarli "asset".

**Il copywriter**: ha cinquanta prompt che si chiamano `email_v3_FINAL_FINAL.txt`. Ogni cliente è una variante. Vuole smettere di riscrivere lo stesso brief da zero ogni volta.

**Il ricercatore**: prepara prompt sperimentali per i suoi paper. Vuole confrontare tre versioni dello stesso prompt e capire quale dà output migliori. Senza diventare matto.

### 6. PaP vs. un file `.txt`

**Titolo**: Perché non basta un file `.txt`.
**Sottotitolo**: Lo abbiamo provato anche noi. Davvero.

Tabella a 2 colonne, tono ironico ma con contenuto. Niente confronti gonfi con Notion o "tool cloud generici": il vero competitor di chi inizia è il file di testo.

| | Prompt a Porter | Un file `.txt` |
|---|---|---|
| Versionamento | Diff stile git, ogni save una versione | `prompt-v2-FINAL.txt`, `prompt-v2-FINAL-davvero.txt`, `prompt-v2-FINAL-questo-funziona.txt` |
| Ricerca | Per significato, locale, on-device | Cmd+F e una preghiera |
| Segnaposti | `{{nome}}`, compilati al volo | Cerca-e-sostituisci a mano |
| Varianti | A/B con confronto fianco a fianco | Tre file diversi che dimentichi quale era quello buono |
| Riuso tra prompt | `{{import "blocco"}}` | Copia-incolla, scordandoti l'aggiornamento |
| Sicurezza | Vault SQLite cifrato AES-256 | "Documenti/prompt/" in chiaro |
| Sync (opzionale) | Server tuo o niente | Te lo emaili da solo. Di nuovo. |

> Un file `.txt` resta una scelta legittima. Quando smetti di trovarlo, sai dove siamo.

### 7. Provare un abito

**Titolo**: In due minuti. Come provare un abito.

**Sotto-sezione**: Windows
1. Scarica `Prompt-a-Porter-portable-windows-x64-vX.Y.Z.zip` dalla release Latest.
2. Estrai dove preferisci.
3. Esegui `Prompt-a-Porter.exe`. Niente installazione, niente UAC, niente domande.

**Sotto-sezione**: Linux / macOS
Build automatiche disponibili dalla [pagina release GitHub](https://github.com/robertomarchioro/prompt-a-porter/releases/latest).

**Box informativo**: Auto-update silenzioso in arrivo con v1.0 (firmato Authenticode). Una volta dentro l'armadio, gli aggiornamenti arrivano da soli.

### 8. Atelier aperto

**Titolo**: Codice aperto. Roadmap pubblica. Le cuciture si vedono.
**Body**:
> PaP è open source con licenza AGPL 3.0. Il codice è su GitHub: contribuire è semplice, le issue documentano lo stato del progetto, la roadmap è pubblica.
>
> Niente telemetria. Niente account obbligatorio. Niente dark pattern. Il modello è in vetrina.

**CTA**: `⭐ Vedi su GitHub` + link.

### 9. Footer

- Link: GitHub | Issue tracker | License (AGPL 3.0) | CHANGELOG
- Copyright: Roberto Marchioro · {anno}
- Nessun "Privacy Policy" complesso perché non raccogliamo dati — solo un disclaimer breve: "Questa pagina usa Matomo self-hosted per analytics anonimizzati. Nessun cookie di terze parti. Nessun dato condiviso."

## Tono ed esempi di copy

### Da NON usare

❌ "Rivoluziona il tuo modo di lavorare con l'AI"
❌ "Sblocca la produttività"
❌ "Sfrutta la potenza dei Large Language Models"
❌ "Powered by AI" come tag generico
❌ "Easy to use" generico — sostituire con esempio concreto
❌ Anglismi gonfi: "leverage", "skill set", "use case" tradotti goffi
❌ Esclamativi gratuiti, emoji marketing, "🚀✨💯"
❌ Riferimenti moda forzati: "PaP è il Chanel dei prompt" (cringe garantito)

### Da usare

✅ "La tua collezione di prompt. Pronti da indossare."
✅ "Versiona, confronta, riusa. In locale."
✅ "Il guardaroba resta a casa tua."
✅ "Cmd+F e una preghiera." (per descrivere alternative)
✅ "Vault cifrato AES-256, modello ONNX on-device." (i fatti tecnici vanno detti dritti, senza svestirli)
✅ "Atelier aperto. Le cuciture si vedono."
✅ "Codice aperto. Nessun account."

### Regola d'oro

I riferimenti moda **adornano**, non sostituiscono. Ogni claim fashion deve avere accanto (sopra o sotto) un'**informazione tecnica concreta**. Se togli il flair fashion, il copy deve ancora funzionare come comunicazione di prodotto.

## Asset richiesti a Claude Design

L'handoff a Claude Design deve produrre:

1. **Mockup high-fidelity** per le 8 sezioni sopra (desktop 1440px + mobile 375px), con trattamento **editoriale fashion** (white space generoso, layout magazine, dettagli tipografici curati).
2. **Componenti riutilizzabili**: card capability (con flair fashion), card persona (illustrazione editoriale no-stock), tabella confronto PaP vs `.txt`, hero, CTA button (primario + secondario).
3. **Palette** proposta dal designer — direzione: fashion brand minimalista (nero profondo + neutri caldi + 1 accent caldo tipo terracotta/ocra/ruggine). Variazioni dark/light se applicabile.
4. **Tipografia**: serif editoriale per titoli (Fraunces / Tiempos / GT Sectra / Newsreader o equivalente) + sans-serif per body (Inter / IBM Plex Sans) + monospace per snippet (JetBrains Mono). Sizes h1-h6, body, mono definiti.
5. **Screenshot del prodotto** preparati e trattati: Libreria con prompt visibili, DetailPane con segnaposti, vista Confronto varianti, Diagnosi linting, e — importante per il concept "à porter" — la **Command Palette aperta sopra un'altra applicazione** (il gesto del tasto magico) con un prompt selezionato/compilato. Trattati con cornice editoriale (es. dispositivo stilizzato, ombre soft, niente browser chrome generico).
   - Opzionale ma di forte impatto: una **animazione/sequenza dell'arco "à porter"** (hotkey → palette → compilazione segnaposti → incolla altrove) per l'hero.
6. **Icona/logo** della pagina (può riusare l'icona esistente del client `Prompt-a-Porter.exe` o reinterpretarla in chiave fashion).
7. **Favicon** + `apple-touch-icon` 180×180.
8. **Meta OG image** 1200×630 per anteprime social, treated come copertina di rivista.
9. **Illustrazioni editoriali** per le 3 persone tipo della sezione 5 (no foto stock, no clipart). Stile coerente col mood editoriale fashion.

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
