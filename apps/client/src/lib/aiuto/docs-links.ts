// Guida interattiva — Fase 0: fonte di verità unica dei link alla
// documentazione utente. Tutti i punti "?" / "Approfondisci" risolvono i
// loro URL da qui, così quando esisterà il sito dedicato basta cambiare
// `urlDoc()` in un solo posto (vedi SITO_BASE).
//
// Stato attuale: i documenti utente vivono nel repo (docs/utente/*.md) e si
// aprono su GitHub nel browser di sistema. È il "sito placeholder" deciso in
// fase di design (blueprint docs/roadmap/guida-interattiva.md §9): online,
// esterno all'app, e già funzionante (nessun 404).

const REPO_ORG_REPO = "robertomarchioro/prompt-a-porter";

/**
 * Base del futuro sito di documentazione dedicato. Finché non esiste, i link
 * puntano ai .md su GitHub (vedi `urlDoc`). Quando il sito sarà pronto, basterà
 * cambiare `urlDoc` per usare questa base + lo slug della pagina.
 */
export const SITO_BASE = "https://prompt-a-porter.app/docs"; // placeholder

/** Chiavi stabili delle pagine di documentazione referenziabili dall'app. */
export type ChiaveDoc =
  | "getting-started"
  | "glossario-sintassi"
  | "segnaposti-globali"
  | "prompt-componibili"
  | "varianti"
  | "rating"
  | "regression-testing"
  | "ricerca-semantica"
  | "linting"
  | "cartelle"
  | "fork"
  | "markdown-import-export"
  | "export-json"
  | "scorciatoie"
  | "troubleshooting"
  | "auto-update"
  | "cli"
  | "mcp";

interface VoceDoc {
  /** Nome file in docs/utente/ (senza estensione). */
  file: string;
  /** Ancora opzionale dentro la pagina (heading slug). */
  ancora?: string;
  /** Etichetta breve leggibile (per tooltip/aria-label). */
  titolo: string;
}

/** Mappa chiave → documento. Unico punto da aggiornare per i contenuti. */
export const DOCS: Record<ChiaveDoc, VoceDoc> = {
  "getting-started": { file: "getting-started", titolo: "Primi passi" },
  "glossario-sintassi": {
    file: "glossario-sintassi",
    titolo: "Sintassi dei segnaposti",
  },
  "segnaposti-globali": {
    file: "glossario-sintassi",
    ancora: "segnaposti-globali",
    titolo: "Segnaposti globali",
  },
  "prompt-componibili": {
    file: "prompt-componibili",
    titolo: "Import componibili",
  },
  varianti: { file: "varianti-prompt", titolo: "Varianti A/B" },
  rating: { file: "rating-prompt", titolo: "Valutazione dei prompt" },
  "regression-testing": {
    file: "regression-testing",
    titolo: "Golden e test di regressione",
  },
  "ricerca-semantica": {
    file: "ricerca-semantica",
    titolo: "Ricerca semantica",
  },
  linting: { file: "linting-regole", titolo: "Regole del linter" },
  cartelle: { file: "cartelle", titolo: "Cartelle e tag" },
  fork: { file: "fork-prompt", titolo: "Fork dei prompt" },
  "markdown-import-export": {
    file: "markdown-import-export",
    titolo: "Import/export Markdown",
  },
  "export-json": { file: "formato-export-json", titolo: "Formato export JSON" },
  scorciatoie: { file: "scorciatoie-tastiera", titolo: "Scorciatoie da tastiera" },
  troubleshooting: { file: "troubleshooting", titolo: "Risoluzione problemi" },
  "auto-update": { file: "auto-update", titolo: "Aggiornamenti automatici" },
  cli: { file: "cli", titolo: "Interfaccia a riga di comando" },
  mcp: { file: "mcp", titolo: "Server MCP" },
};

/**
 * Costruisce l'URL (assoluto, da aprire nel browser) della pagina di doc per
 * la chiave indicata. Oggi risolve al file `.md` su GitHub; domani basterà
 * cambiare questa funzione per puntare a `SITO_BASE`.
 */
export function urlDoc(chiave: ChiaveDoc): string {
  const voce = DOCS[chiave];
  const ancora = voce.ancora ? `#${voce.ancora}` : "";
  return `https://github.com/${REPO_ORG_REPO}/blob/main/docs/utente/${voce.file}.md${ancora}`;
}

/** Titolo breve della pagina, per tooltip/aria-label dei punti "?". */
export function titoloDoc(chiave: ChiaveDoc): string {
  return DOCS[chiave].titolo;
}
