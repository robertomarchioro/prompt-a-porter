// Guida interattiva — Fase 0: fonte di verità unica dei link alla
// documentazione utente. Tutti i punti "?" / "Approfondisci" risolvono i
// loro URL da qui: un solo posto da cambiare per spostare i link.
//
// Fix #554/#555/#557: il sito pubblico è live su www.promptaporter.it
// (landing "Scontrino cucito", VitePress — apps/site/.vitepress/config.ts)
// da prima ancora della registrazione di questo SITO_BASE: il dominio
// `prompt-a-porter.app` usato finora era un placeholder mai registrato, e
// `urlDoc()` risolveva ancora ai file .md su GitHub. Un solo switch da
// accendere: base del sito + slug pagina, niente più `/blob/main/...md`.
// VitePress ha `cleanUrls: true` (vedi config), quindi niente estensione
// `.md` nell'URL pubblico.

/**
 * Base del sito di documentazione pubblico. Le pagine utente vivono sotto
 * `/utente/<slug>` (VitePress `srcDir: ../../docs`, `base: /`).
 */
export const SITO_BASE = "https://www.promptaporter.it";

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
 * la chiave indicata: `SITO_BASE` + `/utente/<file>` (+ ancora se presente).
 * Niente estensione `.md`, niente `/blob/main/` — VitePress ha
 * `cleanUrls: true` e serve i documenti utente da `/utente/<slug>`.
 */
export function urlDoc(chiave: ChiaveDoc): string {
  const voce = DOCS[chiave];
  const ancora = voce.ancora ? `#${voce.ancora}` : "";
  return `${SITO_BASE}/utente/${voce.file}${ancora}`;
}

/** Titolo breve della pagina, per tooltip/aria-label dei punti "?". */
export function titoloDoc(chiave: ChiaveDoc): string {
  return DOCS[chiave].titolo;
}
