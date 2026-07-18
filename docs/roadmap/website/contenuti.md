# Contenuti landing page — v2 (refresh «Arioso Atelier»)

> **Destinatario**: Claude Design (o designer umano) e chi implementa/aggiorna la landing in `apps/site`.
> **Obiettivo**: fornire copy, struttura, audience e CTA aggiornati allo stato reale del prodotto, senza dover ri-chiedere.
> **Nome del prodotto**: Prompt-a-porter
> **Sostituisce**: `_old/contenuti.md` (base della prima landing). La prima versione parlava di un prodotto Windows-only con auto-update "in arrivo": non è più così. Questo documento è la nuova fonte di verità del copy.

## Frame

**Cosa promuoviamo**: versione di Prompt-a-porter **Personale** (single user, local-first), oggi disponibile su **Windows, macOS e Linux** con auto-update firmato attivo. Niente teaser Enterprise — quando v2.0 esisterà, faremo refresh della landing.

**Cosa NON promettere**:
- Multi-utente / workspace team
- SSO / E2E encryption
- Web app / Browser extension
- Sync server obbligatorio (esiste un server di sync **self-hosted e opzionale** nel repo: si può citare come opzione per smanettoni, mai come pilastro del prodotto)
- API pubblica
- Approval workflow / RBAC
- Sync P2P «Ordito» (blueprint pubblico, ma è materia v2.0: non ha data)

Tutto quanto sopra è v2.0, gate domanda-driven. Non va comunicato come "in arrivo" perché non ha data.

**Cosa invece ADESSO possiamo promettere** (ed era vietato o assente nella v1 di questo documento):
- Tre piattaforme vere: installer Windows firmato Authenticode, `.dmg` macOS **universale** (Apple Silicon + Intel) firmato e notarizzato, `.deb`/AppImage Linux.
- Auto-update **attivo** su tutte e tre (updater Tauri con firma Ed25519, riavvio automatico).
- Una **CLI** (`pap`) e un **server MCP** che portano il vault fuori dall'app, in sola lettura.
- **Ritocco**: suggerimenti AI per migliorare un prompt, col provider scelto dall'utente.
- **Test Golden** completi dall'interfaccia (llm-judge, similarità cosine, costo stimato).
- **Diagnosi**: linting dei prompt con regole attivabili e tarabili.

## Lingua

**Solo italiano** in v1 della landing. EN traduzione futura quando il progetto cresce. Tutto il copy qui sotto è già in IT pronto da usare.

## Audience target

**Primaria**:
- Professionisti che lavorano quotidianamente con LLM (Claude, GPT, Gemini) e accumulano prompt sparsi tra appunti, note, screenshot.
- Sviluppatori che usano AI in workflow (code review, documentazione, debugging) e vogliono trattare i prompt come asset versionabili — e richiamarli anche da terminale o dentro l'assistente stesso (CLI + MCP).
- Creatori di contenuti (copywriter, marketer, formatori) che riutilizzano template di prompt con variazioni.
- Appassionati che iniziano a usare più sistemi LLM per passione, divertimento o studio.

**Secondaria**:
- Power user di Notion/Obsidian che cercano un'alternativa specializzata per i prompt.
- Utenti di Claude Desktop / Cursor che vogliono dare ai loro assistenti una libreria di prompt curata (leva MCP).
- Curiosi del movimento "prompt engineering" che vogliono andare oltre il prompt-singolo.

**Non target**:
- Aziende/team (audience v2.0).
- Utenti casuali che usano ChatGPT 1 volta a settimana.

## Brand identity (riferimenti)

Il nome "Prompt a Porter" è un omaggio dichiarato al **prêt-à-porter**: prompt pronti da indossare, cuciti su misura. La landing deve **onorare il gioco di parole** con un mood editoriale che strizza l'occhio al mondo della moda — non come parodia, ma come tono.

### Il doppio senso del nome (concept portante — leggere prima di tutto)

Il nome non è solo un gioco di parole carino: contiene **due significati**, e la landing deve farli sentire entrambi. Sono il cuore concettuale da cui è già nato lo stile grafico della landing «Arioso Atelier».

