import { describe, it, expect } from "vitest";
import { DOCS, urlDoc, titoloDoc, type ChiaveDoc } from "./docs-links";

const CHIAVI = Object.keys(DOCS) as ChiaveDoc[];

describe("docs-links", () => {
  it("ogni chiave risolve a un URL GitHub assoluto e https", () => {
    for (const k of CHIAVI) {
      const url = urlDoc(k);
      expect(url).toMatch(
        /^https:\/\/github\.com\/[^/]+\/[^/]+\/blob\/main\/docs\/utente\/[a-z0-9-]+\.md(#[a-z0-9-]+)?$/,
      );
    }
  });

  it("include l'ancora quando la voce la definisce", () => {
    // segnaposti-globali punta a glossario-sintassi#segnaposti-globali
    expect(urlDoc("segnaposti-globali")).toBe(
      "https://github.com/robertomarchioro/prompt-a-porter/blob/main/docs/utente/glossario-sintassi.md#segnaposti-globali",
    );
  });

  it("non aggiunge l'ancora quando assente", () => {
    expect(urlDoc("getting-started")).toBe(
      "https://github.com/robertomarchioro/prompt-a-porter/blob/main/docs/utente/getting-started.md",
    );
    expect(urlDoc("getting-started")).not.toContain("#");
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
