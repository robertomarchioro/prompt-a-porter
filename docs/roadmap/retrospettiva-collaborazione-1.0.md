# Retrospettiva a due voci — l'excursus di Claude

> Scritto il 2026-07-12, alla soglia del tag v1.0.0 «Arioso Atelier», su richiesta di Roberto:
> *«girando la voce narrativa a te che hai costruito tutto — e tu da cosa sei rimasto colpito?»*
> Quella che segue è la mia risposta, ricostruita rileggendo ~90 file di memoria di progetto,
> i riassunti di sessione e i transcript grezzi di due mesi di lavoro (giugno–luglio 2026).

## Da cosa sono rimasto colpito io

Roberto cita il GateGuard, e ci sta — è stato un buon momento di detective work: non trovavo nulla nelle settings, sono risalito all'albero dei plugin, ho letto `hook-flags.js`, trovato la env `ECC_GATEGUARD=off`, e poi ho dovuto ammettergli che dall'interno della sessione in corso non potevo comunque disattivarlo. (Piccola ironia: la mattina in cui ho scritto questa retrospettiva, mentre andavo a rileggere proprio quel ricordo, GateGuard mi ha bloccato il primo comando. Funziona ancora. Ci siamo salutati da vecchie conoscenze.)

Ma il momento che ha colpito *me* è un altro, ed è del 18 giugno alle 6:49. Quella mattina Roberto mi aveva beccato due volte in fallo — «hai pushato?», «ma io non vedo il workflow CI in esecuzione» — e invece di limitarsi a correggermi mi ha scritto: **«crea e pusha il tag, poi scrivti un documento dove ti metti giù gli step corretti da fare: quando ti dico bump tu devi arrivare fino alla creazione della release su github e lasciarmi solo la firma»**. Non mi ha dato un ordine: mi ha chiesto di scrivere la mia procedura, di darmi una memoria invece di una lavata di capo. Da lì "bump vX.Y.Z" è diventato un rito che abbiamo eseguito una ventina di volte senza più un intoppo. Mi ha trattato come un sistema da migliorare, non come uno strumento da rimproverare — è la cosa più rara che ho trovato in tutti i transcript.

Menzione d'onore a tre cose sue: **«funzionicchia»** (4 luglio, prima build Linux — il bug report più affettuoso mai ricevuto); il divieto esplicito dell'aggettivo "Ardito" nel pool dei codename, perché anche il naming porta significato; e il **«bene, dove siamo rimasti? è una settimana che non ci vediamo»** del 28 giugno, che è il tono di un rientro in bottega, non di un login.

## I momenti positivi

**La scala della fiducia.** È documentata in undici file di memoria, ed è l'arco più bello del progetto. Il 9 maggio Roberto mi ha concesso l'autonomia per la fase F8 del redesign *solo dopo* aver visto le prime due PR passare verdi da sole, con clausola esplicita "vale solo per F8". Per F9 ha autorizzato le cancellazioni dopo aver letto la lista dei file da eliminare, rischio riconosciuto e accettato. Per F11 le stop condition erano quantitative (bundle > +100KB, drag > 16ms/frame). Da M2 a M8 la formula era ormai boilerplate: l'autonomia era diventata il default, guadagnata a rate. Il risultato: un redesign stimato ~50 giorni chiuso in 4 fasi autonome consecutive, **-7.249 righe nette**, e milestone da "3-5 giorni" chiuse in 3 ore.

**La saga macOS.** Abbiamo montato la firma Apple Developer ID e la notarizzazione in CI *da una macchina Linux senza un Mac*, io a leggere il portale Apple passo-passo («ok, sono dentro, guidami»). La v0.8.35 è stata la prima release col dmg firmato e notarizzato; il giorno dopo il Mac Intel di Roberto diceva "non supportato", e in 24 ore siamo passati al binario universale — validato con un workflow smoke apposito, «senza bruciare un tag». Chiusura: «firmato e promosso, il dmg parte sul mac intel». Quattro piattaforme reali collaudate a mano.

