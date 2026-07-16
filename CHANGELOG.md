# Changelog — Prompt a Porter

## v0.8.38 — Ritocco: suggerimenti AI per migliorare un prompt (2026-07-16)

> Nuova funzione **Ritocco**: dal pannello di editing, un provider AI configurato dall'utente suggerisce migliorie al prompt — tarate sul modello per cui è scritto — e la versione riscritta può essere accettata come nuova versione. Design in `docs/roadmap/ritocco.md`. La release porta anche un fix di sicurezza sullo script di setup Linux e gli aggiornamenti di dipendenze del giro di triage confluiti nel frattempo.

### Novità

- **Ritocco — suggerimenti AI sul prompt**: se hai configurato almeno un provider AI (**Impostazioni → Provider AI**), il pulsante ✦ nel pannello di editing apre una modale che invia il prompt e le linee guida ufficiali di prompting del suo **modello target** al provider scelto, e propone dei **suggerimenti** insieme a una **versione riscritta** (mostrata a diff). *Accetta suggerimenti* salva la riscrittura come **nuova versione** del prompt, preservando lo storico. Le linee guida per famiglia (Anthropic/OpenAI/Google/generico) sono incluse nell'app: nessuna connessione extra oltre al provider che hai configurato.

### Sicurezza

- **Passphrase updater transiente nel setup Ubuntu** (#479): `scripts/setup-ubuntu.sh` inseriva nel blocco `.bashrc` un `export` della passphrase della chiave Tauri Updater, lasciandola nell'ambiente di *ogni* processo dell'utente per l'intera sessione (leggibile via `/proc/<pid>/environ`). Ora una funzione `pap-sign` la legge dal keyring e la inietta solo nell'ambiente del singolo comando di firma — stesso pattern per-processo già usato su Windows in `sign-release.ps1` (#466/#477) — e fallisce in modo esplicito se il keyring non la restituisce (niente firma con passphrase vuota).

### Manutenzione

- **Provider Gemini configurabile** (fix): il vincolo del database non elencava `gemini` tra i provider ammessi, pur essendo supportato da codice e interfaccia; ora è possibile salvarlo e usarlo (migrazione `V016`).
- **Vitest + coverage-v8 a 4.x** (#492): `packages/shared-schema` e `apps/mcp-server` erano rimasti su Vitest 3.x mentre `apps/client` era già sulla 4.x; le PR Dependabot separate fallivano perché `@vitest/coverage-v8` 4.x richiede `vitest` 4.x e viceversa. Allineati alla 4.1.10 in un bump atomico, con un nuovo gruppo `vitest` in `dependabot.yml` (major inclusi) così i prossimi salti arrivano già accoppiati.
- **Aggiornamento dipendenze** (#485, #486, #487, #489): `golang.org/x/crypto` 0.54.0 (fix di sicurezza upstream: verifica del tipo di chiave in `authorized_keys`, limiti sui parametri delle chiavi DSA, data race in `acme/autocert`), `go-chi/chi` 5.3.1, `regex` 1.13.0 (client Tauri) e `actions/upload-artifact` v7.

## v0.8.36 — App macOS universale (supporto Mac Intel) (2026-07-11)

> Patch mirato: la v0.8.35, prima release con app macOS, girava solo su Apple Silicon e su Mac **Intel** veniva rifiutata con "non è supportato su questo Mac".

### Fix

- **App macOS come binario universale** (arm64 + Intel x86_64): un solo `.dmg` che gira nativo su tutti i Mac, non più solo Apple Silicon. L'auto-update copre entrambe le architetture. (La versione minima di macOS non era in causa: resta 10.13.)

## v0.8.35 — Hardening di sicurezza: backlog sync-server + batch LOW (2026-07-11)

> Chiusura del backlog di sicurezza aperto dalla review 2026-07-05: 8 issue in 4 gruppi (`/gh-triage`), risolte in 4 PR con review adversariale obbligatoria su ogni gruppo. Le review hanno intercettato **4 difetti gravi in cui il fix stesso non proteggeva davvero** — corretti prima del merge — e fatto emergere **2 vulnerabilità pre-esistenti** non nel piano iniziale (un write bypass cross-tenant e un bug che rompeva silenziosamente il WebSocket in produzione).

### Sicurezza

- **Server di sync: hardening completo** (#450, #451, #452, #453, #454): la paternità dei prompt (`AuthorUserId`) è ora forzata lato server e non più accettata dal client (impediva di falsificare l'autore altrui); rate-limiting sul login con equalizzazione dei tempi (anti user-enumeration) e tetto sulla dimensione dei body; CORS ristretto a una allow-list esplicita (con deny-all reale di default, non solo dichiarato); WebSocket blindato (limite di lettura, ping/pong, token via header invece che in URL, algoritmo JWT fissato a HS256); avvio TLS-only salvo dichiarazione esplicita di reverse-proxy.
- **Server di sync: write bypass cross-tenant** (#482): un utente di un workspace poteva sovrascrivere prompt e tag di *un altro* workspace indovinandone l'Id, perché l'aggiornamento filtrava solo per Id e non per workspace (CWE-639). Ora un Id appartenente a un altro tenant viene rifiutato come conflitto, senza toccare la riga. Vulnerabilità pre-esistente emersa durante la review.
- **Client di sync: token nel vault cifrato** (#455): il token di sincronizzazione non è più scritto in chiaro in `preferenze.json` ma custodito nel vault SQLCipher; il client accetta ora solo endpoint `https://`/`wss://` e trasmette il token via `Sec-WebSocket-Protocol` invece che nella query string (dove finiva nei log dei proxy).
- **Server MCP: difesa da prompt-injection indiretta** (#462): i contenuti del vault restituiti al modello sono ora incorniciati in `<untrusted_vault_content>` con neutralizzazione del delimitatore, così un prompt malevolo non può chiudere la cornice in anticipo e iniettare istruzioni.
- **Passphrase di firma fuori dalla process list** (#466): la passphrase della chiave updater non è più passata come argomento della riga di comando (visibile ad altri processi via `/proc/*/cmdline` o `Win32_Process`), ma tramite variabile d'ambiente del solo processo di firma.
- **Batch di irrigidimenti minori** (#462): password del vault minima a 12 caratteri per i nuovi vault; export dei log di debug fuori da `/tmp` e creato già con permessi `0600`; CSP con `object-src 'none'` e `base-uri 'self'`; DSN SQLite della CLI costruito in modo sicuro; `apps/cli/go.sum` versionato (lock supply-chain); `SECURITY.md` allineato alla policy "ultima release".

### Manutenzione

- **Aggiornamento dipendenze** (#470, #471, #483): patch/minor su tauri (2.11.5), rand (0.10.2), sha2 (0.11.0) e il gruppo npm (vite 8.1.4, CodeMirror, svelte-check, zod, tsx, tra gli altri).



> Security review dell'intero progetto, risolta in 3 gruppi (client, server MCP, CI/script). La review adversariale sulle PR ha intercettato due difetti critici in cui il fix stesso non proteggeva davvero — corretti prima del merge.

### Fix

- **Aggiornamento su Linux/macOS: l'app si riavvia da sola** (#443): dopo aver installato un aggiornamento, su Linux (AppImage/.deb) e macOS l'app non ripartiva automaticamente (funzionava solo su Windows). Ora esegue il riavvio esplicito; se non è possibile (es. installazioni `.deb`) mostra un messaggio che invita a riaprirla manualmente.

### Sicurezza

- **Chiavi API rifiutate su vault non cifrato** (#456): salvando una chiave API di un provider su un vault *non* cifrato, l'app ora rifiuta l'operazione con un errore chiaro, invece di scriverla in chiaro nel database.
- **Validazione dell'endpoint dei provider AI** (#457): il `base_url` dei provider viene ora validato con un vero parser di URL — solo `https://` (o `http://` verso `localhost`), niente `file://` né trucchi tipo `http://localhost@host-esterno`. Impedisce che una chiave API finisca, in chiaro, verso un endpoint non fidato. La verifica scatta anche all'uso, non solo al salvataggio, così valgono anche le configurazioni già esistenti.
- **Verifica di integrità dei file scaricati** (#458): il modello di embeddings e la libreria `onnxruntime` vengono verificati con SHA-256 (fail-closed) prima di essere estratti o caricati, e ri-verificati a ogni avvio — non solo al primo download — per impedire la sostituzione della libreria nativa a posteriori.
- **Azzeramento in memoria delle chiavi del vault** (#459): le chiavi di cifratura e le password del vault vengono ora azzerate dalla memoria appena non servono più (incluse le copie esadecimali passate a SQLCipher), riducendo l'esposizione a memory dump.
- **Server MCP: validazione degli input** (#460): gli argomenti dei tool MCP sono ora validati a runtime con schemi condivisi (nuovo pacchetto `shared-schema`), con limiti di dimensione e un tetto aggregato sul payload, così un client MCP malevolo non può più far crashare il server né saturarne CPU/memoria.
- **Pipeline di rilascio e script** (#445, #446, #447, #461): chiusa una injection via nome del tag in `release.yml`; passphrase della chiave di firma non più in chiaro (DPAPI su Windows, keyring su Linux); **corretto il gate anti-segreti di pubblicazione, che passava sempre** (ora fail-closed, con test di regressione); permessi minimi (`contents: read`) sui workflow di build e verifica dei checksum sui download di setup.

## v0.8.33 — Triage: fix Test Golden, export audit e pacchetto .deb Linux (2026-07-04)

> Ciclo di triage: 4 issue segnalate durante i primi test dal vivo su Linux, risolte in 3 gruppi, più le rifiniture per la tray su XFCE.

### Fix

- **Test Golden: controllo embeddings immediato per "cosine"** (#435): eseguendo un golden con similarità `cosine`, l'app verifica **subito** che il modello embeddings sia inizializzato e — se non lo è — segnala l'errore prima di chiamare (e pagare) il provider AI, invece di scoprirlo alla fine.
- **Test Golden: campi obbligatori evidenziati** (#436): nel modale di creazione/modifica, se salvi con **Etichetta** o **Output atteso** vuoti il campo mancante viene ora evidenziato con bordo rosso, così è chiaro cosa manca.
- **Audit Log AI: esportazione CSV** (#438): risolto l'errore *"invalid type: null"* che compariva cliccando **Esporta CSV**.
- **Pacchetto `.deb` Linux: errori lintian** (#437): corretti il contatto Maintainer (con email), la dipendenza da `libc6`, il file di copyright (dalla LICENSE) e la categoria della voce nel menu applicazioni. Restano fuori scope changelog/man page/Section (limiti del bundler Tauri).
- **Linux/XFCE: guida per l'icona nel tray** (#439): documentato in troubleshooting che su XFCE/Xubuntu serve il plugin *Status Notifier* del pannello perché l'icona compaia; il `.deb` dichiara ora la dipendenza `libayatana-appindicator3-1`.

## v0.8.32 — Build Linux riabilitata + fix CLI e linter (2026-07-04)

> Torna la build Linux nella release, e arrivano correzioni a linter e CLI.

### Feature

- **Build Linux nella release** (#432): la pipeline di rilascio produce di nuovo i pacchetti **`.deb`** e **`.AppImage`** (x86_64) per Linux, con auto-update via AppImage. Restava solo Windows; ora Windows + Linux (macOS ancora escluso, richiede la notarization Apple).
- **CLI: `render` più completo e versione corretta** (#430): il comando `pap render` ora **espande i segnaposti globali** `{{global nome}}` leggendo il vault, e **avvisa** quando lascia intatti segnaposti non compilati, globali non trovati o import `{{import "..."}}` (che la CLI non espande) — l'output non è più silenziosamente incompleto. Inoltre `pap version` non mostra più un numero fisso: riflette la versione dell'app.

### Fix

- **Linter: `{{global}}` e `{{import}}` non più segnalati a torto** (#428): la regola PH003 ("nome segnaposto con caratteri non consentiti") segnalava erroneamente le sintassi valide `{{global nome}}` e `{{import "..."}}`. Ora le riconosce come corrette; un nome globale davvero malformato continua a essere segnalato.

## v0.8.31 — Rifiniture Test Golden + fix editor dopo ripristino (2026-07-04)

> Tre correzioni nate dai test dal vivo su Test Golden e Cronologia.

### Fix

- **Inserimento guidato delle variabili golden** (#422): nel modale di creazione/modifica di un test golden, il campo "Variabili di input" non è più un JSON grezzo con placeholder `{}` poco chiaro: ora l'app legge il prompt, estrae i suoi segnaposti `{{nome}}` (esclusi i globali) e mostra **un campo per ciascuno**. Se il prompt non ha segnaposti lo indica; se il contenuto non è leggibile ricade sulla vecchia casella JSON. Aggiunta anche una breve **descrizione** sotto il selettore della funzione di similarità.
- **Selettore del modello a tendina** (#423): eseguendo un golden, per i provider pubblici (Anthropic, OpenAI, Google/Gemini) il campo "Modello" è ora un **menu a tendina** coi modelli noti; per gli altri provider (Ollama, endpoint compatibili) resta un campo di testo libero. Il modello predefinito configurato nelle impostazioni pre-seleziona la voce.
- **Editor aggiornato dopo un ripristino** (#425): ripristinando una versione precedente dalla Cronologia, tornando al tab Editor si vedeva ancora il testo vecchio. Ora l'editor mostra subito il contenuto della versione ripristinata.

## v0.8.30 — Test Golden completo + connettore varianti solo in A-Z (2026-07-04)

> La funzione Test Golden diventa pienamente usabile dall'interfaccia (esecuzione singola con dettaglio, modifica, giudice per llm-judge, costo stimato), e il connettore delle varianti compare solo con l'ordinamento alfabetico.

### Feature

- **Test Golden — esecuzione singola con dettaglio** (#418): il tasto "Esegui" su un singolo golden ora apre un pannello che sceglie provider/modello, lancia il test e mostra il **dettaglio del risultato**: esito Passato/Fallito, **punteggio di similarità** con barra e soglia, provider · modello · latenza · token, **output ricevuto** a confronto con quello atteso, ed eventuali errori del provider. Un badge di ultimo esito compare accanto a ogni golden.
- **Test Golden — modifica** (#419): ora è possibile **modificare** un golden esistente (prima si poteva solo creare o eliminare).
- **Test Golden — giudice per llm-judge** (#420): i golden che usano la valutazione `llm-judge` mostrano i selettori del **provider giudice** (sia in esecuzione singola sia in batch), così vengono eseguiti correttamente invece di dare errore.
- **Test Golden — costo stimato** (#421): ogni esecuzione mostra una **stima del costo** (`~$…`) basata sui token; prezzi indicativi per Anthropic/OpenAI/Gemini, gratis per i modelli locali Ollama.

### Fix

- **Connettore varianti solo con ordinamento "A-Z"** (#417): il rientro e il connettore **↳** delle varianti ora compaiono solo con l'ordinamento alfabetico, dove padre e variante tendono a stare vicini; con gli altri criteri (Recenti/Popolari/Migliori) la lista resta piatta.

## v0.8.29 — Lista: voto medio in "Migliori" + connettore varianti riparato (2026-06-30)

> Due interventi sulla colonna elenco prompt: l'ordinamento "Migliori" ora mostra il voto, e il connettore delle varianti torna a comparire.

### Feature

- **Voto medio nell'ordinamento "Migliori"** (#415): nella lista, ordinando per **Migliori**, la card mostra ora il **voto medio a due decimali** (media degli ultimi 90 giorni, la stessa finestra usata dall'ordinamento) al posto del conteggio usi. Negli altri ordinamenti resta il numero di usi; in densità "compatta" resta il tempo relativo. Formato coerente con la tab Valutazioni (segno `+` sui positivi, `—` se nessun voto nella finestra).

### Fix

- **Connettore varianti di nuovo visibile in lista** (#416 — chiude una regressione di #412): il rientro e il connettore **↳** ("Variante di «…»") non comparivano più perché la query della lista non restituiva il campo `parent_prompt_id` (era esposto solo nel dettaglio). Ora la lista lo include di nuovo, senza modifiche al frontend.

## v0.8.28 — Impostazioni: scroll indipendente tra sidebar e dettaglio (2026-06-30)

> Rifinitura del riordino di v0.8.27: nel menu Impostazioni la barra delle sezioni a sinistra e l'area dei dati a destra ora scorrono separatamente.

### Fix

- **Scroll indipendente nel menu Impostazioni** (#414): la modale aveva un unico contenitore di scroll, così scorrendo il dettaglio scorreva via anche la sidebar. Ora la **lista delle sezioni** (sinistra) e l'**area di dettaglio** (destra) hanno ciascuna il proprio scroll, e la casella di ricerca resta **pinnata** in cima alla sidebar. Introdotto un prop opt-in `corpoFisso` sulla primitiva `Modale` (master-detail a tutta altezza), senza impatto sugli altri modali.

## v0.8.27 — Impostazioni riordinate (sidebar a 6 gruppi) + rifiniture header e ordinamento (2026-06-30)

> Il menu Impostazioni, cresciuto a 10 voci più un accordion eterogeneo, viene riorganizzato in una sidebar a gruppi con un solo modo di navigare. In più, due rifiniture nate dall'uso e un ordinamento "Migliori" più equo.

### Feature

- **Impostazioni riorganizzate** (#413): la sidebar passa da 10 voci flat + l'accordion "Avanzate" (7 sotto-voci eterogenee) a **17 voci di primo livello raggruppate in 6 sezioni di dominio** — *Personalizzazione*, *Contenuti*, *AI*, *Sicurezza & Sync*, *Sistema*, *Aiuto*. Un solo pattern di navigazione (un clic = una pagina, niente più accordion). **Hotkey**, **Linter** e **Segnaposti globali**, prima sepolti dentro "Avanzate", diventano voci di primo livello; **Aggiornamenti** esce da "Sviluppo" in sezione propria; debutta la pagina **Informazioni** (versione runtime, codename, link al progetto).
- **Ordinamento "Migliori" più equo** (#413): a parità di rating medio negli ultimi 90 giorni, l'ordinamento per qualità mette davanti il prompt con **più voti** (un 1.0 su 10 voti batte un 1.0 su 1 voto): più consenso = stima più affidabile.

### Fix

- **Header più pulito** (#413): rimosso il glifo "P" ridondante accanto al codename (restano codename del ciclo + versione reale).
- **Nome vault subito visibile** (#413): a fine onboarding lo store del vault viene ricaricato, così lo switcher mostra immediatamente il nome scelto invece del default "Personale" letto al boot.

## v0.8.26 — Angolo in alto a sinistra, bottoni editor, varianti e grande manutenzione (Go 1.25, vitest 4) (2026-06-29)

> Tre interventi UI nati dai test dal vivo (#402, #403, #404) più un ampio giro di manutenzione delle dipendenze — incluso lo **sblocco della catena Go 1.25**, rimasta ferma due settimane.

### Feature

- **Angolo in alto a sinistra ridisegnato** (#404):
  - **Header in-app**: rimossa la ripetizione di "Prompt a Porter" (era sia nel titolo finestra OS sia nell'header) — ora resta il solo glifo come logo. Il numero di versione non è più la stringa hardcoded errata "v0.8 redesign shell" ma viene letto a runtime (`getVersion()`), quindi è sempre quello reale.
  - **Nome del vault personalizzabile**: lo slot prima fisso a "Personale" ora mostra il **nome del vault** scelto dall'utente in onboarding (persistito in `Preferenze.nome_vault`, retro-compatibile con i vault esistenti che continuano a mostrare "Personale").
  - **Codename "Ago e Filo"**: l'header riserva uno spazio dedicato al codename editoriale del rilascio. L'intero ciclo pre-1.0 porta il codename di laboratorio **"Ago e Filo"**; alla 1.0 subentrerà il primo nome del pool in ordine alfabetico ("Arioso Atelier"). Convenzione in `docs/roadmap/stagioni-e-nomi-rilascio.md`.
- **Varianti evidenziate nella lista** (#403): le card delle varianti di un prompt ora hanno un **rientro** e un connettore **↳** con tooltip "Variante di «titolo padre»", così si vede il legame con il prompt principale. Scelta di design senza riordino della lista: ordinamento, drag-and-drop e multi-selezione restano invariati.

### Fix

- **Bottoni dell'editor finalmente funzionanti** (#402): nella toolbar dell'editor i tre bottoni in alto a destra — **preferito**, **fork**, **export Markdown** — erano stub rimasti dal redesign (facevano solo `console.log`) e i tooltip finivano con un fuorviante "(F8)". Ora sono cablati ai comandi backend già usati dal menu contestuale (`libreria_toggle_preferito`, `prompt_fork`, `prompt_export_markdown`) e i tooltip sono corretti.

### Dipendenze e infrastruttura

- **Catena Go 1.25 sbloccata** (#405 — chiude #334, #339, #396): il CLI passa a Go 1.25 (`modernc.org/sqlite` 1.53). Il blocco di due settimane non era golangci-lint ma l'**action `golangci-lint-action@v6`** che scaricava un binario non-go1.25; bumpata a **v9** il lint gira correttamente sul modulo go1.25. Sistemati anche i rilievi `errcheck` latenti emersi col lint ora funzionante.
- **Migrazione a vitest 4** (#401): bump di `vitest` e `@vitest/coverage-v8` a 4.x con adeguamento delle tipizzazioni dei mock (in vitest 4 `vi.fn()` non è più assegnabile a una firma specifica senza generico).
- **Major Rust**: `rand` 0.9 → 0.10 (#406, API rinominata `OsRng`→`SysRng`, `TryRngCore`→`TryRng`), `zip` 4 → 8 (#344), `ndarray` 0.16 → 0.17 (#346).
- **Gruppi patch-minor e bump minori**: aggiornamenti cargo e npm raggruppati (#398, #399, #407) e singoli — `go-sqlite3` (#395), `actions/checkout` v7 (#397), `@types/node` 26 (#400), `actions/upload-pages-artifact` v5 (#337).

## v0.8.25 — Triage: identifier `com.pap.client`, tab Valutazioni, golden e pulizia CI (2026-06-19)

> Ciclo di triage: 5 issue risolte in 4 gruppi (3 isole indipendenti + 1 cluster coordinato per il file `release.yml` condiviso). #334 (Go 1.25) resta rinviata per il deadlock golangci-lint/go1.25.

### Feature

- **Tab "Valutazioni" + selettore formato in anteprima** (#390): la nuova tab **Valutazioni** mostra l'aggregato dei rating finora orfano (media, conteggio positivi/neutri/negativi, stato vuoto) leggendolo dal comando già esistente `rating_aggregato`. Il selettore del **formato output** nella maschera Compila è stato spostato dalla colonna del form all'header dell'anteprima, come chip compatti (con `role=group` + `aria-pressed` per gli screen reader).

### Fix

- **Creazione golden funzionante** (#382): il bottone **"+ Golden"** nella tab Test golden era uno stub (`console.log`) e non apriva nulla. Ora apre una modale di creazione collegata al comando backend `golden_crea` (etichetta, variabili di input, output atteso, funzione di similarità, soglia di tolleranza), con ricarica della lista e gestione errori inline.
- **Bundle identifier rinominato in `com.pap.client`** (#389): l'identifier Tauri non termina più con `.app` (sconsigliato, bloccava i build macOS). Allineato in modo coerente nei tre client (desktop, MCP server, CLI) così che risolvano lo stesso vault, con **migrazione one-shot non distruttiva** dei dati dalla vecchia cartella `com.pap.app` alla nuova all'avvio (idempotente, non-fatale, e che non segue i symlink).
- **CI: bump `actions/checkout@v5`** (#388): allineate tutte e 9 le workflow per eliminare il warning di deprecazione di Node 20 (`actions/checkout@v4` veniva forzato su Node 24). DEP0040/DEP0190 annotati come deprecazioni transitive non azionabili.
- **Pulizia 5 warning Rust nei test** (#387): rimossi import e variabili inutilizzati, un nome di test non in snake_case e un `assert` sempre vero, tutto in codice di test (nessun impatto di produzione).

## v0.8.24 — Linter personalizzabile (2026-06-19)

> Il linter dei prompt diventa personalizzabile da **Impostazioni → Linter**: silenzia le regole che non ti servono, cambiane la severità e regola le soglie numeriche. Più un fix all'icona della tab Diagnosi.

### Feature

- **Linter personalizzabile** (#381, #383, #384, #385): nuovo pannello in **Impostazioni → Linter** con il catalogo completo delle regole (titolo, codice, severità, descrizione) come unica fonte di verità dal backend.
  - **Attiva/disattiva** per singola regola o per intera famiglia (Lunghezza, Segnaposti, Privacy, Stile, Import).
  - **Severità regolabile**: declassa o promuovi ogni avviso fra Errore / Avviso / Info (es. portare le email da Avviso a Info).
  - **Soglie numeriche editabili**: caratteri massimi (LEN001), caratteri minimi (LEN002) e ripetizioni minime (STY001).
  - **Ripristina** per singola regola o globale; valori fuori scala riportati entro limiti sensati. Le modifiche si applicano subito alla tab Diagnosi e sono salvate localmente sul dispositivo.

### Fix

- **Icona severità nella tab Diagnosi** (#386): le icone mostravano sempre il simbolo "Info" perché il confronto usava valori capitalizzati mentre il backend li invia in minuscolo. Ora Errore mostra il cerchio, Avviso il triangolo, Info la "i". (Il colore del bordo era già corretto.)

## v0.8.23 — Menu contestuale, checklist "Primi passi" e sintassi `{{global}}` (2026-06-19)

> Tre filoni: un **menu contestuale (tasto destro)** context-aware su tutta l'app, il completamento della **guida** con la checklist "Primi passi", e l'allineamento della **sintassi dei segnaposti globali** all'inglese.

### Feature

- **Menu contestuale (tasto destro)** (#374-#380): un menu che cambia in base a dove clicchi, costruito su un primitivo riusabile (posizionamento al cursore con flip ai bordi, submenu, navigazione da tastiera ↑/↓/←/→/Esc, `role=menu`). Superfici coperte:
  - **Card prompt**: Apri · Apri in Compila · Duplica (fork) · Preferito · Sposta in cartella ▸ · **Gestisci tag ▸** (con spunta sui tag già presenti) · Esporta come Markdown · Elimina.
  - **Cartelle** (sidebar): Nuovo prompt qui · Nuova sottocartella · Rinomina (modifica **inline**) · Elimina (i prompt tornano alla libreria).
  - **Editor**: Taglia/Copia/Incolla/Seleziona tutto + Inserisci segnaposto / segnaposto globale / import.
  - **Chip tag**: Filtra per questo tag · Rimuovi dal prompt. **Pillole varianti**: Passa · Promuovi a principale · Elimina.
  - **Selezione multipla** (≥2 prompt): Confronta · Sposta N in cartella · **Aggiungi tag a N** · Esporta N come Markdown · Elimina N (conferma unica).
  - Le azioni "crea tag su prompt" usano due comandi backend dedicati (`prompt_tag_aggiungi`/`prompt_tag_rimuovi`) che non rigenerano una versione del prompt.
- **Guida — checklist "Primi passi"** (#372): widget dismissibile in basso a destra che traccia 5 prime azioni (crea un prompt, aggiungi un tag, usa un segnaposto, compila, usa un import). Rilevamento basato sulle azioni reali, così il vault demo importato non pre-spunta nulla.

### Modifiche

- **Sintassi segnaposto globale: `{{globale nome}}` → `{{global nome}}`** (#371): per coerenza con la keyword inglese `import`. Aggiornati parser (editor/anteprima/compila), inserimento da toolbar, tutta la documentazione utente e il vault demo. (Cambio netto: i prompt esistenti che usano `{{globale ...}}` vanno aggiornati alla nuova forma.)

### Fix

- **Ancora della guida ai segnaposti globali** (#373): l'heading del glossario conteneva la sintassi, generando uno slug che non combaciava col deep-link `#segnaposti-globali` (atterrava a inizio pagina, sia su GitHub sia sul sito). Heading puliti.

## v0.8.22 — Fix: il tour di benvenuto non partiva (2026-06-18)

> Hotfix della guida: il pulsante "Avvia il tour" non faceva nulla (regressione runtime non coperta dalla CI, che valida solo build/type). Il tour di benvenuto ora parte regolarmente.

### Fix

- **Il tour di benvenuto non partiva dal pulsante** (#370): premendo "Avvia il tour" (dall'hub **Guida e aiuto** o dall'invito post-onboarding) non succedeva nulla. L'`$effect` di `Shell` che avvia il tour annullava i `requestAnimationFrame` nel proprio *cleanup*: `consumaRichiesta()` rimette il flag della richiesta a `false` dentro `untrack`, ma `untrack` sopprime solo la raccolta delle dipendenze in lettura, **non** la notifica di scrittura — quindi l'effect si ri-eseguiva e Svelte lanciava il cleanup del run precedente, annullando i rAF *prima* che il tour partisse. Spostata la cancellazione dei rAF in `onDestroy` (solo smontaggio reale di `Shell`), con dedup dei click ravvicinati. Il micro-tour per-feature (#369) non era affetto (schedula i rAF fuori da un `$effect`).

## v0.8.21 — Tour guidato: offerta automatica e micro-tour (2026-06-18)

> Completa la guida interattiva (Fase 1): il tour di benvenuto ora viene **offerto** automaticamente al primo utilizzo e nascono i primi **micro-tour** contestuali per-feature.

### Feature

- **Invito automatico al tour di benvenuto** (#368): la prima volta che si raggiunge la libreria senza aver mai visto il tour (tipicamente subito dopo l'onboarding) compare un **invito non invadente** in basso a destra — *offerto, mai forzato*. "Avvia il tour" lancia il tour spotlight di benvenuto; "Non ora" lo nasconde e non si ripropone più (il tour resta sempre rilanciabile da **Impostazioni → Guida e aiuto**). Realizzato come toast cortese (`role=status`/`aria-live=polite`) che **non cattura il focus**, per non risultare invadente subito dopo la configurazione.
- **Micro-tour contestuali per-feature** (#369): tour brevi (2 passi) offerti alla **prima apertura** di un'area avanzata, per imparare "appena prima del bisogno" invece di tutto all'inizio. Il primo micro-tour è sulla tab **Import componibili / Varianti A/B/C**: alla prima apertura evidenzia le due sezioni e cosa farci. L'infrastruttura è generalizzata (registro dei tour + flag "già visto" versionato per ciascuno), pronta per aggiungerne altri quando le rispettive aree maturano.

## v0.8.20 — Cestino, warning import e guida interattiva (2026-06-17)

> 3 feature (#302 cestino, #303 warning import, guida/tutorial interattivo) + 2 fix di syntax highlighting (#353, #304) + manutenzione dipendenze/CI. #334 (CLI Go 1.25) resta rinviata (golangci-lint non ancora pronto per go1.25).

### Feature

- **Cestino prompt** (#302): i prompt cancellati non spariscono più definitivamente ma finiscono in un **Cestino** (nuova vista nella sidebar, gruppo VISTE) da cui si possono **ripristinare** o **eliminare in modo definitivo**, oltre a uno **svuota** complessivo. Backend `cestino.rs` (`cestino_lista`/`prompt_ripristina`/`prompt_elimina_definitivo`/`cestino_svuota`); la cancellazione era già soft-delete (`DeletedAt`), quindi i dati erano già conservati. Fix correlato: la cancellazione non distrugge più i tag associati, così il ripristino li riporta intatti. L'eliminazione definitiva è una purge fisica in transazione (versioni, import, rating, golden; varianti/fork promossi a indipendenti).
- **Warning cancellazione prompt importati** (#303): cancellando un prompt **referenziato da altri** via `{{import}}`, ora compare un avviso con la **lista dei prompt impattati** e l'opzione di **rimuovere in massa** quegli import prima di cancellare (invece di lasciare riferimenti rotti). Backend `prompt_dipendenti` + `import_rimuovi_da_dipendenti` (rimozione dei token import dal body dei dipendenti, in transazione, con snapshot di versione). Primo taglio: *annulla* oppure *rimuovi gli import e cancella*; la sostituzione con un altro prompt da dropdown è rinviata.
- **Guida interattiva in-app** (#364-#367): nuovo sistema di aiuto a strati. Un pulsante **"?"** sempre visibile nel titlebar apre l'hub **"Guida e aiuto"** (in Impostazioni) con i link alla documentazione raggruppati per tema; badge **"?"** contestuali accanto ai pannelli più ostici (segnaposti, globali, golden/test, import componibili, varianti, ricerca, linter) aprono la pagina giusta nel browser; e un **tour guidato di benvenuto** (driver.js) evidenzia passo-passo le aree chiave dell'interfaccia (lanciabile dall'hub). I link puntano per ora alla documentazione su GitHub, switchabili al sito dedicato in un solo punto (`docs-links.ts`).

### Fix

- **Colorazione dei segnaposti globali** (#353): in editor i segnaposti globali `{{globale nome}}` non venivano evidenziati perché la regex riconosceva solo la forma a parola singola `{{nome}}`. Estesa la regex (`placeholder-highlight.ts`) per riconoscere anche `{{globale <nome>}}` con un capture group come unica sorgente di verità, e aggiunta una decoration distinta (`cm-segnaposto-globale`, accento viola) per distinguerli visivamente dai segnaposti normali. Un `{{globale}}` senza nome resta trattato come segnaposto normale (comportamento documentato e testato).
- **Colorazione degli import parametrizzati** (#304): gli import con modificatori M4 `{{import "X" with k=v}}` / `{{import "X" version=N}}` non venivano colorati per intero perché la regex frontend (`import-tokens.ts`) si fermava al path tra virgolette, non allineata al backend. Allargata la regex per includere i modificatori nello span evidenziato e nel target hover/click; corretta la stessa regex nella utility `estrai-imports.ts` (gli import parametrizzati erano invisibili anche ai pannelli laterali). Test di adiacenza inclusi (due token su una riga non vengono fusi).

### Manutenzione

- **Aggiornamenti dipendenze e CI**: bump `@types/node` 22→25 e `better-sqlite3` 11→12 (dev), e delle GitHub Actions `pnpm/action-setup` 4→6 e `actions/setup-node` 4→6. Aggiunto un `ignore` per `dtolnay/rust-toolchain` in Dependabot (il toolchain Rust è gestito a mano in lockstep tra workflow e `rust-toolchain.toml`).
- **Pulizia canary**: rimosso il job obsoleto `rust-pinfree` dal `dep-canary` (i pin brotli erano già stati rimossi), che faceva fallire il canary a ogni run schedulato.

> 1 issue (#333) dal triage delle migrazioni dipendenze + pulizia dei pin brotli temporanei segnalata dal canary; #334 (CLI Go 1.25) rinviata (ecosistema golangci-lint non ancora pronto per go1.25).

### Fix

- **Migrazione a `rand` 0.9** (#333): in rand 0.9 `OsRng` diventa fallibile (`TryRngCore`), quindi `OsRng.fill_bytes()` non compilava più. Introdotto un helper centralizzato `riempi_random` (`util_random.rs`) e una nuova variante d'errore opaca `PapErrore::RngNonDisponibile`; tutti i 9 call site crittografici (salt del vault, generatori di ID) ora propagano l'errore con semantica **fail-closed** (un OS RNG non disponibile aborta in modo sicuro, mai un buffer non inizializzato). Il bulk import markdown isola il fallimento per-file nel report invece di abortire l'intero batch. Security review superata (nessun leak, salt invariato 16B piena entropia). Sblocca il Dependabot #331.

### Manutenzione

- **Rimossi i pin brotli temporanei** (#352): i pin di #306 (`brotli`/`brotli-decompressor`/`alloc-stdlib`) non servono più — l'upstream si è allineato (`brotli 8.0.4` risolve di nuovo su una sola `alloc-no-stdlib 2.0.4`). Il canary `dep-canary` li ha rilevati come rimovibili (e ormai dannosi, bloccavano la risoluzione delle altre dipendenze); `[build-dependencies]` ora pulito. Chiude le issue auto-generate dal canary #350/#351.

## v0.8.18 — Creazione cartelle + hardening build/CI + dipendenze (2026-06-14)

> 2 fix UI (#301, #307) + un grosso lavoro infrastrutturale di build/CI emerso durante il triage, più 19 aggiornamenti di dipendenze (2 di sicurezza).

### Fix

- **Impossibile creare nuove cartelle** (#301): il pulsante "+" accanto a CARTELLE nella sidebar era inerte — `Sidebar.svelte` passava `bottonAggiungi` a `NavGroup` ma non il callback `onAggiungi`, quindi il click chiamava `undefined()` (no-op in Svelte 5). Aggiunto un nuovo `NuovaCartellaModal` (invoca il comando backend esistente `folder_crea`), con `onAggiungiCartella` cablato in `Sidebar`/`Shell`, validazione nome allineata al backend (non vuoto, no "/", max 100) e logica estratta in `nuova-cartella-logic.ts` con test.
- **"+" inerte nella sezione TAG rimosso** (#307): i tag non si creano stand-alone (nascono assegnandoli durante la creazione/modifica di un prompt), quindi il "+" accanto a TAG era un'affordance morta → rimosso.

### Manutenzione / Sicurezza

- **Architettura CI stabilità+sicurezza** (#309): `Cargo.lock` ora **committato** + `cargo llvm-cov --locked` + toolchain pinnato (`rust-toolchain.toml`, 1.96.0) → build riproducibili, immuni alle pubblicazioni upstream (l'incidente brotli non può più rendere rosse le PR). Aggiunti **Dependabot** (5 ecosistemi, PR validate dalla corsia `--locked`) e un **canary** non-bloccante (`dep-canary.yml`) che testa le dipendenze più recenti e avvisa via issue sia quando qualcosa si rompe sia quando i pin brotli temporanei (#306) si possono rimuovere (#332). `cargo audit` ora audita il lock committato reale.
- **Aggiornamenti dipendenze** (19): tra cui **sicurezza** `golang.org/x/crypto` 0.51→0.53 e `golang-jwt/jwt` 5.2→5.3; major validati dalla corsia `--locked`: `rusqlite` 0.32→0.40, `zip` 2→4, `vite` 6→8 + `@sveltejs/vite-plugin-svelte` 5→7, `typescript` 5→6, `lucide-svelte` 0.460→1.0, `criterion` 0.5→0.8. Migrazioni `rand` 0.9 e `modernc.org/sqlite` 1.52 rinviate a issue dedicate (#333, #334) perché richiedono lavoro di codice.
- **Pin ecosistema brotli** (#306): `alloc-no-stdlib 3.0.0` rendeva incompatibile `brotli 8.0.x` (E0277 nel macro `implement_allocator!`). Pinnato il set ai pre-bump (`brotli=8.0.2`, `brotli-decompressor=5.0.1`, `alloc-stdlib=0.2.2`); ora retto dal lock committato, da rimuovere quando l'upstream si allinea.

## v0.8.17 — Espansione import nella command palette (2026-06-13)

> 1 issue (#299): completa su superficie palette il fix #293/#297 atterrato in v0.8.16.

### Fix

- **`{{import}}` non espanso dalla command palette** (#299): copiando un prompt con `{{import}}` dalla palette (Ctrl+Shift+P), i token restavano grezzi — `compilaECopia` usava la sola sostituzione regex `compila()` senza passare per il backend. Ora la palette invoca `prompt_compila_inline` per espandere gli import prima di copiare (stesso pattern di `CompilaModal`), con anteprima coerente. Logica estratta in un helper puro (`palette-espansione.ts`) con guard di sequenza monotòna che scarta risposte fuori-ordine al cambio rapido di prompt, attesa dell'espansione in corso prima della copia (Ctrl+Enter non copia mai il body grezzo), guard import preciso (`{{import "`) e gestione errore della clipboard senza swallow.

## v0.8.16 — Triage compila/import + demo globali + errori vault + updater (2026-06-13)

> Triage di 4 issue aperte su v0.8.15 (Windows 11): 4 fix, tutti su file disgiunti → quattro isole indipendenti risolte in parallelo (PR #294/#295/#296/#297). Nessuna feature.

### Fix

- **`{{import}}` non espanso nell'output del modale Compila** (#293): compilando un prompt con `{{import}}` il risultato finale non eseguiva l'import, anche se l'anteprima in hover mostrava correttamente il frammento. Il modale derivava segnaposti e output dal `body` **grezzo**; ora invoca il backend `prompt_compila_inline` (stesso percorso di `AnteprimaTab`) per espandere gli import prima di estrarre i segnaposti, ri-espande allo switch di variante con reset eager dello stato (niente frame con segnaposti stantii) e mostra un errore leggibile se l'espansione fallisce.
- **Segnaposti globali assenti dal vault demo** (#292): gli esempi importati con "importa prompt di esempio" non mostravano alcun caso di segnaposto globale valorizzato. Aggiunto un campo `global_placeholders` (retro-compatibile via `#[serde(default)]`) a `ExportV1`: l'import lo semina con UPSERT-on-skip (`ON CONFLICT DO NOTHING`) e il vault demo ora popola `autore/ruolo/azienda/email`, così l'utente li trova già valorizzati in Impostazioni → Segnaposti globali senza setup manuale. Completato anche il lato **export** (`export_pure_filter` interroga `GlobalPlaceholders` nel full-export; gli export per-cartella restano vuoti perché i globali sono workspace-wide), chiudendo il round-trip.
- **Messaggi di errore poco chiari nel cambio password vault** (#290): completa il fix #280. L'arm `Argon2` di `PapErrore` mostrava ancora "derivazione chiave fallita", opaco e non azionabile. Spezzato in due varianti semantiche — `MetadatiDanneggiati` (salt corrotto nei metadati → "ripristina da un backup") e `DerivazioneFallita` (parametri Argon2 invalidi → "errore interno, non dipende dalla password") — con i call site di `vault.rs` (`hex_a_bytes` vs `deriva_chiave`) ricablati alla variante corretta. Display opaco preservato (nessuna fuga di salt/chiavi).
- **`latest.json`: campo `notes` "signing pending" dopo la firma** (#291): dopo `sign-release.ps1` il campo `notes` di `latest.json` — mostrato dal dialog del Tauri Updater — manteneva il testo CI pre-firma ("release draft / binari NON firmati"), fuorviante su una release in realtà firmata e pubblicata. Lo script rigenerava `latest.json` patchando solo `signature`/`url`/`pub_date`; ora sovrascrive anche `notes` con il testo "published" già usato per il body della release, preservando l'output UTF-8 senza BOM e senza toccare la firma updater.

## v0.8.15 — Triage onboarding + tray + errori vault (2026-06-06)

> Triage di 6 issue aperte dal gate test (Windows 11, v0.8.14): 5 fix + 1 feature P0. Raggruppate per file condiviso — un cluster coordinato sull'onboarding (#283/#284/#281, stesso `OnboardingWizard.svelte`) e due isole indipendenti (#285 tray, #280 errori vault) in parallelo (PR #286/#287/#288); la feature P0 #282 in coda al cluster (PR #289).

### Fix

- **Menu contestuale del tray inerte a finestra chiusa** (#285): chiudendo la finestra principale, le voci del menu del tray (es. "nuovo prompt", "impostazioni") non facevano nulla — la finestra `libreria` veniva **distrutta** dalla chiusura, quindi `get_webview_window` restituiva `None` e `mostra_libreria()` era un no-op. Ora `WindowEvent::CloseRequested` sulla finestra `libreria` viene intercettato con `prevent_close()` + `hide()`, mantenendo la webview viva in background: tutte le azioni del tray restano operative dopo la chiusura.
- **Step "personale/team" rimosso dall'onboarding** (#283): la scelta del profilo nel primo step era UI morta (v1.0 è solo personale, `profilo` era comunque forzato a `personale`). Rimossa la card Team, il componente `ProfileCard` inutilizzato e le chiavi i18n morte; primo step collassato in una welcome-card.
- **Prompt di esempio dal vault demo educativo** (#284): "crea prompt esempio" nello step 3 ora importa il vault demo completo (`docs/demo/demo-vault.json`) via il comando esistente `vault_import_json` (modalità `skip`), invece del singolo prompt hardcoded; import non bloccante con log degli errori parziali.
- **"Salta tour" spiega le decisioni applicate** (#281): saltare il tour applicava silenziosamente dei default (profilo personale, hotkey `Ctrl+Shift+P`, nessun prompt di esempio). Ora un modale di conferma elenca questi default prima di procedere.
- **Messaggi di errore leggibili nel cambio password vault** (#280): gli arm `Argon2`/`Db`/`Io`/`Json` di `PapErrore` esponevano testo grezzo della libreria (offset, dettagli SQLite/OS) all'utente. Resi opachi con messaggi italiani comprensibili senza fuga di informazioni; allineati i catch UI (`erroreElimina`, `embErrore`, `globaliErrore`) con lo strip `^Error: ` già usato altrove.

### Feature

- **Avvio automatico con Windows nel tour di onboarding** (#282): aggiunto un toggle "Avvia con Windows" nel terzo step del wizard (OFF di default, nascosto in versione portable), in stile coerente con il box dei prompt di esempio ma con accento cromatico distinto. Riusa il plugin esistente `@tauri-apps/plugin-autostart`; non si attiva se si salta il tour; non bloccante in caso di errore.

## v0.8.14 — Fix gate test round 1 (2026-06-05)

> Correzioni dal primo giro di gate test su v0.8.13 (Windows 11): 8 issue, risolte in parallelo su file disgiunti (PR #276/#277/#278).

### Fix

- **UI congelata dopo creazione prompt** (#275): loop reattivo infinito in `EditorTab` — `prefsSnapshot` era una `$state` letta **e** riscritta nello stesso `$effect`, quindi dipendeva dalla propria scrittura → flush reattivo senza fine, interfaccia bloccata appena si selezionava un prompt (save disabilitato, tab inerti, persisteva a riavvio). Reso `prefsSnapshot` non-reattivo.
- **Errore spurio al primo avvio** (#268): race di startup — l'onboarding invocava i comandi vault prima che `.setup()` registrasse `VaultState`. Aggiunto probe `vault_aperto` con retry prima di montare l'onboarding.
- **Cambio password del vault falliva** (#272): mismatch dei nomi parametri nell'invoke (`vecchia/nuova` invece di `passwordVecchia/passwordNuova`). Il backend (re-key SQLCipher) era già corretto.
- **"Blocca vault" non faceva nulla** (#273): ora `vault_lock` emette `pap:vault-bloccato` e l'app rimonta la schermata di sblocco.
- **Manca "Elimina vault" in UI** (#274): aggiunta azione distruttiva in Impostazioni → Sicurezza con doppia conferma (digitare `ELIMINA`), che usa il comando backend `vault_elimina` esistente.
- **Prompt di esempio non creato** (#271): la preferenza `crea_prompt_esempio` era un flag morto; ora l'onboarding crea davvero il prompt di esempio via `prompt_crea`.
- **Tema light di default al primo avvio** (#269): il default era `auto` (seguiva il SO) → ora `light`.
- **Criteri password non esplicitati** (#270): aggiunti criteri visibili (min 8 caratteri) + checklist inline; "Continua" disabilitato finché non soddisfatti.

## v0.8.13 — Import/Export JSON nella GUI + avvio automatico (2026-06-05)

> I comandi backend `vault_import_json`/`vault_export_json` (export lossless completo del vault) erano registrati e testati ma irraggiungibili dall'interfaccia: in **Impostazioni → Dati** si poteva importare solo Markdown. Esposti entrambi nella GUI. Aggiunta inoltre l'opzione di avvio automatico al login.

### Feature

- **Avvio automatico al login + avvio nel tray** (#264): nuova sezione
  Impostazioni → **Sistema** con toggle "Avvia all'avvio del computer" (plugin
  ufficiale `tauri-plugin-autostart`: Windows registry Run per-utente — no
  admin —, macOS LaunchAgent, Linux `.desktop`). Quando attivo, al login l'app
  parte **ridotta nel tray** (lanciata con arg `--minimized` → finestra
  nascosta, icona nel tray). L'opzione è **esclusa nella versione portable**
  (il path dell'exe non è stabile): rilevata via marker `.portable` accanto
  all'eseguibile, aggiunto al pacchetto portable in `release.yml`.

### Portabilità ed export

- **Import/Export JSON esposti in Impostazioni → Dati** (#262): nuova card "Importa JSON" (file picker `.json` + selettore modalità conflitti `skip`/`overwrite`/`rename` via `seg-control` a11y + report nuovi/aggiornati/conflitti/errori) e card "Esporta Vault → JSON" (download del backup lossless: storico versioni, tag, cartelle, fork). Prima il JSON era raggiungibile solo via comando, mai dalla UI. Backend invariato (già coperto da test); estratto `nomeFileExport()` in `util/dati-export.ts` (riusato anche dall'export zip, +3 test) e helper locale `scaricaBlob()`; intro Dati aggiornata con link a guida Markdown e formato JSON.

## v0.8.12 — Audit sicurezza + export lossless + installer per-utente (2026-06-02)

> Esito di un audit di sicurezza (`/security-bounty-hunter` sul sync server Go) e di una code review Rust completa (`/rust-review` su tutta la codebase del client). Nessuna vulnerabilità critica/remota trovata; chiusi un controllo di autorizzazione mancante lato server e una serie di hardening/silent-failure lato client. 10 PR atomiche (#239-248), una per modulo (ogni file toccato una sola volta). Inoltre: completato il round-trip dell'export JSON (cartelle + varianti/fork), aggiunto un vault demo per gli screenshot, e rimosso l'installer MSI a favore del solo NSIS per-utente (no admin).

### Sicurezza

- **Server sync — autorizzazione PromptTags (CWE-639)** (#239): il loop `PromptTags` in `pushDelta` inseriva associazioni prompt-tag senza verificare che gli ID appartenessero al workspace del chiamante (a differenza dei loop Tags/Prompts). Un client autenticato poteva creare associazioni cross-workspace. Aggiunta validazione di ownership in transazione + test di regressione. Exploitabilità reale bassa (FK ON + ID a 32 bit casuali + nessun percorso di disclosure), ma controllo mancante che il codice intorno intendeva applicare.
- **Client — validazione `visibility` al trust boundary** (#241): `sync_applica_delta` ora valida `visibility ∈ {private, workspace}` sui record provenienti dal server (skip-with-log) invece di lasciar abortire l'intero delta sul CHECK del DB; existence-check `COUNT(*)` → `EXISTS` con errore DB propagato.
- **Client — `api_key` non esposta via Debug** (#242): rimosso `derive(Debug)` dalle struct provider (Anthropic/OpenAI/Gemini) e da `ProviderConfigItem`/`Input`; un futuro `{:?}`/log non compilerà più. (Nessun leak attivo: il comando frontend già azzera la chiave.)
- **Client — `preferenze.json` con permessi 0600 su Unix** (#248): il file contiene `sync_token` in chiaro; ristretti i permessi al solo owner (best-effort). TODO documentato: spostare i segreti nel keychain OS.
- **Client — cap anti-bomba import a tenuta** (#240): chiusi due bypass del limite `MAX_OUTPUT_BYTES` (1 MB) in `compila_ricorsivo` — il check ora avviene prima di accodare l'espansione del child e include la coda dopo l'ultimo import. Rilevante perché i body possono arrivare via team sync. + 2 test di regressione.

### Robustezza

- **Lock poison-tolerant su stato a lunga vita** (#244, #245): tutti i `Mutex::lock().unwrap()` di `VaultState` (11 site) e `EmbeddingsState` ora recuperano il guard anche su mutex avvelenato (`unwrap_or_else(into_inner)`). Un panic mentre si tiene il lock non crasha più a cascata ogni operazione successiva.
- **Transazione su promozione variante** (#247): i 3 `UPDATE` dello swap variante↔principale in `promuovi_pure` girano in `BEGIN/COMMIT` con `ROLLBACK` su errore (prima un errore a metà lasciava i prompt in stato corrotto).
- **Errori DB non più mascherati** (#247): le existence-check `query_row(...).unwrap_or(false/None)` in `rating`/`fork`/`cartelle` distinguono ora "riga assente" da un vero errore DB (propagato).
- **Errori di scrittura import non più silenziati** (#246): in `import_pure` gli errori su `PromptTags`/`PromptVersions` finiscono in `report.errori`; il rebuild FTS post-bulk-import logga l'errore; i due `unreachable!()` sui rami modalità-conflitto → `Err` esplicito.
- **`audit::registra` osservabile** (#247): il fallimento dell'INSERT di audit è loggato invece di scartato (firma invariata).
- **Guardia su input malformati** (#244): `hex_a_bytes` non panica più su stringa di lunghezza dispari; `version=N` in overflow `i64` dà un errore chiaro invece di mappare a `0`; `app_data_dir()` in setup ritorna errore invece di panicare l'avvio (#248); `remove_file` del DB orfano in `vault_crea` propaga l'errore (#244).

### Qualità

- **Pulizia clippy** (#243): `linting.rs` — `format!` inutile → `.to_string()`, `sort_by` → `sort_by_key(Reverse)`, `HashMap::with_capacity`, rimossa variabile a rami identici (bug di pluralizzazione dormiente). `EmbeddingsState` ora implementa `Default` (#245); rimosso campo morto `AnthropicUsage.input_tokens` (#242); `regression::esegui_pure_con_provider` (solo-test) marcata `#[cfg(test)]` (#248); fix di un doctest che falliva la compilazione (#246).

### Portabilità ed export

- **Round-trip cartelle nell'export JSON** (#185): l'export ora popola `folders[]` (ordinate per `path`, padre prima dei figli) e l'import le ricrea prima dei prompt risolvendo `parent_folder_id` (parent mancante → root); il prompt importato imposta `folder_id` (cartella assente → NULL). Prima le cartelle andavano perse nel round-trip, contraddicendo la garanzia "lossless" del formato.
- **Round-trip varianti e fork nell'export JSON** (#186): aggiunti 4 campi opzionali a `prompts[]` — `parent_prompt_id`, `is_variant`, `variant_label`, `fork_of_prompt_id`. Import a due passate: tutti i prompt prima, poi risoluzione dei FK self-referenziali (mappa id→id effettivo + fallback su prompt già nel vault; riferimento irrisolvibile → NULL). Additivo e retro-compatibile (`#[serde(default)]`, nessun bump `schemaVersion`). L'export è ora lossless tranne audit log e chiavi di sicurezza (esclusi per design).
- **Vault demo per screenshot** (#184): nuovo `docs/demo/demo-vault.json` (17 prompt — 1 variante + 1 fork —, 7 cartelle con nesting, 8 tag, 3 versioni storiche, import fra prompt) per popolare l'app prima di catturare gli screenshot del sito. Coperto dal test `import_export::test::demo_vault_importa_pulito`. Documentazione del formato aggiornata con cartelle e campi varianti/fork.

### Distribuzione

- **Rimosso l'installer MSI, solo NSIS per-utente + portable** (#254): la release Windows non produce più il bundle `.msi` (WiX installa per-machine in `Program Files` con UAC, contro la filosofia local-first single-user). Il job Windows di `release.yml` usa ora `--bundles nsis`: l'installer `…-setup.exe` resta per-utente (`installMode: currentUser`, `%LocalAppData%`, **nessun privilegio admin**) e il portable `.zip` è invariato. L'updater è preservato — con `createUpdaterArtifacts` riusa il bundle NSIS, quindi `latest.json` + `setup.exe.sig` sono generati regolarmente.

### Note

- Falso positivo confermato e **non** modificato: il "deadlock" embeddings segnalato in review — in `unload_se_idle` il guard di `last_used` è un temporary rilasciato prima di acquisire `inner`, i due lock non sono mai tenuti insieme.
- Deferiti consapevolmente (basso valore / alto churn su app local single-user cifrata): conversione `filter_map(|r| r.ok())` nei list-helper residui, N+1 bounded in `libreria::lista_pure`, split di funzioni lunghe.

## v0.8.11 — v1.0 M2-M8: quality gate finali + documentazione utente (2026-05-19)

> Release di chiusura della roadmap v1.0 "Personale". Sette milestone roadmap (M2-M8) consolidate in una singola release: a11y, recupero UI, sintassi import evoluta, editor doppia vista, markdown import/export, gate di coverage e documentazione utente completa.

### Feature

- **Sintassi `{{import}}` evoluta (M4)**: `version=N` per importare una versione specifica dal repository di versioni, `with k=v` per override dei segnaposti del prompt importato. Combinabile: `{{import "path" version=3 with tono=formale}}`. Intellisense CodeMirror sui titoli dei prompt durante la digitazione (#209-211).
- **Editor doppia vista Sorgente/Compilato (M5)**: split-view nell'editor con form valori e compilazione inline live via `prompt_compila_inline`. Pattern dual-source body consolidato (#213-214).
- **Markdown import/export (M6)**: parser front-matter YAML + walker ricorsivo di cartelle `.md`; export bulk vault/cartella → zip Deflated. Compatibile con Obsidian/Foam (#216-219).
- **Recupero UI Fase 4 (M3)**: backend `prompt_promuovi_variante` nuovo + ripristino feature UI sparite nel redesign v0.8 (#203-207).

### Qualità

- **A11y svelte-check con `--fail-on-warnings` (M2)**: 18 warning → 0; gate CI attivo (#197-202).
- **Coverage gap chiusi (M7)** — gate CI di tutti i target raggiunti (#221-226):
  - Rust client `apps/client/src-tauri/`: **74.14% → 80.24%** line, gate alzato 70 → 80.
  - TypeScript client `lib/*.ts`: **37.78% → 81.05%**, gate 70 attivato (`vitest --coverage`).
  - MCP server `lib/*.ts`: **0% → 100%** su funzioni pure estratte, gate 80 attivato.
  - Pattern consolidato: `_pure(&Connection)` / `_impl(&VaultState)` per refactor Tauri command verso unit-test diretto (10 moduli toccati).

### Documentazione utente (M8)

Nuovi documenti in `docs/utente/` (#228-232):

- `getting-started.md` — installazione cross-platform, onboarding 3-step, primo prompt e prima compilazione.
- `glossario-sintassi.md` — reference unificato: `{{nome}}`, `{{globale nome}}`, `{{import "path" version=N with k=v}}`, 11 codici linter LEN/PH/PII/STY/IMP.
- `scorciatoie-tastiera.md` — tabelle complete (global, shell, palette, modali, editor CodeMirror, autocomplete M4, tray) + correzione riferimenti errati a `Ctrl+N`/`Ctrl+S` (non esistono in v1.0; autosave 2s + bottone `+ Nuovo`).
- `troubleshooting.md` — FAQ: SmartScreen, Gatekeeper, AppImage/FUSE, password vault non recuperabile, hotkey su macOS/Linux Wayland, segnaposti malformati, import non risolti, sync, backup, debug log.
- `casi-uso/` con 7 ricette pronte all'uso: email-professionale, code-review, summarize-articolo, riscrittura-tono, brainstorm-idee, traduzione-tecnica, commit-message. Ogni ricetta include prompt completo, esempi di input/output, varianti, anti-pattern.
- `docs/utente/README.md` indice riorganizzato con sezione "Inizio rapido" per nuovi utenti.

### Stato roadmap v1.0

Con questa release, tutte le 8 milestone della roadmap "v1.0 Personale" (`docs/roadmap/v1.0-personale.md`) sono completate: M1 (signing + updater) ✅, M2 (a11y) ✅, M3 (recupero UI) ✅, M4 (import evoluta) ✅, M5 (editor doppia vista) ✅, M6 (markdown) ✅, M7 (coverage) ✅, M8 (docs) ✅.

### Numeri

- **~35 PR** mergiate (#197-#232 incl. doc/bump).
- **578/578** test Rust passati, **157/157** TS client, **21/21** MCP.
- Coverage gate finali: Rust **80%**, TS **70%**, MCP **80%**, svelte-check **0 warning**.

---

## v0.8.10 — v1.0 M1: Authenticode signing + Tauri Updater attivi (2026-05-16)

> Prima release pubblica con **codice firmato Authenticode** (Certum Code Signing Open Source) e **auto-update Tauri** funzionante. SmartScreen Windows mostra il publisher "Open Source Developer, Roberto Marchioro" invece di "Unknown publisher".

### Feature

- **Authenticode signing** su tutti i binari Windows (`Prompt a Porter.exe` portable, NSIS `setup.exe` per-user installer, MSI `.msi`). Catena cert: Open Source Developer Roberto Marchioro → Certum Code Signing 2021 CA → Certum Trusted Network CA 2 → Certum Trusted Network CA. Timestamp Certum (`http://time.certum.pl`) → la firma resta valida anche dopo la scadenza del cert (2027-05-15).
- **Tauri Updater** integrato (plugin `tauri-plugin-updater` v2) con verifica Ed25519 minisign su `latest.json` + ogni binario. Endpoint: GitHub Releases `latest/download/latest.json`. Policy: **check on-demand utente** (no auto-check al boot, no telemetria), disabilitabile da Impostazioni → Sviluppo → Aggiornamenti.
- **NSIS installer per-user** (`installMode: "currentUser"`) — installazione senza UAC, in `%LOCALAPPDATA%\Prompt a Porter\`.
- **Pipeline pre-signing locale** (script `scripts/sign-release.ps1`): CI produce asset unsigned in release draft, maintainer firma da workstation Windows con SimplySign Desktop logged-in + ri-firma Ed25519 + rigenera `latest.json`, poi promuove a Latest published. Vedi `docs/contribuire/release-signing-workflow.md`.

### Scoperte architetturali (cfr. ADR `authenticode-signing.md`)

- **SimplySign Cloud è GUI-only**: 4 iterazioni di test CI hanno confermato che non esiste un metodo headless documentato per il login (gli argomenti CLI suggeriti da fonti community datate non sono supportati in SimplySign Desktop 2026). Adottato approccio C — pre-signing locale, scartato in fase di ADR iniziale ma unica opzione praticabile oggi.
- **Tauri Updater + Authenticode interagiscono**: i `.sig` Ed25519 generati dalla CI sono calcolati sui binari unsigned; dopo `signtool` il contenuto cambia (~+10 KB di firma) e i `.sig` diventano stale, rompendo l'updater. Lo script `sign-release.ps1` rigenera Ed25519 + `latest.json` post-Authenticode.

### Documentazione nuova

- `docs/architettura/decisioni/authenticode-signing.md` — ADR completo (amended 2026-05-16 con scoperte test pipeline)
- `docs/contribuire/release-signing-workflow.md` — procedura step-by-step maintainer
- `docs/contribuire/setup-tauri-updater-keys.md` — generazione chiavi una tantum
- `docs/utente/auto-update.md` — guida utente finale + FAQ
- `scripts/setup-windows.ps1` — setup guidato workstation di signing (Win 10/11 IoT)
- `scripts/setup-ubuntu.sh` — setup guidato dev workstation Linux

### Numeri

- **15+ PR** mergiate per M1 (signing + updater + setup scripts + fix iterativi)
- 4 test tag (`v0.8.9-test*`) per validazione end-to-end pipeline (lasciati come draft pre-release per riferimento)
- Test backend invariati (416), copertura 74.14%

### Skip versions

`v0.8.9` saltata: il numero è stato consumato dai tag di test (`v0.8.9-test1` → `-test6`). Prossima versione "production" = `v0.8.10`.

### Closes

ADR `authenticode-signing.md`, milestone M1 v1.0 (Personale)

---

## v0.8.8 — Hotfix CATASTROFICO: editor input bloccato (2026-05-11)

> ⚠️ **v0.8.6 e v0.8.7 sono DIFETTOSE — non usare.** Aggiornare a v0.8.8.

### Fix critico

- **#170 editor input bloccato** (PR #175) — In v0.8.6/v0.8.7 era impossibile scrivere nel titolo o nel body editor: ogni keystroke veniva immediatamente cancellato. Root cause: la PR #168 (fix #167 data-loss) aveva introdotto in `DetailPane.svelte` un `$effect` su `promptId` che leggeva sincronamente le variabili reattive (`titolo`/`body`/`descrizione`/`dirty`/`dettaglio`) per snapshot. Svelte 5 traccia queste letture come dipendenze: ogni assegnazione (utente digita) ri-eseguiva l'effect → `caricaDettaglio` ricaricava dal DB sovrascrivendo l'input. Fix: `untrack()` da `svelte` per leggere le variabili di snapshot SENZA creare dipendenza reattiva. La sola vera dipendenza resta `promptId`. Comportamento del fix #167 mantenuto.

### Numeri

- **1 PR** mergiata in main (#175) + 1 PR di bump (questa)
- 126 vitest pass invariati
- 0 errori svelte-check
- Severity HOTFIX: v0.8.6 e v0.8.7 marcate come ⚠️ DIFETTOSE sulla release page GitHub

### Closes

#170

---

## v0.8.7 — Sezione Sviluppo + Debug log Telescope-like (2026-05-11)

> Nuova **sezione Impostazioni → Sviluppo** con funzionalità diagnostica "Debug log": logger strutturato su file con rotazione, toggle ON/OFF runtime, viewer in-app con filtri, e export ZIP per allegare a issue GitHub. Architettura non reinventa la ruota: usa `tauri-plugin-log` ufficiale come backbone.

### Feature

- **Sezione Sviluppo** in `ImpostazioniModal` con icona `FlaskConical`. Card "Debug log" con:
  - Toggle ON/OFF (preferenza `debug_log_abilitato`, livello DEBUG/WARN, runtime via cmd `debug_log_imposta_livello` — no riavvio richiesto)
  - Info path cartella + lista file rotati con size/mtime
  - Bottoni **Apri cartella** (xdg-open/open/explorer), **Esporta ZIP per issue** (metadata.txt + tutti i `pap.log*`), **Pulisci log**
  - **Viewer tail in-app** (`LogViewer.svelte`): auto-refresh 2s, filtri livello (TRACE/DEBUG/INFO/WARN/ERROR), regex case-insensitive, highlight colori, bottoni Pause/Refresh/Clear

### Backend

- Dep `tauri-plugin-log = "2"` (ufficiale Tauri team, MIT/Apache-2) per file rotation + JS bridge
- Init plugin in `lib.rs::run` con targets `LogDir + Stdout + Webview`, max file size 5MB, `RotationStrategy::KeepAll`
- Helper `carica_debug_log_abilitato` + `.setup` applica `log::set_max_level` da preferenza
- Nuovo modulo `debug_log.rs` con 5 cmd Tauri: `_info`, `_apri_cartella`, `_pulisci`, `_esporta_zip`, `_leggi(n_righe)` + parser format tauri-plugin-log
- 9 unit test (parser, raccogli file, format ISO)
- Frontend `main.ts`: `attachConsole()` pipe `console.*` → file backend

### Path file log

- Linux: `~/.local/share/com.pap.app/logs/pap.log`
- Windows: `%APPDATA%\com.pap.app\logs\pap.log`
- macOS: `~/Library/Logs/com.pap.app/pap.log`

### Numeri

- **3 PR** mergiate in main (#171 backbone, #172 UI, #173 viewer) + 1 PR di bump (questa)
- 126 vitest pass, 9 nuovi unit test backend (441 totali)
- 0 errori svelte-check
- ~1370 LOC totali (~700 Svelte UI, ~430 Rust backend, ~280 Svelte component)
- Zero codice di file rotation/management custom (delegato a `tauri-plugin-log`)

---

## v0.8.6 — Fix data-loss su switch prompt + hardening sicurezza (2026-05-11)

> Patch urgente per **#167** (data-loss catastrofico su switch prompt) + chiusura audit settimanale fallita (#164, #166).

### Fix critico

- **#167 data-loss su switch prompt via meta-link** (PR #168) — sequenza riproducibile: aperto prompt A, click meta-link a B, click su A nella lista → A veniva sovrascritto con body di B. Root cause: 2 bug interagenti.
  - `EditorTab`: dispatch CodeMirror programmatico per sync body al cambio `promptId` triggava `docChanged=true` → `dirty=true` fantasma in DetailPane senza input utente reale. Fix: flag `ignoraProssimoCambio` blocca propagazione su update programmatico.
  - `DetailPane.salva()`: chiudeva su variabili reattive (`promptId`/`body`/`titolo`/`dettaglio`). `$effect` su cambio promptId chiamava `salvaManuale()` prima che `caricaDettaglio` aggiornasse le reattive → `invoke prompt_aggiorna` con `id=NUOVO` e `body=VECCHIO`. Fix: nuova `salvaConId(args...)` con parametri espliciti, `$effect` cattura snapshot sincrono e ordina `salvaConId(precedente) → caricaDettaglio(nuovo)` in closure async.

### Hardening sicurezza

- **CI security-audit verde** (PR #164, #166) — bump Go 1.25.10 (fix GO-2026-4971 net.Listen panic con NUL byte), `chi/v5` v5.2.5 (fix GO-2026-4316 open redirect), `golang.org/x/crypto` v0.51.0 (4 CVE in ssh/agent). Rename module path `apps/server` da `anthropics/...` a `robertomarchioro/...`. Issue tracking #165 chiusa.

### Numeri

- **3 PR** mergiate in main (#164, #166, #168) + 1 PR di bump (questa)
- 126 vitest pass, 0 errori svelte-check
- Run security-audit dispatch dal branch fix: 4/4 job verdi

### Closes

#164 #166 #167

---

## v0.8.5 — Editor UX + tray fix + segnaposti globali (2026-05-10)

> Sprint patch su v0.8.4 con 3 PR: editor "Salva manuale" + autosave senza snapshot, tray icon singola Win + modelli AI consistenti, e nuova feature **segnaposti globali** (issue #159).

### Feature

- **#159 segnaposti globali** (PR #162) — sintassi `{{globale nome}}` per placeholder riutilizzabili tra prompt diversi con valore di default editabile. Backend: V015 migration `GlobalPlaceholders(Name PK, Value, UpdatedAt)` + 3 cmd Tauri (`globale_placeholder_lista`/`aggiorna(UPSERT)`/`elimina`). Frontend: regex parser estesa (`/\{\{\s*(globale\s+)?(\w+)\s*\}\}/g`), `compila`/`contaCompilati` con 3° arg `valoriGlobali`, `CompilaModal` pre-fill dei globali con auto-UPSERT al copy, nuova sub-sezione "Segnaposti globali" in Impostazioni → Avanzate (CRUD UI), bottone Globe in MarkdownToolbar.

### Fix

- **#156 + #158 editor UX** (PR #160) — `DetailPane` ora separa "Salva manuale" (con snapshot versione, default) da "Salva bozza"/autosave (senza incremento `Version`). Bottoni Save/Trash nell'header, `dirty` state tracking, `onBeforeUnload` warning, snapshot automatico al cambio prompt. Backend `editor::aggiorna_prompt` accetta nuovo flag `crea_snapshot: bool` con SQL `Version = CASE WHEN ?8 THEN Version + 1 ELSE Version END`.
- **#144 tray icon doppia** (PR #161) — root cause finale: `app.trayIcon` in `tauri.conf.json` auto-creava una TrayIcon **in aggiunta** a quella creata manualmente da `TrayIconBuilder` in `lib.rs`. Rimosso il blocco `app.trayIcon` dalla config (single-instance plugin di v0.8.3 non bastava perché le 2 icone erano nello stesso processo).
- **#157 modelli AI inconsistenti** (PR #161) — `Sidebar.svelte` e `RightRail.svelte` hardcodavano sotto-insiemi diversi della lista modelli. Entrambi ora iterano `MODELLI_TARGET` (constante condivisa) garantendo lista identica ovunque.

### Numeri

- **3 PR** mergiate in main (#160, #161, #162) + 1 PR di bump (questa)
- **126 vitest pass** (di cui 34 in `template.test.ts`, +19 nuovi per globali)
- **3 nuovi unit test backend** in `segnaposti_globali` (3/3 verdi)
- **0 errors** svelte-check (3742 files)
- 1 nuova migration `V015__segnaposti_globali.sql` (totale 15)

### Closes

#144 #156 #157 #158 #159

---

## v0.8.4 — Retry release v0.8.3 (fix CI workflow) (2026-05-10)

> **Stesso codice di v0.8.3** (i 7 bugfix Win11 elencati sotto). Il tag v0.8.3 era stato pushato ma `release.yml` aveva fallito (run 25626291738) a causa di un'incompatibilità tra `--no-bundle` (introdotto in PR #147) e `tauri-action` (che cerca artifact bundle MSI/NSIS). Il fix workflow è in PR #154; v0.8.4 ri-trigga la pipeline release con `args: ""` (bundle attivi).

### Cambia rispetto a v0.8.3

- **release.yml**: rimosso `--no-bundle` da `windows-latest` matrix (PR #154). Ora la release pubblica 5 asset Windows: `Prompt-a-Porter_0.8.4_x64_en-US.msi` + `.sig` + `Prompt-a-Porter_0.8.4_x64-setup.exe` + `.sig` + `Prompt-a-Porter-portable-windows-x64-v0.8.4.zip`. Solo il portable .zip è documentato nel release body — gli installer MSI/NSIS sono "bonus" non documentati ma utilizzabili.

I 7 bugfix Win11 di v0.8.3 (PR #148-#152) sono inclusi senza modifiche. Vedi entry v0.8.3 sotto per il dettaglio.

---

## v0.8.3 — Bugfix Win11 multi-issue (2026-05-10)

> Patch su v0.8.2 per 7 issue Win11 segnalate dopo la release portable. Risolte in **5 PR distinte** con focus sulle cause root, non sui sintomi. Schema DB invariato, no breaking change utente. Backend cambia solo aggiungendo `tauri-plugin-single-instance` e un campo `body_preview` al payload `PromptCard`.

### Fix

- **#140 + #141 density UI** (PR #148) — i 3 chip label "Compatta / Comoda / Anteprima" occupavano ~210 px orizzontali sulla colonna stretta (320 px). Sostituiti con 3 bottoni icon-only quadrati (Rows3 / Rows2 / LayoutList lucide). Inoltre la modalità "Anteprima" non funzionava: il flag `abilitata: false` era un placeholder F3 PR-B mai cancellato, e il backend `PromptCard` non includeva il body. Aggiunto `body_preview: String` al payload (SUBSTR truncato a 800 char server-side, max ~80 KB extra per 100 card).
- **#142 sizing barre** (PR #149) — `--titlebar-h` e `--statusbar-h` erano referenziati in 3 punti (Shell.svelte, TitleBar.svelte, StatusBar.svelte) ma **mai definiti** in `tokens.css`. Senza `var(name, default)` la regola `height` collassava → barre prendevano altezza naturale del contenuto. Aggiunti i 2 token con i valori esatti del prototipo (`36px` / `28px`).
- **#143 vault startup error** (PR #151) — `vault_unlock` lanciava `VaultGiaAperto` ("Il vault è già aperto") se la connessione era già cached in memoria backend, e Onboarding mostrava errore bloccante. Helper `isErroreVaultGiaAperto(e)` riconosce il messaggio e procede a `oncompletato()` (no-op success): per l'utente "vault già aperto" = "sbloccato".
- **#144 + #146 tray Windows** (PR #152) — installato `tauri-plugin-single-instance v2.4.2` come primo plugin del Builder: la seconda istanza al lancio focalizza la finestra esistente e termina (no più doppia tray icon). Inoltre `on_menu_event` per "nuovo_prompt" e "impostazioni" ora dopo show+focus emette event Tauri (`tray:nuovo-prompt`, `tray:apri-impostazioni`) verso il webview; Shell.svelte registra listener via `@tauri-apps/api/event` e li traduce in `apriModale({tipo:"impostazioni"})` o `dispatch CustomEvent("pap:nuovo-prompt")`. ListPane ascolta quest'ultimo e chiama `creaNuovoPrompt`.
- **#145 "+ Nuovo" prompt creation** (PR #150) — bottone era cabled a placeholder `console.log("F8 modale crea prompt")`. Funzione `creaNuovoPrompt()` async che invoca `prompt_crea` (cmd backend esistente) con dati minimi default (titolo "Nuovo prompt", body vuoto, visibilità "private", folder = cartella corrente filtrata se ≠ "__nessuna__"); dispatch `pap:lista-mutata` per refresh + `onSelezionaPrompt(id)` per aprire DetailPane in editing immediato.

### Numeri

- **5 PR** mergiate in main (#148, #149, #150, #151, #152) + 1 PR di bump (questa)
- **113 vitest pass** invariati (no nuovi test richiesti per i fix)
- **0 errors** svelte-check (3742 files)
- `cargo check` verde con nuova dep `tauri-plugin-single-instance v2.4.2`
- 1 nuova dep Cargo (~150 KB binari ulteriori, trascurabile vs ~30 MB Win portable)

### Closes

#140 #141 #142 #143 #144 #145 #146

---

## v0.8.2 — Layout fix completo (CSS grid come prototipo) (2026-05-10)

> Patch su v0.8.1 per issue #137 (layout ancora sbagliato dopo i fix v0.8.1). Refactor totale di `Shell.svelte` da `paneforge` percentuali a **CSS grid puro** come da prototipo originale (`docs/architettura/redesign/prototype/redesign.css`). Risolve 5 sintomi con una sola correzione architetturale. Schema DB invariato, no breaking change utente.

### Fix

- **#137 layout grid prototipo** — il fix v0.8.1 (#132) di `html/body/#app` width/height non bastava: `.shell-root` continuava a non stretchare le barre e `paneforge` percentuali (20%/26%) ignoravano le proporzioni del prototipo (248px/320px fissi). Refactor completo:
  - `.shell-root` ora ha `width: 100vw + height: 100vh + overflow: hidden` esplicito → barre full-width su massimizzazione.
  - `.shell-body` usa CSS grid `grid-template-columns: var(--col-sidebar, 248px) 1px var(--col-list, 320px) 1px minmax(0, 1fr)` — stesso pattern del prototipo originale.
  - Quando ListPane è collapsed, lo slot resta **visibile a 36px con un bottone `>>` (`.list-restore`) per riaprirlo** invece di sparire del tutto.
  - Icona collapse cambiata da `>>` a `<<` (semantica corretta: collassa verso sinistra, non espande).
  - Drag handler manuali via `pointermove`/`pointerup`: mouse a destra ⇒ pane sinistro cresce (paneforge era confuso da `collapsedSize=0` e dava drag invertito).
- **Nuovo store `lib/stores/shell-layout.ts`** — persistenza `{colSidebar, colList}` in `localStorage["pap.shell.layout"]`, default 248/320, clamp MIN/MAX (sidebar 200-480, list 240-560). Pattern identico a `sidebar-collapsed.ts` e `densita.ts`.

### Numeri

- 1 PR (#138) merge squash, 1 commit di bump (#139)
- 113 vitest pass invariati
- 0 errors svelte-check (3742 files, +1 store)
- Bundle: app `index.js` 68.7 kB gzip (+1 kB vs v0.8.1, drag handler manuale)
- Closes #137

### Note tecniche

- `paneforge` resta installata come dep ma non più importata (tree-shake la esclude). Cleanup `package.json` deferito a PR separata per non mischiare scope.
- `listCollapsed` è **in-memory only** (non persistito): alla riapertura app la lista riparte espansa, come da prototipo. Le larghezze `colSidebar`/`colList` invece persistono.

---

## v0.8.1 — Bugfix patch redesign UI (2026-05-09)

> Patch immediata su v0.8.0 per 3 bug post-rilascio segnalati in issue. Nessun cambiamento funzionale, solo fix di rendering layout, controllo collassa colonna lista, e display shortcut OS-aware.

### Fix

- **#132 layout root sizing** — TitleBar e StatusBar non si ridimensionavano correttamente su massimizzazione finestra Win11 (l'utente vedeva le barre tronche rispetto al main grid). Aggiunto `html, body, #app { width: 100%; height: 100% }` in `app.css`: `.shell-root` (height 100vh) ora ha parent stretch corretto e tutti i grid items (TitleBar / shell-body / StatusBar) si stretchano alla piena width.
- **#133 ListPane collapse non funzionante** — bottone `>>` in ListPane chiamava un placeholder `console.log`. Ora `Shell.svelte` espone `listPaneRef` con API paneforge (`collapse / expand / isCollapsed / resize`), `<Pane>` ListPane ha `collapsible` + `collapsedSize={0}`. Riapertura via drag PaneResizer adiacente.
- **#134 shortcut display OS-aware** — i glifi macOS-only (⌘ ⌃ ⇧ ↵) erano hardcoded nei `title` e `<kbd>` di TitleBar/StatusBar/PaletteModal/CompilaModal, visibili anche su Windows/Linux. Nuova utility `lib/util/shortcut.ts` con `fmtShortcut(combo)` che rileva piattaforma da `navigator.platform` e ritorna stringa formattata (mac: `⌘K`, `⌃⇧P`, `⌃↵`; win/linux: `Ctrl+K`, `Ctrl+Shift+P`, `Ctrl+Enter`). Frecce ↑↓←→ ed Esc restano universali.

### Numeri

- 1 PR (#135) merge squash, 3 commit (1 per issue)
- **113 vitest pass** (era 98 in v0.8.0, +15 nuovi test su `fmtShortcut` con mock `navigator.platform`)
- 0 errors svelte-check (3741 files)
- Closes #132 #133 #134

---

## v0.8.0 — Redesign UI completo (2026-05-09)

> **Fasi F8-F11 chiuse, 17 sub-PR mergiate**, redesign v0.8 completo. Nuova Shell 3-pannelli + 5 modali primitive-driven + Onboarding consolidato + a11y WCAG 2.1 AA. Net **−7 249 righe codice** vs v0.7.0 nonostante 6 superfici nuove. Schema DB invariato, no breaking change utente.

### Highlights

- **Nuova Shell 3-pannelli + 5 modali** (F8) — Sidebar / ListPane / DetailPane via `paneforge` resizer; modali Compila / Insight / Regressioni / Impostazioni / Palette tutte basate su una primitive `Modale.svelte` riusabile (backdrop scrim + ESC + click-outside + body-scroll-lock + focus trap manuale). Store globale `modale.svelte.ts` discriminated union per stato singleton.
- **Routing semplificato a 2 stati** (F9) — `App.svelte` riscritta: `Onboarding` (caricamento / setup wizard / sblocco vault cifrato) → `Shell`. Default UI è ora la nuova Shell, non più `Libreria`. Cancellate `Libreria.svelte` (2418 righe) + 4 superfici `Auth*` + `DemoComponenti` + 8 superfici legacy orfane (CompilatorePrompt / ConfrontoPrompt / CronologiaPrompt / EditorPrompt / Impostazioni / Insight / Regressioni / ConflittoSync) — totale **−10 749 righe legacy**.
- **WCAG 2.1 AA + 2.3.3** (F10) — focus indicator unificato `:focus-visible` con `--focus-ring` token, focus trap manuale in Modale (Tab/Shift+Tab cycling + return-to-trigger), aria-label su tutti gli icon-only button, contrast tema chiaro 4.5:1+ (`--text-muted` 0.48→0.42, `--accent-team` 0.55→0.48 nei 3 toni), reduced-motion override globale W3C C39 pattern.
- **⌘K Palette globale** + **⌘, Impostazioni** (F8 PR-D1/E) — shortcut globali registrati in Shell. Palette interna sostituisce la window separata legacy (mantenuta per hotkey OS-level). Filtri avanzati slider hybrid alpha persistiti in localStorage.
- **Bundle vendor chunks** (F11 PR-C) — `vite.config.ts` `manualChunks` splitta `codemirror` (185 kB gzip) / `lucide-svelte` (23 kB) / `diff2html` (12 kB) come chunk vendor stabili. App update ora re-scarica solo `index.js` (67.60 kB gzip) invece del monolite (288 kB gzip).
- **Token medi (V014)** in InsightModal — proxy char-count Body / 4 ≈ token cl100k come 7° KPI in Panoramica.

### Numeri

- **17 sub-PR** mergiate F8-F11 (#113-#129)
- **~10 749 righe** legacy cancellate (Libreria + Auth* + Demo + 8 superfici orfane)
- **~3 500 righe** nuove (Shell + 5 modali + Onboarding + Modale primitive + store + tokens)
- **Net: −7 249 righe codice**
- **6 modali nuove** (Compila / Insight / Regressioni / Impostazioni / Palette + primitive)
- **98 vitest pass** (era 88 in v0.7.0, +10 sidebar-collapsed)
- **Bundle gzip total: ~320 kB** (on-target ≤ +100 kB delta vs v0.7.0)
  - `index.js` app: **67.60 kB** (era 287.59 kB pre-split)
  - `codemirror.js`: 184.93 kB (vendor cache)
  - `icons.js`: 23.00 kB (vendor cache)
  - `diff.js`: 12.21 kB (vendor cache)
  - `index.css`: 32.69 kB
- **WCAG 2.1 AA** (contrast text ≥ 4.5:1, UI ≥ 3:1) + **2.3.3** (Animation from Interactions) raggiunti

### Documentazione aggiornata

- `docs/roadmap/redesign-v08/blueprint-F8.md` — primitive Modale + 5 sub-PR modali
- `docs/roadmap/redesign-v08/blueprint-F9.md` — routing/cleanup + Onboarding consolidato
- `docs/roadmap/redesign-v08/blueprint-F10.md` — a11y baseline + keyboard nav + tema chiaro contrast + reduced-motion
- `docs/roadmap/redesign-v08/blueprint-F11.md` — cleanup finale + test + bundle + perf

### Out of scope (rinviato)

- **Profiling Chrome DevTools manuale** drag-resize 60fps + first-paint DetailPane ≤ 300ms — ottimizzazioni preventive applicate (CSS containment + active feedback resizer); profilo dedicato richiede sessione browser interattiva
- **Setup `vitest-plugin-svelte`** per testare runes Svelte 5 + render() su Modale/Onboarding — richiede dep ~30KB, deferito post-release
- **DELETE `OnboardingWizard.svelte`** (assorbito da `Onboarding.svelte` come step "setup") + **DELETE `CommandPalette.svelte`** (window legacy per hotkey OS-level) — refactor architetturale post-v0.8.0
- **E2E test Playwright** + **screen reader smoke** (NVDA/VoiceOver) — manuale, suite non esistente
- **Workspace switcher login/logout funzionale** (placeholder F2 mantenuto)

Tutti i punti deferiti tracciati in [`docs/roadmap/rinvii.md`](docs/roadmap/rinvii.md).

---

## v0.7.0 — Refactor coverage + sprint quick wins import/cartelle (2026-05-08)

> **Sprint v0.7.0 chiuso 6/6 step.** Mix di hardening (refactor `import_export.rs` per testabilità, coverage push 71→74%, gate CI 65→70) e quick wins su flussi di cartelle, import componibili, target model custom. Schema DB invariato, no breaking change.

### Highlights

- **Coverage push 71→74% + gate CI 65→70** — refactor `import_export.rs` estrae `export_pure(conn)` e `import_pure(conn, &ExportV1, modalita)` come helper testabili senza Tauri State. `import_export.rs` 28.95% → 78.87% (+49.92pt). +19 test (10 import_export + 9 embeddings edge case).
- **Esporta singola cartella** — bottone ⬇ nel sb-folder-actions della sidebar Libreria. Filtra Prompts via `Folders.Path` LIKE prefix per il sotto-albero. Comando `vault_export_folder_json(folder_id)`. Folders nel payload restano `Vec::new()` (roundtrip → v0.8).
- **Custom free-text target model** — `<Select>` dei 9 preset sostituito con `<input list>` + `<datalist>` HTML5 nativo. Modelli custom (`claude-opus-5`, `gpt-6`, locali) accettati. Backend `Prompts.TargetModel` invariato.
- **Hover preview import + Ctrl+click navigazione** — i token `{{import "..."}}` nell'editor hanno highlight + tooltip nativo CodeMirror (titolo + snippet body 240 char) + Ctrl/Cmd+click per aprire il prompt target. Nuovo modulo `lib/codemirror/import-tokens.ts` + comando `prompt_resolve_import_preview(path)`.
- **Cross-prompt linting (IMP004)** — nuova regola Info-level che mostra "Questo prompt è importato da N altri" usando `PromptImports` come grafo inverso. Skip se prompt non salvato.
- **Markdown export con YAML front-matter** — bottone "Esporta MD" nel detail pane. Front-matter compatibile Jekyll/Hugo include `title`, `description?`, `target_model?`, `visibility`, `version`, `created_at`, `updated_at`, **`imports: [...]`** parsati dal body (riproducibilità).

### Numeri

- **416 unit test backend** (era 382 a inizio sprint, +34 nuovi: 19 Step 1, +5 Step 2, +10 Step 4 vitest, +4 Step 5, +6 Step 6)
- **17 vitest frontend** (era 7, +10 nuovi su import-tokens parser)
- **Coverage globale 74.14% line / 77.69% function** (era 71.02%/75.61%)
- **CI gate alzato da 65% a 70%** (margine +4pt)
- 6 PR mergiate (#89-#94), tutte con CI verde su `lint-and-test` + `rust-test`

### Documentazione aggiornata

- `docs/operativo/coverage.md` — nuovo snapshot 74.14%, target ridefinito a 78% globale entro v0.8
- `docs/roadmap/rinvii.md` — 6 item Fase 3 atterrati: Esporta cartella, Custom target model, Hover preview import, Ctrl+click navigazione, Cross-prompt linting, Markdown export con front-matter
- `docs/operativo/release-signing-macos.md` — runbook Apple Developer notarization (creato pre-sprint, attesa enrollment KYC)

### Out of scope (rinviato)

- **`embeddings.rs` 41% → 70%** — refactor con HTTP mock per logica di download, target v0.8 (sblocca coverage 78% globale)
- **`embeddings_backfill.rs` 10% → 50%** — dipende da Step embeddings refactor
- **Roundtrip cartelle export/import** (popolare `folders` nel payload + ricreare al target con conflict resolution) — scope v0.8
- **Filtro Libreria sidebar per modelli custom** — query `DISTINCT TargetModel` dal DB invece di iterare i preset
- **UI declutter generale** — sessione dedicata anticipata dall'utente (post-v0.7)
- **Promozione variante a principale** (swap main↔variant) — rivedere col declutter UI
- **CLI `pap test`** + **MCP `pap_test_prompt`** — Fase 5 con MCP HTTP/SSE
- **Signing Authenticode Windows** + **Apple notarization macOS** — gate amministrativo, runbook esistente

Tutti i punti deferiti tracciati in [`docs/roadmap/rinvii.md`](docs/roadmap/rinvii.md).

---

## v0.6.0 — Hardening + secondo sprint quick wins (2026-05-07)

> **Sprint v0.6.0 chiuso 6/6 step.** Mix di hardening (coverage push, riload Session, gate CI) e quick wins UX dai rinvii Fase 3/4 (inline marker linter, statistiche prompt più importati + lint health, vista Confronto varianti, configurazione per-categoria linter). Schema DB invariato.

### Highlights

- **Coverage push 60→65 gate** — alzato il floor CI da 60% a 65% line coverage; coverage globale **71.02%** post-step. Aggiunti 17 unit test edge case su `vault.rs` (43.50% → 50.44%), `audit.rs` (51.89%) e `libreria.rs` (59.28%).
- **Riload automatico Session post idle-unload** — risolve il limite Fase 3 Step 10: dopo idle-unload (default 5min) la ricerca semantica non degrada più a FTS-only. Nuova `assicura_session_caricata(rt, vault)` chiamata da `cerca_semantica` prima di `compute_embedding_opt`. Refactor `init_session_pure` idempotente.
- **Inline marker CodeMirror sul linter** — gli issue PH/PII/IMP/STY/LEN ora compaiono inline nel body con underline wavy colorato per severità + tooltip nativo `code: messaggio`. Nuovo `lib/codemirror/lint-markers.ts` con `StateField<DecorationSet>` + `setLintIssues` effect.
- **Statistiche "Prompt più importati" + "Lint health %"** — vista Insight estesa con 2 nuove metriche: top 10 prompt importati da altri (grafo inverso `PromptImports`) + percentuale prompt senza issue + breakdown top 5 categorie. Tutto client-side, no dati escono.
- **Vista "Confronto varianti" multicolonna** — bottone "Confronta tutte" nella riga delle pillole varianti del detail pane: apre `ConfrontoPrompt` (Step 4 Fase 4) preselezionando principale + tutte le varianti. Riuso completo del componente esistente.
- **Configurazione per-categoria linter** — nuova sezione **Impostazioni → Linter** ✏️ con 5 toggle (LEN/PH/PII/STY/IMP), persistenza in `localStorage`. Backend `prompt_lint` accetta `categorie_disabilitate: Option<Vec<String>>` e filtra a posteriori.

### Numeri

- **382 unit test backend** (era 351 a inizio sprint, +31 nuovi: 17 Step 1 + 3 Step 2 + 7 Step 4 + 4 Step 6)
- **7 nuovi vitest frontend** (Step 3 lint-markers)
- **Coverage globale 71.02% line / 75.61% function** (era 70.27%/75.05%)
- **CI gate alzato da 60% a 65%** line coverage
- 0 svelte-check errors
- 6 PR mergiate (#81-#86), tutte con CI verde su `lint-and-test` + `rust-test`

### Documentazione aggiornata

- `docs/operativo/coverage.md` — nuovo snapshot, target ridefinito a 75% globale entro v0.7
- `docs/roadmap/rinvii.md` — 4 item Fase 3 atterrati (Riload Session, Inline marker, Stats import+lint health, Linter per-categoria) + 1 item Fase 4 atterrato (Confronto varianti multicolonna)

### Out of scope (rinviato)

- **`embeddings.rs` / `import_export.rs`** sotto 50% coverage — refactor con HTTP mock + scenari round-trip JSON/CSV, target v0.7
- **Promozione variante a principale** (swap main ↔ variant) — nessuna domanda forte, in attesa
- **CLI `pap test`** + **MCP `pap_test_prompt`** — Fase 5 con MCP HTTP/SSE
- **Custom free-text target model** — quick win futuro
- **Esporta singola cartella** — quick win futuro
- **Editor doppia vista Sorgente/Compilato integrata** — quick win futuro
- **Hover preview import** + **Ctrl+click navigazione** — quick win futuro
- **Cross-prompt linting** (chi importa X) — quick win futuro
- **Markdown export con front-matter imports** — quick win futuro
- **Signing Authenticode Windows** — decisione costo aperta

Tutti i punti deferiti tracciati in [`docs/roadmap/rinvii.md`](docs/roadmap/rinvii.md).

---

## v0.5.0 — Quick wins UX + 5° provider AI (2026-05-07)

> **Sprint v0.5.0 chiuso 6/6 step.** Polish UX su feature di Fase 4 (varianti, rating, golden, sort) e completamento del set provider AI con Google Gemini. Schema DB invariato, nessun breaking change.

### Highlights

- **Pannello Provider AI in Impostazioni** — sezione dedicata 🤖 con card per ognuno dei 5 provider supportati (Anthropic, OpenAI, OpenAI-compat, Ollama, Gemini). Form modale con API key write-only (placeholder "Lascia vuoto per non modificare"), base URL, modello default, switch abilitato. Sblocca utenti che dovevano configurare provider via SQL diretto.
- **Bottone "+ Variante" nell'Editor** — crea varianti A/B direttamente dall'editor del prompt corrente, senza dover tornare alla Libreria. Auto-naviga al detail pane della nuova variante.
- **Modale "Aggiungi nota" su rating 👎/😐** — il campo `Note` (V013, già nello schema) ora viene popolato. 👍 salva subito senza friction; per voti negativo/neutro si apre una modale opzionale con textarea (max 500 caratteri).
- **"Esegui tutti i golden" batch** — bottone "Esegui tutti (N)" nel pannello Test esegue tutti i golden in sequenza con progress inline `Esecuzione X/Y…` e summary finale colorato `✓ N passed · ✗ M failed · ⚠ K errore`.
- **Sort "Migliori" by rating medio** — nuovo ordinamento nel dropdown della Libreria. Ordina per `AVG(Rating)` ultimi 90 giorni; prompt senza rating in fondo (COALESCE -2). Tie-breaker `UseCount` + `UpdatedAt`.
- **Provider Google Gemini** — 5° e ultimo provider pianificato per Fase 4. Endpoint `/v1beta/models/{model}:generateContent`, auth via header `x-goog-api-key`, parser concatena `candidates[0].content.parts[*].text`, tokens da `candidatesTokenCount`. Modelli supportati: `gemini-2.5-flash`, `gemini-2.5-pro`.

### Numeri

- 351 unit test backend (era 339 post-v0.4.0, +12 nuovi: 12 su Gemini, 2 su libreria sort qualita)
- 6 PR mergiate (#74-#79), tutte con CI verde su lint-and-test + rust-test
- 0 breaking change su schema DB (V013 invariato, nessuna nuova migrazione)
- 0 svelte-check errors

### Documentazione aggiornata

- `docs/utente/regression-testing.md` § Setup provider include riga Google (Gemini); § Limiti noti marcati ✅ atterrati: UI Provider Config, batch golden, Gemini
- `docs/utente/rating-prompt.md` § Limiti noti marcati ✅ atterrati: modale nota, sort qualità

### Out of scope (rinviato)

- **Vista "Confronto varianti" dedicata** multicolonna — riusabile via Confronto fianco-a-fianco esistente
- **Promozione variante a principale** (swap main ↔ variant) — nessuna domanda forte, in attesa
- **CLI `pap test`** + **MCP `pap_test_prompt`** — Fase 5 con MCP HTTP/SSE
- **Inline marker CodeMirror** sul linter — quick win futuro
- **Statistiche "Prompt più importati" / "Lint health %"** — atterrabili in v0.6
- **Signing Authenticode Windows** — decisione costo aperta

Tutti i punti deferiti tracciati in [`docs/roadmap/rinvii.md`](docs/roadmap/rinvii.md).

---

## v0.4.0 — Workflow Avanzati & Quality Assurance (2026-05-07)

> **Fase 4 client-first track chiusa.** 6/8 step atterrati (1, 2, 3, 4, 5, 8). Step 6 (approval workflow) e 7 (RBAC cartelle) rinviati a Fase 5: dipendono da workspace team in produzione e non danno valore aggiunto in single-user. Nessun breaking change su DB/format export rispetto a v0.3.x.

### Highlights

- **Golden examples + regression testing** ⭐ *differenziatore strategico*: trasforma il prompt da testo a contratto comportamentale verificabile. Crei un golden con `input_vars` + `expected_output` + similarity function (`cosine`/`exact-match`/`regex`/`llm-judge`), il client esegue il prompt contro un provider AI (Ollama locale o Anthropic/OpenAI/OpenAI-compat con API key), calcola la similarità, salva l'observation. Vista "Regressioni" in Libreria con tabella drift per (prompt × provider × model) e export CSV.
- **Diff tra versioni** — pannello CronologiaPrompt mostra diff inline e side-by-side fra qualunque due versioni del prompt. Riusa jsdiff (BSD-3) con preserve dei segnaposti `{{...}}` come token unitari. Export Markdown via clipboard.
- **Confronto fianco-a-fianco** di prompt diversi — Cmd/Ctrl+click in Libreria per selezionare 2+ prompt, modale con metadata + body in colonne. Toggle "Diff colorato" riusa il componente `VersionDiff` di Step 3.
- **Varianti / A-B testing** — duplica un prompt come variante B/C/Z (auto-etichetta), ognuna con UseCount/rating/versioning indipendenti. Pillole varianti cliccabili nel detail pane. Riggancio automatico al grandparent (no chain transitive).
- **Fork / Clone** con tracciabilità — clona un prompt team nel tuo workspace privato. Banner "Fork di X" cliccabile per navigare all'originale. Resiliente al soft-delete dell'originale.
- **Rating dopo l'uso** — toast post-copy con 3 emoji (👎/😐/👍), append-only con timestamp. Aggregato % positivi nel detail pane con badge colorato (verde/giallo/rosso).

### Quality gate Fase 4 (Step 9)

- **Coverage globale 69.91% line + 74.30% function** (era 60.12%/67.64% post v0.3.0)
- **Tutti i 6 moduli Fase 4 sopra il target ≥70%**: rating 95.24%, regression 91.27%, fork 91.14%, varianti 90.36%, similarity 86.13%, provider_ai 77.17%
- 337 test backend (era 169 inizio Fase 4)
- 7 stress test sentinel anti-regressione (varianti 100, fork 50, rating 100 misti)
- CI gate `--fail-under-lines 60` invariato (margine prudente)

### Schema DB (V008-V013)

Tutte le migrazioni additive, vault esistenti vengono migrati al primo unlock:

- **V008** `prompt_goldens` — casi di test salvati per prompt
- **V009** `prompt_run_observations` — storia esecuzioni append-only
- **V010** `provider_config` — API key in DB cifrato SQLCipher
- **V011** `prompt_varianti` — `Prompts.ParentPromptId/VariantLabel/IsVariant`
- **V012** `prompt_fork` — `Prompts.ForkOfPromptId`
- **V013** `prompt_ratings` — feedback discreto -1/0/+1 con `Note?` + `UsedWithModel?`

### Documentazione nuova

- [`docs/utente/varianti-prompt.md`](docs/utente/varianti-prompt.md)
- [`docs/utente/fork-prompt.md`](docs/utente/fork-prompt.md)
- [`docs/utente/rating-prompt.md`](docs/utente/rating-prompt.md)
- [`docs/utente/regression-testing.md`](docs/utente/regression-testing.md)
- [`docs/architettura/schema-dati.md`](docs/architettura/schema-dati.md) esteso con V008-V013

### Statistics

- 14 PR mergiate dalla v0.3.0 (#58-#71): #58-#64 Step 8 (golden+regression), #65 Step 3, #66 Step 1, #67 Step 4, #68 Step 5, #69 Step 2, #70 doc roadmap, #71 quality gate
- 337 unit test Rust totali (+154 da v0.3.0)
- 0 errori type check, 0 vulnerabilità note

### Out of scope (rinviato)

- **Step 6 — Approval workflow** e **Step 7 — RBAC cartelle**: gate workspace team, naturalmente Fase 5 con server in produzione
- **Provider Google (Gemini)**: 4 provider su 5 implementati. Quick win `v0.5.0`
- **Modale "Aggiungi nota" su rating negativo**: campo `Note` già nello schema, manca solo UI
- **Sort by quality** "Migliori prompt" in Libreria
- **CLI `pap test`** + **MCP `pap_test_prompt`**: rinviati `v0.5.0`/Fase 5
- **UI Provider Config in Impostazioni**: oggi via SQL/MCP
- **"Esegui tutti i golden" batch**: quick win `v0.5.0`

Tutti i punti deferiti tracciati in [`docs/roadmap/rinvii.md`](docs/roadmap/rinvii.md).

---

## v0.3.0 — Intelligenza & Authoring (2026-05-06)

> **Fase 3 chiusa.** Tutti gli 11 step della roadmap completati: ricerca semantica (sqlite-vec + ONNX), linting, cartelle, prompt componibili, statistiche, quality gate. Nessun breaking change su DB/format export rispetto a v0.2.x.

### Highlights

- **Ricerca semantica + ibrida** — Comprendi i prompt per significato, non solo per keyword. RRF pesata (alpha configurabile) fra FTS5 lessicale e sqlite-vec semantico. Modello locale 384 dim (`paraphrase-multilingual-MiniLM-L12-v2`, ~118 MB), download lazy al primo uso. Niente cloud, niente leak.
- **Linting in tempo reale** — 11 regole su body (LEN/PH/PII/STY/IMP) con pannello Diagnosi nell'editor. Cattura PII (email/CC/API key), segnaposti malformati, ripetizioni, import non risolti, cicli, profondità eccessiva.
- **Cartelle gerarchiche** — Modello dati piatto + `Path` denormalizzato. Drag & drop, rinomina inline, sposta cascata, anti-ciclo. Stress test passa con 100 cartelle e profondità 5.
- **Prompt componibili** — Sintassi `{{import "path"}}` con risoluzione cartella+titolo, parser ricorsivo, cycle detection, depth limit 5, anti-bomba 1 MB.
- **Tag suggeriti** — Suggeritore semantico (top-K vicini per cosine) con fallback su frequenza per workspace ancora "freddi".
- **Statistiche / Insight** — Vista dedicata con KPI, top usati, candidati cleanup, distribuzioni per tag/target/visibilità. Lint health % aggregata.
- **Auto-init Session al boot** — Se modello + runtime sono su disco, il client carica la Session ort senza richiedere click manuale.
- **Idle-unload Session** — Configurabile (5/10/30/60 min, o disattivata). Libera ~150 MB di RAM quando inattiva.

### Quality gate Fase 3 (Step 10)

- **Grace degradation** verificata su tutti i path: backfill ora skippa graceful invece di crashare se Session None
- **Bench P95 ricerca ibrida**: 8.29 ms su 10 000 prompt (lex+sem+RRF) → ~38 ms includendo encoding query MiniLM. Sotto target 100 ms con margine ~2.5×
- **Stress cartelle**: 14 unit test, 100 cartelle profondità 5, invariante `Path` ↔ `ParentFolderId` validato
- **Coverage gate**: cargo-llvm-cov nel CI con threshold 60 % line. Coverage attuale: 60.12 %. Roadmap esplicita verso 70 % per v0.4

### Schema DB (V005-V007)

- **V005** `embeddings` — Tabella vec0 `PromptsEmbeddings` (sqlite-vec)
- **V006** `tag_embeddings` — Tabella vec0 `TagsEmbeddings`
- **V007** `prompt_imports` — Tabella `PromptImports` come grafo dipendenze import

Tutte le migrazioni sono additive. Vault esistenti vengono migrati al primo unlock post-update.

### Documentazione nuova

- [`docs/utente/ricerca-semantica.md`](docs/utente/ricerca-semantica.md)
- [`docs/utente/linting-regole.md`](docs/utente/linting-regole.md)
- [`docs/utente/cartelle.md`](docs/utente/cartelle.md)
- [`docs/utente/prompt-componibili.md`](docs/utente/prompt-componibili.md)
- [`docs/operativo/bench-ricerca-ibrida.md`](docs/operativo/bench-ricerca-ibrida.md)
- [`docs/operativo/coverage.md`](docs/operativo/coverage.md)
- ADR completi: `embedding-model.md`, `sqlite-vec-sqlcipher.md`, `onnx-bundle.md`

### Statistics

- 26 PR mergiate dalla v0.2.1 (Fase 3 effettiva: PR #28-#53)
- 169 unit test Rust totali (+58 da v0.2.1)
- 0 errori type check, 0 vulnerabilità note (audit verde)
- ~5 800 righe Rust strumentate, 60.12 % line coverage

### Out of scope (rinviato)

- **Riload automatico Session post idle-unload** — oggi serve riavviare il client. Issue tracker per v0.3.x patch
- **Sintassi `with k=v` su import** — variabili scopate per import. Roadmap Fase 4
- **Pinning a versione storica negli import** (`{{import "x" version=N}}`) — schema `PromptVersions` già pronto, manca parser. Roadmap Fase 4
- **Coverage 70 % globale** — roadmap incrementale in `docs/operativo/coverage.md`

---

## v0.2.1 (2026-05-05)

> **Status**: patch della linea `v0.2.x` con quick wins anticipati di Fase 3 e infrastruttura release. 4 PR funzionali + 1 CI in un singolo ciclo, niente AI introdotta (foundations rimangono stabili). Spike pre-Fase 3 chiusi con verdict prima dei feature step.

### Quick wins anticipati di Fase 3

#### Step 6 — Modello target dichiarato (PR #23)
- Backend `editor.rs`: `NuovoPrompt`/`AggiornamentoPrompt` accettano `target_model: Option<String>`, persistito in `Prompts.TargetModel`
- Backend `libreria.rs`: `FiltroLista` filtra per `target_model`
- Frontend: nuovo `apps/client/src/lib/modelli-target.ts` con preset (Claude Opus/Sonnet/Haiku, GPT-4/Mini, Gemini Pro/Flash, Llama 3, Generic)
- UI Editor: dropdown Select sopra Visibilità, autosave-aware
- UI Libreria: gruppo "Modello target" in sidebar, badge nel detail panel
- 5 test unit Rust per `normalizza_target_model`

#### Step 9 — Statistiche / Insight (PR #24)
- Nuovo modulo backend `statistiche.rs` con comando Tauri `statistiche_query`
- Aggregazioni: totali (prompt attivi/eliminati, tag, creati/aggiornati 30g, versioni), top 10 usati, candidati cleanup (>90g inattivi), distribuzioni per tag/target_model/visibilità
- Nuova superficie `Insight.svelte`: KPI grid + bar charts SVG inline custom (no librerie esterne)
- Privacy: tutto calcolato localmente sul vault SQLCipher, disclaimer esplicito
- 6 test unit Rust per le aggregazioni

#### Step 7 — Cartelle gerarchiche (PR #25 backend + UI base, PR #26 D&D + polish)
- **Schema V004**: tabella `Folders` (Id, WorkspaceId, ParentFolderId, Name, Path denormalizzato), indice unique sibling-name, `Prompts.FolderId`
- 6 comandi Tauri: `folder_lista/crea/rinomina/sposta/elimina` + `prompt_sposta`
- Rinomina/sposta cascade aggiornano `Path` di tutti i discendenti via prefix replace SQL in transazione (helper `atomicamente`, no unsafe transmute)
- Anti-ciclo: bloccato spostamento dentro sé stessi o discendenti
- Soft-delete cascade: cartella + sottocartelle marcate, prompt dentro tornano a root
- 8 test unit Rust per validazione, calcolo path, cascade rinomina/sposta, anti-ciclo, unique sibling
- UI Libreria sidebar: tree gerarchico con indent, "Senza cartella" come voce speciale, conteggio prompt accanto al nome
- **Drag & drop** (PR #26): prompt → cartella, cartella → cartella, drop su "Senza cartella" sposta a root, visual feedback dashed-outline durante dragover
- **Filter chip** "Cartella corrente" nella head lista: pill con path, click rimuove filtro
- **Rinomina inline**: input field al posto di NavItem, Enter conferma, Escape annulla, blur conferma
- UI Editor: Select cartella sotto Modello target, autosave-aware

### Infrastruttura release

#### Versione portable Windows (PR #27)
- Step Windows-only post `tauri-action`: copia `Prompt a Porter.exe` standalone in cartella staging + `README.txt`, zippa, carica come asset extra della draft release
- Asset risultante: `Prompt-a-Porter-portable-windows-x64-{tag}.zip` accanto a NSIS / MSI installer
- Pattern Chrome/VSCode portable: estrai e lancia, niente installer, niente registro modificato
- WebView2 runtime requirement documentato nel README e nel body release

### Spike chiusi pre-Fase 3 (release ciclo precedente, ricapitolati)

I 3 spike sotto sono stati eseguiti e mergiati a `v0.2.0-foundations` ma sbloccano lo sviluppo di Fase 3 e meritano una nota:

- **Spike 1 — sqlite-vec ⊕ SQLCipher** (PR #20): tutti e 6 gli stage chiusi su Linux con SQLCipher 4.5.7 + sqlite-vec v0.1.9. Step 2 di Fase 3 procede col path standard (`vec0` dentro vault SQLCipher), niente fallback architetturali.
- **Spike 2 — ONNX Runtime bundle size** (PR #21): bundle Tauri cresce da ~3-4 MB a ~19-26 MB con `ort` + `libonnxruntime` (4-8× crescita). Accettabile, decisione presa di andare con bundle inclusivo via `ort 2.x default features (download-binaries + copy-dylibs)`. ⚠️ ort 2.x rc.9/.10/.12 attualmente instabili su Rust stable, da rivalutare all'inizio Step 1 di Fase 3.
- **Spike 3 — modello embedding IT/EN** (PR #22): qualitative test su 30 prompt + 10 query in `@huggingface/transformers`. `paraphrase-multilingual-MiniLM-L12-v2` (118 MB) batte `bge-small-en-v1.5` (33 MB) di +30 punti recall@5 sul mix linguistico (97.5% vs 65.0%). Decisione: si adotta multilingual-MiniLM-L12-v2 in Step 1, lazy download al primo uso.

### Statistics

- 5 PR mergiate (#23, #24, #25, #26, #27)
- ~1.500 righe di codice nuovo (Rust + TypeScript + SQL)
- 19 nuovi test unit Rust (5 target_model + 6 statistiche + 8 cartelle)
- 0 vulnerabilità note (audit security verde)

---

## v0.2.0-foundations (2026-05-04)

> **Status**: Fase 2 chiusa sui 6 step controllabili (1, 2, 3, 4, 7, 8). Step 5 (auto-update silenzioso) riposizionato a patch line `v0.2.x` — sblocca con cert Authenticode Certum (KYC in corso). Step 6 (server cross-platform senza Docker) spostato a Fase 5 Step 0a — domanda-driven, riprende con workspace team enterprise. Razionale completo in `docs/roadmap/fase-2-foundations.md` e `docs/roadmap/quality-gate-fase-2.md`.

### Breaking changes

- **Licenza**: GPL 2.0 → **AGPL 3.0** (`LICENSE`, `package.json`, `Cargo.toml`). Chiude il loophole SaaS: chi ospita il codice come servizio è obbligato a pubblicare le modifiche. Fork e redistribution restano liberi sotto AGPL 3.0. Vedi commit `4e365c9`.

### Step 1 — Cambio licenza GPL 2.0 → AGPL 3.0
- `LICENSE` sostituito con testo ufficiale AGPL 3.0
- SPDX `AGPL-3.0-only` in tutti i manifest (`package.json` root + client, `Cargo.toml` Tauri)
- README sezione Licenza riscritta con razionale anti-SaaS-loophole

### Step 2 — Versioning completo prompt + rollback
- **Migration V002**: `PromptVersions` esteso con `Visibility` + `TargetModel`, indice composito `(PromptId, Version DESC)`, backfill v1 per prompt esistenti
- Nuovo modulo Rust `versioning.rs`: `snapshot_versione` (idempotente via INSERT OR IGNORE), `prompt_get_history`, `prompt_rollback` (soft, preserva storia)
- Hook in `prompt_crea`/`prompt_aggiorna`: snapshot automatico ad ogni create/update
- Rolling delete oltre 100 versioni per prompt (configurabile in futuro)
- UI Svelte `CronologiaPrompt.svelte`: modale split pane con lista versioni + preview + ripristino con doppia conferma
- Bottone "Cronologia" nel pannello dettaglio Libreria
- 5 test Rust nuovi
- PR #6, commit `ee0c4e3`

### Step 3 — Audit log query-able
- **Migration V003**: 3 indici performance su `AuditLog` (`(WorkspaceId, OccurredAt)`, `(UserId, OccurredAt)`, `(EntityType, EntityId)`)
- Tauri commands `audit_query` (filtri date+user+action+text+entity, paginazione), `audit_export_csv` (RFC 4180 con quoting), `audit_cleanup_oltre_giorni` (retention manuale)
- UI Impostazioni > Registro attività: filtri estesi (segmented entity, search action/testo, range date), paginazione 50/pag, bottone "Esporta CSV", inline-confirm cleanup
- 4 test Rust nuovi
- PR #7, commit `6af4bd9`

### Step 4 — Import/export JSON con schema v1
- **Schema documentato**: `docs/utente/formato-export-json.md` — versionato (`schemaVersion: 1`), forward/backward compatible, round-trip lossless
- Tauri commands `vault_export_json` (workspace completo), `vault_import_json` con modalità conflitti (`skip`/`overwrite`/`rename`)
- Helper `ora_iso()` in pure Rust (zero `chrono`, algoritmo Howard Hinnant)
- UI Impostazioni > Vault: bottoni Esporta/Importa con segmented modalità, report inline post-import (nuovi/aggiornati/conflitti/errori)
- Audit log: `vault.exported`, `vault.imported`
- 5 test Rust nuovi
- Markdown export/import rinviato a sub-step
- PR #8, commit `1eda4f8`

### Step 7 — MCP server (Model Context Protocol)
- **Nuovo modulo `apps/mcp-server/`** in TypeScript con `@modelcontextprotocol/sdk` + `better-sqlite3`
- Trasporto stdio (Claude Desktop, Cursor)
- 4 tool read-only: `pap_search`, `pap_get`, `pap_list_recent`, `pap_render`
- Vault discovery via env `PAP_VAULT_PATH` o default per piattaforma
- Solo vault non cifrati in MVP (SQLCipher in arrivo)
- Documentazione completa `docs/utente/mcp.md` (Claude Desktop, Cursor, troubleshooting)
- Workflow CI dedicato `mcp-server-build.yml` (lint + build TS) con `workflow_dispatch` manuale
- HTTP/SSE transport e `pap_create_draft` rinviati a sub-step
- PR #9, commit `cfbe546`

### Step 8 — CLI `pap`
- **Nuovo modulo `apps/cli/`** in Go con `cobra` + `modernc.org/sqlite` (pure-Go, zero CGO) + `yaml.v3`
- 5 comandi: `pap version|search|get|recent|render` + `completion` automatico (bash/zsh/fish/powershell)
- Output formats: `table` (default, tabwriter), `json`, `yaml`, `plain` (id<TAB>title)
- Vault read-only strict (DSN `?mode=ro`)
- CI cross-compile matrix 6 build (linux/darwin/windows × amd64/arm64) con `CGO_ENABLED=0`, ldflags `-s -w`, upload-artifact
- Documentazione `docs/utente/cli.md` con esempi tab-completion per ogni shell
- 10 test unit Go
- Comandi `login`/`new`/`import`/`export` rinviati (richiedono server API o IPC client desktop)
- PR #11, commit `12a1214`

### Infrastructure & repo

- File standard di presentazione GitHub: `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1 IT), `SECURITY.md` (disclosure policy + tempi risposta), `.github/ISSUE_TEMPLATE/{config,bug_report,feature_request}.yml`, `.github/PULL_REQUEST_TEMPLATE.md`
- `CONTRIBUTING.md` esteso con DCO sign-off + Conventional Commits
- Filter-repo per unificare autori commit
- Workflow `bootstrap.yml` per generare lockfile + icone in CI senza Node locale
- Workflow `release.yml` per build multi-OS Tauri (NSIS perUser su Windows, dmg arm64 su macOS, deb/AppImage/rpm su Linux) con `tauri-action` + draft Release
- Patch CI workflow: `paths-ignore` per `*.md`/`LICENSE`/`CHANGELOG.md` dentro `apps/*` per evitare build inutili; `workflow_dispatch` su tutti i workflow per re-run manuale; `pnpm-lock.yaml` aggiunto ai trigger paths
- Nuovo workflow `security-audit.yml`: `cargo audit` + `govulncheck` (server + CLI) + `pnpm audit`, schedulato settimanalmente + dispatch manuale

### Bug fix significativi (post-v0.1.0-fase1)

- **#4 critical**: preferenze Windows non persistevano causando re-onboarding e errore "vault già aperto". Fix: `App.svelte` usa `vault_esiste()` come fallback robusto del check `onboarding_completato`.
- **#3 high**: tray menu Windows non appariva. Fix: `lib.rs` configura `show_menu_on_left_click(false)` + handler `on_tray_icon_event` per click sinistro → mostra libreria; click destro → menù contestuale.
- **#2 low**: onboarding mancava toggle tema light/dark. Fix: segmented control nel wizard, applicato live via `data-theme`.

### Quality gate (PR #17, #18, #19)

- **PR #17** — Server `go.sum` rigenerato (hash inconsistenti con `sum.golang.org` per tutti i moduli, probabile generazione originale con `GOSUMDB=off`); bump server Go 1.23 → 1.25 + `golang-jwt/jwt/v5` aggiornato + `chi/v5 v5.2.1 → v5.2.2` (fix `GO-2025-3770` open-redirect). Risultato `govulncheck`: 22 vuln (1.23.4) → 0 (1.25.9).
- **PR #18** — Coverage CLI `53.3% → 72.7%` con 3 test mirati su `recent` (70.6%), `formatPrompt` (93.5%), `tagsFor` (81.8%).
- **PR #19** — Riposizionamento Step 5 (→ patch line `v0.2.x`) e Step 6 (→ Fase 5 Step 0a). Scope `v0.2.0-foundations` formalizzato.

### Audit security finale

| Audit | Stato |
|---|---|
| `cargo audit` (Tauri client) | ✅ clean |
| `pnpm audit` (workspace) | ✅ clean |
| `govulncheck` CLI (Go 1.24) | ✅ clean |
| `govulncheck` server (Go 1.25) | ✅ clean — 0 vulnerabilità |
| `licensee` AGPL 3.0 | ✅ consistente in tutti i manifest |

### Roadmap successiva

- **Patch line `v0.2.x`** — Auto-update silenzioso (Step 5): NSIS per-user + Tauri Updater + firma Authenticode. Sblocco: cert Certum OSS.
- **Fase 5 Step 0a** — Server Go cross-platform senza Docker (`modernc.org/sqlite`, Win Service + systemd). Domanda-driven.
- **Fase 3 (`v0.3.0`)** — Intelligenza & authoring: ricerca semantica via embeddings ONNX locali, prompt componibili, linting proattivo. Vedi `docs/roadmap/fase-3-intelligence.md`.

### Statistics

- 14 PR mergiate (#6 – #19)
- ~5500 righe di codice nuovo (Rust + TypeScript + Go + SQL)
- Coverage CLI 72.7%, server 56.2% (cross-package via test integrazione)
- 0 vulnerabilità note (audit settimanale via `security-audit.yml`)

---

## v0.1.0-fase1 (2026-05-03)

Prima release MVP. Tutte le funzionalità core implementate.

### Step 0 — Bootstrap repo
- Inizializzazione repo con LICENSE GPL 2.0, README, .gitignore
- Setup pnpm workspace monorepo (`apps/client`, `apps/server`, `packages/`)
- GitHub Actions baseline (lint check client + server)

### Step 1 — Client Tauri + Svelte
- Scaffolding Tauri 2 + Svelte 5 + TypeScript
- Configurazione multi-window (libreria 1200×800 + palette 640×480 frameless)
- Struttura directory: components, superfici, stores, i18n
- File i18n: it.json + en.json con stringhe per tutte le superfici
- Icone SVG sorgenti (Lucide `braces`)

### Step 2 — Vault SQLite + SQLCipher
- Integrazione `rusqlite` con `bundled-sqlcipher` (AES-256)
- Schema V001: 8 tabelle + FTS5 + 8 indici
- Migration system embedded via `include_str!()`
- Comandi: vault_crea, vault_unlock, vault_lock, vault_cambia_password
- Derivazione chiave Argon2id (m=32MiB, t=3, p=4)
- 7 test unitari

### Step 3 — Componenti UI base
- 16 primitive Svelte 5: Button, Input, Textarea, Select, Field, Switch, Kbd, Tag, Badge, Placeholder, NavItem, ListItem, EmptyState, Toast, Skeleton, Tooltip
- Classi utility globali in app.css
- Pagina demo `?demo` con switch tema/tono
- Accessibilità: aria attributes, focus-visible, keyboard nav

### Step 4 — Onboarding
- Wizard 3 step (Profilo → Password vault → Hotkey)
- Strength meter password (4 livelli, calcolo entropia)
- Supporto vault non cifrato ("Salta cifratura")
- Navigazione tastiera (Enter=avanti, Esc=reset)

### Step 5 — Tray icon + global hotkey
- Tray con menu contestuale (5 voci)
- Hotkey globale registrabile a runtime
- Toggle palette: show+center+focus / hide
- Caricamento hotkey da preferenze all'avvio

### Step 6 — Command Palette
- Window frameless dedicata, fuzzy search FTS5
- Navigazione tastiera (↑↓ naviga, Enter seleziona, Escape chiudi)
- Espansione inline form segnaposti
- Ctrl+Enter = compila e copia negli appunti

### Step 7 — Libreria
- Layout 3 pannelli CSS Grid (sidebar + lista + dettaglio)
- Sidebar con workspace switcher, viste, tag dinamici
- Lista con search debounced, sort (recente/popolare/A-Z)
- Status bar con sync dot, versione, hotkey

### Step 8 — Editor prompt
- Modale 2 colonne con CodeMirror 6
- Highlight {{segnaposti}} con ViewPlugin + Decoration
- Tag picker con autocomplete
- Autosave con debounce (2s)

### Step 9 — Compilatore
- Vista 2 colonne (form + preview)
- Form auto-generato dai segnaposti
- Progress bar compilazione
- Toggle output Testo / Markdown / JSON
- Copy to clipboard + toast

### Step 10 — Impostazioni
- Layout sidebar + content con 7 sezioni
- Hotkey configurabile con HotkeyInput
- Tema scuro/chiaro + tono zinc/slate/stone
- Gestione vault: percorso, cifratura, cambio password, elimina
- Toggle lingua it/en

### Step 11 — Server Go
- chi router con middleware (CORS, logger, JWT, recoverer)
- Schema SQLite server + SyncChangelog
- Auth: Argon2id + JWT HS256 (login + refresh)
- Sync: pull delta + push con last-write-wins
- WebSocket broadcast per workspace
- Dockerfile multistage (golang:1.23-alpine → alpine:3.20)
- 12 test di integrazione

### Step 12 — Auth e Sync client
- 3 schermate auth: Login, Reset password, Recupera workspace
- Store sync singleton (polling + WebSocket reconnect)
- Conflict UI con scelta locale/server per entità
- Preferenze estese con campi sync (serde default backward compat)
- Sezione Sync in Impostazioni con stato live

### Step 13 — Audit log
- Modulo `audit.rs` con `registra()` fire-and-forget
- Hook su editor, libreria, vault, sync (9 azioni tracciate)
- Vista "Registro attività" in Impostazioni con filtro per tipo
- Comando `audit_lista` con limite e filtro tipo entità

### Step 14 — Quality gate
- 37 test Rust su 8 moduli
- 22 test TypeScript per template.ts (vitest)
- CI aggiornata: job rust-test + vitest + coverage 70% server

### Step 15 — Documentazione
- Architettura completa con diagrammi e tabelle moduli
- Setup sviluppo con comandi e struttura directory
- Deploy produzione con Docker e variabili d'ambiente
- Prompt di ricostruzione con lezioni apprese
- Changelog completo
- API server aggiornata