**1. "à porter" — pronti da portare con sé, indossabili al volo.**
Premi la **combinazione di tasti magici** (hotkey globale, configurabile) da qualsiasi applicazione: si apre una palette al volo, **selezioni** il prompt, lo **compili** riempiendo i segnaposti, lo **copi** e lo **incolli dove vuoi** — chat AI, editor, email. Niente "apri l'app, cerca, copia, torna indietro". Il prompt giusto è addosso a te, a un tasto di distanza, ovunque tu stia scrivendo. _Questo è un comportamento reale del prodotto (Command Palette via global shortcut), non una metafora — va comunicato come feature concreta._
Il significato "à porter" oggi ha **due estensioni reali** in più: il prompt ti segue anche **nel terminale** (`pap render` in pipe) e **dentro i tuoi assistenti AI** (server MCP per Claude Desktop, Cursor e simili). Il guardaroba è lo stesso; cambiano le porte da cui entri.

**2. "prêt-à-porter" — il passaggio dalla sartoria all'industrializzazione.**
Nella moda, il prêt-à-porter è stato la rivoluzione che ha portato l'abito **dalla sartoria su misura** (un capo, un cliente, cucito a mano) **alla produzione industriale** (collezioni, taglie, riproducibilità) — senza perdere il gusto. Prompt a Porter fa lo stesso con i prompt: si parte dall'**artigianato** del prompt salvato in un file di testo, copiato e riadattato a mano ogni volta, e si arriva all'**industrializzazione** — versioni, segnaposti, varianti A/B, ricerca semantica, riuso modulare, golden test, controllo qualità (linting), perfino il **ritocco del sarto** (suggerimenti AI). Lo stesso prompt smette di essere un capo unico e diventa una **collezione riproducibile**. La landing deve raccontare questo arco: _da fondo del cassetto a guardaroba curato; da `prompt-FINAL-davvero.txt` a collezione._

> Regola per Claude Design: ogni scelta visiva forte dovrebbe poggiare su almeno uno di questi due assi — la **velocità/portabilità** ("à porter", il gesto del tasto magico) o la **trasformazione artigianato→industria** (prêt-à-porter, sartoria che diventa collezione). Sono il punto di partenza, non un dettaglio.

**Mood**: rivista di moda contemporanea (think Wallpaper, COS Magazine, Aesop website). Pulito, arioso, white space generoso, dettagli tipografici raffinati. Il prodotto è tecnico ma si presenta come una **collezione curata**, non come un tool.

**Vocabolario suggerito** (usare con misura, mai forzato): _collezione · guardaroba · cassetto · armadio · sartoria · atelier · capi · vetrina · su misura · cuciti · tagli · stagione · indossare · fondo del cassetto · ritocco · prova in camerino · etichetta di manutenzione_.

**Tipografia**: **decisa e in produzione** — Newsreader (serif editoriale, titoli), Inter (body), JetBrains Mono (snippet, label, segnaposti). Non riaprire la scelta: riusarla.

**Palette**: **decisa e in produzione** — dark editoriale (`#0B0B0C`) con **viola `#8470D5` = brand & azione** e **ambra `#D8A463` = segnaposti/template**. La regola cromatica viola/ambra è un vincolo, non un suggerimento (i token `{{…}}` sono sempre ambra su tint ambra). Il dettaglio completo dei token è nel `README.md` di questa cartella. L'icona ufficiale `{ P }` viola è l'unico mark ammesso.

**Stagioni e codename** (convenzione reale del progetto, da usare nel copy di lancio): le release cardine hanno etichetta `Autunno-Inverno ANNO · vX.Y.Z` e un codename tessile. La **v1.0 è «Arioso Atelier»** e "subentra a «Ago e Filo», il codename di laboratorio" della fase beta. La landing di lancio ha già ribbon e sezione «Debutto stagione» costruiti su questa convenzione: mantenerla viva nei refresh futuri (ogni release cardine = nuova stagione in vetrina).

**Tono**: **ironico e auto-consapevole**. Diciamo cose serie con leggerezza. Mai marketing-speak gonfio ("rivoluziona la tua produttività"), mai techno-speak rigido ("leverage your AI stack"). Riferimento: come una rivista di moda parla di vestiti — con autorevolezza ma con humor.

**Voce**: italiano corretto, registro medio-alto editoriale. Anglismi solo dove inevitabili nel dominio tecnico (prompt, software, file). Niente "leverage", "skill set", "use case" tradotti goffi. Niente esclamativi gratuiti.

## Struttura proposta (9 sezioni + footer)