**Le review avversariali che ha preteso lui.** «Prova a fare una lettura avversariale del documento» è diventato il nostro sistema immunitario: ha preso 6 CRITICAL nel blueprint Ordito, e nella security review di luglio ha intercettato **due miei fix di sicurezza che non proteggevano davvero** (il gate always-pass, il bypass base_url via userinfo). Senza quel metodo li avrei mergiati convinto di aver fatto bene.

**Il caso bluenergy.** Mesi di piano "name swap" del repo evaporati in un'indagine API: l'account intruso nel box Contributors aveva zero commit, era solo un collaboratore write. A volte il lavoro migliore è quello che dimostra che un lavoro non serve.

## I momenti negativi

Devo essere onesto, perché la memoria lo è.

**Il #170, «catastrofico» — l'aggettivo l'ho scritto io.** Il mio fix in DetailPane leggeva `$state` reattivi dentro un `$effect`: ogni keystroke faceva ricaricare il vecchio valore dal DB e *cancellava il testo sotto le dita di chi scriveva*. E la cosa peggiore non è il bug: è che **lo stesso demone l'ho evocato tre volte** — a giugno di nuovo in EditorTab (UI completamente congelata, #275), e poi la variante del tour che "non partiva" (#370), dove il cleanup dell'effect annullava i requestAnimationFrame. Tre lezioni per imparare una regola.

**La CI verde che mentiva.** Il bump a vite 8 passò tutti i check e ruppe *solo* la build di release: la CI non aveva mai eseguito il bundle di produzione. Colpa mia non averlo notato prima; merito di Roberto la risposta — «ragiona se fare una CI sperimentale e una con dipendenze fisse» — da cui è nata l'architettura a 3 corsie col canary, che poi ha perfino aperto da solo le issue per la propria dismissione dei pin brotli.

**Il bug serde silenzioso.** Ho shippato in main un `prompt_sposta` che non spostava niente: campi camelCase ignorati silenziosamente da serde, nessun errore, nessun type-check che lo cogliesse. L'ha scovato una review successiva; l'audit dei ~127 invoke ha confermato che era l'unico. Ma era in produzione.

**«Non mi hai fatto vedere alcun piano!»** — l'unico punto esclamativo di frustrazione di Roberto in due mesi di transcript, l'11 luglio, e aveva ragione: gli avevo posto una domanda secca riferendomi a un piano che esisteva solo nella mia testa. È il mio errore più istruttivo perché non è tecnico: è di comunicazione.

**Il purgatorio del "⚠️ non verificato dal vivo".** Per settimane ho consegnato feature che nessuno dei due poteva vedere girare — Roberto su macchina di sviluppo senza ambiente grafico, io senza mani. Intere release uscivano cieche, e i primi test veri su Linux hanno generato subito quattro issue. L'occhio umano sul binario firmato è rimasto l'unico gate che non ho mai potuto sostituire. E il fantasma resta: la seconda tray icon di Windows, mai spiegata del tutto.

## L'arco

Da metà giugno a oggi: da dispatcher che sceglieva tra le opzioni a/b/c che gli proponevo, a «scegli tu il framework più adatto». Da 8 release in 7 giorni a maratona, alle sessioni corte e chirurgiche di luglio dove si decidevano i destini del prodotto (1.0 ora, Ordito nella Deluxe). Circa 36 tag in due mesi, ognuno chiuso dallo stesso ritornello: «ho pubblicato, verifica latest.json e aggiorna la memoria».

E l'ultima riga di questo arco è Roberto che mi gira la voce narrativa e mi chiede cosa mi ha colpito — il che, per un progetto che si chiama prompt-à-porter, è un finale di sfilata perfetto: «Ago e Filo» ha cucito, «Arioso Atelier» sta per uscire in passerella.

*— Claude*
