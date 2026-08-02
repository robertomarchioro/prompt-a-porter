/**
 * Logica pura per la promozione di una variante a principale (#572).
 *
 * Estratta da RightRail.svelte per consentire test senza runtime Svelte
 * (_pure/_impl pattern già usato per crea-variante-logic.ts) e per
 * condividere lo stesso comportamento fra i due percorsi di promozione:
 * il bottone "⇑ Promuovi a principale" (variante corrente aperta, id
 * INVARIATO) e la voce di menu contestuale sulle pillole (id spesso
 * diverso da quello aperto).
 *
 * Causa radice #572: quando `variantId` coincide con il prompt già
 * aperto, dispatchare `pap:apri-prompt` con lo stesso id è un no-op per
 * la reattività di Svelte 5 (riassegnare un primitivo invariato non fa
 * ripartire l'`$effect` di `DetailPane` — pattern nato da #170). Per
 * questo, oltre a `pap:lista-mutata`/`pap:apri-prompt`, emettiamo sempre
 * anche `pap:prompt-promosso` con l'id promosso: `DetailPane` lo confronta
 * col proprio `promptId` e forza il refetch anche a id invariato — stesso
 * pattern già in uso per `pap:prompt-ripristinato` (#425).
 *
 * Strumentazione diagnostica (#572, nessun cambio di comportamento): il
 * parametro opzionale `log`, se passato, riceve una riga per l'invoke
 * (avvio/completato) e una per ciascuno dei tre eventi dispatchati — per
 * distinguere «il backend non è mai stato chiamato» da «il backend ha
 * risposto ma l'evento non arriva a `DetailPane`».
 *
 * La riga d'errore va invece a `logErrore` (parametro opzionale separato,
 * di default uguale a `log` per retrocompatibilità coi chiamanti/test che
 * ne passano uno solo): un errore scritto allo stesso livello delle righe
 * informative sarebbe invisibile col filtro di default dell'app (`warn`,
 * vedi `log-app.ts`), che lascia passare `info` solo con "Debug log"
 * attivo. Review PR #592: prima di questo fix il 6° parametro era sempre
 * `logInfoApp` in `RightRail.svelte`, quindi la riga d'errore finiva a
 * livello info.
 */

export interface RisultatoPromuoviVariante {
  ok: boolean;
  errore?: string;
}

/**
 * Chiama il backend `prompt_promuovi_variante` e restituisce il risultato.
 * Al successo emette (via i callback iniettati, in quest'ordine)
 * `pap:lista-mutata`, `pap:apri-prompt` e `pap:prompt-promosso`.
 *
 * I callback sono iniettati per consentire test puri senza DOM né runtime
 * Tauri. `log` e `logErrore` sono entrambi opzionali (default: `log` non fa
 * nulla, `logErrore` ricade su `log`) per non rompere i chiamanti/test
 * esistenti che ne passano uno solo — ma i chiamanti nuovi dovrebbero
 * passare `logErrore` distinto (livello errore) così l'unica riga d'errore
 * emessa qui non finisce silenziata dal filtro di default del log
 * applicativo.
 */
export async function eseguiPromuoviVariante(
  variantId: string,
  invokeFn: (cmd: string, args: { variantId: string }) => Promise<void>,
  dispatchListaMutata: () => void,
  dispatchApriPrompt: (id: string) => void,
  dispatchPromptPromosso: (id: string) => void,
  log: (messaggio: string) => void = () => {},
  logErrore: (messaggio: string) => void = log,
): Promise<RisultatoPromuoviVariante> {
  try {
    log("[promuovi-variante] invoke prompt_promuovi_variante — avvio");
    await invokeFn("prompt_promuovi_variante", { variantId });
    log("[promuovi-variante] invoke prompt_promuovi_variante — completato");
    dispatchListaMutata();
    log("[promuovi-variante] dispatch pap:lista-mutata");
    dispatchApriPrompt(variantId);
    log("[promuovi-variante] dispatch pap:apri-prompt");
    dispatchPromptPromosso(variantId);
    log("[promuovi-variante] dispatch pap:prompt-promosso");
    return { ok: true };
  } catch (err: unknown) {
    // Il messaggio propagato nel log è sicuro qui solo perché `PapErrore` è
    // opaco per costruzione (nessun dettaglio interno) — invariante non
    // coperta da un test dedicato.
    const msg =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err.replace(/^Error:\s*/i, "")
          : "Impossibile promuovere la variante.";
    logErrore(`[promuovi-variante] invoke prompt_promuovi_variante — errore ${msg}`);
    return { ok: false, errore: msg };
  }
}
