/**
 * Logica pura della modale «Cartamodello» (scomposizione di prompt piatti
 * in moduli componibili). Tipi specchio dei comandi Rust
 * `cartamodello_analizza` / `cartamodello_applica`, scelte dell'utente in
 * revisione, motivi che bloccano l'applicazione e costruzione della
 * richiesta. Testabile senza runtime Svelte.
 *
 * Blueprint: docs/roadmap/cartamodello.md.
 */

export type TipoModulo =
  | "ruolo"
  | "vincoli"
  | "formato"
  | "contesto"
  | "esempi"
  | "altro";

export interface Riuso {
  prompt_id: string;
  titolo: string;
  similarita: number;
  fonte: "semantica" | "lessicale";
}

export interface ModuloVerificato {
  chiave: string;
  titolo: string;
  tipo: TipoModulo;
  corpo: string;
  usato_da: string[];
  problemi: string[];
  riuso: Riuso | null;
  titolo_in_conflitto: string | null;
}

export interface PromptVerificato {
  id: string;
  titolo_originale: string;
  titolo: string;
  descrizione: string | null;
  corpo: string;
  corpo_originale: string;
  anteprima_espansa: string | null;
  import_non_risolti: string[];
  problemi: string[];
  segnaposti: string[];
}

export interface Proposta {
  moduli: ModuloVerificato[];
  prompt: PromptVerificato[];
  note: string[];
  avvisi: string[];
  riuso_semantico: boolean;
  cartella_moduli: string;
  tokens_used: number | null;
  costo_stimato: number | null;
  provider: string;
  model: string;
  troncato: boolean;
}

/** Cosa fare di un modulo, deciso in revisione. */
export type AzioneModulo = "crea" | "riusa" | "scarta";

/** Stato editabile di un modulo nella revisione. */
export interface SceltaModulo {
  chiave: string;
  azione: AzioneModulo;
  /** Titolo eventualmente rinominato dall'utente. */
  titolo: string;
}

/** Titolo con cui salvare ogni prompt ricomposto (default: l'originale). */
export interface SceltaPrompt {
  id: string;
  titolo: string;
}

/**
 * Di default il prompt conserva il suo titolo: quello proposto dal modello
 * è un suggerimento visibile in revisione, mai una rinomina silenziosa.
 */
export function scelteInizialiPrompt(proposta: Proposta): SceltaPrompt[] {
  return proposta.prompt.map((p) => ({ id: p.id, titolo: p.titolo_originale }));
}

export function conTitoloPrompt(
  scelte: SceltaPrompt[],
  id: string,
  titolo: string,
): SceltaPrompt[] {
  return scelte.map((s) => (s.id === id ? { ...s, titolo } : s));
}

export interface RiferimentoPrompt {
  id: string;
  titolo: string;
}

export interface EsitoApplica {
  moduli_creati: RiferimentoPrompt[];
  moduli_riusati: RiferimentoPrompt[];
  prompt_aggiornati: RiferimentoPrompt[];
  cartella_moduli: string | null;
}

/** Etichette leggibili dei tipi di modulo. */
export const ETICHETTA_TIPO: Record<TipoModulo, string> = {
  ruolo: "Ruolo",
  vincoli: "Vincoli",
  formato: "Formato",
  contesto: "Contesto",
  esempi: "Esempi",
  altro: "Altro",
};

/**
 * Scelta di partenza per ogni modulo: se il vault ha già un equivalente si
 * propone il riuso, altrimenti la creazione. Non si scarta mai in
 * automatico: è una decisione dell'utente.
 */
export function scelteIniziali(proposta: Proposta): SceltaModulo[] {
  return proposta.moduli.map((m) => ({
    chiave: m.chiave,
    azione: m.riuso ? "riusa" : "crea",
    titolo: m.titolo,
  }));
}

/** Aggiorna una scelta senza mutare l'array. */
export function conScelta(
  scelte: SceltaModulo[],
  chiave: string,
  patch: Partial<Omit<SceltaModulo, "chiave">>,
): SceltaModulo[] {
  return scelte.map((s) => (s.chiave === chiave ? { ...s, ...patch } : s));
}

/**
 * Motivi che impediscono di applicare. Vuoto = si può applicare. Ogni
 * motivo è una frase pronta per la UI.
 */
