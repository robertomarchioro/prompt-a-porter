import { describe, it, expect } from "vitest";
import { estraiImports } from "./estrai-imports";

describe("estraiImports", () => {
  it("ritorna array vuoto su body senza import", () => {
    expect(estraiImports("Hello world")).toEqual([]);
    expect(estraiImports("")).toEqual([]);
  });

  it("trova un singolo import", () => {
    expect(estraiImports('Pre {{import "marketing/intro"}} post')).toEqual([
      "marketing/intro",
    ]);
  });

  it("trova multipli import e deduplica preservando ordine", () => {
    const body =
      '{{import "a/intro"}} {{import "b/footer"}} text {{import "a/intro"}} {{import "c/cta"}}';
    expect(estraiImports(body)).toEqual(["a/intro", "b/footer", "c/cta"]);
  });

  it("supporta whitespace tra `import` e path", () => {
    const body = '{{ import   "spaces"  }} {{import\t"tabs"}}';
    expect(estraiImports(body)).toEqual(["spaces", "tabs"]);
  });

  it("ignora segnaposti normali {{var}} (no parola 'import')", () => {
    const body = '{{nome}} {{import "x/y"}} {{cognome}} {{import "z"}}';
    expect(estraiImports(body)).toEqual(["x/y", "z"]);
  });

  it("non lancia su body con sole graffe", () => {
    expect(() => estraiImports("{{ }} { } {{import }}")).not.toThrow();
    expect(estraiImports("{{ }} { } {{import }}")).toEqual([]);
  });

  // Parametrized imports (M4 syntax — issue #304)
  it("trova import con modificatore `with k=v`", () => {
    expect(
      estraiImports('{{import "marketing/email" with tone=formal}}'),
    ).toEqual(["marketing/email"]);
  });

  it("trova import con modificatore `version=N`", () => {
    expect(estraiImports('pre {{import "intro" version=2}} post')).toEqual([
      "intro",
    ]);
  });

  it("deduplica import parametrizzati e semplici dello stesso path", () => {
    const body =
      '{{import "a"}} {{import "a" with k=v}} {{import "b" version=1}}';
    expect(estraiImports(body)).toEqual(["a", "b"]);
  });

  it("nessuna regressione: plain `{{import \"x\"}}` ancora riconosciuto", () => {
    expect(estraiImports('{{import "plain"}}')).toEqual(["plain"]);
  });
});
