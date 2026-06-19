/// Persistenza locale (localStorage) della lista categorie linter
/// disabilitate dall'utente in Impostazioni → Linter.
///
/// Locale-only: queste preferenze NON vengono sincronizzate al server
/// (sono UX personali, non workspace). Storage key fissa, lettura
/// difensiva contro JSON malformato.
///
/// v0.6.0 Step 6.

const STORAGE_KEY = "pap.linter.categorie_disabilitate";

export type CategoriaLinter = "LEN" | "PH" | "PII" | "STY" | "IMP";

export const CATEGORIE_LINTER: CategoriaLinter[] = [
  "LEN",
  "PH",
  "PII",
  "STY",
  "IMP",
];

/// Etichette user-friendly per ogni categoria.
export const ETICHETTE: Record<CategoriaLinter, string> = {
  LEN: "Lunghezza body (LEN)",
  PH: "Segnaposti (PH)",
  PII: "Privacy / dati personali (PII)",
  STY: "Stile / ripetizioni (STY)",
  IMP: "Import / dipendenze (IMP)",
};

/// Descrizione one-liner per tooltip / hint.
export const DESCRIZIONI: Record<CategoriaLinter, string> = {
  LEN: "Avvisi su body troppo corti o troppo lunghi.",
  PH: "Segnaposti malformati o con caratteri non consentiti.",
  PII: "Email, carte di credito, API key rilevati nel body.",
  STY: "Pattern stilistici (es. ripetizioni n-gram).",
  IMP: "Import non risolti, cicli, profondità eccessiva.",
};

/// Legge la lista delle categorie disabilitate dal localStorage.
/// Difensivo: ritorna [] se il valore è assente o malformato.
export function leggiCategorieDisabilitate(): CategoriaLinter[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((c): c is CategoriaLinter =>
      CATEGORIE_LINTER.includes(c as CategoriaLinter),
    );
  } catch {
    return [];
  }
}

/// Salva la lista nel localStorage. Filtra valori non validi.
export function salvaCategorieDisabilitate(
  disabilitate: CategoriaLinter[],
): void {
  const validi = disabilitate.filter((c) => CATEGORIE_LINTER.includes(c));
  localStorage.setItem(STORAGE_KEY, JSON.stringify(validi));
}

/// Helper: ritorna true se la categoria è attiva (non disabilitata).
export function categoriaAttiva(
  categoria: CategoriaLinter,
  disabilitate: CategoriaLinter[],
): boolean {
  return !disabilitate.includes(categoria);
}

/// Toggle: aggiunge/rimuove la categoria dalla lista disabilitate.
/// Ritorna la nuova lista (non muta l'input).
export function toggleCategoria(
  categoria: CategoriaLinter,
  disabilitate: readonly CategoriaLinter[],
): CategoriaLinter[] {
  if (disabilitate.includes(categoria)) {
    return disabilitate.filter((c) => c !== categoria);
  }
  return [...disabilitate, categoria];
}

// ─────────── Granularità per-regola (linter personalizzabile Fase 1) ───────────
//
// La lista può contenere sia prefissi di categoria ("PII") sia code completi
// di singola regola ("PII001"). Il backend (`filtra_disabilitate`) nasconde un
// issue se il suo code O il suo prefisso è nella lista.

const STORAGE_KEY_REGOLE = "pap.linter.regole_disabilitate";

/// Legge la lista (code o prefissi) delle regole disabilitate. Difensivo.
/// Migrazione one-shot: se la nuova key è assente ma esiste la vecchia
/// `pap.linter.categorie_disabilitate`, la copia (le categorie sono prefissi
/// validi) e la mantiene.
export function leggiRegoleDisabilitate(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_REGOLE);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return [];
      return parsed.filter(
        (x): x is string => typeof x === "string" && x.length > 0,
      );
    }
    // Migrazione dalla vecchia key (solo categorie/prefissi).
    const vecchie = leggiCategorieDisabilitate();
    if (vecchie.length > 0) {
      salvaRegoleDisabilitate(vecchie);
      return [...vecchie];
    }
    return [];
  } catch {
    return [];
  }
}

/// Salva la lista nel localStorage. Filtra stringhe vuote.
export function salvaRegoleDisabilitate(regole: readonly string[]): void {
  const validi = regole.filter((r) => typeof r === "string" && r.length > 0);
  localStorage.setItem(STORAGE_KEY_REGOLE, JSON.stringify(validi));
}

/// Toggle di un token (code completo o prefisso) nella lista. Immutabile.
export function toggleRegola(
  token: string,
  lista: readonly string[],
): string[] {
  if (lista.includes(token)) {
    return lista.filter((t) => t !== token);
  }
  return [...lista, token];
}

// ─────────── Config completa: tuning soglie + severità (Fase 2) ───────────
//
// Blob unico `pap.linter.config` ConfigLinter-shaped, passato al backend come
// parametro `config` di `prompt_lint`. ATTENZIONE: i campi sono in snake_case
// perché il backend li deserializza via serde con i nomi esatti della struct
// (i campi nested NON vengono convertiti da Tauri — solo gli arg top-level).

