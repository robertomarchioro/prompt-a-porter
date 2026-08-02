/**
 * Logica pura del viewer log live (issue #558).
 *
 * Estratta da LogViewer.svelte per consentire test senza runtime Svelte
 * (stesso pattern `_logic.ts` di debug-log-export-logic.ts /
 * crea-variante-logic.ts): filtro per livello, filtro per regex e
 * ordinamento sono funzioni pure, testabili senza mockare `invoke`.
 */

export interface RigaLog {
  timestamp: string;
  level: string;
  target: string;
  message: string;
  raw: string;
}

export type LivelloFiltro = "" | "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";

/**
 * Righe da chiedere al backend di default. Il backend (`debug_log.rs`)
 * clampa comunque a [1, N_RIGHE_MAX]. Prima di questo fix il frontend
 * chiedeva sempre e solo 200 righe hardcoded: con sessioni lunghe le righe
 * più vecchie (spesso quelle utili per diagnosticare un problema già
 * passato) uscivano dalla finestra prima ancora di poterle leggere.
 */
export const N_RIGHE_DEFAULT = 1000;

/** Limite massimo, allineato a `MAX_RIGHE` in `debug_log.rs`. */
export const N_RIGHE_MAX = 5000;

/** Opzioni selezionabili dall'utente per il numero di righe caricate. */
export const OPZIONI_N_RIGHE = [200, 500, N_RIGHE_DEFAULT, 2000, N_RIGHE_MAX] as const;

/**
 * Compila l'input regex dell'utente. Ritorna `null` (nessun filtro attivo)
 * per stringa vuota o pattern non valido — l'errore va segnalato a parte
 * (vedi `erroreRegex`) senza far fallire il filtro.
 */
export function compilaRegex(input: string): RegExp | null {
  const s = input.trim();
  if (!s) return null;
  try {
    return new RegExp(s, "i");
  } catch {
    return null;
  }
}

/** Messaggio d'errore di compilazione regex, "" se valida o vuota. */
export function erroreRegex(input: string): string {
  const s = input.trim();
  if (!s) return "";
  try {
    new RegExp(s, "i");
    return "";
  } catch (e) {
    return String(e);
  }
}

/**
 * Filtra le righe per livello e/o regex. Non muta l'array in input: se
 * nessun filtro è attivo ritorna lo stesso riferimento (nessuna copia
 * inutile), altrimenti costruisce un nuovo array via `.filter()`.
 *
 * `livello === ""` ("Tutti") non applica alcun filtro di livello: è il
 * comportamento corretto, mai stato un bug qui — vedi nota in
 * LogViewer.svelte sull'indagine del sintomo #558 punto 1.
 */
export function filtraRighe(
  righe: readonly RigaLog[],
  livello: LivelloFiltro,
  regex: RegExp | null,
): RigaLog[] {
  let out: readonly RigaLog[] = righe;
  if (livello) {
    out = out.filter((r) => r.level === livello);
  }
  if (regex) {
    out = out.filter(
      (r) => regex.test(r.message) || regex.test(r.target) || regex.test(r.raw),
    );
  }
  return out as RigaLog[];
}

/**
 * Ordina le righe dal più recente al più vecchio, senza mutare l'array
 * sorgente (il backend le restituisce in ordine cronologico crescente).
 * `.slice().reverse()` invece di `.reverse()` in place — il progetto
 * impone pattern immutabili sullo state Svelte.
 */
export function ordinaRecentiPrimi(righe: readonly RigaLog[]): RigaLog[] {
  return righe.slice().reverse();
}
