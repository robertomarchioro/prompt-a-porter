# Getting started

Guida al primo utilizzo di Prompt a Porter: installazione, creazione del vault, primo prompt, prima compilazione.

> **Tempo richiesto:** 5 minuti.

## Prerequisiti

- Windows 10+, macOS 12+ o Linux con GTK 3 (la maggior parte delle distro recenti).
- ~150 MB di spazio su disco (binario + vault iniziale).
- Nessun account: PaP è local-first.

## Installazione

Scarica l'ultima release da [GitHub Releases](https://github.com/robertomarchioro/prompt-a-porter/releases/latest):

| Piattaforma | Bundle |
|---|---|
| Windows | `prompt-a-porter_x.y.z_x64-setup.exe` (firmato Authenticode) o `.zip` portable |
| macOS | `prompt-a-porter_x.y.z_universal.dmg` |
| Linux | `prompt-a-porter_x.y.z_amd64.AppImage` o `.deb` |

Su Windows, l'installer chiede l'approvazione SmartScreen al primo avvio: clicca "Esegui comunque" — il binario è firmato con certificato Certum EV (vedi [`auto-update.md`](./auto-update.md) per i dettagli sulla firma).

Su macOS, al primo avvio Gatekeeper potrebbe segnalare l'app come "non identificato": Tasto destro → Apri → conferma. È un comportamento atteso finché non viene completata la notarization (in roadmap post-v1.0).

## Onboarding: il primo avvio

Al primo avvio l'app mostra un wizard a 3 step:

### 1. Crea il vault

Il vault è il file SQLite cifrato (SQLCipher, AES-256) che contiene tutti i tuoi prompt. Viene salvato di default in:

- Windows: `%APPDATA%\com.pap.app\pap-vault.db`
- macOS: `~/Library/Application Support/com.pap.app/pap-vault.db`
- Linux: `~/.local/share/com.pap.app/pap-vault.db`

Scegli una password robusta (12+ caratteri, mix lettere/numeri/simboli). **La password non è recuperabile:** se la dimentichi devi ricreare il vault. Considera di salvarla in un password manager.

### 2. Imposta la hotkey globale

La hotkey di default è `Ctrl+Shift+P` (Windows/Linux) o `⌃⇧P` (macOS). La hotkey funziona anche quando l'app non è in primo piano: premila in qualunque momento per evocare la Command Palette.

Puoi cambiarla in qualsiasi momento da **Impostazioni → Generale → Hotkey**.

### 3. Scegli se creare i prompt di esempio

L'opzione "Crea prompt di esempio" (attiva di default) popola il vault con prompt dimostrativi. Disattivala se preferisci partire da vuoto.

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

- **Sidebar (sinistra):** viste predefinite (Tutti, Preferiti, Privati, Team), cartelle, tag.
- **List Pane (centro):** lista dei prompt nella vista corrente, con anteprima del body.
- **Right Rail (destra):** dettaglio del prompt selezionato — titolo, body, tag, modello target, varianti, cronologia.

In alto: barra di ricerca + filtri rapidi. In basso: status bar con conteggio prompt e hotkey suggerite.

## Il tuo primo prompt

1. Clicca **Nuovo prompt** in alto a destra (o `Ctrl+N`).
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
5. Salva con `Ctrl+S` o il pulsante in alto.

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

## Prossimi passi

- **Sintassi avanzata:** segnaposti tipizzati, import fra prompt, valori globali — vedi [`glossario-sintassi.md`](./glossario-sintassi.md).
- **Scorciatoie:** [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) per la lista completa.
- **Casi d'uso pratici:** [`casi-uso/`](./casi-uso/) raccoglie ricette concrete (email, code review, summarize, …).
- **Problemi?** [`troubleshooting.md`](./troubleshooting.md) per FAQ e soluzioni.
- **Integrazione AI:** [`mcp.md`](./mcp.md) per esporre il vault a Claude Desktop, Cursor e altri agenti via MCP.
- **CLI:** [`cli.md`](./cli.md) per usare il vault dal terminale.
