# Istruzioni di sviluppo — Landing «Scontrino cucito» (Cloud Dancer)

> **Destinatario**: chi implementa la nuova landing in `apps/site` (VitePress/Vue).
> **Fonti vincolanti**: handoff desktop [`desktop/README.md`](./desktop/README.md) (schermata `#4a`), handoff mobile [`mobile/README.md`](./mobile/README.md) (schermata `#3a`), copy [`contenuti.md`](./contenuti.md).
> **Regola di precedenza**: in caso di conflitto, **il copy lo governa `contenuti.md`**, **il visivo lo governano gli handoff**. La vecchia landing «Arioso Atelier» dark (in `archivio/`) resta come riferimento storico: questa nuova direzione la **sostituisce**.
> **Stato**: documento operativo, aggiornato al 2026-07-18.

## 1. Cosa si costruisce

Una **sola pagina** con **due layout della stessa direzione creativa**:

- **Desktop `#4a`** — 1280px di riferimento, container max 1080–1180px, CTA «Scarica l'app» con detection OS.
- **Mobile `#3a`** — 392px di riferimento, colonna singola, CTA «Mandami il link» (form email, vedi §5).

Il breakpoint di collasso è **≤680px**: sotto quella soglia si passa **alla composizione mobile 3a**, non a un desktop compresso (regola esplicita del handoff desktop; breakpoint intermedi 1080/900 descritti lì). Palette **Cloud Dancer** (chiara), design token identici tra le due varianti: implementarli **una volta sola** come CSS custom properties.

### Cosa NON implementare (dagli handoff, ribadito)

- Il runtime di prototipazione (`support.js`) e la struttura `.dc.html`: servono solo markup e stili come riferimento.
- I turni/opzioni scartati (`#t1`, `#t2`, opzioni 1a–1d, 2a–2d): l'unica coppia valida è `#3a` + `#4a`.
- Il cartellino "PaP vs .txt" mobile: **rimosso in revisione**, non reintrodurlo.
- Il QR code nella sezione desktop-da-mobile: scelta deliberata, l'utente è già sul telefono.
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

Non ripartire da zero: la struttura attuale in `theme/components/landing/` copre già gran parte dei blocchi. Mappa indicativa:

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
- **Contrasto**: i toni faint (`#AEAB9F` su `#F1F0EC`) non passano WCAG AA per testo informativo — ammessi **solo** per elementi decorativi (eyebrow ridondanti, meta ornamentali). Ogni testo che porta informazione usa almeno `#8C8A80`/`#54524A` e va verificato.
- **Corpi minimi**: il prototipo usa mono a 8–9px; sul web reale non scendere sotto ~10px effettivi (usare `rem`), a costo di allargare leggermente le label.
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

1. **Dal sito**: l'email serve esclusivamente a recapitare il link di download. Base giuridica: esecuzione della richiesta dell'utente. L'indirizzo **si cancella automaticamente** dopo un tempo breve (es. 30 giorni).
2. **Dalla mail**: il corpo della mail col link contiene una **CTA di iscrizione** (es. *«Vuoi sapere quando debutta una nuova stagione? Avvisami →»*) che porta a una **pagina di iscrizione dedicata** con **double opt-in**. Chi clicca compie un'azione esplicita e la mail di conferma la sigilla: consenso pulito, lista costruita solo con chi la vuole davvero.

La mail del link è l'unico punto in cui la newsletter viene proposta — una volta, senza insistere. È l'approccio più coerente col brand "niente dark pattern": la conversione sarà più bassa di una checkbox, ma ogni iscritto è genuino.

### 5.2 Architettura

Il sito è statico (GitHub Pages): il form fa **POST cross-origin** verso Giganto. L'iscrizione alla lista **non** passa dall'endpoint: avviene solo dopo, via CTA nella mail, sulla pagina pubblica dell'applicativo di supporto.

```
form (apps/site) ──POST──► endpoint Go «mandami-il-link» (Giganto)
                              │  rate limit IP · honeypot · validazione
                              │  errori opachi (pattern PapErrore, CWE-209)
                              └──► API transazionale (app di supporto) ──► relay SMTP ──► mail col link
                                                                                            │
              pagina di iscrizione «stagioni» (double opt-in, app di supporto) ◄── CTA nel corpo della mail
```

