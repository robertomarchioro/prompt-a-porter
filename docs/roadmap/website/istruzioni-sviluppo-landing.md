# Istruzioni di sviluppo — Landing «Scontrino cucito» (Cloud Dancer)

> **Destinatario**: chi implementa la nuova landing in `apps/site` (VitePress/Vue).
> **Fonti vincolanti**: handoff desktop [`desktop/README.md`](./desktop/README.md) (schermata `#4a`), handoff mobile [`mobile/README.md`](./mobile/README.md) (schermata `#3a`), copy [`contenuti.md`](./contenuti.md).
> **Regola di precedenza**: in caso di conflitto, **il copy lo governa `contenuti.md`**, **il visivo lo governano gli handoff**. La vecchia landing «Arioso Atelier» dark (in `archivio/`) resta come riferimento storico: questa nuova direzione la **sostituisce**.
> **Stato**: documento operativo. F1-F3 live; aggiornato al **2026-08-02** (riscritta la §5: il transazionale passa da n8n e non più da un endpoint Go, anti-abuso con ALTCHA, relay Lettermint).
> **Documento gemello**: per l'infrastruttura su Giganto (CT 150, listmonk, nginx, DNS, workflow n8n) la fonte è la pagina Notion *«CT 150 — mailer (Listmonk + Lettermint) — Piano di implementazione»*. Qui sta ciò che riguarda `apps/site`; là ciò che riguarda il cluster. Dove si toccano — contratto dell'endpoint, CORS, ALTCHA, privacy — vale quanto scritto nella §5 di questo file.

## 1. Cosa si costruisce

Una **sola pagina** con **due layout della stessa direzione creativa**:

- **Desktop `#4a`** — 1280px di riferimento, container max 1080–1180px, CTA «Scarica l'app» con detection OS.
- **Mobile `#3a`** — 392px di riferimento, colonna singola, CTA «Mandami il link» (form email, vedi §5).

Il breakpoint di collasso è **≤680px**: sotto quella soglia si passa **alla composizione mobile 3a**, non a un desktop compresso (regola esplicita del handoff desktop; breakpoint intermedi 1080/900 descritti lì). Palette **Cloud Dancer** (chiara), design token identici tra le due varianti: implementarli **una volta sola** come CSS custom properties.

### 1.1 Architettura dei layout: due alberi, un URL (decisione 2026-07-18)

Valutata l'alternativa "due pagine distinte" e scartata (su GitHub Pages non esistono redirect server-side: servirebbe sniffing user-agent via JS con flash di pagina sbagliata, più doppio URL con annotazioni SEO sconsigliate da Google e statistiche Matomo spezzate). La decisione è:

- **Un solo URL**, ma **due alberi di componenti completamente separati**: `LandingDesktop.vue` (composizione `#4a`) e `LandingMobile.vue` (composizione `#3a`). **Nessuna media query condivisa tra le due composizioni**: ognuna è scritta pixel-perfect sul proprio handoff, senza CSS che serve due padroni.
- Entrambi gli alberi sono montati nella pagina; la commutazione a **680px** avviene in **CSS puro** (`display:none` sull'albero non attivo) — SSG-friendly, niente flash, niente JS di detection. I breakpoint interni al desktop (1080/900) vivono **solo** dentro `LandingDesktop.vue`.
- Si **condividono solo** i pezzi identici per natura: design token (custom properties), `download.ts`/`os.ts`, il mockup palette (`CmdkPalette.vue` parametrizzato), il modulo form (§5). Il resto non si forza in comune: se un blocco diverge tra i due handoff, si duplica — la divergenza è del design, non un difetto del codice.
- Costo accettato: DOM doppio (~decine di KB, tutto inline — irrilevante per una landing). Beneficio: sviluppo, review e fix di un layout **senza rischio di regressione sull'altro**.
- **Verifica obbligatoria in ogni PR**: screenshot Playwright alle larghezze chiave **392 / 680 / 900 / 1280** confrontati coi render `3a-mobile.png` / `4a-desktop.png`.

### Cosa NON implementare (dagli handoff, ribadito)

- Il runtime di prototipazione (`support.js`) e la struttura `.dc.html`: servono solo markup e stili come riferimento.
- La status bar finta ("9:41") del mockup mobile: solo cornice di prototipo.
- Il trucco `<i style="font-style:inherit">{</i>{nome}}`: nel codice reale i token sono testo semplice `{{nome}}` nella pill ambra.

## 2. Considerazioni tecniche (leggere prima di scrivere codice)

### 2.1 Ambiente reale: `apps/site` e i suoi gotcha

- `config.ts` sta in `apps/site/.vitepress/` con `srcDir: "../../docs"` e `base: "/prompt-a-porter/"` **hardcoded**: ogni asset va prefissato con la base.
- La landing è un **custom layout** (`markdown.html: false` → la home non passa dal tema default). I componenti vivono in `.vitepress/theme/components/landing/`.
- **`{{nome}}` dentro gli SFC Vue è un'interpolazione**: i token ambra nei template vanno scritti con `v-pre` sull'elemento o come stringa (`{{ '{{nome}}' }}`). Nei `.md` vanno sempre in code span. **Non** ridefinire i delimitatori Vue globali (rompono il tema default — commento in `config.ts`).
- L'`head` della landing (font, meta og:) sta nel **frontmatter di `docs/index.md`**, non nell'head globale di `config.ts` (che inietta su ogni pagina docs).
- `pnpm preview` (sirv) **cachea gli asset all'avvio**: riavviare il preview dopo modifiche agli asset.
- La nuova landing è **chiara** mentre docs e tema VitePress hanno lo switch dark/light: la landing usa la **propria palette Cloud Dancer fissa**, indipendente dal tema del resto del sito. Non ereditare variabili del tema default nei componenti landing.

### 2.2 Riuso dei componenti esistenti

Non ripartire da zero: la struttura attuale in `theme/components/landing/` copre già gran parte dei blocchi. I componenti nuovi/restylati vanno organizzati sotto i **due alberi di §1.1** (`LandingDesktop.vue` / `LandingMobile.vue`); condivisi solo token, `download.ts`/`os.ts`, palette mockup e modulo form. Mappa indicativa:

| Blocco nuovo | Componente esistente | Azione |
|---|---|---|
| Topbar / header mobile | `TopBar.vue` | restyle (palette chiara, nav nuova); aggiungere hamburger mobile |
| Ribbon di lancio | `SeasonDebut.vue` (o affine) | restyle su gradiente viola nuovo |
| Hero + eyebrow kbd | `HeroStage.vue` | restyle; H1 98px desktop / 54px mobile con `clamp()` |
| Palette in evidenza | `CmdkPalette.vue` + `scenes/SceneCmdk.vue` | riuso forte: è lo stesso mockup, cambiano superficie e contorno "documento" |
| Manifesto | `ManifestoSection.vue` | restyle (banda `#EDEBF6`, palette light inclinata a destra) |
| Scontrino (collezione) | `CollectionGrid.vue` | **riscrittura**: da card-tessuto a scontrino con spilli, dentellatura, voci "incl." |
| Servizi atelier (2×2) | — | nuovo componente |
| Clienti tipo (cartellini) | — | nuovo componente |
| Banda download / sezione email | `download.ts`, `os.ts` | riusare la **detection OS esistente**; il form email è nuovo (§5) |
| Footer | `SiteFooter.vue` | restyle (rocchetto, barcode, care label) |

Il **carosello** attuale (`ShowcaseCarousel.vue` + scenes) nel nuovo design non c'è: la palette in evidenza è una scena unica. Il handoff desktop lo cita come **opzionale** futuro ("mini-carosello con Ritocco e Test Golden"): tenere le scene nel repo, non cablarle nella prima consegna.

### 2.3 Font e asset

- **Self-host obbligatorio** dei tre font (Newsreader, Inter, JetBrains Mono): non solo per performance (handoff), ma per **privacy** — il CDN Google Fonts trasmette l'IP del visitatore a Google e in UE è considerato trasferimento di dati personali. Nessuna richiesta a domini terzi da tutta la pagina.
- Tutti gli altri asset (spilli, ago, filo, rocchetto, icone OS, barcode, dentellature) sono **SVG/CSS inline** come da handoff: nessuna immagine esterna.
- Logo: icona ufficiale `{ P }` viola già in `docs/public/icons/` — non il placeholder "P" del prototipo.

### 2.4 Accessibilità e qualità (quality floor, non opzionale)

- `prefers-reduced-motion`: disattiva il blink del caret e ogni transizione decorativa.
- Focus visibile su link, bottoni, input (la palette chiara rende facile perderlo: outline viola).
- Hit target ≥44px su CTA e voci nav mobile.
- **Contrasto — scala scurita rispetto al handoff** (decisione Roberto 2026-07-18, in produzione): i grigi originali erano illeggibili su `#F1F0EC` (`#8C8A80` = 3.0:1, `#AEAB9F` = 2.0:1). La scala implementata è spostata di un gradino: `--pap-muted: #6E6C60` (4.6:1, AA ✓) per descrizioni/body, `--pap-faint: #8C8A80` per label secondarie, `--pap-ornamento: #AEAB9F` riservato ai soli ornamenti. Inoltre **Newsreader 300 solo sopra i 30px**: i serif medi (sottotitoli, tagline, titoli card) usano peso 400.
- **Corpi minimi — scala derivata dalla landing archiviata** (decisione Roberto 2026-07-19, in produzione): i corpi del prototipo (8–12px) erano troppo leggeri; il pavimento è quello che funzionava nella landing dark (`apps/site/archivio/landing-arioso/landing.css`): **desktop**: label mono ≥11px, descrizioni card 14px, body di sezione 16px, titoli card serif 21px+, voci palette 14.5/12.5px. **Mobile (dal blocco ≤900px dell'archivio)**: pavimento **13px per tutto il micro-testo** (label, badge, kbd, hint, mono) e **15px per i body** — su mobile non si scende mai sotto i 12px. Il prototipo resta il riferimento per composizione e proporzioni, non per i px assoluti dei corpi piccoli.
- La regola cromatica **viola = brand/azione, ambra = segnaposti** è un vincolo: i token `{{…}}` sono sempre ambra JetBrains Mono su tint ambra.
- Lighthouse ≥95 in Performance/Accessibility/SEO (checklist PR di `contenuti.md`).
- CSS: attenzione alle specificità incrociate tra selettori di sezione e di elemento (padding/margini tra sezioni che si annullano a vicenda); tenere i token in custom properties e gli stili per-componente scoped.

## 3. Matomo — tracciamento comportamenti

### 3.1 Istanza esistente su Giganto

L'istanza Matomo **esiste già** e risponde su **`https://matomo.giganto.it`**: non va creata, va **preparata**. Da fare:

- Creare nel pannello il sito/misurazione per la landing e annotare l'`idSite` da usare nel tag.
- **Audit della configurazione** rispetto ai requisiti di esenzione (§3.2): anonimizzazione IP ≥2 byte, rispetto DNT, retention log grezzi — sono impostazioni **server-side**, il tag cookieless da solo non basta.
- **Verificare la raggiungibilità pubblica del tracker**: al 2026-07-18 la connessione HTTPS da rete esterna risulta rifiutata (probabile restrizione IP/firewall). Va bene proteggere il pannello, ma `matomo.php` e `matomo.js` devono essere raggiungibili da qualsiasi visitatore, altrimenti la landing non traccia nulla.
- Verificare versione/aggiornamenti dell'istanza prima di collegarla a un sito pubblico.

### 3.2 Configurazione privacy-first (obbligatoria, non facoltativa)

Questa configurazione è ciò che rende **superfluo il banner cookie** (§4). Nel Matomo server-side:

- **Cookieless**: `disableCookies` nel tag JS — nessun cookie di tracciamento.
- **Anonimizzazione IP ≥2 byte** (meglio 3).
- Rispetto del **DoNotTrack**.
- Retention dei log grezzi limitata (90–180 giorni), report aggregati conservabili.
- Nessun incrocio con altri dati, nessuna condivisione con terzi, nessun tracciamento cross-site (condizioni dell'esenzione analytics del Garante).

### 3.3 Inserimento del tag nel sito

- Il tag va nell'**head globale** di `config.ts` (a differenza dei font della landing, per gli analytics è corretto coprire landing **e** docs: il funnel landing→guida è un dato utile).
- **Solo in build di produzione**: guardia su `process.env.NODE_ENV` (o variabile dedicata) per non sporcare i dati con lo sviluppo locale.
- Snippet `_paq` standard con `disableCookies` **prima** di `trackPageView`.

### 3.4 Eventi da tracciare (mai dati personali)

| Evento | Dove | Nota |
|---|---|---|
| Click CTA download, con OS rilevato | hero + banda download desktop | goal «Download avviato» |
| Submit form «Mandami il link» | sezione mobile | **solo l'evento, mai l'indirizzo email** |
| Iscrizioni newsletter | — | non tracciabili da Matomo del sito (avvengono via CTA in mail → pagina listmonk): usare le statistiche di listmonk |
| Click «Scopri il debutto →» | ribbon | |
| Outbound GitHub | hero + footer | |
| Navigazione ancore (collezione/servizi/come funziona) | topbar | opzionale |

Verificare il tracking **in staging** prima del merge (checklist PR in `contenuti.md`) e, una volta attivo, accendere il disclaimer footer già previsto da `contenuti.md` §10.

## 4. Banner cookie — verdetto: NON serve (a queste condizioni)

Con Matomo **cookieless + IP anonimizzato + nessun incrocio/cessione**, gli analytics rientrano nell'esenzione dal consenso riconosciuta dal Garante (analytics assimilati ai cookie tecnici) e la config è quella raccomandata da CNIL/Matomo per operare senza banner. La pagina inoltre non ha embed di terzi né font remoti (§2.3): **niente cookie banner**.

Obblighi che restano:
- **Disclaimer breve in footer** (già nel copy di `contenuti.md` §10: "Questa pagina usa Matomo self-hosted per analytics anonimizzati…").
- **Pagina privacy** quando il form email va live (§5.4) — è un'informativa trattamento dati, non un banner.

Il banner diventa necessario solo se in futuro si attivano cookie Matomo "pieni", embed di terzi (video, ecc.) o qualsiasi tracciamento non esente. Raccomandazione: **restare cookieless**; la perdita di precisione (visitatori unici stimati peggio) è irrilevante per questo sito e la coerenza col posizionamento "niente telemetria, niente dark pattern" vale più del dato.

Fonti: [Matomo senza consenso/banner](https://matomo.org/faq/new-to-piwik/how-do-i-use-matomo-analytics-without-consent-or-cookie-banner/) · [ePrivacy e implementazioni nazionali (incl. Garante)](https://matomo.org/faq/general/eprivacy-directive-national-implementations-and-website-analytics/) · [Cookieless tracking](https://matomo.org/cookie-consent-banners/)

## 5. «Mandami il link» + raccolta email

### 5.1 La soluzione: nessuna iscrizione dal sito

Il design mobile promette testualmente: *«un solo link. Niente newsletter, niente account, niente scuse.»* La raccolta contatti per comunicazioni future **non passa dal sito** — così la promessa resta vera alla lettera, il form resta un campo solo e non c'è micro-copy da rinegoziare:

1. **Dal sito**: l'email serve esclusivamente a recapitare il link di download. Base giuridica: esecuzione della richiesta dell'utente. L'indirizzo **non viene mai memorizzato in un archivio nostro**: transita nel workflow e finisce nei log di consegna del relay. ⚠️ n8n però **conserva i dati di esecuzione**, email inclusa: la retention delle execution del workflow va configurata esplicitamente (o il claim qui sopra è falso). Vedi §5.4.
2. **Dalla mail**: il corpo della mail col link contiene una **CTA di iscrizione** (es. *«Vuoi sapere quando debutta una nuova stagione? Avvisami →»*) che porta a una **pagina di iscrizione dedicata** con **double opt-in**. Chi clicca compie un'azione esplicita e la mail di conferma la sigilla: consenso pulito, lista costruita solo con chi la vuole davvero.

La mail del link è l'unico punto in cui la newsletter viene proposta — una volta, senza insistere. È l'approccio più coerente col brand "niente dark pattern": la conversione sarà più bassa di una checkbox, ma ogni iscritto è genuino.

**Copy della landing: invariato** (decisione Roberto, 2026-08-02). La promessa *«niente newsletter»* resta scritta così com'è sulla card mobile. La tensione con la CTA dentro la mail si scioglie **nel template della mail**, con una nota autoironica del tipo *«ti avevamo promesso niente newsletter — e infatti non ti ci abbiamo iscritto. Se però vuoi…»*. Non si ammorbidisce il copy del sito: è la promessa che rende credibile il resto.

### 5.2 Architettura (rev. 2026-08-02 — sostituisce l'endpoint Go)

> ⚠️ **Questa sezione è cambiata.** La versione precedente prescriveva un **endpoint proxy in Go** su Giganto. Non si fa più: il transazionale passa da **n8n**, che è già in produzione, già HA, già esposto e già in backup. Il documento operativo dell'infrastruttura è la pagina Notion *«CT 150 — mailer (Listmonk + Lettermint) — Piano di implementazione»*; qui sta solo ciò che riguarda `apps/site` e i vincoli che il sito impone al backend.

Il principio: **n8n gestisce i messaggi, Listmonk gestisce le liste.** Un link di download non ha consenso, disiscrizione o bounce da gestire — è un messaggio, non una campagna. I due sistemi **non si parlano mai**: l'unico punto di contatto è un URL scritto nel corpo di una mail.

```
apps/site (GitHub Pages, statico)
    │  1. GET  challenge ALTCHA
    │  2. POST { email, altcha, honeypot }
    ▼
n8n CT 110 — webhook /pap-download                [CORS gestito da nginx CT 100]
    │  verifica ALTCHA (HMAC) · honeypot · formato email · dedup indirizzo
    │  GET api.github.com/…/releases/latest → estrazione asset per pattern
    ▼
Lettermint (Paesi Bassi) — route `transactional`
    ▼
mail: 4 link download  +  CTA iscrizione ──────┐
                                               ▼
                        Listmonk CT 150 — tikki.giganto.it
                        pagina pubblica · double opt-in · route `broadcast`
```

Vincoli che il **sito** deve rispettare:

- **CORS**: l'origin da autorizzare è **`https://www.promptaporter.it`**, non l'apex — `promptaporter.it` fa 301 verso `www` e il browser manda l'origin *dopo* il redirect. Il CORS è gestito da nginx CT 100, non dall'opzione nativa del nodo Webhook n8n (documentata solo per le richieste non-preflight e riportata come inaffidabile).
- **Risposte non enumeranti**: sempre lo stesso messaggio di esito («Fatto, controlla la posta»), qualunque sia l'errore — coerente col lavoro #512 sugli errori opachi (CWE-209). Vale anche per il rifiuto anti-abuso: chi attacca non deve capire quale difesa è scattata.
- **Niente email in chiaro nei log**, né lato n8n né lato nginx.
- **Il contratto dell'endpoint** (forma di richiesta e risposta, status code, comportamento a quota esaurita) va fissato **prima** di scrivere il componente: senza, il form non è scrivibile. Da definire con la Fase 2 del piano Notion.

### 5.2bis Anti-abuso: ALTCHA — e dove trova posto nella nostra infrastruttura

**Decisione (2026-08-02): ALTCHA open source, `altcha` + `altcha-lib`, entrambi MIT.** Proof-of-work con challenge firmata HMAC: il widget contatta **solo un nostro endpoint**, nessun servizio di terzi entra nel giro, nessun cookie, nessun fingerprinting.

**Mai reCAPTCHA, hCaptcha o Turnstile.** Non è una preferenza estetica: caricano uno script da un dominio terzo (USA), mandano lì l'IP del visitatore, violano la regola «nessuna richiesta a domini terzi» del §2.3 e aggiungono un responsabile extra-UE all'informativa. Turnstile era stato ipotizzato nella prima stesura del piano infrastruttura ed è stato **scartato per questo motivo**.

ALTCHA non richiede un servizio nuovo, ma **richiede comunque quattro posti** nell'infrastruttura. Vanno assegnati esplicitamente, non dati per scontati:

| Pezzo | Dove vive | Nota |
|---|---|---|
| **Widget** | bundle di `apps/site` (npm `altcha`, MIT) | impacchettato da Vite come i font `@fontsource` — zero richieste a domini terzi, la regola del §2.3 resta intatta |
| **Emissione della challenge** | **n8n CT 110**, secondo webhook (`/webhook/pap-challenge`) | stesso host del POST → coperto dalla stessa regex CORS su nginx, nessun vhost nuovo |
| **Verifica della soluzione** | nodo Code del workflow `/pap-download`, prima di ogni altra difesa | HMAC-SHA256 + SHA-256: logica di `altcha-lib`, zero dipendenze |
| **Chiave HMAC** | pagina **Segreti** + variabile d'ambiente su CT 110 | mai nel repo, mai nel bundle, mai in un file versionato |

**Prerequisito da verificare PRIMA di impegnarsi**: il nodo Code di n8n deve poter usare `crypto`, che su n8n dipende da `NODE_FUNCTION_ALLOW_BUILTIN`. Se risultasse chiuso, la soluzione è **quella variabile d'ambiente su CT 110** — una riga di configurazione su un servizio già gestito, non un servizio in più.

**Protezione replay**: la libreria nuda non impedisce di risolvere il PoW una volta e riusare la stessa soluzione. Si copre con due accorgimenti dentro il workflow, entrambi voluti a prescindere:

- `expires` breve sulla challenge (supportato nativamente da `altcha-lib`);
- **deduplica per indirizzo** su finestra recente — impedisce anche a un utente legittimo di farsi recapitare venti volte la stessa mail.

> ⛔ **Trappola da non prendere.** Su `altcha.org` esiste anche un'**API antispam ospitata da loro**. Non c'entra col protocollo ed è un servizio di terzi: usarla rimetterebbe dentro dalla finestra esattamente il problema per cui abbiamo scartato Turnstile. Nella configurazione non deve comparire nessun endpoint `altcha.org`.

**Valutati e tenuti in panchina**, entrambi da riaprire solo se il form diventasse un bersaglio vero:

- **ALTCHA Sentinel** — backend commerciale self-hosted, da €24/mese a licenza fissa. Aggiunge dashboard, rate limiting, threat intelligence, classificatore ML, protezione replay. Costa più del servizio che protegge, e ogni sua funzione qui è già coperta altrove (Matomo, Grafana, nginx). Usa **lo stesso widget**: passarci un domani significa cambiare un attributo e aggiungere una chiave — non è una scelta che ci incastra.
- **GateCHA** — server MIT in Go che parla il protocollo ALTCHA, alternativa gratuita a Sentinel. Scartato **non per qualità** (metodo di sviluppo curato) ma per livello e maturità: è un **servizio in esecuzione sul percorso critico del form**, di una persona sola, creato a febbraio 2026, a v0.3.2 con 10 stelle. Una libreria abbandonata è codice che possiedi; un server abbandonato è un servizio di rete non più aggiornato da sostituire di corsa. In cambio darebbe multi-sito, gestione API key e dashboard: cose che qui non servono.

### 5.2ter Relay di consegna: Lettermint, e il legame fra quota e captcha

**Relay SMTP esterno obbligatorio**: **non** si spedisce SMTP direttamente da Giganto (reputazione IP, PTR, blacklist = link che finiscono in spam). Scelto **Lettermint** (Paesi Bassi, transito UE), con **due route distinte**: `transactional` per n8n e Grafana, `broadcast` per Listmonk — se le campagne uscissero dalla transazionale, il webhook dei bounce non riceverebbe nulla.

Condizioni verificate il 2026-08-02:

| Piano | Da | Incluse/mese | Overage |
|---|---|---|---|
| Free | €0 | **300** | ❌ nessuno: muro rigido, poi si smette di spedire |
| Starter | €10/mese | 10.000 | €1,10–1,50 / 1.000 |
| Growth | €13/mese | 10.000 | €0,85–1,15 / 1.000 |
| Pro | €15/mese | 10.000 | €0,60–1,10 / 1.000 |

Non esistono pacchetti di crediti una-tantum: dal free si esce solo cambiando piano. Per il picco del lancio 1.0 la leva è **Starter per il mese del lancio, poi ritorno al free** — è un abbonamento mensile, non un impegno.

> **Il tetto e il captcha sono la stessa decisione.** Sul piano free il tetto esiste per costruzione ed è invalicabile: il caso peggiore di un abuso è che si brucino 300 mail e il form si spenga da solo — fastidio, non disastro. **Nel momento in cui si passa a Starter con overage attivo quel muro sparisce**, e uno script che gira una notte può spedire diecimila mail non richieste col nostro dominio come mittente. Il captcha va quindi costruito **prima** del lancio, non durante: serve esattamente quando arriva il traffico vero.

### 5.2quater Applicativo di supporto: listmonk o altro? — ✅ DECISO: listmonk

> **Decisione presa (2026-08-01/02): listmonk**, su **CT 150** (hostname `mailer`, dominio pubblico `tikki.giganto.it`). Il confronto qui sotto resta come traccia del ragionamento. ⚠️ Rispetto a quanto scritto sotto, il ruolo di listmonk si è **ristretto**: fa **solo liste**, non più la mail transazionale del link (che passa da n8n, §5.2). Il requisito (a) è quindi decaduto come criterio di scelta.

Requisiti: **(a)** API transazionale per la mail del link, **(b)** pagina di iscrizione pubblica con double opt-in, **(c)** gestione lista per le campagne future (unsubscribe, bounce, export/cancellazione GDPR), **(d)** self-hosted su Giganto con footprint contenuto.

| | **listmonk** | **Keila** | **Mailtrain** | Custom (Go + DB) |
|---|---|---|---|---|
| Stack | Go, binario singolo + Postgres | Elixir/Phoenix | Node.js | Go |
| API transazionale (a) | ✅ nativa | limitata (focus campagne) | ❌ | da scrivere |
| Pagina iscrizione + double opt-in (b) | ✅ nativi | ✅ (form builder) | ✅ | da scrivere |
| Campagne/lista (c) | ✅ completo | ✅ completo, UI più curata | ✅ ma base | da scrivere |
| Footprint (d) | <100 MB RAM | più pesante (BEAM) | medio | minimo |
| Manutenzione | attiva | attiva, progetto più giovane | **rallentata** | tutta a carico nostro |
| Licenza | AGPL | AGPL | GPL | — |

**Raccomandazione: listmonk.** Copre da solo tutti e quattro i requisiti, è nello **stesso stack Go di `apps/server`** (manutenzione familiare), un binario + Postgres, AGPL come PaP. **Keila** è la riserva se in futuro contasse di più l'editor visuale delle campagne — ma introduce uno stack Elixir estraneo al monorepo. **Mailtrain** scartato (sviluppo rallentato, niente transazionale). La soluzione custom reinventerebbe double opt-in, unsubscribe e bounce handling: da evitare.

Fonti confronto: [Keila vs listmonk (openalternative)](https://openalternative.co/compare/keila/vs/listmonk) · [panoramica piattaforme self-hosted](https://mailflowauthority.com/email-comparisons/open-source-newsletter-platforms) · [Mailtrain vs listmonk](https://stackshare.io/stackups/listmonk-vs-mailtrain)

### 5.3 Impatto e fasi

Il form è **solo mobile**: la landing desktop non ne dipende. Quindi:

- **Fase A** — landing live senza backend: su mobile la CTA degrada a link diretto alla release Latest ("Apri la pagina di download") o resta il form con messaggio "in arrivo". **Decisione consigliata: lanciare con il fallback**, non tenere la landing in ostaggio del backend.
- **Fase B** — backend attivo: form live, mail transazionale col link + CTA di iscrizione (§5.1).

Stato al 2026-08-02: **siamo in Fase A** — `SezioneDesktop.vue` (albero mobile) espone la CTA di ripiego verso la pagina release, non il form.

Cosa serve su Giganto perché parta la Fase B (riepilogo aggiornato — Matomo esiste già, §3.1):

| # | Pezzo | Stato |
|---|---|---|
| 1 | Account **Lettermint** + route `transactional`/`broadcast` + DNS (DKIM, DMARC, CNAME bounces) | ✅ fatto 2026-08-02, `dkim/spf/dmarc=pass` verificati |
| 2 | **CORS** su nginx CT 100 per il webhook n8n (origin `www`, regex che copre anche `/webhook-test/`) | ⏳ patcher pronto, non applicato |
| 3 | **Workflow n8n** del Flusso A (7 nodi) + anti-abuso | ⏳ da costruire |
| 4 | **ALTCHA**: challenge endpoint su n8n, chiave HMAC nei Segreti, `crypto` nel Code node (§5.2bis) | ⏳ da fare — prerequisito `crypto` da verificare per primo |
| 5 | **CT 150 + listmonk + Postgres** su CT 200, vhost `tikki.giganto.it` | ⏳ da fare (indipendente da 2-4: il Flusso A parte anche senza) |
| 6 | **Widget ALTCHA + componente form** in `apps/site` | ⏳ da fare, dipende dal contratto dell'endpoint (§5.2) |

I flussi A (download) e B (liste) sono **indipendenti**: si può mandare live il form senza che listmonk esista, mettendo nella mail una CTA che punterà alla pagina di iscrizione solo quando ci sarà.

### 5.4 Obblighi privacy (dal momento in cui il form va live)

- **Pagina «Privacy»** nel sito: informativa art. 13 GDPR — titolare (Roberto Marchioro), le due finalità con le rispettive basi giuridiche e retention (§5.1), diritti dell'interessato, contatto. Linkata dal form, dalla pagina di iscrizione e dal footer.
- Il claim di `contenuti.md` §10 (*"Nessun 'Privacy Policy' complesso perché non raccogliamo dati"*) **decade**: aggiornare `contenuti.md` quando la Fase B parte.
- **Retention delle execution n8n** — ⚠️ il punto più facile da sbagliare. n8n salva i dati di esecuzione, **email inclusa**: finché non è configurata una retention breve (o il pruning), la frase «non conserviamo il tuo indirizzo» è **falsa** e l'informativa sarebbe sbagliata. Va verificato sul workflow prima di andare live, non dopo.
- **Responsabili esterni da elencare** nell'informativa: **Lettermint** (Paesi Bassi, transito UE — IP di uscita ad Amsterdam, verificato). Con ALTCHA self-hosted **non se ne aggiungono altri**: è la ragione principale per cui è stato scelto al posto di Turnstile.
- Unsubscribe in ogni mail della lista (listmonk lo fa da sé); nessun indirizzo nei log applicativi.
- Il relay scelto è UE: nessun trasferimento extra-UE da dichiarare. Se un domani cambiasse, va detto nell'informativa.

## 6. Piano di lavoro e Definition of Done

Ordine suggerito (ogni fase = PR autonoma verso `main`):

1. **F1 — Token + desktop `#4a`**: custom properties Cloud Dancer, albero `LandingDesktop.vue` (§1.1), restyle/riscrittura componenti (§2.2), layout 1280 con breakpoint interni 1080/900.
2. **F2 — Mobile `#3a`**: albero `LandingMobile.vue` separato, commutazione CSS a 680px, hamburger/drawer, form in modalità fallback (Fase A §5.3).
3. **F3 — Matomo**: preparazione istanza esistente (audit §3.1), tag cookieless, eventi §3.4, disclaimer footer.
4. **F4 — Form email**: workflow n8n + ALTCHA + Lettermint (§5.2), componente form al posto della CTA di ripiego, mail col link + CTA di iscrizione, pagina Privacy, aggiornamento `contenuti.md`. Listmonk/CT 150 è parallelo e non blocca il resto (§5.3).

Checklist di ogni PR (estende quella di `contenuti.md`):

- [ ] Copy conforme a `contenuti.md`; visivo conforme agli handoff (pixel-perfect sui render `4a-desktop.png` / `3a-mobile.png`)
- [ ] Screenshot Playwright a 392 / 680 / 900 / 1280 confrontati coi render; nessuna regressione sull'altro layout (§1.1)
- [ ] Regola viola/ambra rispettata; token `{{…}}` sempre ambra
- [ ] Nessuna richiesta a domini terzi (font e widget ALTCHA impacchettati; unica eccezione runtime: challenge + POST del form verso Giganto — **nessun endpoint `altcha.org`**, §5.2bis)
- [ ] `prefers-reduced-motion`, focus visibile, hit target 44px, contrasti verificati (§2.4)
- [ ] Build statico verde + Lighthouse ≥95 (Perf/A11y/SEO)
- [ ] Matomo: eventi verificati in staging, nessun dato personale negli eventi
- [ ] Form: errori opachi, rate limit testato, ALTCHA verificato lato server, double opt-in verificato end-to-end (solo F4)
- [ ] **Prova dal vivo su telefono reale** (solo F4): form compilato da mobile, mail ricevuta, i 4 link scaricano il file giusto da desktop. I criteri di accettazione del piano infrastruttura sono tutti server-side: questo è l'unico che collauda il flusso vero.
