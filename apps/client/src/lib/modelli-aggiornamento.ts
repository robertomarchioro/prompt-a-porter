// Logica pura di aggiornamento del registro modelli (`modelli-registro.json`).
//
// Volutamente SENZA rete e SENZA filesystem: prende il listino grezzo di
// models.dev e il registro corrente, restituisce il registro nuovo. Il runner
// (`scripts/aggiorna-modelli.ts`) si occupa di scaricare e riscrivere.
//
// Perché models.dev e non le API dei provider: è pubblico e senza chiave
// (niente secret in CI), copre i tre provider in un colpo solo e porta anche
// i prezzi, che le API ufficiali NON restituiscono.

import type { ModelloAI, ProviderConElenco, RegistroModelli } from "./modelli-provider";

/** Data di rilascio sotto la quale un modello non entra in registro. */
const SOGLIA_RILASCIO = "2025-01-01";

/** Chiave del provider in models.dev → nome del provider in PAP. */
const MAPPA_PROVIDER: Record<string, ProviderConElenco> = {
  anthropic: "anthropic",
  openai: "openai",
  google: "gemini",
};

/**
 * Varianti specialistiche di OpenAI escluse dalle tendine: `-pro` (fino a
 * 150 $/Mtok, fuori scala rispetto alle altre voci), i modelli `codex`, la
 * serie `o` di ragionamento e gli alias mobili `-chat-latest`.
 */
const OPENAI_SPECIALISTICI = /(-pro$)|codex|(^o\d)|(-chat-latest$)/;

/**
 * Anteprime ammesse: solo la linea Pro di Gemini, perché Google **non**
 * pubblica una versione stabile di `gemini-N-pro` — escluderle vorrebbe dire
 * non offrire affatto il suo modello di punta. Esclude di proposito le
 * anteprime di nicchia (`-customtools`, `robotics`, `computer-use`).
 */
const ANTEPRIMA_AMMESSA = /^gemini-[\d.]+-pro-preview$/;

/** Alias datati (`gpt-4o-2024-05-13`): teniamo solo l'id stabile. */
const ALIAS_DATATO = /-20\d{2}(-\d{2}){2}$|-20\d{6}$/;

/** Forma minima di una voce di models.dev; il resto dei campi è ignorato. */
export interface VoceListino {
  id?: string;
  name?: string;
  release_date?: string;
  open_weights?: boolean;
  modalities?: { input?: string[]; output?: string[] };
  limit?: { context?: number };
  cost?: { input?: number; output?: number };
}

export interface ListinoGrezzo {
  [provider: string]: { models?: Record<string, VoceListino> };
}

/**
 * True se la voce è un modello di chat testuale che vogliamo in tendina.
 *
 * Gli embedding vanno esclusi esplicitamente: dichiarano `output: ["text"]`
 * come i modelli di chat, ma hanno costo di output 0 — è quello il segnale.
 */
export function ammesso(id: string, v: VoceListino, provider: ProviderConElenco): boolean {
  const output = v.modalities?.output ?? [];
  const input = v.modalities?.input ?? [];
  if (output.length !== 1 || output[0] !== "text") return false;
  if (!input.includes("text")) return false;
  if (v.open_weights) return false;
  if (ALIAS_DATATO.test(id)) return false;
  if (id.includes("embedding")) return false;
  if ((v.cost?.output ?? 0) <= 0) return false;
  if ((v.release_date ?? "9999") < SOGLIA_RILASCIO) return false;
  if (id.includes("-preview") && !ANTEPRIMA_AMMESSA.test(id)) return false;
  if (provider === "openai" && OPENAI_SPECIALISTICI.test(id)) return false;
  return true;
}

