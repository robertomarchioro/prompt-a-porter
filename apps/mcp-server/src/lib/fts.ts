/**
 * Sanitizzazione query FTS5 per SQLite.
 *
 * Spezza la query in token, rimuove caratteri non alfanumerici/underscore,
 * aggiunge `*` come suffisso wildcard per match prefix. Token vuoti sono
 * eliminati. Risultato concatenato con spazi (operatore AND implicito FTS5).
 *
 * Esempi:
 *   "hello world"  -> "hello* world*"
 *   "foo-bar baz"  -> "foobar* baz*"
 *   "  !!  "       -> ""
 */
export function sanitizzaFts(query: string): string {
  return query
    .split(/\s+/)
    .map((w) => {
      const pulito = w.replace(/[^\p{L}\p{N}_]/gu, "");
      return pulito ? `${pulito}*` : "";
    })
    .filter(Boolean)
    .join(" ");
}
