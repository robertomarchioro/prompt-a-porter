/**
 * Rendering sicuro delle note di rilascio dell'updater (Impostazioni →
 * Aggiornamenti). `update.body` arriva da un contenuto esterno non fidato
 * (il testo della release su GitHub), quindi va trattato come markdown
 * potenzialmente ostile: prima convertito in HTML, poi sanificato.
 */

import { marked } from "marked";
import DOMPurify from "dompurify";

/**
 * Converte markdown non fidato in HTML sicuro da iniettare con `{@html}`.
 *
 * @param markdown testo grezzo delle note di rilascio (`update.body`)
 * @returns HTML sanificato, pronto per il rendering
 */
export function renderNotesHtml(markdown: string): string {
  const html = marked.parse(markdown, { async: false });
  return DOMPurify.sanitize(html);
}