export type SeveritaLinter = "error" | "warning" | "info";

export interface SoglieLinter {
  len_max_body: number;
  len_min_body: number;
  ngram_threshold: number;
}

export interface ConfigLinter {
  /// Code completi o prefissi di famiglia da nascondere (riusa Fase 1).
  disabilitate: string[];
  /// Code esatto → severità forzata (assente = severità di default).
  severita_override: Record<string, SeveritaLinter>;
  soglie: SoglieLinter;
}

/// Default = costanti storiche del backend (regression-safe).
export const DEFAULT_SOGLIE: SoglieLinter = {
  len_max_body: 4000,
  len_min_body: 30,
  ngram_threshold: 4,
};

const STORAGE_KEY_CONFIG = "pap.linter.config";

const SEVERITA_VALIDE: readonly SeveritaLinter[] = ["error", "warning", "info"];

function isSeverita(v: unknown): v is SeveritaLinter {
  return typeof v === "string" && SEVERITA_VALIDE.includes(v as SeveritaLinter);
}

/// Normalizza un valore di soglia: intero finito ≥ 0, altrimenti il default.
function soglia(v: unknown, def: number): number {
  return typeof v === "number" && Number.isFinite(v) && v >= 0
    ? Math.floor(v)
    : def;
}

function configDefault(): ConfigLinter {
  return {
    disabilitate: [],
    severita_override: {},
    soglie: { ...DEFAULT_SOGLIE },
  };
}

/// Parsing difensivo di un oggetto sconosciuto in ConfigLinter valido.
function normalizzaConfig(parsed: unknown): ConfigLinter {
  if (typeof parsed !== "object" || parsed === null) return configDefault();
  const o = parsed as Record<string, unknown>;

  const disabilitate = Array.isArray(o.disabilitate)
    ? o.disabilitate.filter((x): x is string => typeof x === "string" && x.length > 0)
    : [];

  const overrideRaw =
    typeof o.severita_override === "object" && o.severita_override !== null
      ? (o.severita_override as Record<string, unknown>)
      : {};
  const severita_override: Record<string, SeveritaLinter> = {};
  for (const [code, sev] of Object.entries(overrideRaw)) {
    if (code.length > 0 && isSeverita(sev)) severita_override[code] = sev;
  }

  const soglieRaw =
    typeof o.soglie === "object" && o.soglie !== null
      ? (o.soglie as Record<string, unknown>)
      : {};
  const soglie: SoglieLinter = {
    len_max_body: soglia(soglieRaw.len_max_body, DEFAULT_SOGLIE.len_max_body),
    len_min_body: soglia(soglieRaw.len_min_body, DEFAULT_SOGLIE.len_min_body),
    ngram_threshold: soglia(
      soglieRaw.ngram_threshold,
      DEFAULT_SOGLIE.ngram_threshold,
    ),
  };

  return { disabilitate, severita_override, soglie };
}

/// Legge la config completa. Difensivo su JSON malformato. Migrazione one-shot:
/// se la nuova key è assente, eredita `disabilitate` dalla key Fase 1 (che a
/// sua volta migra dalle categorie legacy), con override vuoto e soglie default.
export function leggiConfig(): ConfigLinter {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_CONFIG);
    if (raw) return normalizzaConfig(JSON.parse(raw));
    // Migrazione da Fase 1.
    const disabilitate = leggiRegoleDisabilitate();
    const cfg = { ...configDefault(), disabilitate };
    if (disabilitate.length > 0) salvaConfig(cfg);
    return cfg;
  } catch {
    return configDefault();
  }
}

/// Salva la config (ri-normalizzata per sicurezza).
export function salvaConfig(cfg: ConfigLinter): void {
  const valida = normalizzaConfig(cfg);
  localStorage.setItem(STORAGE_KEY_CONFIG, JSON.stringify(valida));
}

/// Imposta/rimuove l'override di severità per un code. Immutabile.
/// Se `sev` coincide con `def` (severità di default della regola) rimuove
/// l'override per non sporcare il payload.
export function setSeverita(
  cfg: ConfigLinter,
  code: string,
  sev: SeveritaLinter,
  def: SeveritaLinter,
): ConfigLinter {
  const severita_override = { ...cfg.severita_override };
  if (sev === def) {
    delete severita_override[code];
  } else {
    severita_override[code] = sev;
  }
  return { ...cfg, severita_override };
}

/// Aggiorna un singolo campo soglia. Immutabile.
/// `min` è il minimo per-campo (allineato al clamp del backend, es. ngram ≥ 2):
/// un valore sotto il minimo viene portato al minimo, così il blob salvato non
/// diverge mai da ciò che il backend applica davvero. Input non numerico
/// (campo svuotato → NaN) ⇒ default storico.
export function setSoglia(
  cfg: ConfigLinter,
  campo: keyof SoglieLinter,
  valore: number,
  min = 0,
): ConfigLinter {
  const v = Number.isFinite(valore)
    ? Math.max(Math.floor(valore), min)
    : DEFAULT_SOGLIE[campo];
  return { ...cfg, soglie: { ...cfg.soglie, [campo]: v } };
}
