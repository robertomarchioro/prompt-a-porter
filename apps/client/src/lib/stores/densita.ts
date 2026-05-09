/**
 * Persistenza locale (localStorage) della densità lista, righe preview
 * e ordine selezionato (V0.8 F3 List pane).
 *
 * Locale-only: preferenze UI personali. Pattern equivalente a
 * `sidebar-collapsed.ts` e `preferenze-linter.ts`.
 *
 * Riferimenti:
 * - Blueprint: docs/roadmap/redesign-v08/blueprint-F3.md §3
 */

const STORAGE_KEY = "pap.lista.densita";

export type Densita = "compatta" | "comoda" | "anteprima";
export type Ordine = "recente" | "popolare" | "qualita" | "alfabetico";

export interface StatoLista {
  densita: Densita;
  righePreview: number; // 1-8, usato solo da densità "anteprima" (F3 PR-B)
  ordine: Ordine;
}

const DENSITA_AMMESSE: readonly Densita[] = ["compatta", "comoda", "anteprima"];
const ORDINI_AMMESSI: readonly Ordine[] = [
  "recente",
  "popolare",
  "qualita",
  "alfabetico",
];

export const DEFAULTS: StatoLista = {
  densita: "comoda",
  righePreview: 3,
  ordine: "recente",
};

const RIGHE_MIN = 1;
const RIGHE_MAX = 8;

interface PartialStato {
  densita?: unknown;
  righePreview?: unknown;
  ordine?: unknown;
}

function clampRighe(n: unknown): number {
  if (typeof n !== "number" || !Number.isFinite(n)) return DEFAULTS.righePreview;
  return Math.min(RIGHE_MAX, Math.max(RIGHE_MIN, Math.round(n)));
}

function pickEnum<T extends string>(
  v: unknown,
  ammessi: readonly T[],
  fallback: T,
): T {
  return typeof v === "string" && (ammessi as readonly string[]).includes(v)
    ? (v as T)
    : fallback;
}

export function caricaStato(): StatoLista {
  try {
    const raw =
      typeof localStorage !== "undefined"
        ? localStorage.getItem(STORAGE_KEY)
        : null;
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as PartialStato;
    return {
      densita: pickEnum(parsed.densita, DENSITA_AMMESSE, DEFAULTS.densita),
      righePreview: clampRighe(parsed.righePreview),
      ordine: pickEnum(parsed.ordine, ORDINI_AMMESSI, DEFAULTS.ordine),
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export function salvaStato(stato: StatoLista): void {
  try {
    if (typeof localStorage === "undefined") return;
    const sanitizzato: StatoLista = {
      densita: pickEnum(stato.densita, DENSITA_AMMESSE, DEFAULTS.densita),
      righePreview: clampRighe(stato.righePreview),
      ordine: pickEnum(stato.ordine, ORDINI_AMMESSI, DEFAULTS.ordine),
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sanitizzato));
  } catch {
    // localStorage pieno o disabilitato: silently ignore
  }
}
