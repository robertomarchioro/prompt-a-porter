/**
 * Fix #134 — formatter OS-aware per shortcut tastiera.
 *
 * Le label dei kbd nei title/UI v0.8.0 usavano i glifi macOS (⌘ ⌃ ⇧ ↵)
 * anche su Windows/Linux, dove l'utente si aspetta "Ctrl+K" e simili.
 * Questa utility rileva la piattaforma da `navigator.platform` (Tauri
 * WebView lo espone correttamente) e formatta la stringa di conseguenza.
 *
 * Esempi:
 *   fmtShortcut("mod+k")          → "⌘K" (mac) / "Ctrl+K" (win/linux)
 *   fmtShortcut("ctrl+shift+p")   → "⌃⇧P" (mac) / "Ctrl+Shift+P"
 *   fmtShortcut("mod+,")          → "⌘," / "Ctrl+,"
 *   fmtShortcut("ctrl+enter")     → "⌃↵" / "Ctrl+Enter"
 *   fmtShortcut("esc")            → "Esc" / "Esc"
 *
 * Riferimento: github.com/robertomarchioro/prompt-a-porter#134
 */

export const isMac =
  typeof navigator !== "undefined" &&
  /Mac|iPhone|iPad|iPod/i.test(navigator.platform);

const MOD_GLYPHS: Record<string, [string, string]> = {
  // [mac, win/linux]
  mod: ["⌘", "Ctrl"],
  ctrl: ["⌃", "Ctrl"],
  shift: ["⇧", "Shift"],
  alt: ["⌥", "Alt"],
  meta: ["⌘", "Win"],
};

const KEY_GLYPHS: Record<string, [string, string]> = {
  enter: ["↵", "Enter"],
  return: ["↵", "Enter"],
  esc: ["Esc", "Esc"],
  escape: ["Esc", "Esc"],
  up: ["↑", "↑"],
  down: ["↓", "↓"],
  left: ["←", "←"],
  right: ["→", "→"],
  space: ["␣", "Space"],
  tab: ["⇥", "Tab"],
  backspace: ["⌫", "Backspace"],
  delete: ["⌦", "Delete"],
};

const MODIFIERS = new Set(Object.keys(MOD_GLYPHS));

/**
 * Formatta una stringa shortcut tipo "mod+k" o "ctrl+shift+p" in label
 * leggibile per l'OS corrente.
 *
 * Convenzione `combo`:
 * - tutti lowercase, separati da `+`
 * - modifier validi: mod / ctrl / shift / alt / meta
 *   (mod = ⌘ su mac, Ctrl altrove — è il "leader" generico)
 * - key finale: una lettera/cifra/symbolo, o un nome speciale
 *   (enter, esc, up, down, left, right, space, tab, backspace, delete)
 */
export function fmtShortcut(combo: string): string {
  const parts = combo
    .toLowerCase()
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean);
  if (parts.length === 0) return "";

  const platformIdx = isMac ? 0 : 1;
  const formatted: string[] = [];

  for (const part of parts) {
    if (MODIFIERS.has(part)) {
      formatted.push(MOD_GLYPHS[part][platformIdx]);
    } else if (KEY_GLYPHS[part]) {
      formatted.push(KEY_GLYPHS[part][platformIdx]);
    } else {
      // tasto singolo o multi-char (es. "f4", "f12"): uppercase totale.
      // Cifre e punteggiatura sono invarianti su toUpperCase().
      formatted.push(part.toUpperCase());
    }
  }

  // Su mac, modifier glifo + key senza separatore: "⌘K"
  // Su win/linux, "Ctrl+K"
  return isMac ? formatted.join("") : formatted.join("+");
}
