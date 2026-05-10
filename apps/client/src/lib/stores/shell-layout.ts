/**
 * Persistenza locale (localStorage) della larghezza in pixel delle 2
 * colonne resizable della Shell (Sidebar + ListPane). Pattern identico
 * a `sidebar-collapsed.ts` e `densita.ts`.
 *
 * Default 248px / 320px replicati dal prototipo originale
 * (`docs/architettura/redesign/prototype/redesign.css:117`):
 *
 *   .body {
 *     grid-template-columns:
 *       var(--col-sidebar, 248px) 1px var(--col-list, 320px) 1px minmax(0, 1fr);
 *   }
 *
 * Riferimenti:
 * - Issue #137 (layout proporzioni iniziali sbagliate, drag-resize
 *   invertito): rifatto Shell.svelte con CSS grid puro come prototipo
 *   invece di paneforge percentuali.
 */

const STORAGE_KEY = "pap.shell.layout";

export interface StatoLayout {
  colSidebar: number;
  colList: number;
}

export const DEFAULTS: StatoLayout = {
  colSidebar: 248,
  colList: 320,
};

export const COL_SIDEBAR_MIN = 200;
export const COL_SIDEBAR_MAX = 480;
export const COL_LIST_MIN = 240;
export const COL_LIST_MAX = 560;

interface PartialStato {
  colSidebar?: unknown;
  colList?: unknown;
}

function clampSidebar(v: unknown): number {
  if (typeof v !== "number" || !Number.isFinite(v)) return DEFAULTS.colSidebar;
  return Math.min(COL_SIDEBAR_MAX, Math.max(COL_SIDEBAR_MIN, Math.round(v)));
}

function clampList(v: unknown): number {
  if (typeof v !== "number" || !Number.isFinite(v)) return DEFAULTS.colList;
  return Math.min(COL_LIST_MAX, Math.max(COL_LIST_MIN, Math.round(v)));
}

export function caricaStato(): StatoLayout {
  try {
    const raw =
      typeof localStorage !== "undefined"
        ? localStorage.getItem(STORAGE_KEY)
        : null;
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as PartialStato;
    return {
      colSidebar: clampSidebar(parsed.colSidebar),
      colList: clampList(parsed.colList),
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export function salvaStato(stato: StatoLayout): void {
  try {
    if (typeof localStorage === "undefined") return;
    const sanitizzato: StatoLayout = {
      colSidebar: clampSidebar(stato.colSidebar),
      colList: clampList(stato.colList),
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sanitizzato));
  } catch {
    // localStorage pieno o disabilitato: silently ignore
  }
}
