# Cartelle

> Come organizzare i prompt in cartelle, quando preferire i tag, e le regole che governano l'albero: creazione, spostamenti, filtri, eliminazione.

Finché il vault contiene una manciata di prompt, la ricerca basta e avanza. Ma quando i prompt diventano decine, e magari coprono progetti e clienti diversi, serve una struttura: le **cartelle** sono il modo di Prompt a Porter di dare a ogni prompt una casa. Ogni prompt sta in una sola cartella — oppure a *root*, cioè fuori da qualsiasi cartella — e le cartelle possono annidarsi in sottocartelle, formando un albero che vedi nella barra laterale della Libreria.

Le cartelle convivono con i **tag**, ma risolvono problemi diversi. La cartella risponde alla domanda "dove sta questo prompt?", e la risposta è una sola: `marketing/email`, oppure root. I tag rispondono a "che caratteristiche ha?", e le risposte possono essere molte: lo stesso prompt può essere insieme `formale`, `outreach` e `inglese`. Una buona regola pratica: cartelle per il dominio (il cliente, il progetto, l'area di lavoro), tag per le dimensioni trasversali (tono, scopo, destinatario).

C'è anche un motivo in più per curare l'albero: i path degli import (`{{import "cartella/titolo"}}`) seguono esattamente la struttura delle cartelle, quindi un albero pulito rende anche gli import più leggibili.

## Creare e gestire le cartelle

Tutta la gestione passa dal menu contestuale: nella barra laterale della Libreria, clicca col tasto destro su una cartella per aprire le azioni disponibili.

- **Nuovo prompt qui** — crea un prompt direttamente dentro la cartella.
- **Nuova sottocartella** — aggiunge un livello sotto la cartella corrente.
- **Rinomina** — cambia il nome della cartella; i percorsi di tutte le sottocartelle si aggiornano di conseguenza.
- **Elimina cartella** — elimina la cartella e tutte le sue sottocartelle, previa conferma esplicita.

Sull'eliminazione vale la pena fermarsi un momento: eliminare una cartella **non elimina i prompt che contiene**. I prompt tornano a root (li ritrovi nel filtro **Nessuna cartella**), mentre la cartella e le sue sottocartelle spariscono dall'albero. È per questo che l'app chiede conferma prima di procedere.

Le cartelle non si spostano fra loro dall'interfaccia: non c'è drag&drop né una voce "Sposta in...". I prompt invece sì: tasto destro sul prompt → **Sposta in cartella** e scegli la destinazione.

## Filtrare la lista per cartella

Nella Libreria, il click su una cartella filtra la lista ai prompt di quella cartella **e di tutto il suo sotto-albero**: selezionando `marketing` vedi anche i prompt di `marketing/email` e di ogni altra sottocartella. Il sotto-albero è sempre incluso, non c'è un interruttore per escluderlo.

L'opzione **Nessuna cartella** mostra i soli prompt a root — quelli che non hai ancora sistemato, o che hai scelto di tenere fuori dall'albero.

## Regole sui nomi e sull'albero

Le regole sono poche e pensate per evitare ambiguità. Due cartelle sorelle (stesso genitore) non possono avere lo stesso nome, ma lo stesso nome è consentito in rami diversi dell'albero: puoi avere `clienti/acme/email` e `clienti/globex/email` senza conflitti. Il nome può essere lungo al massimo 100 caratteri, non può essere vuoto e non può contenere `/` (che è riservato a separare i livelli nei percorsi).

Sulla profondità non c'è un limite rigido: l'albero regge senza problemi anche strutture articolate. È l'ergonomia a suggerire moderazione, come spiegato qui sotto.

## Consigli di organizzazione

Se ti accorgi di scendere oltre i 4-5 livelli di annidamento, probabilmente stai codificando nelle cartelle informazione che starebbe meglio nei tag: una struttura come `clienti/acme/2026/email/formali` si appiattisce bene in una cartella `clienti/acme` più i tag `email` e `formale`. Un segnale indiretto arriva anche dal linter: quando un import attraversa una catena troppo profonda scatta `IMP003`, ed è spesso il sintomo di un albero da semplificare.

Attenzione anche alla cartella-contenitore che mescola ruoli ortogonali: se in `marketing` convivono "email cold", "presentazioni keynote" e "social copy", la cartella sta facendo il lavoro dei tag. Meglio una cartella per dominio (cliente o progetto) e i tag per la tipologia.

## Limiti noti

Le cartelle sono visibili a tutti gli utenti del workspace: oggi non esistono permessi per cartella, quindi non è possibile riservare una cartella a un singolo utente o a un sottogruppo.

Le cartelle, inoltre, non si possono spostare o riordinare dall'interfaccia: per ristrutturare l'albero devi creare la nuova struttura e spostarci i prompt.

## Vedi anche

- [`glossario-sintassi.md`](./glossario-sintassi.md) — la sintassi dei path negli import, che segue l'albero delle cartelle.
- [`prompt-componibili.md`](./prompt-componibili.md) — i prompt modulari: dove una buona struttura di cartelle ripaga di più.
- [`getting-started.md`](./getting-started.md) — l'anatomia dell'interfaccia, barra laterale inclusa.
