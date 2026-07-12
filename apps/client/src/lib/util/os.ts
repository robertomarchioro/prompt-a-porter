/**
 * Rilevamento del sistema operativo per etichette UI OS-aware.
 *
 * Alcune etichette (es. "Avvia con Windows" nell'onboarding) citavano
 * l'OS sbagliato su macOS/Linux: la funzione di avvio automatico è
 * cross-platform (tauri-plugin-autostart), quindi il nome dell'OS va
 * personalizzato, non nascosto.
 *
 * Stessa strategia di shortcut.ts: `navigator.platform` è esposto
 * correttamente dalle WebView Tauri (WebView2/WKWebView/WebKitGTK) e
 * viene valutato una volta al module load.
 */

export type SistemaOperativo = "windows" | "macos" | "linux";

/** Nome dell'OS da mostrare in UI, indicizzato per identificatore. */
const NOMI_OS: Record<SistemaOperativo, string> = {
  windows: "Windows",
  macos: "macOS",
  linux: "Linux",
};

/**
 * Classifica una stringa `navigator.platform` (es. "Win32", "MacIntel",
 * "Linux x86_64"). Esportata pura per testabilità; il fallback è "linux"
 * perché è l'unica piattaforma desktop supportata rimanente.
 */
export function classificaPiattaforma(platform: string): SistemaOperativo {
  if (/Mac|iPhone|iPad|iPod/i.test(platform)) return "macos";
  if (/Win/i.test(platform)) return "windows";
  return "linux";
}

export const sistemaOperativo: SistemaOperativo = classificaPiattaforma(
  typeof navigator !== "undefined" ? navigator.platform : "",
);

/** Nome leggibile dell'OS corrente: "Windows" | "macOS" | "Linux". */
export const nomeOS: string = NOMI_OS[sistemaOperativo];
