import { describe, expect, it } from "vitest";
import { _matchSegnaposti } from "./placeholder-highlight";

describe("placeholder-highlight / _matchSegnaposti", () => {
  it("ritorna array vuoto se nessun segnaposto nel testo", () => {
    expect(_matchSegnaposti("Lorem ipsum dolor sit amet.")).toEqual([]);
  });

  it("riconosce un segnaposto semplice {{nome}}", () => {
    const matches = _matchSegnaposti("Ciao {{nome}}!");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
    expect(matches[0].from).toBe(5);
    expect(matches[0].to).toBe(13);
  });

  it("riconosce {{globale autore}} come globale", () => {
    const matches = _matchSegnaposti("Scritto da {{globale autore}}.");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(true);
    expect(matches[0].from).toBe(11);
    expect(matches[0].to).toBe(29);
  });

  it("{{globaleautore}} senza spazio NON è globale (è segnaposto normale)", () => {
    const matches = _matchSegnaposti("{{globaleautore}}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });

  it("riconosce sia segnaposti semplici sia globali nello stesso testo", () => {
    const testo = "{{nome}} scrive {{globale data}} per {{titolo}}";
    const matches = _matchSegnaposti(testo);
    expect(matches).toHaveLength(3);
    expect(matches[0].globale).toBe(false);
    expect(matches[1].globale).toBe(true);
    expect(matches[2].globale).toBe(false);
  });

  it("tollera whitespace extra dentro le graffe", () => {
    const matches = _matchSegnaposti("{{  nome  }}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });

  it("tollera whitespace extra in {{globale  nome}}", () => {
    const matches = _matchSegnaposti("{{ globale  autore }}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(true);
  });

  it("non matcha {{import \"path\"}} (lasciato a import-tokens)", () => {
    // import-tokens gestisce {{import "..."}}: non è un \w+ semplice
    const matches = _matchSegnaposti('{{import "foo/bar"}}');
    expect(matches).toHaveLength(0);
  });

  it("from e to puntano ai caratteri corretti nel testo", () => {
    const testo = "aaa {{globale xyz}} bbb";
    const matches = _matchSegnaposti(testo);
    expect(matches).toHaveLength(1);
    expect(testo.slice(matches[0].from, matches[0].to)).toBe(
      "{{globale xyz}}",
    );
  });

  it("caso negativo: testo senza doppie graffe", () => {
    expect(_matchSegnaposti("{nome} e [altro]")).toEqual([]);
  });

  // HIGH-1/3: pinning del comportamento di {{globale}} (parola chiave sola,
  // senza nome): il primo ramo della regex non può fare match (manca \s+\w+),
  // quindi ricade nel ramo \w+ e viene trattato come segnaposto normale.
  it("{{globale}} senza nome è trattato come segnaposto normale (globale=false)", () => {
    const matches = _matchSegnaposti("{{globale}}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });

  it("segnaposto semplice ha sempre globale=false (verifica booleano esplicito)", () => {
    const matches = _matchSegnaposti("{{titolo}}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });
});
