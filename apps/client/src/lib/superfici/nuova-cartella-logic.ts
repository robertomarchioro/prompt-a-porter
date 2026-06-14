/**
 * Logica pura per la creazione di una nuova cartella (issue #301).
 *
 * Estratta dalla modale per consentire test senza runtime Svelte (_pure/_impl
 * pattern già usato in cartelle.rs). Importata sia da NuovaCartellaModal.svelte
 * sia da nuova-cartella.test.ts per evitare drift.
 */

export interface RisultatoSubmit {
  ok: boolean;
  id?: string;
  errore?: string;
}

/**
 * Restituisce true se il nome è accettabile: non vuoto dopo trim, senza "/".
 * Specchia le stesse regole di `nome_valido()` in cartelle.rs.
 */
export function nomeValido(nome: string): boolean {
  const t = nome.trim();
  return t.length > 0 && !t.includes("/");
}

/**
 * Chiama il backend `folder_crea` e restituisce il risultato.
 * Al successo chiama dispatchFn (emette `pap:lista-mutata`) e onChiudi.
 *
 * I due callback sono iniettati per consentire test puri senza DOM né
 * runtime Tauri.
 */
export async function eseguiCreaCartella(
  nome: string,
  invokeFn: (
    cmd: string,
    args: { dati: { nome: string; parent_folder_id: null } },
  ) => Promise<string>,
  dispatchFn: () => void,
  onChiudi: () => void,
): Promise<RisultatoSubmit> {
  if (!nomeValido(nome)) {
    return { ok: false, errore: "Il nome non può essere vuoto o contenere '/'." };
  }

  try {
    const id = await invokeFn("folder_crea", {
      dati: { nome: nome.trim(), parent_folder_id: null },
    });
    dispatchFn();
    onChiudi();
    return { ok: true, id };
  } catch (err: unknown) {
    const msg =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err.replace(/^Error:\s*/i, "")
          : "Impossibile creare la cartella.";
    return { ok: false, errore: msg };
  }
}
