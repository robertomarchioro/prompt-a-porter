/**
 * Estrae i path degli `{{import "..."}}` dal body di un prompt.
 *
 * Pattern stesso usato lato backend (`prompt_componibili::re_import`)
 * e dal plugin CodeMirror (`import-tokens.ts`). Risultato deduplicato
 * preservando l'ordine di prima occorrenza.
 *
 * Riferimenti:
 * - Blueprint F6 §3
 */

const RE = /\{\{\s*import\s+"([^"]+)"\s*\}\}/g;

export function estraiImports(body: string): string[] {
  const visti = new Set<string>();
  const acc: string[] = [];
  let m: RegExpExecArray | null;
  RE.lastIndex = 0;
  while ((m = RE.exec(body)) !== null) {
    const path = m[1];
    if (!visti.has(path)) {
      visti.add(path);
      acc.push(path);
    }
  }
  return acc;
}
