/**
 * Helper per generare avatar colorati deterministici (V0.8 F2 Sidebar).
 *
 * Usato dal WorkspaceSwitcher per produrre un avatar stabile a partire
 * dal nome workspace. F5 Cronologia usa un algoritmo separato (SHA1)
 * conforme alla decisione designer #13 per gli autori delle revisioni.
 *
 * Riferimenti:
 * - Decisione designer #2 (workspace switcher placeholder)
 * - Blueprint: docs/roadmap/redesign-v08/blueprint-F2.md §3
 */

/**
 * Hash djb2 sincrono (Daniel J. Bernstein). Buone proprietà di
 * distribuzione su stringhe brevi, sufficiente per derivare un hue
 * di avatar. NON usare per email utente: F5 ha SHA1 dedicato.
 */
export function hashDjb2(input: string): number {
  let hash = 5381;
  for (let i = 0; i < input.length; i++) {
    hash = ((hash << 5) + hash + input.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

export interface ColoreAvatar {
  background: string;
  foreground: string;
}

/**
 * Genera coppia { background, foreground } HSL deterministica per un
 * input stringa.
 *
 * - hue derivato dall'hash (0-359)
 * - sat 55%, light 58% (coerente con palette dark/light tokens)
 * - foreground bianco se light < 60, nero altrimenti (AA contrast euristica)
 */
export function coloreAvatar(input: string): ColoreAvatar {
  const hash = hashDjb2(input);
  const hue = hash % 360;
  const sat = 55;
  const light = 58;
  return {
    background: `hsl(${hue} ${sat}% ${light}%)`,
    foreground: light < 60 ? "#fff" : "#000",
  };
}
