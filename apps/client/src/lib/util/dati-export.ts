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

/**
 * Slug filesystem-safe da un titolo (per i nomi file di export per-prompt).
 * Minuscolo, spazi→trattini, rimuove i caratteri non `[a-z0-9-]`, niente
 * trattini doppi/iniziali/finali. Fallback "prompt" se resta vuoto.
 */
export function slugFile(titolo: string): string {
  const slug = titolo
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return slug || "prompt";
}

/**
 * Scarica un Blob come file nel browser (pattern `<a download>`). Effetto DOM
 * puro, isolato qui per riuso fra le sezioni che esportano (vault + per-prompt).
 */
export function scaricaBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
