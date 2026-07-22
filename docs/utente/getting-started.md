# Getting started

> Guida al primo utilizzo di Prompt a Porter: installazione, creazione del vault, primo prompt, prima compilazione. Tempo richiesto: 5 minuti.

Se usi l'AI ogni giorno, prima o poi ti accorgi che i prompt migliori li riscrivi sempre daccapo: li cerchi nella cronologia della chat, li ricopi da un file di appunti, li ricostruisci a memoria. Prompt a Porter nasce per chiudere questo cerchio: è un'app desktop che custodisce i tuoi prompt in un archivio locale, li rende ricercabili in un istante e ti permette di riusarli compilando solo le parti che cambiano.

Il percorso di questa pagina è quello che farai davvero la prima volta: installi l'app, crei il tuo **vault** (l'archivio che conterrà tutto), scrivi un primo prompt con qualche parte variabile, e infine lo compili con valori reali per incollarlo dove ti serve. Alla fine avrai toccato con mano il ciclo completo — scrivi una volta, riusa per sempre — e saprai orientarti nell'interfaccia.

Non serve alcun account e nessun dato lascia il tuo computer: Prompt a Porter è local-first. Tutto quello che scrivi vive in un file sul tuo disco, che puoi cifrare, esportare e portare con te.

## Prerequisiti

- Windows 10+, macOS 12+ o Linux con GTK 3 (la maggior parte delle distro recenti).
- ~150 MB di spazio su disco (binario + vault iniziale).
- Nessun account: PaP è local-first.

## Installazione

Scarica l'ultima release da [GitHub Releases](https://github.com/robertomarchioro/prompt-a-porter/releases/latest):

| Piattaforma | Bundle |
|---|---|
| Windows | `Prompt.a.Porter_X.Y.Z_x64-setup.exe` (installer, firmato Authenticode) o `Prompt-a-Porter-portable-windows-x64-vX.Y.Z.zip` (portable) |
| macOS | `Prompt.a.Porter_X.Y.Z_universal.dmg` (universale: Apple Silicon + Intel) |
| Linux | `Prompt.a.Porter_X.Y.Z_amd64.AppImage` o `Prompt.a.Porter_X.Y.Z_amd64.deb` |

Su Windows, l'installer chiede l'approvazione SmartScreen al primo avvio: clicca "Altre informazioni" e poi "Esegui comunque" — il binario è firmato con certificato Certum EV (vedi [`auto-update.md`](./auto-update.md) per i dettagli sulla firma).

Su macOS l'app è firmata Developer ID e notarizzata da Apple: il `.dmg` si apre normalmente, senza avvisi Gatekeeper.

## Onboarding: il primo avvio

Al primo avvio l'app ti accompagna con un wizard in 3 step. Serve a prendere le tre decisioni iniziali — come chiamare il vault, se cifrarlo, quale scorciatoia usare per richiamare l'app — e richiede meno di un minuto.

### 1. Benvenuto e nome del vault

Il vault è il file che contiene tutti i tuoi prompt: un unico archivio SQLite, facile da copiare e da salvare in backup. Scegli il nome nel campo **"Nome del vault"**. Il file viene salvato di default in:

- Windows: `%APPDATA%\com.pap.client\pap-vault.db`
- macOS: `~/Library/Application Support/com.pap.client/pap-vault.db`
- Linux: `~/.local/share/com.pap.client/pap-vault.db`

### 2. Cifra il tuo vault

La cifratura (SQLCipher, AES-256) è **opzionale ma consigliata**: protegge i tuoi prompt anche se qualcun altro mette le mani sul file. Scegli una password robusta (minimo 8 caratteri, mix lettere/numeri/simboli). **La password non è recuperabile:** se la dimentichi devi ricreare il vault. Considera di salvarla in un password manager.

Se preferisci un vault in chiaro, attiva il toggle **"Salta cifratura del vault"** (segnalato con badge "Sconsigliato").

### 3. Hotkey e opzioni finali

La hotkey è la scorciatoia che richiama Prompt a Porter da qualunque punto del sistema, anche quando l'app non è in primo piano: è il gesto che userai più spesso. Quella di default è `Ctrl+Shift+P` (Windows/Linux) o `⌃⇧P` (macOS); premila in qualunque momento per evocare la Command Palette. Puoi cambiarla in qualsiasi momento da **Impostazioni → Sistema → Hotkey** (la modifica ha effetto al prossimo riavvio dell'app).

Nello stesso step trovi due switch:

