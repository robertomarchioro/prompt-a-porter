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
