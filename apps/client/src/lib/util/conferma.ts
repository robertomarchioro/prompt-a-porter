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
 */

import { sistemaOperativo } from "./os";
import { accodaConferma, accodaAvviso } from "$lib/stores/dialogo.svelte";

/**
 * Chiede conferma per un'azione (tipicamente distruttiva). Su macOS mostra
 * una Modale in-app (via `DialogoHost.svelte`); altrove è `window.confirm`
 * nativo, invariato rispetto a oggi.
 */
export async function conferma(messaggio: string): Promise<boolean> {
  if (sistemaOperativo === "macos") {
    return accodaConferma(messaggio);
  }
  return window.confirm(messaggio);
}

/**
 * Mostra un messaggio (tipicamente un errore) all'utente. Su macOS mostra
 * un Toast in-app (via `DialogoHost.svelte`); altrove è `window.alert`
 * nativo, invariato rispetto a oggi.
 */
export async function avvisa(messaggio: string): Promise<void> {
  if (sistemaOperativo === "macos") {
    return accodaAvviso(messaggio);
  }
  window.alert(messaggio);
}
