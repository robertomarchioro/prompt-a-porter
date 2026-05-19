import { describe, it, expect } from "vitest";
import { sanitizzaFts } from "./fts.js";

describe("sanitizzaFts", () => {
  it("aggiunge wildcard ai token alfanumerici separati da spazi", () => {
    expect(sanitizzaFts("hello world")).toBe("hello* world*");
  });

  it("rimuove caratteri non alfanumerici/underscore mantenendo il resto", () => {
    expect(sanitizzaFts("foo-bar baz!")).toBe("foobar* baz*");
  });

  it("conserva underscore", () => {
    expect(sanitizzaFts("snake_case PascalCase")).toBe(
      "snake_case* PascalCase*",
    );
  });

  it("ritorna stringa vuota se tutti i token sono fatti solo di simboli", () => {
    expect(sanitizzaFts("!!! ???")).toBe("");
  });

  it("ritorna stringa vuota su input solo whitespace", () => {
    expect(sanitizzaFts("   \t  \n  ")).toBe("");
  });

  it("ritorna stringa vuota su stringa vuota", () => {
    expect(sanitizzaFts("")).toBe("");
  });

  it("supporta caratteri Unicode (accentati italiani)", () => {
    expect(sanitizzaFts("città però")).toBe("città* però*");
  });

  it("collassa whitespace multipli (split su /\\s+/)", () => {
    expect(sanitizzaFts("  foo   bar  ")).toBe("foo* bar*");
  });
});
