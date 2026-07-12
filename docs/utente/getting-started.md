# Getting started

> Guida al primo utilizzo di Prompt a Porter: installazione, creazione del vault, primo prompt, prima compilazione. Tempo richiesto: 5 minuti.

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

Su Windows, l'installer chiede l'approvazione SmartScreen al primo avvio: clicca "Esegui comunque" — il binario è firmato con certificato Certum EV (vedi [`auto-update.md`](./auto-update.md) per i dettagli sulla firma).

Su macOS l'app è firmata Developer ID e notarizzata da Apple: il `.dmg` si apre normalmente, senza avvisi Gatekeeper.

## Onboarding: il primo avvio

Al primo avvio l'app mostra un wizard a 3 step:

### 1. Benvenuto e nome del vault

Il vault è il file SQLite che contiene tutti i tuoi prompt. Scegli il nome nel campo **"Nome del vault"**. Il file viene salvato di default in:

- Windows: `%APPDATA%\com.pap.client\pap-vault.db`
- macOS: `~/Library/Application Support/com.pap.client/pap-vault.db`
- Linux: `~/.local/share/com.pap.client/pap-vault.db`

### 2. Cifra il tuo vault

La cifratura (SQLCipher, AES-256) è **opzionale ma consigliata**. Scegli una password robusta (minimo 12 caratteri, mix lettere/numeri/simboli). **La password non è recuperabile:** se la dimentichi devi ricreare il vault. Considera di salvarla in un password manager.

Se preferisci un vault in chiaro, attiva il toggle **"Salta cifratura del vault"** (segnalato con badge "Sconsigliato").

### 3. Hotkey e opzioni finali

La hotkey di default è `Ctrl+Shift+P` (Windows/Linux) o `⌃⇧P` (macOS). La hotkey funziona anche quando l'app non è in primo piano: premila in qualunque momento per evocare la Command Palette. Puoi cambiarla in qualsiasi momento da **Impostazioni → Sistema → Hotkey** (la modifica ha effetto al prossimo riavvio dell'app).

Nello stesso step trovi due switch:

- **"Importa prompt di esempio al primo avvio"** (attivo di default): popola il vault con prompt dimostrativi. Disattivalo se preferisci partire da vuoto.
- **"Avvia automaticamente con \<nome OS\>"** (l'etichetta si adatta al tuo sistema operativo): avvia PaP al login.

### Dopo il wizard: tour di benvenuto

Al primo ingresso nell'app parte un breve **tour guidato** dell'interfaccia e una checklist **"Primi passi"** con le azioni essenziali. Puoi rilanciare il tour in qualunque momento da **Impostazioni → Guida e aiuto → "Avvia il tour guidato"**.

## Anatomia dell'interfaccia

Dopo l'onboarding entri nella **Shell**, l'interfaccia principale a 3 colonne:

```
┌──────────┬──────────────┬───────────────────────────┐
│ Sidebar  │ List Pane    │ Right Rail / Editor       │
│          │              │                           │
│ • Tutti  │  Prompt 1    │  Titolo del prompt        │
│ • Pref.  │  Prompt 2    │  Body con {{segnaposti}}  │
│ • Cart.  │  Prompt 3    │                           │
│ • Tag    │  ...         │  Tag, modello, varianti   │
└──────────┴──────────────┴───────────────────────────┘
```

- **Sidebar (sinistra):** viste predefinite (Preferiti, Tutti i prompt, Cestino, Privati, Team), cartelle, tag; nel footer il link "Regressioni". Il **Cestino** raccoglie i prompt eliminati (soft-delete): da lì puoi ripristinarli o eliminarli definitivamente con "Svuota cestino".
- **List Pane (centro):** lista dei prompt nella vista corrente, con anteprima del body.
- **Right Rail (destra):** dettaglio del prompt selezionato — titolo, body, tag, modello target, varianti, cronologia.

In alto: barra di ricerca + filtri rapidi. In basso: status bar con conteggio prompt e hotkey suggerite.

## Il tuo primo prompt

1. Clicca **Nuovo** in alto nel ListPane (la colonna centrale).
2. Compila i campi:
   - **Titolo:** "Email di reclamo professionale"
   - **Descrizione:** una riga che spieghi cosa fa il prompt (opzionale ma consigliata: appare nella ricerca).
   - **Body:** il testo del prompt, con segnaposti per le parti variabili. Esempio:

     ```
     Scrivi un'email di reclamo professionale per {{servizio}} riguardo {{problema}}.

     Tono: {{tono}}. Lingua: italiano.

     Includi:
     - Riferimento al problema in oggetto
     - Conseguenze concrete subite
     - Richiesta esplicita di rimborso o soluzione
     - Scadenza per la risposta (7 giorni)
     ```

3. Imposta **Modello target** (es. `claude-sonnet`, `gpt-4`) se il prompt è ottimizzato per un modello specifico.
4. Aggiungi **tag** (es. `email`, `clienti`, `reclami`) per ritrovarlo facilmente.
5. Le modifiche vengono salvate **automaticamente** ~2 secondi dopo l'ultima edit (autosave debounced; il ritardo è configurabile in **Impostazioni → Editor**). Nessuna azione esplicita richiesta.

## La tua prima compilazione

Ora compila il prompt con valori reali:

1. Premi la **hotkey globale** (`Ctrl+Shift+P`) — anche da fuori l'app.
2. La **Command Palette** si apre al centro dello schermo: digita "reclamo" per trovare il prompt.
3. Premi `Enter` sul risultato: si apre la modale **Compila**.
4. Inserisci i valori dei segnaposti:
   - `servizio`: `Trenitalia`
   - `problema`: `treno cancellato senza preavviso`
   - `tono`: `formale ma fermo`
5. Premi `Ctrl+Enter` (o "Compila & copia"): il testo compilato è automaticamente copiato negli appunti.
6. Incolla in ChatGPT / Claude / dovunque: ✅

## Vedi anche

- [`glossario-sintassi.md`](./glossario-sintassi.md) — sintassi avanzata: segnaposti, import fra prompt, valori globali.
- [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) — la lista completa delle scorciatoie.
- [`casi-uso/`](./casi-uso/README.md) — ricette concrete pronte all'uso (email, code review, summarize, …).
- [`troubleshooting.md`](./troubleshooting.md) — FAQ e soluzioni ai problemi più comuni.