- **"Importa prompt di esempio al primo avvio"** (attivo di default): popola il vault con prompt dimostrativi. Disattivalo se preferisci partire da vuoto.
- **"Avvia automaticamente con \<nome OS\>"** (l'etichetta si adatta al tuo sistema operativo): avvia PaP al login.

### Dopo il wizard: tour di benvenuto

Al primo ingresso nell'app parte un breve **tour guidato** dell'interfaccia e una checklist **"Primi passi"** con le azioni essenziali. Puoi rilanciare il tour in qualunque momento da **Impostazioni → Guida e aiuto → "Avvia il tour guidato"**.

## Anatomia dell'interfaccia

Finito l'onboarding entri nella finestra principale, organizzata in 3 colonne — da sinistra a destra: dove cerchi, cosa hai trovato, cosa stai guardando:

```
┌──────────────┬──────────────┬───────────────────────────┐
│ Barra        │ Lista dei    │ Pannello di dettaglio     │
│ laterale     │ prompt       │                           │
│              │              │  Titolo del prompt        │
│ • Tutti      │  Prompt 1    │  Body con {{segnaposti}}  │
│ • Preferiti  │  Prompt 2    │                           │
│ • Cartelle   │  Prompt 3    │  Tag, modello, varianti   │
│ • Tag        │  ...         │                           │
└──────────────┴──────────────┴───────────────────────────┘
```

La **barra laterale** (a sinistra) è il punto di partenza di ogni ricerca: raccoglie le viste predefinite (Preferiti, Tutti i prompt, Cestino, Privati, Team), l'albero delle cartelle e i tag; nel footer trovi il link "Regressioni". Il **Cestino** merita una nota: i prompt eliminati non spariscono subito, ma finiscono lì — da dove puoi ripristinarli o eliminarli definitivamente con "Svuota cestino".

La **lista dei prompt** (colonna centrale) mostra i prompt della vista corrente, ognuno con un'anteprima del body: è qui che scorri e selezioni.

Il **pannello di dettaglio** (a destra) mostra tutto del prompt selezionato — titolo, body, tag, modello target, varianti, cronologia — ed è dove lo modifichi.

In alto trovi la barra di ricerca con i filtri rapidi; in basso, la status bar con il conteggio dei prompt e le hotkey suggerite.

## Il tuo primo prompt

1. Clicca **Nuovo** in alto nella colonna della lista (quella centrale).
2. Compila i campi:
   - **Titolo:** "Email di reclamo professionale"
   - **Descrizione:** una riga che spieghi cosa fa il prompt (opzionale ma consigliata: appare nella ricerca).
   - **Body:** il testo del prompt, con segnaposti per le parti variabili.

     Questo esempio mostra un prompt riusabile: le parti fra doppie graffe sono i segnaposti, cioè i punti che cambieranno a ogni uso.

     ```
     Scrivi un'email di reclamo professionale per {{servizio}} riguardo {{problema}}.

     Tono: {{tono}}. Lingua: italiano.

     Includi:
     - Riferimento al problema in oggetto
     - Conseguenze concrete subite
     - Richiesta esplicita di rimborso o soluzione
     - Scadenza per la risposta (7 giorni)
     ```

     Al momento dell'uso l'app ti chiederà i valori di `servizio`, `problema` e `tono`: il resto del testo resta identico, già rifinito una volta per tutte.

3. Imposta **Modello target** (es. `claude-sonnet`, `gpt-4`) se il prompt è ottimizzato per un modello specifico.
4. Aggiungi **tag** (es. `email`, `clienti`, `reclami`) per ritrovarlo facilmente.
5. Non c'è un pulsante Salva: le modifiche vengono salvate **automaticamente** circa 2 secondi dopo che smetti di scrivere (il ritardo è configurabile in **Impostazioni → Editor**).

## La tua prima compilazione

Ora chiudi il cerchio: usa il prompt appena scritto compilandolo con valori reali. È il gesto che ripeterai ogni giorno.

1. Premi la **hotkey globale** (`Ctrl+Shift+P`) — funziona anche se stai lavorando in un'altra app.
2. La **Command Palette** si apre al centro dello schermo: digita "reclamo" per trovare il prompt.
3. Premi `Enter` sul risultato: si apre la modale **Compila**, con un campo per ogni segnaposto.
4. Inserisci i valori:
   - `servizio`: `Trenitalia`
   - `problema`: `treno cancellato senza preavviso`
   - `tono`: `formale ma fermo`
5. Premi `Ctrl+Enter` (o il pulsante "Compila & copia"): il testo compilato finisce automaticamente negli appunti.
6. Incolla in ChatGPT, Claude o dovunque ti serva: il prompt è pronto, completo dei tuoi valori.

Da qui in poi il flusso è sempre lo stesso: hotkey, due lettere nella palette, `Enter`, valori, `Ctrl+Enter`. Più prompt accumuli nel vault, più il gesto ripaga.

## Vedi anche

- [`glossario-sintassi.md`](./glossario-sintassi.md) — la sintassi completa del body: segnaposti, valori globali, import fra prompt.
- [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) — la lista completa delle scorciatoie, per usare l'app senza mouse.
- [`cartelle.md`](./cartelle.md) — come organizzare il vault quando i prompt iniziano a crescere.
- [`troubleshooting.md`](./troubleshooting.md) — soluzioni ai problemi più comuni di installazione e primo avvio.
