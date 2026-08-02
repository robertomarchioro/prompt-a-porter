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
 * Tauri.
 */
export async function eseguiPromuoviVariante(
  variantId: string,
  invokeFn: (cmd: string, args: { variantId: string }) => Promise<void>,
  dispatchListaMutata: () => void,
  dispatchApriPrompt: (id: string) => void,
  dispatchPromptPromosso: (id: string) => void,
): Promise<RisultatoPromuoviVariante> {
  try {
    await invokeFn("prompt_promuovi_variante", { variantId });
    dispatchListaMutata();
    dispatchApriPrompt(variantId);
    dispatchPromptPromosso(variantId);
    return { ok: true };
  } catch (err: unknown) {
    const msg =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err.replace(/^Error:\s*/i, "")
          : "Impossibile promuovere la variante.";
    return { ok: false, errore: msg };
  }
}
