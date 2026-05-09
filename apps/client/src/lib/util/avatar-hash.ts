/**
 * Avatar hash deterministico SHA1+HSL per email autori (V0.8 F5 PR-D).
 *
 * Conforme alla decisione designer #13: l'avatar autore della Cronologia
 * deve essere stabile (gravatar-like) ma calcolato offline senza
 * fingerprinting esterno. Algoritmo:
 *
 *   hue = parseInt(SHA1(email).slice(0, 6), 16) % 360
 *   sat = 55%
 *   light = 58%
 *
 * F2 usa un djb2 sync (`avatar-color.ts`) per workspace name. Qui SHA1
 * async è la specifica del designer per le email autori.
 *
 * Riferimenti:
 * - Decisione designer #13
 * - Blueprint F5 PR-D §4
 */

const cache = new Map<string, ColoreAvatar>();

export interface ColoreAvatar {
  background: string;
  foreground: string;
}

async function sha1Hex(input: string): Promise<string> {
  const enc = new TextEncoder().encode(input);
  const hash = await crypto.subtle.digest("SHA-1", enc);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Risolve un avatar color deterministico per una email.
 * Usa cache in-memory durante la sessione (key = email lowercase trimmed).
 */
export async function avatarColorePerEmail(
  email: string,
): Promise<ColoreAvatar> {
  const key = email.trim().toLowerCase();
  const hit = cache.get(key);
  if (hit) return hit;

  const h = await sha1Hex(key);
  const hex = h.slice(0, 6);
  const hue = parseInt(hex, 16) % 360;
  const colore: ColoreAvatar = {
    background: `hsl(${hue} 55% 58%)`,
    foreground: "#fff",
  };
  cache.set(key, colore);
  return colore;
}

/**
 * Reset cache (esposto per testabilità).
 */
export function _resetCache(): void {
  cache.clear();
}