/** Estrae dal listino grezzo i modelli ammessi, ordinati per id. */
export function estraiModelli(listino: ListinoGrezzo, oggi: string): ModelloAI[] {
  const fuori: ModelloAI[] = [];
  for (const [chiave, provider] of Object.entries(MAPPA_PROVIDER)) {
    const voci = listino[chiave]?.models ?? {};
    for (const [id, v] of Object.entries(voci)) {
      if (!ammesso(id, v, provider)) continue;
      fuori.push({
        id,
        provider,
        etichetta: v.name ?? id,
        obsoleto: false,
        anteprima: id.includes("-preview"),
        visto_il: oggi,
        contesto: v.limit?.context ?? null,
        prezzo:
          v.cost?.input !== undefined && v.cost?.output !== undefined
            ? { input: v.cost.input, output: v.cost.output }
            : null,
      });
    }
  }
  return fuori.sort((a, b) => a.id.localeCompare(b.id));
}

/**
 * Fonde il listino nel registro corrente.
 *
 * Regola richiesta: un modello già in registro che **sparisce** dal listino
 * NON viene rimosso, viene marcato `obsoleto` — chi lo ha configurato non se
 * lo deve veder sparire da sotto. Un modello che ricompare torna attivo.
 */
export function fondi(
  corrente: readonly ModelloAI[],
  listino: readonly ModelloAI[],
  oggi: string,
): ModelloAI[] {
  const perId = new Map(listino.map((m) => [m.id, m]));
  const idCorrenti = new Set(corrente.map((m) => m.id));

  const aggiornati = corrente.map((vecchio) => {
    const fresco = perId.get(vecchio.id);
    if (!fresco) return { ...vecchio, obsoleto: true };
    return { ...fresco, visto_il: oggi, obsoleto: false };
  });

  const nuovi = listino.filter((m) => !idCorrenti.has(m.id));
  return [...aggiornati, ...nuovi].sort(
    (a, b) => a.provider.localeCompare(b.provider) || a.id.localeCompare(b.id),
  );
}

/** Differenza leggibile tra due registri, per il corpo della PR settimanale. */
export function descriviDelta(
  prima: readonly ModelloAI[],
  dopo: readonly ModelloAI[],
): { nuovi: string[]; obsoleti: string[]; tornati: string[] } {
  const primaPerId = new Map(prima.map((m) => [m.id, m]));
  const nuovi: string[] = [];
  const obsoleti: string[] = [];
  const tornati: string[] = [];

  for (const m of dopo) {
    const vecchio = primaPerId.get(m.id);
    if (!vecchio) nuovi.push(m.id);
    else if (!vecchio.obsoleto && m.obsoleto) obsoleti.push(m.id);
    else if (vecchio.obsoleto && !m.obsoleto) tornati.push(m.id);
  }
  return { nuovi, obsoleti, tornati };
}

/**
 * Soglia di sanità: sotto questo numero di modelli per provider il listino è
 * considerato rotto (schema cambiato, risposta troncata, CDN che serve una
 * pagina d'errore) e l'aggiornamento va **fermato**, non scritto a metà.
 *
 * Senza questo controllo un listino vuoto marcherebbe obsoleto l'intero
 * registro in un colpo solo, e la PR sembrerebbe legittima.
 */
export const MINIMO_PER_PROVIDER = 3;

export function verificaSanita(listino: readonly ModelloAI[]): string | null {
  for (const p of Object.values(MAPPA_PROVIDER)) {
    const n = listino.filter((m) => m.provider === p).length;
    if (n < MINIMO_PER_PROVIDER) {
      return `Listino sospetto: solo ${n} modelli per '${p}' (minimo ${MINIMO_PER_PROVIDER}). Aggiornamento annullato.`;
    }
  }
  return null;
}

export function costruisciRegistro(
  corrente: RegistroModelli,
  listino: ListinoGrezzo,
  oggi: string,
): { registro: RegistroModelli; errore: string | null } {
  const estratti = estraiModelli(listino, oggi);
  const errore = verificaSanita(estratti);
  if (errore) return { registro: corrente, errore };

  return {
    registro: {
      ...corrente,
      aggiornato_a: oggi,
      modelli: fondi(corrente.modelli, estratti, oggi),
    },
    errore: null,
  };
}
