/**
 * Logica pura di espansione `{{import "..."}}` per la Command Palette.
 *
 * Estratta da CommandPalette.svelte per permettere unit test senza Tauri
 * (MEDIUM-1 issue #299 code review).
 *
 * Il token corretto è `{{import "path"}}` (virgolette obbligatorie dopo
 * `import `). La guard usa `{{import "` per non colpire segnaposti come
 * `{{importanza}}` (MEDIUM-2).
 */

/** Risultato dell'espansione import. */
export interface RisultatoEspansione {
  bodyEspanso: string | null;
  erroreEspansione: string | null;
}

/** Token import minimo: `{{import "` (include la virgoletta aperta). */
const IMPORT_TOKEN = '{{import "';

/**
 * Determina se il body contiene almeno un token `{{import "...}}`.
 * Tightened (MEDIUM-2): `{{importanza}}` non viene più considerato import.
 */
export function contieneImport(rawBody: string): boolean {
  return rawBody.includes(IMPORT_TOKEN);
}

/**
 * Espande i token `{{import "..."}}` nel body tramite una funzione invoke
 * iniettabile (di solito `invoke<string>` di Tauri, ma mockabile nei test).
 *
 * Supporta la cancellation token pattern (HIGH-1): il chiamante passa un
 * `token` corrente e la funzione restituisce `null` se il token non coincide
 * più con `getToken()` dopo l'await, così le risposte fuori-ordine vengono
 * scartate silenziosamente.
 *
 * @param rawBody   Body grezzo del prompt.
 * @param promptId  Id del prompt corrente (per cycle detection backend).
 * @param invokeFn  Funzione async che chiama il backend (mockabile in test).
 * @param token     Numero sequenziale associato a questa invocazione.
 * @param getToken  Getter del contatore corrente — se `token !== getToken()`
 *                  dopo l'await, il risultato viene scartato (ritorna null).
 */
export async function espandiImportConToken(
  rawBody: string,
  promptId: string,
  invokeFn: (body: string, pid: string) => Promise<string>,
  token: number,
  getToken: () => number,
): Promise<RisultatoEspansione | null> {
  if (!contieneImport(rawBody)) {
    return { bodyEspanso: null, erroreEspansione: null };
  }
  try {
    const espanso = await invokeFn(rawBody, promptId);
    // HIGH-1: scarta risposta fuori-ordine
    if (token !== getToken()) return null;
    return { bodyEspanso: espanso, erroreEspansione: null };
  } catch (e) {
    // HIGH-1: scarta risposta fuori-ordine anche in caso di errore
    if (token !== getToken()) return null;
    return {
      bodyEspanso: null,
      erroreEspansione: String(e).replace(/^Error: /, ""),
    };
  }
}
