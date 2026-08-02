/**
 * Sostituti di `window.confirm`/`window.alert` specifici per macOS.
 *
 * Causa verificata nel sorgente: wry 0.55.1 implementa `WKUIDelegate` in
 * `src/wkwebview/class/wry_web_view_ui_delegate.rs` con due soli metodi
 * (`runOpenPanelWithParameters` e `requestMediaCapturePermissionForOrigin`).
 * Non implementa `runJavaScriptConfirmPanelWithMessage` né i pannelli
 * alert/prompt. Comportamento osservato dal vivo su macOS: nessun dialogo
 * appare e l'azione viene eseguita comunque (`window.confirm` risolve un
 * valore veritiero) — l'opposto di un blocco, è una conferma saltata e
 * data per accettata. Su Windows (WebView2) i dialoghi nativi funzionano
 * regolarmente, quindi fuori da macOS questi helper delegano invariati a
 * `window.confirm`/`window.alert`.
 *
 * Rilevamento piattaforma: riusa `sistemaOperativo` di `./os.ts` (già
 * testato in os.test.ts), niente nuovo plugin/permesso.
 *
 * Strumentazione diagnostica (nessun cambio di comportamento): #585
 * (cancellazione prompt non fa nulla su Windows), #572 (promozione
 * variante non fa nulla), #584 («Pulisci log» non pulisce, Windows). Ogni
 * chiamata a `conferma()` logga durata, ramo, esito grezzo e piattaforma —
 * vedi `conferma-log.ts` per il perché della soglia sulla durata e il
 * vincolo di privacy (mai il testo del messaggio, solo la lunghezza).
 */

import { sistemaOperativo, classificaPiattaforma } from "./os";
import { accodaConferma, accodaAvviso } from "$lib/stores/dialogo.svelte";
import { logInfoApp } from "./log-app";
import {
  formattaRigaConferma,
  formattaRigaAvviso,
  type RamoConferma,
} from "./conferma-log";

function ramoCorrente(): RamoConferma {
  return sistemaOperativo === "macos" ? "macos" : "nativo";
}

function platformCorrente(): string {
  return typeof navigator !== "undefined" ? navigator.platform : "";
}

/**
 * Chiede conferma per un'azione (tipicamente distruttiva). Su macOS mostra
 * una Modale in-app (via `DialogoHost.svelte`); altrove è `window.confirm`
 * nativo, invariato rispetto a oggi.
 *
 * `azione` è un identificativo diagnostico opzionale (es.
 * "elimina-prompt"), MAI testo utente — usato solo nella riga di log, non
 * cambia il comportamento della funzione. Parametro opzionale: nessun
 * chiamante esistente si rompe.
 */
export async function conferma(
  messaggio: string,
  azione?: string,
): Promise<boolean> {
  const ramo = ramoCorrente();
  const platform = platformCorrente();
  const inizio = performance.now();
  const esito =
    ramo === "macos" ? await accodaConferma(messaggio) : window.confirm(messaggio);
  const durataMs = performance.now() - inizio;
  logInfoApp(
    formattaRigaConferma({
      azione: azione ?? "(non specificata)",
      ramo,
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
 * Mostra un messaggio (tipicamente un errore) all'utente. Su macOS mostra
 * un Toast in-app (via `DialogoHost.svelte`); altrove è `window.alert`
 * nativo, invariato rispetto a oggi.
 *
 * `azione`: vedi `conferma()` sopra, stesso vincolo di privacy.
 */
export async function avvisa(messaggio: string, azione?: string): Promise<void> {
  const ramo = ramoCorrente();
  logInfoApp(
    formattaRigaAvviso({
      azione: azione ?? "(non specificata)",
      ramo,
      lunghezzaMessaggio: messaggio.length,
    }),
  );
  if (ramo === "macos") {
    return accodaAvviso(messaggio);
  }
  window.alert(messaggio);
}
