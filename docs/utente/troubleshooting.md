# Troubleshooting

> Risposte ai problemi più comuni durante l'uso di Prompt a Porter. Se non trovi quello che cerchi, apri un'issue su [GitHub](https://github.com/robertomarchioro/prompt-a-porter/issues/new/choose).

## Installazione e primo avvio

### Windows: "SmartScreen ha bloccato l'app"

Il binario è firmato con certificato Authenticode Certum EV, ma Microsoft costruisce reputazione sui singoli hash di binario nel tempo. Per le prime release la reputazione è zero e SmartScreen è cauto.

**Soluzione**: clicca "Altre informazioni" → "Esegui comunque". Una volta installato, l'app funziona normalmente. Vedi [`auto-update.md`](./auto-update.md) per i dettagli sulla firma.

### macOS: "L'app non può essere aperta perché lo sviluppatore non è verificato"

L'app è firmata Developer ID e **notarizzata da Apple**: questo avviso **non è lo stato atteso**. Se compare, probabilmente il download è corrotto o proviene da una fonte non ufficiale.

**Soluzione**: riscarica il `.dmg` dalla [pagina release ufficiale](https://github.com/robertomarchioro/prompt-a-porter/releases/latest) e reinstalla. Se il problema persiste, apri un'issue.

### Linux: AppImage non si avvia

L'AppImage richiede FUSE 2 (non FUSE 3). Su Ubuntu 22.04+ potresti dover installare `libfuse2`:

```bash
sudo apt install libfuse2
```

In alternativa, usa il pacchetto `.deb`.

### Linux (XFCE / Xubuntu): l'icona nel tray non appare

Su XFCE — quindi anche **Xubuntu** — l'area di notifica di default ("Area di notifica (legacy)") mostra solo icone vecchio stile *xembed* e **ignora** le icone moderne *StatusNotifier/AppIndicator* che usa Prompt a Porter. L'app pubblica correttamente l'icona, ma manca chi la mostra nel pannello.

Aggiungi al pannello il plugin **Status Notifier**:

```bash
sudo apt install xfce4-statusnotifier-plugin
```

Poi tasto destro sul pannello → **Pannello → Aggiungi nuovi elementi…** → aggiungi **"Status Notifier Plugin"**. Riavvia l'app: l'icona compare accanto all'orologio.

Su GNOME serve un'estensione tipo *AppIndicator/KStatusNotifier Support*; su KDE Plasma il tray funziona nativamente. (La libreria di sistema `libayatana-appindicator3-1` è comunque richiesta: il pacchetto `.deb` la installa come dipendenza.)

### "Vault non trovato" / dialog vault vuoto

L'app cerca il vault nei path di default (vedi [`getting-started.md`](./getting-started.md) — sezione Onboarding). Se il file non esiste, l'onboarding si avvia e ti guida nella creazione.

L'app desktop **non legge variabili d'ambiente** per il path del vault. La variabile `PAP_VAULT_PATH` vale solo per la CLI `pap` e per il server MCP:

```bash
PAP_VAULT_PATH=/percorso/del/vault.db pap list
```

## Password e vault

### Ho dimenticato la password del vault

**Non c'è recupero possibile.** Il vault è cifrato con SQLCipher AES-256: senza password il contenuto è inaccessibile, anche a noi.

**Cosa puoi fare:**
1. Se hai un export JSON recente (Impostazioni → Dati → Esporta), crea un nuovo vault con una nuova password e reimporta.
2. Se hai un backup del file `.db` con la vecchia password, ripristinalo.
3. Altrimenti: ricrea il vault da zero.

> **Per evitarlo in futuro**: salva la password in un password manager (1Password, Bitwarden, KeePass). Considera anche un export JSON periodico in cloud personale.

### La password ha caratteri speciali e non funziona

Verifica il layout di tastiera. Su Windows/Linux, alcune password copiate da macOS contengono caratteri Unicode ambigui (es. `⌘`). Reimposta la password da macOS o ricreala usando solo ASCII.

### Cambio password: "Errore durante il rekey"

Il vault è in uso da un altro processo. Chiudi:
- Tutte le istanze dell'app desktop
- La CLI `pap` (se attiva)
- Il server MCP (se collegato)

Poi ripeti il cambio password da **Impostazioni → Sicurezza**.

## Hotkey globale

### La hotkey non funziona

Verifica nell'ordine:

1. **Conflitto con altra app**: prova a chiudere altri programmi che potrebbero usare la stessa combinazione (clipboard manager, Spotlight custom, Raycast, AutoHotkey su Windows). Riavvia PaP dopo aver chiuso il conflitto.
2. **Permessi macOS**: la prima volta che PaP registra una global shortcut, macOS chiede l'autorizzazione Accessibilità. Vai in **System Settings → Privacy & Security → Accessibility** e abilita PaP.
3. **Linux Wayland**: alcuni desktop environment (GNOME Wayland) hanno restrizioni sulle global shortcut. Considera X11 o usa la palette dall'interno dell'app (`Ctrl+K`).
4. **Reimposta default**: **Impostazioni → Sistema → Hotkey**, reinserisci `Ctrl+Shift+P` nel campo e salva. La modifica ha effetto al prossimo riavvio dell'app.

### La hotkey funziona ma apre il programma sbagliato

Un'altra app ha registrato la stessa combinazione **dopo** PaP. Su Windows succede tipicamente con programmi installati di recente. Cambia la hotkey PaP in qualcosa di più specifico (es. `Ctrl+Alt+Shift+P`).

## Editor e compilazione

### Le modifiche all'editor non vengono salvate

PaP usa **autosave** con debounce di ~2 secondi: ferma di scrivere per 2-3 secondi e il salvataggio scatta. Verifica:

- L'indicatore "Salvato" / spinner accanto al titolo del prompt.
- La cronologia (right rail → Cronologia) mostra nuove voci?
- Se nessuna delle due risponde, può essere un bug: chiudi e riapri il prompt; se persiste, apri issue con screenshot.

### "Vault locked" durante l'edit

Il vault non si blocca da solo: si blocca solo se premi il pulsante **"Blocca vault"** in **Impostazioni → Sicurezza**. Reinserisci la password per sbloccare. Le modifiche non salvate (dentro la finestra di autosave) potrebbero essere perse — la lock scatta dopo l'autosave, ma può succedere edge case.

### `{{segnaposto}}` non viene sostituito durante la compilazione

Cause possibili:
- **Nome non valido**: spazi o trattini nel nome (`{{nome con spazi}}`, `{{nome-con-trattini}}`). Il linter `PH003` avvisa. Usa underscore: `{{nome_con_spazi}}`.
- **Singola graffa**: `{nome}` non è un segnaposto. Devono essere doppie: `{{nome}}`. Il linter `PH001` lo segnala come errore.
- **Form lasciata vuota**: se non scrivi un valore, il segnaposto resta intatto by design (così puoi compilarlo manualmente dopo).

### Import `{{import "..."}}` non risolve

Cause comuni:
- **Path errato**: verifica il titolo esatto del prompt importato (case-insensitive, ma niente typos). Il linter `IMP001` segnala "Import non risolto".
- **Ciclo**: A importa B che importa A. Il linter `IMP002` blocca la compilazione. Rivedi la catena.
- **Profondità eccessiva**: oltre 5 livelli di nidificazione. Refactor: piatti gli import o accorpa in un unico prompt.
- **Path con cartella vs root**: `"marketing/email/cold"` cerca esattamente in `Folders.Path = '/marketing/email'`. Se il prompt è a root, usa solo `"cold"`.

Vedi [`prompt-componibili.md`](./prompt-componibili.md) per la sintassi completa.

## Performance

### L'app è lenta all'apertura

Il primo avvio carica il modello di embedding (se la ricerca semantica è abilitata, ~150 MB). Successivi avvi sono cached.

Se persiste lentezza:
- Disabilita temporaneamente la ricerca semantica (**Impostazioni → Ricerca & Embeddings → Ricerca semantica abilitata**).
- Verifica spazio disco libero (< 1 GB libero rallenta SQLite).
- Controlla i log: **Impostazioni → Sviluppo → Debug log** (toggle ON, effetto immediato, nessun riavvio richiesto).

### La Command Palette è lenta con molti prompt

PaP è testato fino a ~10 000 prompt. Oltre, considera di organizzare in cartelle (vedi [`cartelle.md`](./cartelle.md)) o di eliminare prompt obsoleti (Cestino → Svuota cestino).

Per dataset > 50 000 prompt: apri un'issue, il design corrente non è ottimizzato per quel volume.

## Sync (server team)

### Sync non si connette al server

La sezione **Impostazioni → Sync** mostra solo lo **Stato**, il pulsante **"Sincronizza ora"** e il **"Logout"**; il login e la configurazione del server avvengono nella superficie legacy (il redesign completo della configurazione Sync è previsto per la v0.9).

Verifica nell'ordine:
1. **Stato**: controlla in **Impostazioni → Sync** che risulti connesso.
2. **Credenziali**: fai **Logout** da **Impostazioni → Sync**, poi ripeti il login dalla superficie legacy.
3. **Server raggiungibile**: prova `curl https://server-url/health` (deve rispondere `ok`).
4. **CORS / certificati**: se usi reverse proxy, verifica certificato TLS e header `Access-Control-Allow-Origin`.

### Conflitti di merge dopo sync

PaP usa **last-write-wins** sui campi del prompt (no merge automatico Git-style). Se due utenti editano lo stesso prompt:
- Vince l'ultimo `UpdatedAt`.
- Le versioni precedenti restano in **Cronologia** del prompt (right rail) — puoi recuperare il body precedente.

Per editing collaborativo intenso, considera di lavorare in branch separati (varianti A/B/C) o di coordinarsi via canale esterno.

## Backup ed export

### Come faccio backup del vault?

Tre opzioni:

1. **Copia del file**: chiudi l'app → copia `pap-vault.db` in altra posizione. È un singolo file SQLite.
2. **Export JSON** (consigliato): **Impostazioni → Dati → Esporta vault** → `prompt-a-porter-export-YYYY-MM-DD.json`. Round-trip lossless. Vedi [`formato-export-json.md`](./formato-export-json.md).
3. **Export Markdown** (M6): **Impostazioni → Dati → Esporta come Markdown** → zip con un file `.md` per ogni prompt. Compatibile con Obsidian / Foam. Vedi [`markdown-import-export.md`](./markdown-import-export.md).

### Migrazione da altro tool (Obsidian, Notion, …)

- **Da Obsidian / Foam / qualsiasi cartella di `.md`**: usa l'import Markdown (M6). Front-matter YAML interpretato come campi del prompt.
- **Da JSON arbitrario**: scrivi uno script che lo trasformi nel formato di [`formato-export-json.md`](./formato-export-json.md), poi usa import JSON.
- **Da Notion**: esporta come Markdown da Notion, poi import M6.

## Aggiornamenti

Per problemi con l'auto-update, vedi la sezione dedicata in [`auto-update.md`](./auto-update.md):
- "Errore di rete durante il controllo aggiornamenti"
- "Firma non valida"
- "Aggiornamento manuale"

## Debug e log

### Come abilito i log dettagliati?

**Impostazioni → Sviluppo → Debug log**: toggle ON, con **effetto immediato** (nessun riavvio richiesto). I log vengono scritti in `pap.log`:

- Windows: `%APPDATA%\com.pap.client\logs\pap.log`
- macOS: `~/Library/Logs/com.pap.client/pap.log`
- Linux: `~/.local/share/com.pap.client/logs/pap.log`

Il toggle ON aumenta il livello da `WARN` a `DEBUG`; nella stessa sezione Sviluppo trovi il viewer in-app **"Visualizza log"**. Disabilita quando non serve (i file possono crescere).

### Come segnalo un bug?

1. Abilita Debug log (vedi sopra) e riproduci il problema.
2. Apri [GitHub Issues → New issue → Bug report](https://github.com/robertomarchioro/prompt-a-porter/issues/new?template=bug_report.yml).
3. Allega:
   - Versione PaP (Impostazioni → Informazioni)
   - Sistema operativo
   - Estratto del log al momento del bug (cerca timestamp dell'evento)
   - Step per riprodurre

> **Privacy**: prima di allegare log, verifica che non contengano segreti o dati personali. I body dei prompt **non** sono loggati di default; lo sono i metadati (titoli, ID).
