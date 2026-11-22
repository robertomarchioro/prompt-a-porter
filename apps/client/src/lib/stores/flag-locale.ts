/**
 * Flag booleani di UI persistiti in localStorage (#747).
 *
 * Locale-only: preferenze personali di layout, non sincronizzate.
 * Formato del valore: "1" / "0". Qualsiasi altro valore, una chiave
 * assente o un localStorage non disponibile ricadono sul predefinito.
 */

export function caricaFlag(chiave: string, predefinito = false): boolean {
  try {
    const raw = localStorage.getItem(chiave);
    if (raw === "1") return true;
    if (raw === "0") return false;
    return predefinito;
  } catch {
    return predefinito;
  }
}

export function salvaFlag(chiave: string, valore: boolean): void {
  try {
    localStorage.setItem(chiave, valore ? "1" : "0");
  } catch {
    // localStorage pieno o disabilitato: la preferenza resta solo in memoria
  }
}
