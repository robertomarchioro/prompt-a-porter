# Fork dei prompt

> Quando e come clonare un prompt in una copia indipendente ma tracciata:
> il banner "Fork di X", le catene di fork, e la differenza con le varianti.

Prima o poi ti capita un prompt che non è del tutto tuo — uno di team,
scritto da un collega — che vorresti adattare al tuo modo di lavorare. Non
puoi modificarlo direttamente senza pestare i piedi a chi lo mantiene, e
duplicarlo con un copia-incolla ti lascia fra le mani una copia orfana: fra
un mese non ricorderai da dove veniva né se nel frattempo l'originale è
cambiato.

Un **fork** è la copia fatta bene. È una copia indipendente del prompt — tua,
privata, modificabile a piacere — che però conserva un filo verso
l'originale. Puoi sperimentare quanto vuoi senza toccare il prompt di
partenza, e in ogni momento sai da dove sei partito, con un clic per tornare
alla sorgente.

Il caso d'uso tipico è proprio questo: adattare al proprio flusso un prompt
di team senza chiedere permesso e senza rischi. Ma il fork serve anche per un
refactor sperimentale su un prompt critico — forki, modifichi, provi con i
[test golden](./regression-testing.md), e solo se convince decidi se
sostituire l'originale — o semplicemente per conservare una tua copia stabile
di qualcosa che altri potrebbero cambiare.

## La prima volta

1. Apri il prompt che vuoi clonare nel pannello di dettaglio.
2. Clicca l'icona **Fork** nella barra degli strumenti dell'editor. In
   alternativa, dalla lista della Libreria, usa la voce **"Duplica (fork)"**
   del menu contestuale (clic destro sul prompt).
3. Viene creata la copia e l'app la apre subito, pronta da modificare.

La copia nasce con alcune proprietà fissate, pensate per renderla davvero
"tua e privata":

| Campo | Valore del fork |
|---|---|
| Titolo | `<originale> (fork)` |
| Visibilità | privata (forzata, anche se l'originale era di team) |
| Workspace | il tuo workspace personale |
| Tag | tutti copiati dall'originale |
| Body, modello di destinazione, cartella | ereditati dall'originale |

Il body, il modello e la cartella li erediti dall'originale come punto di
partenza; visibilità e workspace vengono invece forzati per garantire che
il fork resti una faccenda privata.

## Il filo verso l'originale

Quando apri un fork, sotto i metadati compare un banner che lo lega alla sua
origine:

```
Fork di "Nome originale"
```

Cliccando il banner salti al prompt originale. Se l'originale è stato
spostato nel cestino, il banner lo segnala come `(eliminato)` e smette di
essere cliccabile; nel caso raro in cui l'originale sia sparito del tutto, il
banner mostra `Fork di un prompt non più disponibile` — ma il fork continua
a funzionare senza problemi, perché è a tutti gli effetti autonomo.

## Fork di un fork: le catene

Puoi forkare un fork. Ogni fork punta sempre al suo **diretto** genitore, non
all'origine remota della catena:

```
prm-orig  ←  prm-fork-A  ←  prm-fork-B
```

Qui `prm-fork-B` è fork di `prm-fork-A`, che a sua volta è fork di
`prm-orig`. Per risalire all'origine di una catena lunga, segui i banner un
passo alla volta.

## Differenza con le varianti

Fork e [varianti](./varianti-prompt.md) partono entrambi da un prompt
esistente, ma servono a cose diverse. La variante resta nella famiglia del
prompt — stesso intento, stesso workspace, stessa visibilità — e serve a
confrontare formulazioni alternative fianco a fianco. Il fork invece stacca
una copia con responsabilità separata, sempre privata e nel tuo workspace
personale, pensata per lavorare in autonomia su qualcosa che non controlli
del tutto. In breve: la variante è "un altro modo di dire la stessa cosa"; il
fork è "la mia copia, con cui faccio quello che voglio".

## Confrontare il fork con l'originale

Per vedere in cosa il tuo fork si è discostato dall'originale, selezionali
entrambi nella lista e apri il **Confronto** (bottone **Confronta**,
disponibile con 2-4 prompt selezionati): i due body compaiono affiancati,
colonna contro colonna.

## Limiti noti

- Il banner del fork non mostra le differenze rispetto all'originale: per
  vederle devi usare il Confronto, selezionando fork e originale.
- Non c'è un contatore di "quanti fork" ha un prompt originale: dal lato
  dell'originale non vedi chi lo ha forkato.
- Non esiste un modo per riproporre le modifiche di un fork all'originale
  ("proponi questa modifica"): il fork resta una copia separata, e riportare
  i cambiamenti indietro è un'operazione manuale.

## Vedi anche

- [`varianti-prompt.md`](./varianti-prompt.md) — l'alternativa quando vuoi confrontare formulazioni nello stesso workspace, non staccare una copia privata.
- [`regression-testing.md`](./regression-testing.md) — i test golden per validare un fork prima di sostituire l'originale.
- [`cartelle.md`](./cartelle.md) — come sono organizzati workspace e cartelle dove finiscono i fork.
