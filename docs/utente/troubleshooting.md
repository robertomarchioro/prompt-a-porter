# Troubleshooting

> Risposte ai problemi più comuni durante l'uso di Prompt a Porter. Se non trovi quello che cerchi, apri un'issue su [GitHub](https://github.com/robertomarchioro/prompt-a-porter/issues/new/choose).

Qualcosa non funziona come dovrebbe? Quasi sempre la causa è una di quelle raccolte in questa pagina: un avviso di sicurezza del sistema operativo al primo avvio, una scorciatoia contesa fra due programmi, un dettaglio di sintassi nel body. Ogni voce spiega prima quando il problema si presenta, poi come uscirne — nella maggior parte dei casi bastano un paio di minuti.

Le sezioni seguono il ciclo di vita dell'app: installazione e primo avvio, password e vault, hotkey, editor e compilazione, prestazioni, sync, backup, aggiornamenti, log. Usa l'indice della pagina per saltare direttamente al tuo caso.

## Installazione e primo avvio

### Windows: "SmartScreen ha bloccato l'app"

Il binario è firmato con certificato Authenticode Certum **EV**, che dà a SmartScreen reputazione immediata: download e installazione, verificati su macchine reali, non mostrano avvisi. Questo avviso **non è lo stato atteso**: se compare, con ogni probabilità il download è corrotto o proviene da una fonte non ufficiale.

**Soluzione**: riscarica l'installer dalla [pagina release ufficiale](https://github.com/robertomarchioro/prompt-a-porter/releases/latest) e riprova. Se l'avviso persiste, clicca "Altre informazioni" e verifica il publisher prima di scegliere "Esegui comunque"; poi apri un'issue: è un caso che vogliamo vedere. Dettagli sulla firma in [`auto-update.md`](./auto-update.md).

### macOS: "L'app non può essere aperta perché lo sviluppatore non è verificato"

L'app è firmata Developer ID e **notarizzata da Apple**, quindi questo avviso **non è lo stato atteso**: se compare, con ogni probabilità il download è corrotto o proviene da una fonte non ufficiale.

**Soluzione**: riscarica il `.dmg` dalla [pagina release ufficiale](https://github.com/robertomarchioro/prompt-a-porter/releases/latest) e reinstalla. Se il problema persiste, apri un'issue: è un caso che vogliamo vedere.

### Linux: AppImage non si avvia

Succede tipicamente su distribuzioni recenti (Ubuntu 22.04+) dove il pacchetto `libfuse2` non è più installato di default: l'AppImage richiede FUSE 2 (non FUSE 3) per montarsi.

**Soluzione**: installa la libreria mancante e riprova:

```bash
sudo apt install libfuse2
```

In alternativa, usa il pacchetto `.deb`, che non ha questa dipendenza.

### Linux (XFCE / Xubuntu): l'icona nel tray non appare

Se usi XFCE — quindi anche **Xubuntu** — è probabile che l'app parta regolarmente ma l'icona nel pannello non si veda. L'area di notifica di default ("Area di notifica (legacy)") mostra solo icone vecchio stile *xembed* e **ignora** le icone moderne *StatusNotifier/AppIndicator* che usa Prompt a Porter: l'app pubblica correttamente l'icona, ma manca chi la mostra nel pannello.

**Soluzione**: aggiungi al pannello il plugin **Status Notifier**:

```bash
sudo apt install xfce4-statusnotifier-plugin
```

Poi tasto destro sul pannello → **Pannello → Aggiungi nuovi elementi…** → aggiungi **"Status Notifier Plugin"**. Riavvia l'app: l'icona compare accanto all'orologio.

Su GNOME serve un'estensione tipo *AppIndicator/KStatusNotifier Support*; su KDE Plasma il tray funziona nativamente. (La libreria di sistema `libayatana-appindicator3-1` è comunque richiesta: il pacchetto `.deb` la installa come dipendenza.)

### "Vault non trovato" / dialog vault vuoto

Compare quando l'app non trova il file del vault nei percorsi di default (elencati in [`getting-started.md`](./getting-started.md), sezione Onboarding) — per esempio dopo aver spostato il file, cambiato macchina o cancellato la cartella dati. Niente di irreparabile: se il file non esiste, l'onboarding si avvia e ti guida nella creazione di un nuovo vault.

Una precisazione utile se arrivi dalla riga di comando: l'app desktop **non legge variabili d'ambiente** per il path del vault. La variabile `PAP_VAULT_PATH` vale solo per la CLI `pap` e per il server MCP:

```bash
PAP_VAULT_PATH=/percorso/del/vault.db pap list
```

## Password e vault

### Ho dimenticato la password del vault

È la situazione più delicata di questa pagina, quindi diciamolo subito con chiarezza: **non c'è recupero possibile**. Il vault è cifrato con SQLCipher AES-256 e senza password il contenuto è inaccessibile, anche a noi. È il prezzo (voluto) di una cifratura seria.

**Cosa puoi fare:**
1. Se hai un export JSON recente (**Impostazioni → Dati → Esporta**), crea un nuovo vault con una nuova password e reimporta.
2. Se hai un backup del file `.db` con la vecchia password, ripristinalo.
3. Altrimenti: ricrea il vault da zero.

> **Per evitarlo in futuro**: salva la password in un password manager (1Password, Bitwarden, KeePass). Considera anche un export JSON periodico in cloud personale.

### La password ha caratteri speciali e non funziona

Capita soprattutto passando da un sistema operativo all'altro: la password è giusta, ma i caratteri che digiti non sono quelli che credi. Verifica il layout di tastiera. Su Windows/Linux, alcune password copiate da macOS contengono caratteri Unicode ambigui (es. `⌘`). Reimposta la password da macOS o ricreala usando solo ASCII.

### Cambio password: "Errore durante il rekey"

Compare quando provi a cambiare la password mentre il vault è in uso da un altro processo: la stessa app aperta due volte, la CLI, il server MCP. Chiudi:

- Tutte le istanze dell'app desktop
- La CLI `pap` (se attiva)
- Il server MCP (se collegato)

Poi ripeti il cambio password da **Impostazioni → Sicurezza**. Il vault non subisce danni: l'operazione viene semplicemente rifiutata finché il file è condiviso.

## Hotkey globale

### La hotkey non funziona

Di solito succede in due momenti: subito dopo l'installazione (permessi di sistema non ancora concessi) oppure dopo l'installazione di un altro programma che si è preso la stessa combinazione. Verifica nell'ordine:

1. **Conflitto con altra app**: prova a chiudere altri programmi che potrebbero usare la stessa combinazione (clipboard manager, Spotlight custom, Raycast, AutoHotkey su Windows). Riavvia PaP dopo aver chiuso il conflitto.
2. **Permessi macOS**: la prima volta che PaP registra una scorciatoia globale, macOS chiede l'autorizzazione Accessibilità. Vai in **System Settings → Privacy & Security → Accessibility** e abilita PaP.
3. **Linux Wayland**: alcuni desktop environment (GNOME Wayland) hanno restrizioni sulle scorciatoie globali. Considera X11 o usa la palette dall'interno dell'app (`Ctrl+K`).
4. **Reimposta default**: **Impostazioni → Sistema → Hotkey**, reinserisci `Ctrl+Shift+P` nel campo e salva. La modifica ha effetto al prossimo riavvio dell'app.

### La hotkey funziona ma apre il programma sbagliato

Segno che un'altra app ha registrato la stessa combinazione **dopo** PaP e ha "vinto" la scorciatoia. Su Windows succede tipicamente con programmi installati di recente. La via più rapida è cambiare la hotkey di PaP in qualcosa di più specifico (es. `Ctrl+Alt+Shift+P`), così i due programmi smettono di contendersela.

## Editor e compilazione

### Le modifiche all'editor non vengono salvate

Prima di preoccuparti: quasi sempre il salvataggio è avvenuto, solo non c'è un pulsante che lo dica. PaP salva **automaticamente** circa 2 secondi dopo l'ultima modifica: smetti di scrivere per 2-3 secondi e il salvataggio scatta da solo. Per verificarlo:

- Guarda l'indicatore "Salvato" / spinner accanto al titolo del prompt.
- Apri la tab **Cronologia** nel pannello di dettaglio: compaiono nuove voci?
- Se nessuna delle due risponde, può essere un bug: chiudi e riapri il prompt; se persiste, apri un'issue con screenshot.

### "Vault locked" durante l'edit

Il vault non si blocca da solo: si blocca solo se premi il pulsante **"Blocca vault"** in **Impostazioni → Sicurezza**. Reinserisci la password per sbloccare e riprendi da dove eri. In condizioni normali il blocco scatta dopo il salvataggio automatico, quindi non perdi nulla; solo le modifiche degli ultimissimi secondi (quelle ancora in attesa del salvataggio) potrebbero, in rari casi, andare perse.

### `{{segnaposto}}` non viene sostituito durante la compilazione

Se nel testo compilato ritrovi il segnaposto tale e quale, la causa è quasi sempre una di queste tre:

- **Nome non valido**: spazi o trattini nel nome (`{{nome con spazi}}`, `{{nome-con-trattini}}`). Il linter avvisa con `PH003`. Usa underscore: `{{nome_con_spazi}}`.
- **Singola graffa**: `{nome}` non è un segnaposto. Devono essere doppie: `{{nome}}`. Il linter `PH001` lo segnala come errore.
- **Form lasciata vuota**: se non scrivi un valore, il segnaposto resta intatto di proposito (così puoi compilarlo manualmente dopo). Non è un errore: è il comportamento previsto.

### Import `{{import "..."}}` non risolve

Quando la tab Diagnosi segnala un import rotto, o la compilazione non espande il contenuto atteso, controlla queste cause in ordine di frequenza:

- **Path errato**: verifica il titolo esatto del prompt importato (il confronto ignora maiuscole/minuscole, ma non perdona i refusi). Il linter `IMP001` segnala "Import non risolto".
- **Ciclo**: A importa B che importa A. Il linter `IMP002` blocca la compilazione. Rivedi la catena.
- **Profondità eccessiva**: oltre 5 livelli di nidificazione. Appiattisci gli import o accorpa in un unico prompt.
- **Path con cartella vs root**: `"marketing/email/cold"` cerca il prompt esattamente nella cartella `marketing/email`. Se il prompt è a root, usa solo `"cold"`.

Vedi [`prompt-componibili.md`](./prompt-componibili.md) per la sintassi completa.

## Prestazioni

### L'app è lenta all'apertura

Se la lentezza riguarda solo il primo avvio, è normale: con la ricerca semantica abilitata l'app carica il modello di embedding (~150 MB), e gli avvii successivi riusano la copia già scaricata.

Se la lentezza persiste anche dopo:

- Disabilita temporaneamente la ricerca semantica (**Impostazioni → Ricerca & Embeddings → Ricerca semantica abilitata**).
- Verifica lo spazio disco libero (sotto 1 GB il database rallenta).
- Controlla i log: **Impostazioni → Sviluppo → Debug log** (toggle ON, effetto immediato, nessun riavvio richiesto).

### La Command Palette è lenta con molti prompt

Può presentarsi su vault molto grandi. PaP è testato fino a ~10 000 prompt: se ti avvicini a quella soglia, organizzare in cartelle (vedi [`cartelle.md`](./cartelle.md)) e ripulire i prompt obsoleti (Cestino → Svuota cestino) aiuta la palette a restare reattiva.

Per collezioni oltre i 50 000 prompt: apri un'issue e raccontaci il tuo caso — il design attuale non è ottimizzato per quel volume, e ci interessa capire chi ci arriva.

## Sync (server team)

### Sync non si connette al server

Prima un orientamento, perché è facile cercare le opzioni nel posto sbagliato: la sezione **Impostazioni → Sync** mostra solo lo **Stato**, il pulsante **"Sincronizza ora"** e il **"Logout"**. Il login e la configurazione del server si fanno invece dalla schermata **Impostazioni → Sincronizzazione** (è l'app stessa a indicartela quando la sync non è configurata).

Verifica nell'ordine:
1. **Stato**: controlla in **Impostazioni → Sync** che risulti connesso.
2. **Credenziali**: fai **Logout** da **Impostazioni → Sync**, poi ripeti il login da **Impostazioni → Sincronizzazione**.
3. **Server raggiungibile**: prova `curl https://server-url/health` (deve rispondere `ok`).
4. **CORS / certificati**: se usi un reverse proxy, verifica certificato TLS e header `Access-Control-Allow-Origin`.

### Conflitti di merge dopo sync

Capita quando due utenti modificano lo stesso prompt fra una sincronizzazione e l'altra. PaP risolve con la regola **last-write-wins** sui campi del prompt (nessun merge automatico in stile Git): vince la modifica più recente. Niente però va perduto: le versioni precedenti restano nella tab **Cronologia** del pannello di dettaglio, da cui puoi recuperare il body sovrascritto.

Per editing collaborativo intenso, considera di lavorare su varianti separate (A/B/C) o di coordinarvi su un canale esterno.

## Backup ed export

### Come faccio backup del vault?

Non è un problema da risolvere ma una buona abitudine da prendere — soprattutto se il vault è cifrato, perché il backup è la tua rete di sicurezza se dimentichi la password. Tre opzioni:

1. **Copia del file**: chiudi l'app → copia `pap-vault.db` in un'altra posizione. È un singolo file SQLite.
2. **Export JSON** (consigliato): **Impostazioni → Dati → Esporta vault** → `prompt-a-porter-export-YYYY-MM-DD.json`. Senza perdita di dati: quello che esporti si reimporta identico. Vedi [`formato-export-json.md`](./formato-export-json.md).
3. **Export Markdown**: **Impostazioni → Dati → Esporta come Markdown** → zip con un file `.md` per ogni prompt. Compatibile con Obsidian / Foam. Vedi [`markdown-import-export.md`](./markdown-import-export.md).

### Migrazione da altro tool (Obsidian, Notion, …)

Se i tuoi prompt vivono già da un'altra parte, non serve ricopiarli a mano:

- **Da Obsidian / Foam / qualsiasi cartella di `.md`**: usa l'import Markdown; il front-matter YAML viene interpretato come campi del prompt.
- **Da JSON arbitrario**: scrivi uno script che lo trasformi nel formato di [`formato-export-json.md`](./formato-export-json.md), poi usa l'import JSON.
- **Da Notion**: esporta come Markdown da Notion, poi usa l'import Markdown.

## Aggiornamenti

Per problemi con l'auto-update, vedi la sezione dedicata in [`auto-update.md`](./auto-update.md):
- "Errore di rete durante il controllo aggiornamenti"
- "Firma non valida"
- "Aggiornamento manuale"

## Debug e log

### Come abilito i log dettagliati?

Servono quando un problema non rientra in nessuna voce di questa pagina e vuoi capire (o farci capire) cosa succede dietro le quinte. **Impostazioni → Sviluppo → Debug log**: toggle ON, con **effetto immediato** (nessun riavvio richiesto). I log vengono scritti in `pap.log`:

- Windows: `%APPDATA%\com.pap.client\logs\pap.log`
- macOS: `~/Library/Logs/com.pap.client/pap.log`
- Linux: `~/.local/share/com.pap.client/logs/pap.log`

Il toggle ON aumenta il livello di dettaglio da `WARN` a `DEBUG`; nella stessa sezione Sviluppo trovi il viewer in-app **"Visualizza log"**. Disabilita quando non serve (i file possono crescere).

### Come segnalo un bug?

Una segnalazione ben fatta si risolve molto più in fretta. Il percorso ideale:

1. Abilita il Debug log (vedi sopra) e riproduci il problema.
2. Apri [GitHub Issues → New issue → Bug report](https://github.com/robertomarchioro/prompt-a-porter/issues/new?template=bug_report.yml).
3. Allega:
   - Versione PaP (Impostazioni → Informazioni)
   - Sistema operativo
   - Estratto del log al momento del bug (cerca il timestamp dell'evento)
   - Step per riprodurre

> **Privacy**: prima di allegare log, verifica che non contengano segreti o dati personali. I body dei prompt **non** sono loggati di default; lo sono i metadati (titoli, ID).