Tutte le sezioni applicano il tone-of-voice editoriale fashion definito sopra. Rispetto alla v1 del documento cambia: CTA multi-piattaforma, una sezione nuova (**I servizi dell'atelier**) per le capacità arrivate dopo, e l'auto-update promosso da promessa a fatto.

### 1. Hero

**Titolo (h1)**: Prompt-à-porter
**Sottotitolo**: La tua collezione di prompt AI. Pronti da indossare.
**Pitch breve (1 frase)**: Versionali, riusali, cuciscili su misura con i segnaposti. Tutto in locale, niente cloud obbligatorio.
**Riga d'aggancio (sotto il pitch, opzionale ma consigliata)**: Un tasto magico e il prompt giusto è già tra le dita: lo compili e lo incolli dove stai scrivendo.

> Nota a Claude Design: l'hero resta il posto naturale per **mostrare il gesto "à porter"** — l'hotkey globale che apre la palette sopra qualsiasi app. Il carosello showcase della landing attuale (palette → libreria → compila → cronologia → import → giorno/sera) assolve già questo compito: nei refresh, valutare l'aggiunta di scene per Ritocco e Test Golden invece di stravolgere l'impianto.

**CTA primario**: `Scarica per Windows` con rilevamento piattaforma se fattibile (macOS → `.dmg` universale, Linux → `.deb`/AppImage); in fallback link "Altre piattaforme →" alla release Latest.
**CTA secondario**: `Vedi su GitHub` (link al repo)
**Meta sotto i CTA**: `gratis · local-first · Windows / macOS / Linux`

**Visual**: screenshot reali del client (Libreria + DetailPane con segnaposti evidenziati, palette sopra un'altra app) trattati in stile editoriale — la landing attuale usa mockup HTML/CSS: quando possibile sostituirli con **screenshot o registrazioni della release Latest**.

### 2. Il problema

**Titolo**: I tuoi prompt valgono. Ma sono in fondo al cassetto.
**Body**:
> File di testo sparsi. Note in app diverse. Screenshot di conversazioni. Prompt copiati e incollati che ogni volta vanno adattati a mano. Quando trovi quello buono, finisce nel buco nero degli appunti.
>
> Prompt a Porter è il tuo guardaroba: ogni prompt al suo posto, pronto da indossare quando serve. Senza ricuciture, senza ricordarsi dove l'avevi lasciato.

### 3. La collezione (6 card capability)

Le **capacità fondanti del prodotto** presentate come capi di una collezione (nella landing attuale: card con texture-tessuto, gruccia e cartellino SKU — mantenere). Ogni card ha titolo con flair fashion e 1-2 righe di descrizione funzionale, chiara e tecnica.

**Card 0 — Un tasto magico e lo indossi**
Premi l'hotkey globale da qualsiasi app: si apre la palette, cerchi il prompt, lo compili al volo e lo incolli dove stai scrivendo. Nessun salto di finestra. _(Questa è la card che incarna il "à porter": metterla per prima o subito dopo l'hero.)_

**Card 1 — Ogni modifica, in vetrina**
Ogni save crea una versione. Diff stile git, ripristino in un click. E se cancelli un prompt, c'è il cestino: niente sparisce per un gesto maldestro.

**Card 2 — Capi su misura**
`{{nome}}` per variabili ad-hoc, `{{global autore}}` per valori che vivono nel vault. Il prompt diventa template, il template diventa abito su misura.

**Card 3 — Trova quello giusto. Anche al buio dell'armadio.**
Cerca per significato, non solo per parola. Modello ONNX MiniLM embedded — niente API, niente cloud, niente quote.

**Card 4 — Prova prima di comprare**
Crea varianti dello stesso prompt, confrontale fianco a fianco, scegli la migliore con i **Test Golden**: regression testing sui prompt, con giudice LLM o similarità cosine, e il costo stimato *prima* di lanciare.

**Card 5 — Lo stesso tono, in tagli diversi**
`{{import "altro-prompt"}}` — anche con parametri, `{{import "stile" with tono=formale}}` — per riusare blocchi. Definisci una volta lo stile, riusalo in dieci prompt diversi. Sartoria modulare.

### 4. I servizi dell'atelier (sezione NUOVA)

Le capacità arrivate dopo la prima landing, presentate come **i servizi della maison**: non capi in collezione, ma ciò che l'atelier fa *per* te e *intorno* al guardaroba. 4 card più sobrie delle card-tessuto (niente competizione visiva con la collezione).

**Ritocco — il sarto guarda il capo** ✦
Un provider AI a tua scelta (Anthropic, OpenAI, Google, Ollama o endpoint compatibile) esamina il prompt e propone migliorie **tarate sul modello per cui è scritto** — le linee guida di prompting per famiglia sono incluse nell'app. Vedi i suggerimenti, il testo riscritto a diff, e se ti piace lo accetti come nuova versione. Lo storico resta intatto.

**Diagnosi — il controllo qualità**
Un linter dedicato ai prompt segnala segnaposti malformati, incoerenze e fragilità. Ogni regola si attiva, disattiva e tara dalle impostazioni: severità decisa da te, non dal tool.

**La CLI `pap` — il guardaroba in terminale**
Un singolo eseguibile, stesso vault del client, in sola lettura: `pap search`, `pap get`, `pap render --var k=v` e via in pipe al comando successivo. Per chi vive nella shell, il prompt arriva senza aprire finestre.

**MCP — il guardaroba per i tuoi assistenti**
Il server MCP incluso espone il vault a Claude Desktop, Cursor e ogni client Model Context Protocol: l'assistente cerca, legge e compila i tuoi prompt su richiesta. In sola lettura, solo via stdio: nessuna porta aperta, nessun dato che lascia il computer.

> Copy di raccordo suggerito per aprire la sezione: _"La collezione è tua. L'atelier la tiene in forma: la controlla, la ritocca, te la porge ovunque tu stia lavorando — in un'altra app, nel terminale, dentro il tuo assistente."_

### 5. Il guardaroba resta a casa tua.

**Titolo**: Il guardaroba resta a casa tua.
**Body**:
> Vault SQLite cifrato AES-256 (SQLCipher) con master password tua — dentro ci stanno anche le chiavi API dei provider, mai in chiaro su disco. Il modello di ricerca semantica gira on-device; i file scaricati sono verificati SHA-256; le chiavi di cifratura vengono azzerate dalla memoria appena non servono. L'app parla con internet solo se glielo chiedi tu: per gli aggiornamenti firmati, o verso i provider AI che *tu* hai configurato.
>
> Open source AGPL 3.0: chiunque può ispezionare il codice. La privacy non è una feature opt-in, è il taglio di partenza.

Nota di copy: i fatti tecnici vanno detti dritti ma **selezionati** — sulla landing bastano vault cifrato, on-device, niente account, niente telemetria; il resto vive nella docs. Chi vuole il dettaglio del hardening lo trova nel CHANGELOG e in `SECURITY.md`.

Visual: schema architettura semplificato (cassaforte locale + cervello on-device + freccia sbarrata verso cloud), trattato come illustrazione editoriale, non come "diagramma tecnico SaaS".

### 6. Tre clienti tipo

**Titolo**: A chi sta bene Prompt a Porter.

Mini-card con illustrazione editoriale (no foto stock) + scenario in 2 frasi. Tono leggero, riconoscibile.

**Lo sviluppatore**: usa Claude/GPT trenta volte al giorno per code review, refactor, documentazione. Vuole template ripetibili e versionabili come il codice — e li richiama con `pap render` in pipe o direttamente dal suo assistente via MCP. Si rifiuta di chiamarli "asset".

**Il copywriter**: ha cinquanta prompt che si chiamano `email_v3_FINAL_FINAL.txt`. Ogni cliente è una variante. Vuole smettere di riscrivere lo stesso brief da zero ogni volta — e quando un prompt gli sembra fiacco, chiede il Ritocco invece di rimuginare.

**Il ricercatore**: prepara prompt sperimentali per i suoi paper. Vuole confrontare tre versioni dello stesso prompt e capire quale dà output migliori — coi Test Golden, non a occhio. Senza diventare matto.

### 7. PaP vs. un file `.txt`

**Titolo**: Perché non basta un file `.txt`.
**Sottotitolo**: Lo abbiamo provato anche noi. Davvero.

Tabella a 2 colonne, tono ironico ma con contenuto. Niente confronti gonfi con Notion o "tool cloud generici": il vero competitor di chi inizia è il file di testo.

| | Prompt a Porter | Un file `.txt` |
|---|---|---|
| Versionamento | Diff stile git, ogni save una versione, cestino incluso | `prompt-v2-FINAL.txt`, `prompt-v2-FINAL-davvero.txt`, `prompt-v2-FINAL-questo-funziona.txt` |
| Ricerca | Per significato, locale, on-device | Cmd+F e una preghiera |
| Segnaposti | `{{nome}}`, `{{global autore}}`, compilati al volo | Cerca-e-sostituisci a mano |
| Varianti | A/B con confronto fianco a fianco e Test Golden | Tre file diversi che dimentichi quale era quello buono |
| Riuso tra prompt | `{{import "blocco" with k=v}}` | Copia-incolla, scordandoti l'aggiornamento |
| Migliorare un prompt | Ritocco: suggerimenti AI a diff, accetti → nuova versione | Lo incolli in ChatGPT, ricopi a mano, perdi l'originale |
| Dal terminale | `pap render \| pbcopy` | `cat` e speranza |
| Dai tuoi assistenti AI | Server MCP in sola lettura, via stdio | Trascini il file nella chat. Di nuovo. |
| Sicurezza | Vault SQLite cifrato AES-256 | "Documenti/prompt/" in chiaro |
| Aggiornamenti | Auto-update firmato, si riavvia da solo | Il `.txt` almeno quello non si aggiorna mai |

> Un file `.txt` resta una scelta legittima. Quando smetti di trovarlo, sai dove siamo.

Nota: 10 righe sono il tetto — se in futuro se ne aggiungono, toglierne. La tabella funziona finché si legge in un respiro.

### 8. Provare un abito

**Titolo**: In due minuti. Come provare un abito.

**Sotto-sezione: Windows**
1. Scarica l'installer dalla release Latest (firmato Authenticode) — oppure il portable `.zip` se preferisci niente installazione.
2. Avvia. Installazione per-utente: niente UAC, niente domande.
3. Al primo avvio un breve tour di benvenuto ti mostra il guardaroba; la checklist "Primi passi" fa il resto.

**Sotto-sezione: macOS**
1. Scarica il `.dmg` dalla release Latest: **un solo file, universale** — gira nativo su Apple Silicon e Intel.
2. Trascina in Applicazioni. Firmato e notarizzato Apple: nessun avviso da aggirare.

**Sotto-sezione: Linux**
1. Scarica `.deb` (Debian/Ubuntu) o AppImage dalla release Latest.
2. `sudo apt install ./prompt-a-porter_*.deb`, oppure rendi eseguibile l'AppImage e via.

**Box informativo**: Gli aggiornamenti arrivano da soli, su tutte e tre le piattaforme: updater firmato (Ed25519 + Authenticode/notarizzazione), scarichi una volta e l'armadio si tiene in ordine da sé.

### 9. Atelier aperto

**Titolo**: Codice aperto. Roadmap pubblica. Le cuciture si vedono.
**Body**:
> PaP è open source con licenza AGPL 3.0. Il codice è su GitHub: contribuire è semplice, le issue documentano lo stato del progetto, la roadmap è pubblica. C'è perfino la documentazione di come viene firmata ogni release.
>
> Niente telemetria. Niente account obbligatorio. Niente dark pattern. Il modello è in vetrina.

**CTA**: `⭐ Vedi su GitHub` + link.

### 10. Footer

- Link: GitHub | Release | Documentazione | Issue tracker | License (AGPL 3.0) | CHANGELOG
- "Etichetta di manutenzione" (già in produzione, mantenere): `100% locale · no cloud · nessun account · lavabile in git · AGPL 3.0`
- Copyright: Roberto Marchioro · {anno} · firma "Disegnato da Roberto · cucito da ✦ Claude" (già in produzione)
- Nessun "Privacy Policy" complesso perché non raccogliamo dati — solo un disclaimer breve: "Questa pagina usa Matomo self-hosted per analytics anonimizzati. Nessun cookie di terze parti. Nessun dato condiviso." *(Se Matomo non è ancora attivo sull'hosting corrente, omettere il disclaimer finché non lo è.)*

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
❌ Vendere il Ritocco come "AI-powered magic": è un servizio del sarto, concreto — provider *tuo*, suggerimenti a diff, accetti o rifiuti

### Da usare

✅ "La tua collezione di prompt. Pronti da indossare."
✅ "Versiona, confronta, riusa. In locale."
✅ "Il guardaroba resta a casa tua."
✅ "Cmd+F e una preghiera." (per descrivere alternative)
✅ "Vault cifrato AES-256, modello ONNX on-device." (i fatti tecnici vanno detti dritti, senza svestirli)
✅ "Atelier aperto. Le cuciture si vedono."
✅ "Codice aperto. Nessun account."
✅ "Se ci puoi scrivere, ci puoi incollare il tuo prompt." (slogan manifesto, già in produzione)
✅ "Il sarto guarda il capo, propone il ritocco. Decidi tu se farlo cucire." (per Ritocco)
✅ "Windows, macOS e Linux. Lo stesso guardaroba, tre armadi."

### Regola d'oro

I riferimenti moda **adornano**, non sostituiscono. Ogni claim fashion deve avere accanto (sopra o sotto) un'**informazione tecnica concreta**. Se togli il flair fashion, il copy deve ancora funzionare come comunicazione di prodotto.

## Asset richiesti (delta rispetto alla landing esistente)

La landing «Arioso Atelier» è **già implementata** (`apps/site`, design in `prompt-a-porter-landing.html` + `README.md` di questa cartella). Non serve rifare l'impianto: serve l'aggiornamento incrementale. Deliverable del refresh:

1. **Screenshot reali della release Latest** al posto dei mockup HTML/CSS del carosello: Libreria con prompt visibili, DetailPane con segnaposti, Command Palette **sopra un'altra applicazione** (il gesto del tasto magico), Confronto varianti, modale Ritocco con diff, esecuzione Test Golden. Trattati con cornice editoriale (ombre soft, niente browser chrome generico).
2. **Due scene nuove per il carosello** (o sostituzione delle meno rappresentative): *Ritocco* (suggerimenti + diff) e *Test Golden* (varianti a confronto con esito). Rispettare la regola viola/ambra e il toggle via `display` documentato nel README.
3. **CTA multi-piattaforma**: bottone primario con detection OS o tripletta Windows/macOS/Linux; badge "universale (Apple Silicon + Intel)" sul `.dmg`.
4. **Sezione "I servizi dell'atelier"**: 4 card sobrie (Ritocco, Diagnosi, CLI, MCP) coerenti col design system esistente — non nuove card-tessuto.
5. **Aggiornamento sezione stagione/debutto** al momento del tag v1.0: etichetta `Autunno-Inverno 2026 · v1.0`, codename «Arioso Atelier». Fino ad allora la landing base (senza elementi di lancio) resta il riferimento.
6. **Meta OG image** 1200×630 aggiornata (copertina di rivista) se gli screenshot cambiano il visual dell'hero.
7. Icona/logo: usare l'icona ufficiale `{ P }` viola già integrata in app e sito. Favicon e `apple-touch-icon` esistono: non toccarli senza motivo.

## Tracciamento progetto

I refresh della landing viaggiano su PR verso `main` (sito in `apps/site`). Checklist PR:
- [ ] Copy IT verificato contro questo documento
- [ ] Screenshot prodotto aggiornati alla release Latest
- [ ] CTA di download puntano alla release Latest per tutte e tre le piattaforme
- [ ] Build statico verde
- [ ] Lighthouse score ≥ 95 in Performance/Accessibility/SEO
- [ ] Regola cromatica viola/ambra rispettata (segnaposti sempre ambra)
- [ ] Matomo (se attivo) tracking verificato in staging

## Manutenzione di questo documento

- **Al tag v1.0.0**: attivare ribbon + sezione «Debutto stagione» della versione di lancio; verificare che ogni "in arrivo" residuo sia sparito.
- **A ogni release cardine successiva**: nuova stagione (etichetta + codename tessile dal pool), refresh della sezione stagione.
- Quando il **sync self-hosted** diventa una feature documentata per utenti finali: aggiungere una riga in §5 e in tabella §7 ("Server tuo o niente"). Fino ad allora, non promuoverlo.
- Quando un differenziatore emerge nei feedback utenti: aggiungere a §7.
- Quando il copy attuale si rivela non funzionante (es. bounce rate alto su una sezione tracciata da Matomo): iterare.
- Se una capability citata qui cambia comportamento (es. la CLI smette di essere read-only), aggiornare **prima** questo documento, poi la landing.
