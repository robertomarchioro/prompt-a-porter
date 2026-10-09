/**
 * Esito di un import (`ImportReport` in `import_export.rs`) e regole di
 * presentazione comuni a Impostazioni, Collezioni e Onboarding.
 */

export interface ImportReport {
  nuovi: number;
  aggiornati: number;
  conflitti: number;
  /** Solo fallimenti reali: il messaggio di annullamento NON è qui dentro. */
  errori: string[];
  /** `true` se il backend ha annullato l'import per intero (nulla scritto). */
  annullato: boolean;
}

export const MESSAGGIO_IMPORT_ANNULLATO =
  "Importazione annullata: nessuna modifica è stata salvata.";

/**
 * Un import è fallito se è stato annullato o ha riportato errori: in
 * entrambi i casi la UI non deve mostrare un successo né contatori di righe
 * che il backend ha annullato.
 */
export function importFallito(report: ImportReport): boolean {
  return report.annullato || report.errori.length > 0;
}