export function motiviBloccanti(
  proposta: Proposta,
  scelte: SceltaModulo[],
  sceltePrompt: SceltaPrompt[] = scelteInizialiPrompt(proposta),
): string[] {
  const motivi: string[] = [];
  if (proposta.prompt.length === 0) {
    motivi.push("Il modello non ha restituito nessun prompt ricomposto.");
  }
  for (const sp of sceltePrompt) {
    if (!sp.titolo.trim()) {
      const p = proposta.prompt.find((x) => x.id === sp.id);
      motivi.push(`Il prompt «${p?.titolo_originale ?? sp.id}» non può restare senza titolo.`);
    }
  }
  const titoliCrea = new Map<string, string>();
  for (const m of proposta.moduli) {
    const s = scelte.find((x) => x.chiave === m.chiave);
    if (!s || s.azione !== "crea") continue;
    const titolo = s.titolo.trim();
    if (!titolo) {
      motivi.push(`Il modulo «${m.titolo || m.chiave}» non ha titolo.`);
      continue;
    }
    if (titolo.includes("/")) {
      motivi.push(`Il titolo «${titolo}» non può contenere «/».`);
    }
    const chiaveTitolo = titolo.toLowerCase();
    const altro = titoliCrea.get(chiaveTitolo);
    if (altro) {
      motivi.push(`Due moduli da creare hanno lo stesso titolo «${titolo}».`);
    } else {
      titoliCrea.set(chiaveTitolo, m.chiave);
    }
    // Il conflitto con il vault vale solo se il titolo non è stato cambiato.
    if (m.titolo_in_conflitto && titolo.toLowerCase() === m.titolo.trim().toLowerCase()) {
      motivi.push(
        `Esiste già un prompt intitolato «${titolo}»: rinominalo o usa quello esistente.`,
      );
    }
    for (const p of m.problemi) {
      motivi.push(`Modulo «${titolo}»: ${p}`);
    }
  }
  for (const p of proposta.prompt) {
    for (const imp of p.import_non_risolti) {
      motivi.push(`In «${p.titolo}» l'import «${imp}» non risolve a nessun modulo.`);
    }
    for (const e of p.problemi) {
      motivi.push(`«${p.titolo}»: ${e}`);
    }
  }
  return motivi;
}

/** Richiesta per `cartamodello_applica`, dalla proposta e dalle scelte. */
export function costruisciRichiesta(
  proposta: Proposta,
  scelte: SceltaModulo[],
  sceltePrompt: SceltaPrompt[] = scelteInizialiPrompt(proposta),
): {
  moduli: Array<{
    titolo: string;
    tipo: TipoModulo;
    corpo: string;
    azione: { tipo: "crea" } | { tipo: "riusa"; prompt_id: string } | { tipo: "scarta" };
  }>;
  prompt: Array<{
    id: string;
    titolo: string;
    corpo: string;
    corpo_originale_atteso: string;
  }>;
} {
  return {
    moduli: proposta.moduli.map((m) => {
      const s = scelte.find((x) => x.chiave === m.chiave);
      const azione = s?.azione ?? "crea";
      return {
        titolo: (s?.titolo ?? m.titolo).trim(),
        tipo: m.tipo,
        corpo: m.corpo,
        azione:
          azione === "riusa" && m.riuso
            ? { tipo: "riusa", prompt_id: m.riuso.prompt_id }
            : azione === "scarta"
              ? { tipo: "scarta" }
              : { tipo: "crea" },
      };
    }),
    prompt: proposta.prompt.map((p) => ({
      id: p.id,
      titolo: (sceltePrompt.find((s) => s.id === p.id)?.titolo ?? p.titolo_originale).trim(),
      corpo: riscriviImportRinominati(p.corpo, proposta, scelte),
      corpo_originale_atteso: p.corpo_originale,
    })),
  };
}

/**
 * Se l'utente ha rinominato un modulo da creare, gli `{{import "vecchio"}}`
 * dei prompt devono seguire il nuovo titolo. Sostituzione sui token
 * `import "…"` con confronto case-insensitive del titolo.
 */
export function riscriviImportRinominati(
  corpo: string,
  proposta: Proposta,
  scelte: SceltaModulo[],
): string {
  let out = corpo;
  for (const m of proposta.moduli) {
    const s = scelte.find((x) => x.chiave === m.chiave);
    if (!s || s.azione !== "crea") continue;
    const nuovo = s.titolo.trim();
    if (!nuovo || nuovo.toLowerCase() === m.titolo.trim().toLowerCase()) continue;
    const re = new RegExp(
      `(\\{\\{\\s*import\\s+")${escapeRegExp(m.titolo.trim())}(")`,
      "gi",
    );
    out = out.replace(re, `$1${nuovo}$2`);
  }
  return out;
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Riga di riepilogo dopo l'applicazione. */
export function formattaEsito(esito: EsitoApplica): string {
  const parti: string[] = [];
  if (esito.moduli_creati.length > 0) {
    parti.push(
      esito.moduli_creati.length === 1
        ? "1 modulo creato"
        : `${esito.moduli_creati.length} moduli creati`,
    );
  }
  if (esito.moduli_riusati.length > 0) {
    parti.push(
      esito.moduli_riusati.length === 1
        ? "1 modulo riusato"
        : `${esito.moduli_riusati.length} moduli riusati`,
    );
  }
  parti.push(
    esito.prompt_aggiornati.length === 1
      ? "1 prompt ricomposto (nuova versione)"
      : `${esito.prompt_aggiornati.length} prompt ricomposti (nuova versione)`,
  );
  const base = parti.join(" · ");
  return esito.cartella_moduli ? `${base} · moduli in ${esito.cartella_moduli}` : base;
}

export function formattaSimilarita(r: Riuso): string {
  const pct = Math.round(r.similarita * 100);
  return r.fonte === "semantica" ? `${pct}% simile` : `${pct}% di parole in comune`;
}

export function formattaCosto(c: number | null): string {
  if (c == null) return "n/d";
  return c < 0.01 ? `~$${c.toFixed(4)}` : `~$${c.toFixed(2)}`;
}
