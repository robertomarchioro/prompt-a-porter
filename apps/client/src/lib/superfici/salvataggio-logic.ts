/**
 * Logica pura per la condizione di salvataggio del prompt (issue #642).
 *
 * Estratta da DetailPane.svelte per consentire test senza runtime Svelte
 * (_pure/_impl pattern già usato per nuova-cartella-logic.ts).
 *
 * Root cause: `salvaConId()` richiedeva sia titolo sia body non vuoti per
 * procedere. I prompt nuovi nascono con `body: ""` (vedi `creaNuovoPrompt`
 * in ListPane.svelte e `nuovoPromptInCartella` in Sidebar.svelte), quindi
 * rinominare un prompt appena creato senza ancora scriverne il body
 * lasciava il salvataggio silenziosamente bloccato (`statoSalvataggio`
 * incastrato su "dirty"). Il backend (`editor.rs`) accetta già un body
 * vuoto: basta un titolo non vuoto per avere un salvataggio valido.
 */

/**
 * Restituisce true se il prompt è salvabile: basta un titolo non vuoto
 * dopo trim. Il body può essere vuoto.
 */
export function puoSalvare(titolo: string, body: string): boolean {
  void body;
  return titolo.trim().length > 0;
}
