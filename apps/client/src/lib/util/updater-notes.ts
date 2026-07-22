/**
 * Rendering sicuro delle note di rilascio dell'updater (Impostazioni →
 * Aggiornamenti). `update.body` arriva da un contenuto esterno non fidato
 * (il testo della release su GitHub), quindi va trattato come markdown
 * potenzialmente ostile: prima convertito in HTML, poi sanificato con un
 * profilo ristretto (solo i tag/attributi che il dialog sa stilare).
 */

import { marked } from "marked";
import DOMPurify from "dompurify";

// Solo i tag effettivamente stilati da `.updater-notes` in ImpostazioniModal.
const ALLOWED_TAGS = [
  "h1",
  "h2",
  "h3",
  "p",
  "ul",
  "ol",
  "li",
  "code",
  "pre",
  "a",
  "strong",
  "em",
  "br",
  "blockquote",
];
// Niente <style>/style= (che DOMPurify permetterebbe di default) e niente
// altri attributi arbitrari. `target`/`rel` sono aggiunti dall'hook sotto.
const ALLOWED_ATTR = ["href"];

// Registrato una sola volta a livello di modulo (non a ogni sanitize()):
// come da app-convention (AboutSezione.svelte, ImpostazioniModal.svelte),
// ogni link esterno nel dialog apre una nuova scheda invece di navigare via
// la webview single-window di Tauri, perdendo lo stato dell'app.
DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (node.tagName === "A") {
    node.setAttribute("target", "_blank");
    node.setAttribute("rel", "noopener noreferrer");
  }
});

/**
 * Converte markdown non fidato in HTML sicuro da iniettare con `{@html}`.
 *
 * @param markdown testo grezzo delle note di rilascio (`update.body`)
 * @returns HTML sanificato, pronto per il rendering
 */
export function renderNotesHtml(markdown: string): string {
  const html = marked.parse(markdown, { async: false });
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ADD_ATTR: ["target", "rel"],
  });
}