- **Endpoint proxy in Go** (stile `apps/server`): non esporre l'applicativo di supporto direttamente. Fa rate limiting per IP, honeypot, validazione sintattica email, CORS ristretto al dominio del sito, risposte **non enumeranti** (sempre "Fatto, controlla la posta" — coerente con il lavoro #512 sugli errori opachi). Niente email in chiaro nei log.
- **Pagina di iscrizione**: è quella pubblica dell'applicativo di supporto (double opt-in nativo, unsubscribe nativo) — non serve proxarla, ha le sue protezioni; va solo brandizzata (template) e linkata all'informativa privacy.
- **Relay SMTP esterno** per la consegna: **non** inviare SMTP direttamente da Giganto (reputazione IP, PTR, blacklist = link che finiscono in spam). Serve un account presso un provider transazionale (preferenza UE) e la configurazione **SPF + DKIM + DMARC** sul dominio mittente. Questa è l'unica dipendenza esterna dell'intero impianto: scelta del provider da fare con Roberto.
- **Anti-abuso**: se il rate limit non basta, aggiungere [Altcha](https://altcha.org) (proof-of-work self-hosted, zero cookie). **Mai reCAPTCHA/Turnstile**: terze parti in contrasto col posizionamento privacy e col §4.

### 5.2bis Applicativo di supporto: listmonk o altro?

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

Piattaforme nuove da creare su Giganto (riepilogo): **1)** listmonk + Postgres, **2)** endpoint Go `mandami-il-link` — Matomo esiste già (§3.1). Più l'account relay SMTP (esterno). Entrambe dietro il reverse proxy TLS esistente.

### 5.4 Obblighi privacy (dal momento in cui il form va live)

- **Pagina «Privacy»** nel sito: informativa art. 13 GDPR — titolare (Roberto Marchioro), le due finalità con le rispettive basi giuridiche e retention (§5.1), diritti dell'interessato, contatto. Linkata dal form, dalla pagina di iscrizione e dal footer.
- Il claim di `contenuti.md` §10 (*"Nessun 'Privacy Policy' complesso perché non raccogliamo dati"*) **decade**: aggiornare `contenuti.md` quando la Fase B parte.
- Unsubscribe in ogni mail della lista (listmonk lo fa da sé); cancellazione automatica delle email non iscritte; nessun indirizzo nei log applicativi.
- Niente trasferimenti extra-UE se il relay SMTP scelto è UE; in caso contrario, va detto nell'informativa.

## 6. Piano di lavoro e Definition of Done

Ordine suggerito (ogni fase = PR autonoma verso `main`):

1. **F1 — Token + desktop `#4a`**: custom properties Cloud Dancer, restyle/riscrittura componenti (§2.2), layout 1280 con breakpoint 1080/900.
2. **F2 — Mobile `#3a`**: collasso ≤680px, hamburger/drawer, form in modalità fallback (Fase A §5.3).
3. **F3 — Matomo**: preparazione istanza esistente (audit §3.1), tag cookieless, eventi §3.4, disclaimer footer.
4. **F4 — Form email**: listmonk + endpoint Go + relay SMTP, mail col link + CTA di iscrizione, pagina Privacy, aggiornamento `contenuti.md`.

Checklist di ogni PR (estende quella di `contenuti.md`):

- [ ] Copy conforme a `contenuti.md`; visivo conforme agli handoff (pixel-perfect sui render `4a-desktop.png` / `3a-mobile.png`)
- [ ] Regola viola/ambra rispettata; token `{{…}}` sempre ambra
- [ ] Nessuna richiesta a domini terzi (font self-hosted; unica eccezione runtime: POST del form verso Giganto)
- [ ] `prefers-reduced-motion`, focus visibile, hit target 44px, contrasti verificati (§2.4)
- [ ] Build statico verde + Lighthouse ≥95 (Perf/A11y/SEO)
- [ ] Matomo: eventi verificati in staging, nessun dato personale negli eventi
- [ ] Form: errori opachi, rate limit testato, double opt-in verificato end-to-end (solo F4)
