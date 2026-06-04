/**
 * Helper puri per l'export del vault (sezione Impostazioni → Dati).
 *
 * Estratti per DRY (zip markdown + json condividono il naming) e per
 * testabilità senza montare il componente o mockare il DOM.
 */

/**
 * Costruisce il nome file di un export del vault datato.
 *
 * @example
 * nomeFileExport("json", "2026-06-03T10:30:00.000Z")
 * // -> "prompt-a-porter-export-2026-06-03.json"
 *
 * @param estensione estensione del file, con o senza punto iniziale
 * @param dataIso timestamp ISO 8601 (tipicamente `new Date().toISOString()`)
 */
export function nomeFileExport(estensione: string, dataIso: string): string {
  const giorno = dataIso.slice(0, 10);
  const ext = estensione.startsWith(".") ? estensione.slice(1) : estensione;
  return `prompt-a-porter-export-${giorno}.${ext}`;
}
