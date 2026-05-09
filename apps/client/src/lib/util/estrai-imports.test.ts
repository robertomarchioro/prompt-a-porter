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
});
