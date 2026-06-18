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

  it("riconosce {{global autore}} come globale", () => {
    const matches = _matchSegnaposti("Scritto da {{global autore}}.");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(true);
    expect(matches[0].from).toBe(11);
    expect(matches[0].to).toBe(28);
  });

  it("{{globalautore}} senza spazio NON è globale (è segnaposto normale)", () => {
    const matches = _matchSegnaposti("{{globalautore}}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });

  it("riconosce sia segnaposti semplici sia globali nello stesso testo", () => {
    const testo = "{{nome}} scrive {{global data}} per {{titolo}}";
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

  it("tollera whitespace extra in {{global  nome}}", () => {
    const matches = _matchSegnaposti("{{ global  autore }}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(true);
  });

  it("non matcha {{import \"path\"}} (lasciato a import-tokens)", () => {
    // import-tokens gestisce {{import "..."}}: non è un \w+ semplice
    const matches = _matchSegnaposti('{{import "foo/bar"}}');
    expect(matches).toHaveLength(0);
  });

  it("from e to puntano ai caratteri corretti nel testo", () => {
    const testo = "aaa {{global xyz}} bbb";
    const matches = _matchSegnaposti(testo);
    expect(matches).toHaveLength(1);
    expect(testo.slice(matches[0].from, matches[0].to)).toBe(
      "{{global xyz}}",
    );
  });

  it("caso negativo: testo senza doppie graffe", () => {
    expect(_matchSegnaposti("{nome} e [altro]")).toEqual([]);
  });

  // HIGH-1/3: pinning del comportamento di {{global}} (parola chiave sola,
  // senza nome): il primo ramo della regex non può fare match (manca \s+\w+),
  // quindi ricade nel ramo \w+ e viene trattato come segnaposto normale.
  it("{{global}} senza nome è trattato come segnaposto normale (globale=false)", () => {
    const matches = _matchSegnaposti("{{global}}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });

  it("segnaposto semplice ha sempre globale=false (verifica booleano esplicito)", () => {
    const matches = _matchSegnaposti("{{titolo}}");
    expect(matches).toHaveLength(1);
    expect(matches[0].globale).toBe(false);
  });
});
