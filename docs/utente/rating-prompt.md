# Valutazione dei prompt

> Come dare un voto rapido a un prompt dopo averlo usato (👎 / 😐 / 👍) e
> leggere l'aggregato nella tab Valutazioni per capire quali prompt reggono
> nel tempo.

Un prompt sembra buono finché non lo usi davvero, ripetutamente. Con l'uso
scopri che una formulazione che leggevi con soddisfazione produce, nella
pratica, output che devi sempre ritoccare — oppure, al contrario, che un
prompt scritto in fretta funziona ogni volta. Ma questa conoscenza vive
nella tua testa e svanisce: fra due settimane non ricordi più quali prompt
ti hanno deluso e quali no.

La valutazione cattura quel giudizio nel momento in cui lo hai, con il
minimo attrito. Dopo aver compilato e copiato un prompt, puoi lasciare un
**feedback a tre valori** — 👎 negativo, 😐 neutro, 👍 positivo — con un
clic. Niente stelle, niente commenti obbligatori: un pollice, e via.

La cosa importante è che i voti non si sovrascrivono: ogni voto è una riga a
sé, con la sua data. Così non ottieni una media piatta, ma una **traiettoria**.
Un prompt molto usato che comincia a raccogliere voti bassi è un candidato
al refactor — e te ne accorgi guardando la tendenza, non un singolo numero.

## Come dare un voto

1. Compila il prompt: dalla Command Palette (`Ctrl+Shift+P`) oppure dal
   pannello di dettaglio con il bottone **Compila**.
2. Sotto l'output compilato, la modale mostra il blocco **"Valuta il
   risultato"** con tre bottoni: **Negativo**, **Neutro**, **Positivo**
   (👎 😐 👍).
3. Clicca il bottone che corrisponde al tuo giudizio: il voto viene
   registrato e accanto al titolo del blocco compare la conferma
   "— grazie!".
4. Puoi accompagnare il voto con una nota. I voti **Neutro** e **Positivo**
   si salvano subito, ma restano affiancati da un campo nota opzionale se
   vuoi annotare qualcosa. Il voto **Negativo** apre invece la modale
   **"Cosa non ha funzionato?"** con una textarea: scrivi cosa è andato
   storto, oppure premi **"Salta e registra voto"** per registrare il
   pollice verso senza spiegazioni.
5. Se chiudi la modale senza cliccare alcun bottone, non viene registrato
   nulla: il voto è sempre facoltativo.

Se qualcosa va storto nel salvataggio (rete, disco), l'errore resta
silenzioso: la valutazione non deve mai intralciare il flusso di "compila e
usa".

## Leggere l'aggregato

I singoli voti diventano utili quando li guardi insieme. Il pannello di
dettaglio della Libreria ha la tab **Valutazioni**, che riassume tutti i
voti di un prompt:

- una **media firmata** a due decimali (per esempio `+0.75`) sul numero
  totale di voti;
- le **barre di distribuzione** Positivi / Neutri / Negativi, ciascuna col
  proprio conteggio.

Se un prompt non ha ancora voti, la tab mostra uno stato vuoto che ti invita
a valutarlo dalla modale di compilazione. Nella riga dei metadati del
pannello di dettaglio compare intanto il numero di utilizzi, come chip
**Usato N×**.

## Cosa significano i tre valori

I tre livelli hanno un significato preciso, riferito al modello con cui hai
usato il prompt:

- **👍 positivo (+1)** — l'output rispetta l'intento del prompt, così com'è.
- **😐 neutro (0)** — parziale: funziona, ma serve un ritocco manuale.
- **👎 negativo (−1)** — il prompt non ha prodotto quello che ti aspettavi.

La scala è volutamente ridotta a tre valori. Le scale a cinque stelle
soffrono di un bias culturale (per qualcuno "3 su 5" è già un buon voto, per
altri solo "5" lo è): tre livelli netti — no, così così, sì — riducono
l'ambiguità e rendono i voti confrontabili.

## Ordinare i prompt per qualità

Le valutazioni non restano confinate nella loro tab: alimentano un
ordinamento della Libreria. Nel menu dell'ordine c'è l'opzione **"Migliori"**,
che dispone i prompt per voto medio degli ultimi 90 giorni (dai migliori in
giù), lasciando in fondo quelli senza voti. Con questo ordinamento attivo le
card della lista mostrano il voto medio accanto al prompt, così vedi al volo
quali funzionano meglio.

## Limiti noti

- Le valutazioni sono personali e legate alla tua installazione: non
  esistono ancora aggregati condivisi di team.
- Ogni voto registra anche il modello con cui hai usato il prompt, ma
  l'aggregato non permette oggi di filtrare i voti per modello: la media
  mescola gli esiti ottenuti con modelli diversi.

## Vedi anche

- [`varianti-prompt.md`](./varianti-prompt.md) — le valutazioni indipendenti di ogni variante fanno emergere la formulazione migliore.
- [`regression-testing.md`](./regression-testing.md) — la misura automatica degli output, complementare al tuo giudizio umano.
- [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) — la scorciatoia per aprire la Command Palette e compilare in fretta.
