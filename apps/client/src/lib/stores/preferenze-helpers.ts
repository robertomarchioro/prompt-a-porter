/**
 * Funzioni pure per la gestione tema/tono — separate da `preferenze.svelte.ts`
 * (che usa $state runes Svelte 5) per consentire unit test diretti via Vitest
 * senza il plugin svelte-vitest.
 *
 * Riferimenti:
 * - Blueprint: docs/roadmap/redesign-v08/blueprint-F0.md §6 §9
 */

/**
 * Risolve un valore di tema a "dark" o "light" concreto.
 *
 * - "dark" → "dark"
 * - "light" → "light"
 * - "auto" o sconosciuto → matchMedia("prefers-color-scheme: dark") →
 *   "dark" se sistema preferisce dark, altrimenti "light"
 * - In ambienti senza `window.matchMedia` (test SSR) ritorna "light" come
 *   fallback, coerente con il default light-first del primo avvio
 *   (issue #269).
 */
export function risolviTema(tema: string): "dark" | "light" {
  if (tema === "dark") return "dark";
  if (tema === "light") return "light";
  if (typeof window !== "undefined" && window.matchMedia) {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return "light";
}

/**
 * Applica `data-theme` (risolto) e `data-tone` su `<html>`. Pure side-effect
 * sul DOM: testabile direttamente senza Tauri. La risoluzione di "auto"
 * avviene al momento dell'applicazione, NON al momento del save.
 */
export function applicaThemeTone(tema: string, tono: string): void {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", risolviTema(tema));
  document.documentElement.setAttribute("data-tone", tono);
}
