/**
 * Estrae i path degli `{{import "..."}}` dal body di un prompt.
 *
 * Pattern stesso usato lato backend (`prompt_componibili::re_import`)
 * e dal plugin CodeMirror (`import-tokens.ts`). Risultato deduplicato
 * preservando l'ordine di prima occorrenza.
 *
 * #643: gli import dentro un commento `{{!-- --}}` non contano, come nel
 * backend `parse_imports`.
 *
 * Riferimenti:
 * - Blueprint F6 §3
 */

import { rimuoviCommenti } from "../commenti";

/// Gruppo 1: path. Gruppo 2 (opzionale): modificatori M4 (`with k=v` / `version=N`).
const RE = /\{\{\s*import\s+"([^"]+)"([^}]*?)\s*\}\}/g;

export function estraiImports(body: string): string[] {
  const pulito = rimuoviCommenti(body);
  const visti = new Set<string>();
  const acc: string[] = [];
  let m: RegExpExecArray | null;
  RE.lastIndex = 0;
  while ((m = RE.exec(pulito)) !== null) {
    const path = m[1];
    if (!visti.has(path)) {
      visti.add(path);
      acc.push(path);
    }
  }
  return acc;
}
