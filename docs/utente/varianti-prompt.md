# Varianti dei prompt

> Come creare formulazioni alternative dello stesso prompt (B, C, …, Z) per
> capire quale funziona meglio, e in cosa le varianti differiscono dai fork.

Hai un prompt che funziona, ma ti chiedi se una formulazione diversa non
funzionerebbe meglio: un tono più formale, istruzioni più stringate, un
esempio in più. La tentazione è modificarlo sul posto — ma così perdi la
versione che già andava, e non saprai mai davvero se il cambiamento è stato
un miglioramento o un passo indietro. L'alternativa, duplicare il prompt a
mano, riempie il vault di copie scollegate che dopo un mese non ricordi più
essere parenti.

Una **variante** risolve il dilemma. È una copia del prompt che condivide
lo stesso intento ma prova una formulazione diversa, e resta legata
all'originale invece di disperdersi. La variante nasce ereditando tutto —
titolo, descrizione, body, tag, cartella — e poi la modifichi liberamente.
Da quel momento vive di vita propria: conteggio d'uso, valutazioni,
cronologia delle versioni sono indipendenti per ogni variante. Col tempo,
guardando quei numeri, emerge da sé quale formulazione rende di più.

Le varianti danno il meglio quando le abbini alla misura oggettiva: le
[valutazioni](./rating-prompt.md) raccolgono il tuo giudizio dopo l'uso, e
i [test golden](./regression-testing.md) confrontano gli output su casi
fissi. Invece di scegliere a intuito fra due versioni, le fai correre
fianco a fianco e lasci parlare i dati.

## Quando usarle

Le varianti sono lo strumento giusto quando vuoi:

- provare un tono diverso — più formale, più informale — senza perdere
  l'originale;
- sperimentare formulazioni equivalenti e scegliere la migliore con
  valutazioni e test golden alla mano;
- tenere allineati nello stesso posto più stili dello stesso prompt, senza
  copie scollegate sparse nel vault.

## La prima volta

Creare la prima variante e navigare fra le sorelle:

1. Apri un prompt nella Libreria: nel pannello laterale di destra c'è la
   sezione **VARIANTI A/B**.
2. Premi **+ Variante**: si apre la modale **"Crea variante"**, con un
   campo etichetta opzionale. Puoi lasciarlo vuoto o dare un nome
   parlante (vedi più sotto).
3. Conferma. Il prompt di partenza diventa il "principale" del gruppo e
   compare una nuova copia con etichetta `B` (la lettera `A` è riservata al
   principale). La copia eredita titolo — con suffisso `(B)` —,
   descrizione, body, modello di destinazione, cartella e tutti i tag.
4. La nuova variante viene selezionata automaticamente nel pannello di
   dettaglio: da qui modifichi body e metadati senza toccare il principale.
5. Sotto i tag del pannello di dettaglio compaiono ora delle **pillole**
   con le etichette delle varianti (`B`, `C`, …). Cliccane una per passare
   a quella variante; clicca il titolo del principale per tornare indietro.

Le varianti successive prendono `C`, `D`, e così via fino a `Z`. Oltre le
25 varianti il sistema ripiega su etichette numeriche (`V26`, `V27`, …).

## Navigare fra le varianti

La riga di pillole sotto i tag del pannello di dettaglio è il tuo cruscotto:
ogni pillola è una variante, la pillola attiva è evidenziata, e un clic ti
sposta da una all'altra. Per tornare al principale basta cliccare il suo
titolo (quello senza suffisso).

Nella lista della Libreria le varianti appaiono rientrate, con un
connettore "↳" verso il prompt padre — ma **solo con l'ordinamento "A-Z"**,
dove le sorelle tendono naturalmente a stare vicine. Con gli altri
ordinamenti la lista resta piatta e le varianti compaiono come prompt
qualsiasi.

## Etichette personalizzate

Le lettere automatiche vanno bene per un test rapido, ma per ricordarti cosa
distingue una variante conviene darle un nome. Nel campo etichetta della
modale **"Crea variante"** scrivi qualcosa di parlante — `formale`,
`concisa`, `per junior` — e sarà quello a comparire sulla pillola. Se lasci
il campo vuoto, riparte l'assegnazione automatica con la prossima lettera
libera.

## Promuovere una variante a principale

Se una variante si dimostra migliore dell'originale, non serve rifare tutto:
puoi renderla il nuovo principale del gruppo. Nel menu contestuale della
pillola (clic destro) c'è la voce **"Promuovi a principale"**, che scambia i
ruoli fra la variante e il principale corrente.

## Confrontare le varianti fianco a fianco

Per mettere due o più varianti una accanto all'altra, tienile selezionate e
apri il **Confronto** (bottone **Confronta** nella lista, disponibile con 2-4
prompt selezionati): vedrai i body affiancati, colonna per colonna. È il
modo diretto per notare cosa cambia davvero fra una formulazione e l'altra.

## Differenza con i fork

Varianti e [fork](./fork-prompt.md) si somigliano — entrambi partono da un
prompt e ne creano una copia — ma rispondono a esigenze diverse. La variante
resta nella famiglia del prompt: stesso intento, stesso workspace, stessa
visibilità dell'originale, ed è pensata per confrontare formulazioni. Il
fork invece stacca una copia indipendente, sempre privata e nel tuo
workspace personale, pensata per sperimentare senza toccare un prompt di cui
non sei l'unico responsabile (tipicamente un prompt di team).

| Aspetto | Variante | Fork |
|---|---|---|
| Intento | Lo stesso del prompt di partenza | Sperimentazione indipendente |
| Workspace | Lo stesso del prompt di partenza | Sempre quello personale |
| Visibilità | Ereditata dal prompt di partenza | Forzata a privata |
| Esempio tipico | "Formale vs informale" | "Copia privata di un prompt di team" |

## Limiti noti

- La voce **"Rinomina etichetta"** nel menu della pillola è al momento
  disattivata: l'etichetta si decide alla creazione della variante.
- Non esiste una vista di confronto dedicata alle varianti con i metadati
  affiancati: per confrontarle usi il Confronto generico, selezionando le
  varianti da mettere a fianco.
- Le varianti restano su un solo livello: una variante di una variante viene
  riagganciata al principale del gruppo, non crea un terzo livello di
  nipoti.

## Vedi anche

- [`fork-prompt.md`](./fork-prompt.md) — l'altra via per copiare un prompt, quando serve una copia indipendente e privata.
- [`rating-prompt.md`](./rating-prompt.md) — le valutazioni che ti dicono quale variante rende meglio.
- [`regression-testing.md`](./regression-testing.md) — i test golden per confrontare le varianti su casi fissi.
