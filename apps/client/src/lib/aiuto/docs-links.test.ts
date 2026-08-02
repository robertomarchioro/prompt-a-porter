import { describe, it, expect } from "vitest";
import { DOCS, SITO_BASE, urlDoc, titoloDoc, type ChiaveDoc } from "./docs-links";

const CHIAVI = Object.keys(DOCS) as ChiaveDoc[];

describe("docs-links", () => {
  // Fix #554/#555/#557: SITO_BASE punta ora al sito pubblico
  // www.promptaporter.it (prima: placeholder mai registrato + fallback
  // github.com/.../blob/main). Questo test asserisce esplicitamente il
  // nuovo pattern per evitare regressioni verso GitHub o verso il vecchio
  // dominio.
  it("ogni chiave risolve a un URL del sito pubblico sotto /utente/", () => {
    for (const k of CHIAVI) {
      const url = urlDoc(k);
      expect(url).toMatch(
        /^https:\/\/www\.promptaporter\.it\/utente\/[a-z0-9-]+(#[a-z0-9-]+)?$/,
      );
    }
  });

  it("usa SITO_BASE come prefisso di ogni URL risolto", () => {
    for (const k of CHIAVI) {
      expect(urlDoc(k).startsWith(`${SITO_BASE}/utente/`)).toBe(true);
    }
  });

  it("non punta più a github.com/.../blob/main (regressione #554/#555/#557)", () => {
    for (const k of CHIAVI) {
      expect(urlDoc(k)).not.toContain("github.com");
      expect(urlDoc(k)).not.toContain("blob/main");
      expect(urlDoc(k)).not.toContain(".md");
    }
  });

  it("include l'ancora quando la voce la definisce", () => {
    // segnaposti-globali punta a glossario-sintassi#segnaposti-globali
    expect(urlDoc("segnaposti-globali")).toBe(
      "https://www.promptaporter.it/utente/glossario-sintassi#segnaposti-globali",
    );
  });

  it("non aggiunge l'ancora quando assente", () => {
    expect(urlDoc("getting-started")).toBe(
      "https://www.promptaporter.it/utente/getting-started",
    );
    expect(urlDoc("getting-started")).not.toContain("#");
  });

  it("costruisce correttamente file e ancora insieme per una chiave arbitraria", () => {
    // export-json non ha ancora: verifica che urlDoc non ne inventi una e
    // che il file risolto sia esattamente quello mappato in DOCS.
    const voce = DOCS["export-json"];
    expect(voce.ancora).toBeUndefined();
    expect(urlDoc("export-json")).toBe(`${SITO_BASE}/utente/${voce.file}`);
  });

  it("ogni voce ha file (slug valido) e titolo non vuoto", () => {
    for (const k of CHIAVI) {
      expect(DOCS[k].file).toMatch(/^[a-z0-9-]+$/);
      expect(titoloDoc(k).trim().length).toBeGreaterThan(0);
    }
  });

  it("titoloDoc restituisce l'etichetta della voce", () => {
    expect(titoloDoc("prompt-componibili")).toBe("Import componibili");
  });
});
