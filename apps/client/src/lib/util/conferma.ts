/**
 * Sostituti di `window.confirm`/`window.alert` — **su tutte le piattaforme**.
 *
 * CAUSA VERIFICATA NEL SORGENTE (non ipotizzata). `tauri-plugin-dialog`,
 * aggiunto in #570 per il dialogo «salva con nome» dell'export ZIP, inietta
 * uno script di init (`src/init-iife.js` nella crate 2.7.2) che **sostituisce
 * le funzioni globali della webview**:
 *
 *     window.alert   = (m) => invoke("plugin:dialog|message", ...)
 *     window.confirm = (m) => invoke("plugin:dialog|confirm", ...)
 *
 * Due conseguenze, entrambe osservate dal vivo:
 *
 * 1. `plugin:dialog|confirm` **non è un comando registrato**: l'handler della
 *    crate (`src/lib.rs`, `generate_handler!`) espone solo `open`, `save`,
 *    `message`. Ogni `window.confirm` fallisce quindi con
 *    «Command plugin:dialog|confirm not allowed by ACL». Concedere un
 *    permesso NON risolve: `permissions/confirm.toml` dichiara
 *    `allow-confirm` deprecato e aliasato ad `allow-message`, cioè a un
 *    comando diverso. È un difetto della libreria, non una configurazione
 *    mancante.
 * 2. `window.alert` finisce su `plugin:dialog|message`, che esiste ma
 *    richiede un permesso `dialog:` che questa app non concede.
 *
 * Questo spiega, con una causa sola, i difetti #585 (cancellazione prompt
 * senza effetto), #572 (promozione variante senza effetto), #584 («Pulisci
 * log» senza effetto) e il blocco dell'aggiornamento automatico su
 * «Installa e riavvia» — tutti su Windows e Linux, mentre su macOS
 * funzionavano perché la #576 aveva già instradato macOS sulla modale
 * interna, aggirando `window.confirm` per un'altra ragione.
 *
 * Rimedio: **non dipendere più da `window.confirm`/`window.alert`**. La coda
 * in-app (`$lib/stores/dialogo.svelte.ts` → `DialogoHost.svelte`) è ora
 * l'unica strada, su ogni sistema operativo. Nessun permesso `dialog:`
 * necessario, nessuna dipendenza dal comportamento della webview.
 *
 * `DialogoHost` è montato in App.svelte per **tutte** le finestre: se non lo
 * fosse in una, una `conferma()` chiamata da lì resterebbe appesa per sempre
 * invece di fallire — un blocco silenzioso, peggio dell'errore.
 *
 * Strumentazione diagnostica: ogni chiamata logga durata, esito grezzo e
 * piattaforma — vedi `conferma-log.ts` per il vincolo di privacy (mai il
 * testo del messaggio, solo la lunghezza).
 */

import { classificaPiattaforma } from "./os";
import { accodaConferma, accodaAvviso } from "$lib/stores/dialogo.svelte";
import { logInfoApp } from "./log-app";
import {
  formattaRigaConferma,
  formattaRigaAvviso,
  type RamoConferma,
} from "./conferma-log";

/**
 * Unico ramo possibile: la coda in-app. Resta esplicito nella riga di log
 * perché un log che non dica quale meccanismo è stato usato renderebbe
 * ambigue le righe raccolte prima di questa correzione.
 */
const RAMO: RamoConferma = "in-app";

function platformCorrente(): string {
  return typeof navigator !== "undefined" ? navigator.platform : "";
}

/**
 * Chiede conferma per un'azione (tipicamente distruttiva). Mostra sempre una
 * Modale in-app via `DialogoHost.svelte`, su ogni piattaforma.
 *
 * `azione` è un identificativo diagnostico opzionale (es. "elimina-prompt"),
 * MAI testo utente — usato solo nella riga di log, non cambia il
 * comportamento. Parametro opzionale: nessun chiamante esistente si rompe.
 */
export async function conferma(
  messaggio: string,
  azione?: string,
): Promise<boolean> {
  const platform = platformCorrente();
  const inizio = performance.now();
  const esito = await accodaConferma(messaggio);
  const durataMs = performance.now() - inizio;
  logInfoApp(
    formattaRigaConferma({
      azione: azione ?? "(non specificata)",
      ramo: RAMO,
      durataMs,
      lunghezzaMessaggio: messaggio.length,
      tipoEsito: typeof esito,
      valoreEsito: esito,
      platform,
      piattaformaClassificata: classificaPiattaforma(platform),
    }),
  );
  return esito;
}

/**
 * Mostra un messaggio (tipicamente un errore) all'utente, sempre come Toast
 * in-app via `DialogoHost.svelte`, su ogni piattaforma.
 *
 * `azione`: vedi `conferma()` sopra, stesso vincolo di privacy.
 */
export async function avvisa(messaggio: string, azione?: string): Promise<void> {
  logInfoApp(
    formattaRigaAvviso({
      azione: azione ?? "(non specificata)",
      ramo: RAMO,
      lunghezzaMessaggio: messaggio.length,
    }),
  );
  return accodaAvviso(messaggio);
}
