# Cartamodello: scomporre un prompt in moduli

> Prendi uno o più prompt scritti «piatti» e lascia che l'app li tagli nei loro pezzi riusabili — ruolo, vincoli, formato, contesto — ricomponendoli con `{{import}}` e segnaposti. Dal dettaglio di un prompt (icona forbici) o dal menu della selezione multipla.

In sartoria il *cartamodello* è il modello di carta da cui si tagliano i pezzi di un capo. Qui fa lo stesso con i prompt: quello che hai scritto in un blocco unico — o che hai importato da un file, da un sito, da un collega — viene analizzato da un modello AI che conosce la sintassi di Prompt à Porter e proposto **scomposto**: un modulo per il ruolo, uno per il formato di output, un altro per i vincoli che ripeti sempre, e il prompt originale riscritto come composizione di quei moduli più la sua parte specifica, con i valori variabili trasformati in `{{segnaposti}}`.

Non è una riscrittura: il testo, una volta espanso, deve dire le stesse cose dell'originale. Per migliorare un prompt c'è **Ritocco** (l'icona bacchetta nel dettaglio); Cartamodello si limita a tagliare e ricucire.

## Quando conviene

- Hai **più prompt che ripetono lo stesso pezzo** (lo stesso «Sei un…», le stesse regole di stile): selezionali insieme e il pezzo comune diventa un solo modulo importato da tutti. Migliorarlo una volta lo migliora ovunque.
- Hai importato prompt da fuori (una [collezione](./collezioni.md), un file Markdown, un sito) e vuoi renderli «alla PaP».
- Un prompt è cresciuto troppo e vuoi separare ciò che è stabile da ciò che cambia a ogni uso.

## Come si usa

1. Apri il prompt e premi l'icona **forbici** nella barra del dettaglio, oppure seleziona da 2 a 10 prompt nella lista (Ctrl/Cmd+click) e scegli **Scomponi N in moduli (AI)** dal tasto destro.
2. Scegli **provider e modello** (serve almeno un provider AI abilitato in Impostazioni → Provider AI). Premi **Analizza**: il testo dei prompt viene inviato al modello; nulla viene ancora scritto nel vault.
3. **Rivedi la proposta.** Per ogni modulo puoi:
   - **Crea nuovo** — con il titolo proposto o uno tuo (il titolo è la chiave degli `{{import}}`, l'app lo aggiorna nei prompt se lo cambi);
   - **Usa esistente** — se nel vault c'è già un prompt equivalente, l'app lo ha trovato e te lo propone al posto di un doppione;
   - **Scarta** — il pezzo non diventa modulo (e i prompt che lo importavano vanno sistemati: l'app te lo dice).

   Il **titolo** di ogni prompt resta il tuo: se il modello ne propone uno diverso lo vedi come suggerimento e lo adotti solo cliccandolo. Descrizione, cartella e tag non vengono toccati.

   Per ogni prompt ricomposto hai tre viste: il **Diff con l'originale** (confronta l'originale con il ricomposto *espanso*: se è quasi vuoto la scomposizione è fedele), l'**Anteprima espansa** e il **Sorgente** con gli import.
4. **Applica.** In un colpo solo: i moduli nuovi vengono creati nella sottocartella `Moduli` accanto agli originali, e ogni prompt originale viene salvato come **nuova versione** con il corpo ricomposto. Il bottone resta disabilitato finché c'è qualcosa da sistemare, e ti dice cosa.

## Cosa succede all'originale

Diventa la versione N+1 di sé stesso. La versione precedente è nella tab **Cronologia**: se il risultato non ti convince, **Ripristina** e torni al testo piatto, mentre i moduli creati restano (puoi cancellarli o riusarli). Cartella, preferiti, tag e conteggi degli usi non cambiano.

## Cosa controlla l'app prima di applicare

La proposta del modello è un suggerimento, non un ordine. Prima di mostrartela l'app verifica che:

- ogni modulo abbia titolo e corpo, non contenga a sua volta `{{import}}`, passi il linter e non abbia lo stesso titolo di un prompt già nel vault (sarebbe ambiguo importarlo);
- ogni `{{import}}` dei prompt ricomposti punti a un modulo proposto o a un prompt che hai già;
- due moduli con lo stesso contenuto vengano fusi in uno;
- per ogni modulo, se nel vault esiste un prompt molto simile, te lo proponga come riuso (confronto semantico se il modello di ricerca è caricato, altrimenti sulle parole in comune).

E prima di scrivere ricontrolla tutto sul vault attuale: se nel frattempo hai modificato uno degli originali, si ferma e ti chiede di rilanciare l'analisi.

## Limiti e consigli

- Massimo **10 prompt** e **60 000 caratteri** per volta: oltre, spezza la selezione.
- La qualità dipende dal modello: con modelli piccoli la proposta può essere povera. Il diff di fedeltà è la tua rete: se è pieno di modifiche, il modello ha riscritto invece di scomporre — rifiuta e riprova, magari con un modello più capace.
- I segnaposti globali (`{{global …}}`) vengono usati solo se ne hai definiti: al modello arrivano i **nomi**, mai i valori.
- I commenti `{{!-- … --}}` dell'originale fanno parte del testo inviato al modello.

## Vedi anche

- [`prompt-componibili.md`](./prompt-componibili.md) — la sintassi degli `{{import}}` e di `with`.
- [`collezioni.md`](./collezioni.md) — prompt importati da fuori, candidati tipici alla scomposizione.
- [`glossario-sintassi.md`](./glossario-sintassi.md) — segnaposti, globali, commenti.
