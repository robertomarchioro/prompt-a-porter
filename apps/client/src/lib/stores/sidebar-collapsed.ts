/**
 * Persistenza locale (localStorage) dello stato collapsed della sidebar
 * e dei singoli NavGroup (V0.8 F2).
 *
 * Locale-only: lo stato UI della sidebar è preferenza personale, non
 * sincronizzata. Pattern equivalente a `preferenze-linter.ts`.
 *
 * Riferimenti:
 * - Blueprint: docs/roadmap/redesign-v08/blueprint-F2.md §8
 */

const STORAGE_KEY = "pap.sidebar.collapsed";

export interface StatoSidebar {
  sidebarCollapsed: boolean;
  gruppi: {
    viste: boolean;
    visibilita: boolean;
    cartelle: boolean;
    tag: boolean;
    modelTarget: boolean;
  };
}

export const DEFAULTS: StatoSidebar = {
  sidebarCollapsed: false,
  gruppi: {
    viste: false,
    visibilita: false,
    cartelle: false,
    tag: false,
    modelTarget: true, // collassato di default come da prototipo
  },
};

interface PartialStato {
  sidebarCollapsed?: unknown;
  gruppi?: Partial<Record<keyof StatoSidebar["gruppi"], unknown>>;
}

function bool(v: unknown, fallback: boolean): boolean {
  return typeof v === "boolean" ? v : fallback;
}

export function caricaStato(): StatoSidebar {
  try {
    const raw =
      typeof localStorage !== "undefined"
        ? localStorage.getItem(STORAGE_KEY)
        : null;
    if (!raw) return structuredClone(DEFAULTS);
    const parsed = JSON.parse(raw) as PartialStato;
    return {
      sidebarCollapsed: bool(
        parsed.sidebarCollapsed,
        DEFAULTS.sidebarCollapsed,
      ),
      gruppi: {
        viste: bool(parsed.gruppi?.viste, DEFAULTS.gruppi.viste),
        visibilita: bool(parsed.gruppi?.visibilita, DEFAULTS.gruppi.visibilita),
        cartelle: bool(parsed.gruppi?.cartelle, DEFAULTS.gruppi.cartelle),
        tag: bool(parsed.gruppi?.tag, DEFAULTS.gruppi.tag),
        modelTarget: bool(parsed.gruppi?.modelTarget, DEFAULTS.gruppi.modelTarget),
      },
    };
  } catch {
    return structuredClone(DEFAULTS);
  }
}

export function salvaStato(stato: StatoSidebar): void {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(stato));
  } catch {
    // localStorage pieno o disabilitato: silently ignore
  }
}
