export interface Segnaposto {
  nome: string;
  indice: number;
  /** Issue #159: true se sintassi `{{globale nome}}`, false se `{{nome}}`. */
  globale: boolean;
}

/**
 * Issue #159: il prefisso `globale ` (case-sensitive, con spazio
 * obbligatorio) marca il segnaposto come globale. Il nome stesso resta
 * `\w+` (no underscore-prefix né altri trattamenti speciali).
 *
 * Esempi:
 *   {{nome}}             → normale, nome="nome"
 *   {{ nome }}           → normale, nome="nome" (whitespace permesso)
 *   {{globale autore}}   → globale, nome="autore"
 *   {{globale  autore }} → globale, nome="autore" (whitespace permesso)
 *   {{globaleautore}}    → normale (no spazio dopo "globale")
 */
const RE_SEGNAPOSTO = /\{\{\s*(globale\s+)?(\w+)\s*\}\}/g;

export function estraiSegnaposti(body: string): Segnaposto[] {
  const risultati: Segnaposto[] = [];
  const visti = new Set<string>();
  let match;
  RE_SEGNAPOSTO.lastIndex = 0;
  while ((match = RE_SEGNAPOSTO.exec(body)) !== null) {
    const globale = match[1] !== undefined;
    const nome = match[2];
    // Chiave dedup: distingue globale-vs-normale con stesso nome
    // (in teoria possibile avere {{nome}} e {{globale nome}} nello
    // stesso prompt — sono 2 segnaposti distinti).
    const chiave = globale ? `globale:${nome}` : nome;
    if (!visti.has(chiave)) {
      risultati.push({ nome, indice: match.index, globale });
      visti.add(chiave);
    }
  }
  return risultati;
}

/**
 * Issue #159: 3° parametro opzionale `valoriGlobali` per resolver
 * separato. Default `{}` per back-compat (segnaposti globali non
 * vengono compilati senza valori globali).
 */
export function compila(
  body: string,
  valori: Record<string, string>,
  valoriGlobali: Record<string, string> = {},
): string {
  return body.replace(RE_SEGNAPOSTO, (_, glob, nome) => {
    if (glob) {
      return valoriGlobali[nome]?.trim() || `{{globale ${nome}}}`;
    }
    return valori[nome]?.trim() || `{{${nome}}}`;
  });
}

export function contaCompilati(
  segnaposti: Segnaposto[],
  valori: Record<string, string>,
  valoriGlobali: Record<string, string> = {},
): number {
  return segnaposti.filter((s) => {
    const map = s.globale ? valoriGlobali : valori;
    return map[s.nome]?.trim();
  }).length;
}
