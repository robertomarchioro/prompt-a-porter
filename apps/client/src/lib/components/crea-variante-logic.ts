/**
 * Logica pura per la creazione di una variante (#559).
 *
 * Estratta da RightRail.svelte per consentire test senza runtime Svelte
 * (_pure/_impl pattern già usato per nuova-cartella-logic.ts). Al successo
 * deve emettere `pap:lista-mutata` — senza questo evento la nuova variante
 * non compare nell'elenco Libreria ordinato per "recenti" finché non si
 * ricarica manualmente.
 */

export interface RisultatoCreaVariante {
  ok: boolean;
  id?: string;
  errore?: string;
}

/**
 * Chiama il backend `prompt_crea_variante` e restituisce il risultato.
 * Al successo emette (via dispatchListaMutata) `pap:lista-mutata` così
 * Libreria/Sidebar si rinfrescano, poi (via dispatchApriPrompt) apre la
 * nuova variante — stesso pattern di promuoviVariante/eliminaVariante.
 *
 * I callback sono iniettati per consentire test puri senza DOM né runtime
 * Tauri.
 */
export async function eseguiCreaVariante(
  parentId: string,
  etichetta: string,
  invokeFn: (
    cmd: string,
    args: { parentId: string; etichetta: string | null },
  ) => Promise<string>,
  dispatchListaMutata: () => void,
  dispatchApriPrompt: (id: string) => void,
): Promise<RisultatoCreaVariante> {
  try {
    const trimmed = etichetta.trim();
    const id = await invokeFn("prompt_crea_variante", {
      parentId,
      etichetta: trimmed.length > 0 ? trimmed : null,
    });
    dispatchListaMutata();
    dispatchApriPrompt(id);
    return { ok: true, id };
  } catch (err: unknown) {
    const msg =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err.replace(/^Error:\s*/i, "")
          : "Impossibile creare la variante.";
    return { ok: false, errore: msg };
  }
}
