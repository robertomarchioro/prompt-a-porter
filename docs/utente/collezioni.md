# Collezioni di prompt

> Raccolte di prompt curate dal progetto, una per tipo di lavoro, da importare nel vault con un click. Impostazioni → Dati → **Collezioni di prompt**.

Un vault vuoto è una pagina bianca: sai che l'app può fare molto, ma non da dove cominciare. Le collezioni rispondono a questo. Ognuna raccoglie una dozzina di prompt scritti per un tipo di lavoro — chi sviluppa, chi scrive — e costruiti apposta per mostrare ciò che distingue Prompt à Porter da un blocco note: segnaposti da compilare, ruoli riusabili con `{{import}}`, variabili passate con `with`, varianti A/B, commenti che non partono col prompt.

Sono prompt originali del progetto, in italiano, sotto la stessa licenza del repository. Non c'è nulla da pagare e nessun account: i file stanno nel [repository pubblico](https://github.com/robertomarchioro/prompt-a-porter/tree/main/docs/collezioni), e chiunque può proporne di nuove.

## Come si importano

1. Apri **Impostazioni → Dati**.
2. Nella card **Collezioni di prompt** premi **Sfoglia collezioni…**. È l'unico momento in cui l'app si collega a GitHub per scaricare l'elenco: non succede da solo all'apertura delle impostazioni.
3. Scegli una collezione e premi **Importa**. In pochi secondi i prompt compaiono nella libreria, dentro la cartella `Collezioni` (una sottocartella per collezione).

Il riepilogo sotto la collezione ti dice quanti elementi sono stati aggiunti e quanti erano già presenti.

## Cosa succede ai prompt che hai già

Niente. L'importazione lavora sempre in modalità *salta*: un prompt, un tag o una cartella già presenti nel vault non vengono mai sovrascritti. Puoi importare la stessa collezione due volte senza creare doppioni — la seconda volta il riepilogo dirà «già presenti».

Un dettaglio utile: se hai già un tag con lo stesso nome di uno della collezione (per esempio `codice`), i prompt importati si agganciano al tuo tag invece di crearne un secondo.

Una volta importati, i prompt sono tuoi come tutti gli altri: modificali, spostali, cancellali. Aggiornare l'app non li tocca e non li rimette a posto.

## Le collezioni disponibili

| Collezione | Cosa contiene |
|---|---|
| **Sviluppatore** | Code review di una pull request, test unitari mirati, refactoring guidato (con variante *conservativa*), spiegazione del codice, messaggio di commit e descrizione di PR, diagnosi di un errore, documentazione di funzioni, piano di implementazione, SQL spiegata e ottimizzata. Due ruoli da importare: *tech lead* e *reviewer rigoroso*. |
| **Scrittura** | Email di follow-up e email difficili, riscrittura di registro (con variante *accorcia*), sintesi in punti, dieci titoli, post per i social, revisione critica, correzione di bozze, traduzione naturale. Due ruoli da importare: *editor di testi* e *copywriter*. |

Ogni collezione è pensata per essere letta, non solo usata: apri un prompt, guarda come importa il ruolo, come fissa un segnaposto con `with`, dove usa un commento `{{!-- --}}` per lasciarsi una nota. Sono esempi da copiare per i tuoi.

## Se qualcosa non va

- **«Verifica la connessione»**: l'elenco e i file arrivano da `raw.githubusercontent.com`. Se sei offline, o quel dominio è bloccato dalla rete aziendale, l'importazione non può funzionare; l'app non tiene copie locali delle collezioni.
- **«Il file della collezione non corrisponde all'indice»**: ogni file scaricato viene verificato contro un'impronta pubblicata nell'indice. Il caso tipico è una collezione appena aggiornata, con la cache di GitHub non ancora allineata: riprova fra qualche minuto. Se persiste, segnalalo.
- **Un import in rosso nell'editor** (`IMP001`) dopo l'importazione: succede se hai rinominato o spostato il prompt "ruolo" che un altro prompt della collezione importa. Rimetti il titolo originale, o aggiorna l'import nel prompt che lo usa.

## Per chi vuole proporre una collezione

Le collezioni sono file JSON nel formato di [export del vault](./formato-export-json.md), in `docs/collezioni/` del repository. La procedura per aggiungerne una — formato, indice, test che la verificano — è nel [README di quella cartella](https://github.com/robertomarchioro/prompt-a-porter/blob/main/docs/collezioni/README.md).

## Vedi anche

- [`prompt-componibili.md`](./prompt-componibili.md) — gli `{{import}}` che le collezioni usano per i ruoli.
- [`varianti-prompt.md`](./varianti-prompt.md) — le varianti *conservativo* e *accorcia*.
- [`formato-export-json.md`](./formato-export-json.md) — il formato dei file.
