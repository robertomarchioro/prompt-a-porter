/**
 * Template engine MCP — segnaposti `{{nome}}`.
 *
 * Subset minimo del template engine client (Rust): supporta solo segnaposti
 * semplici `{{nome}}`, identificatori `\w+`, whitespace interno opzionale.
 * Non supporta `{{import "..."}}` (riservato al client desktop, dove gli
 * import vengono risolti contro il vault aperto).
 *
 * #643: i commenti `{{!-- … --}}` vengono tolti prima di tutto — il livello
 * MCP non li espone mai a un client (che è un modello).
 */

import { rimuoviCommenti } from "@pap/shared-schema";

const RE_SEGNAPOSTO = /\{\{\s*(\w+)\s*\}\}/g;

/**
 * Compila `body` sostituendo `{{nome}}` con `valori[nome]` (trim).
 * Se il valore manca o e' vuoto/whitespace, il segnaposto resta inalterato
 * (forma canonica `{{nome}}` senza spazi).
 */
export function compila(
  body: string,
  valori: Record<string, string>,
): string {
  return rimuoviCommenti(body).replace(
    RE_SEGNAPOSTO,
    (_, nome: string) => valori[nome]?.trim() || `{{${nome}}}`,
  );
}

/**
 * Estrae i nomi unici dei segnaposti `{{nome}}` in `body`, preservando
 * l'ordine di prima apparizione.
 */
export function estraiSegnaposti(body: string): string[] {
  const pulito = rimuoviCommenti(body);
  const visti = new Set<string>();
  const out: string[] = [];
  RE_SEGNAPOSTO.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = RE_SEGNAPOSTO.exec(pulito)) !== null) {
    if (!visti.has(m[1])) {
      visti.add(m[1]);
      out.push(m[1]);
    }
  }
  return out;
}
