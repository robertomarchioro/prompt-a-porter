# Aggiornamenti automatici

> Come Prompt a Porter gestisce gli aggiornamenti del software e che controllo hai tu sul processo.

Molte app desktop si aggiornano da sole, quando vogliono loro. Prompt a Porter fa una scelta diversa: **il controllo aggiornamenti parte solo quando lo chiedi tu**. All'avvio l'app non contatta internet, non scarica nulla in background, non applica update silenziosi al riavvio successivo. Se preferisci restare su una versione per mesi, nessuno te lo impedisce.

Quando decidi di aggiornare, l'app fa le cose per bene: i file arrivano direttamente da **GitHub Releases** (mai da server di terze parti) e ogni aggiornamento è **firmato digitalmente**. L'app verifica la firma prima di applicare qualsiasi cosa; se la firma non corrisponde, l'update viene rifiutato, senza eccezioni.

E come per il resto del prodotto: niente telemetria, niente identificativi utente, niente tracking degli aggiornamenti.

## Come funziona

Quando vuoi sapere se c'è una versione nuova, vai in **Impostazioni → Sistema → Aggiornamenti** e clicca **"Verifica aggiornamenti"**.

L'app contatta GitHub Releases, scarica un piccolo file di metadati (`latest.json`), verifica che sia firmato con la chiave del progetto, e ti dice:

- **Sei aggiornato.** Niente di nuovo, nessun bottone, nessun pop-up.
- **C'è una versione più recente.** L'app ti mostra la versione disponibile, la data di rilascio e le note di rilascio. Decidi tu se installare o no.

Se confermi, l'app scarica il nuovo binario, verifica anche quella firma, e applica l'update. A processo finito, l'app si chiude e si riavvia con la nuova versione.

## Cosa NON fa l'app

- **Non controlla aggiornamenti all'avvio.** L'app parte, ti apre la libreria, non parla con internet senza un tuo gesto esplicito.
- **Non scarica nulla in background.** Niente download pre-emptivi, niente "applica al prossimo riavvio" silenzioso.
- **Non invia analytics sul controllo update.** L'unica chiamata web è il download del file `latest.json` da GitHub Releases.
- **Non installa downgrade.** Se per qualche motivo `latest.json` indicasse una versione **inferiore** alla tua, l'app la ignora.

## Come disabilitare il check

In **Impostazioni → Sistema → Aggiornamenti** c'è il toggle "Abilita verifica aggiornamenti". Se lo disattivi:

- Il bottone "Verifica aggiornamenti" sparisce.
- L'app non contatta mai GitHub Releases per i metadati di update.
- Resti sulla versione installata fino a quando non aggiorni manualmente (vedi sotto).

## Aggiornamento manuale

In qualsiasi momento puoi scaricare la versione più recente direttamente da:

