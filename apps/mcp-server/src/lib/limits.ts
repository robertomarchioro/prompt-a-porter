/**
 * Limiti condivisi lato server MCP: clamp di `limit` e guardia aggregata
 * sulla dimensione degli argomenti di una CallTool request.
 */

/** Limite minimo/massimo di risultati per pap_search / pap_list_recent. */
export const LIMIT_MIN = 1;
export const LIMIT_MAX = 50;

/**
 * Blocca (clamp) `limit` nell'intervallo [LIMIT_MIN, LIMIT_MAX].
 *
 * Il tipo (numero intero) è già garantito dallo schema Zod a monte;
 * qui preserviamo il comportamento storico di "clamp silenzioso" invece
 * di un hard-fail su valori fuori range (es. 0, negativi, >50), per non
 * rompere client che mandano limit non normalizzati.
 */
export function clampLimit(limit: number): number {
  return Math.min(Math.max(limit, LIMIT_MIN), LIMIT_MAX);
}

/**
 * Dimensione massima (in caratteri, su `JSON.stringify`) accettata per
 * l'intero oggetto `arguments` di una CallTool request, come guardia
 * aggregata economica PRIMA della validazione Zod per-tool. Protegge da
 * payload con moltissime chiavi/elementi che altrimenti verrebbero
 * comunque interamente attraversati dal parser Zod prima di essere
 * rifiutati (es. milioni di chiavi in `vars` o elementi in `tags`).
 */
export const MAX_ARGS_JSON_LENGTH = 100_000;

/** True se la rappresentazione JSON di `args` supera MAX_ARGS_JSON_LENGTH. */
export function argsTroppoGrandi(args: unknown): boolean {
  return JSON.stringify(args ?? {}).length > MAX_ARGS_JSON_LENGTH;
}