[https://github.com/robertomarchioro/prompt-a-porter/releases/latest](https://github.com/robertomarchioro/prompt-a-porter/releases/latest)

Asset disponibili:

- **Windows — NSIS installer** (`Prompt.a.Porter_X.Y.Z_x64-setup.exe`) — installazione standard per-user senza UAC. Sostituisce la versione precedente preservando i dati.
- **Windows — Portable .zip** (`Prompt-a-Porter-portable-windows-x64-vX.Y.Z.zip`) — estrazione e uso senza installer. Manuale.
- **macOS — DMG universale** (`Prompt.a.Porter_X.Y.Z_universal.dmg`) — Apple Silicon + Intel, firmato e notarizzato.
- **Linux — AppImage** (`Prompt.a.Porter_X.Y.Z_amd64.AppImage`) e **`.deb`** (`Prompt.a.Porter_X.Y.Z_amd64.deb`).

Per gli installer, fai doppio click e segui la procedura. I tuoi dati (vault, preferenze, log) restano intatti durante l'aggiornamento.

## Privacy

Sull'aggiornamento vale la regola del prodotto: **i dati restano sul tuo computer**.

- L'app non invia user-agent identificabili oltre lo standard del runtime HTTPS.
- Non c'è fingerprinting del sistema.
- GitHub registra normalmente gli accessi al repo (è un loro log standard, vale per qualunque visitatore della pagina release).
- Se questo livello di privacy non ti basta, disabilita il check (vedi sopra) e fai update manuali quando vuoi.

## Sicurezza

Ogni file scaricato dall'updater è verificato in due passaggi:

1. **Firma Ed25519 del manifesto** `latest.json` confrontata con la chiave pubblica del progetto, incorporata nel binario installato.
2. **Firma Ed25519 del binario stesso** indicato nel manifesto.

Se una delle due verifiche fallisce, l'update viene rifiutato con errore. Niente "applica comunque" — niente bypass.

Inoltre il binario Windows è firmato **Authenticode EV** dalla CA Certum: Windows lo riconosce come publisher fidato e né il download né l'installazione mostrano avvisi SmartScreen.

## Troubleshooting

### "Errore di rete durante il controllo aggiornamenti"

Probabili cause:

- Connessione internet assente o intermittente.
- Firewall aziendale che blocca `github.com` o `objects.githubusercontent.com`.
- DNS pubblico in fallimento.

**Cosa fare**: verifica la connessione, riprova. Se persiste, fai update manuale da [github.com/robertomarchioro/prompt-a-porter/releases/latest](https://github.com/robertomarchioro/prompt-a-porter/releases/latest).

### "Firma non valida"

L'app ha scaricato `latest.json` o il binario ma la firma non corrisponde alla chiave attesa.

**Cosa fare**: **non bypassare** mai questo errore. Significa che qualcosa è andato storto fra GitHub e te (corruzione del download, intercettazione, ecc.). Fai update manuale dalla pagina release ufficiale.

### "Update interrotto a metà"

Il download è iniziato ma si è interrotto.

**Cosa fare**: rilancia il check. L'updater non lascia file parziali installati. Se persiste, update manuale.

### "Voglio tornare alla versione precedente"

L'updater non supporta il downgrade automatico (per sicurezza). Per tornare a una versione precedente:

1. Disinstalla la versione corrente (Windows: Impostazioni → App; macOS: elimina l'app da `Applications`; Linux: rimuovi l'AppImage o `sudo apt remove` per il `.deb`).
2. Scarica la versione che vuoi da [github.com/robertomarchioro/prompt-a-porter/releases](https://github.com/robertomarchioro/prompt-a-porter/releases) (tutte le release sono permanenti).
3. Installa.

I tuoi dati restano: il vault è nella cartella dati dell'app (vedi [`getting-started.md`](./getting-started.md)), indipendente dalla versione dell'eseguibile.

### "SmartScreen mi dice che l'eseguibile non è riconosciuto"

Con il certificato **EV** attuale questo avviso non è lo stato atteso: la reputazione SmartScreen è legata al certificato, non ai download della singola versione, quindi vale anche per le release appena pubblicate.

**Cosa fare**: riscarica il file da [github.com/robertomarchioro/prompt-a-porter/releases/latest](https://github.com/robertomarchioro/prompt-a-porter/releases/latest) e riprova. Se l'avviso persiste, clicca "Maggiori informazioni" e verifica che il publisher sia "Open Source Developer, Roberto Marchioro" prima di scegliere "Esegui comunque"; poi segnalacelo con un'issue.

## FAQ

**D: L'app si aggiorna da sola di notte?**
R: No. Niente check automatici, niente download in background. Tutto su tuo comando.

**D: Posso usare PaP offline per sempre?**
R: Sì. Disabilita il check aggiornamenti e non lanciarlo mai manualmente. L'app funziona senza internet (eccetto i provider AI cloud che configuri tu, opzionali).

**D: Cosa succede se cambia la chiave di firma del progetto?**
R: L'updater rifiuta gli aggiornamenti firmati con chiave diversa da quella che hai installato. Dovrai fare un update manuale "ponte" per saltare alla nuova chiave. Sarà comunicato chiaramente nella release.

**D: Posso bloccare gli aggiornamenti per un mese?**
R: Sì. Disabilita il check dalle impostazioni. Riattiva quando vuoi controllare.

**D: Gli aggiornamenti rompono i miei dati?**
R: No: il vault è in una cartella separata (`%APPDATA%\com.pap.client\`). Gli update aggiornano solo l'eseguibile. I dati restano. Eventuali migrazioni di schema database sono retro-compatibili e documentate nel CHANGELOG.

**D: Posso aggiornare PaP installato in modalità portable?**
R: Per la versione portable serve sostituzione manuale: scarica la nuova `.zip`, sovrascrivi i file della cartella. Il check aggiornamenti dell'updater è disponibile anche in portable, ma il "Applica" richiede privilegi di scrittura nella cartella dove hai estratto il portable.

## Vedi anche

- [`getting-started.md`](./getting-started.md) — dove vivono i tuoi dati, che restano intatti a ogni aggiornamento.
- [`troubleshooting.md`](./troubleshooting.md) — soluzioni agli altri problemi comuni dell'app.
