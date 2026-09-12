import { describe, it, expect } from "vitest";
import { intervalliCommenti, rimuoviCommenti } from "./commenti";
import { compila, estraiSegnaposti } from "./template";
// #643: tabella di conformità condivisa con MCP server, Rust e Go CLI.
// Un caso nuovo si aggiunge nella fixture, non qui.
import fixture from "../../../../packages/shared-schema/fixtures/commenti-conformita.json";

interface CasoConformita {
  nome: string;
  body: string;
  atteso: string;
  segnaposti: string[];
}

const casi: CasoConformita[] = fixture.casi;

describe("rimuoviCommenti — conformità cross-linguaggio (#643)", () => {
  it("la fixture non è vuota", () => {
    expect(casi.length).toBeGreaterThan(10);
  });

  for (const caso of casi) {
    it(caso.nome, () => {
      expect(rimuoviCommenti(caso.body)).toBe(caso.atteso);
      expect(
        estraiSegnaposti(caso.body)
          .filter((s) => !s.globale)
          .map((s) => s.nome),
      ).toEqual(caso.segnaposti);
    });
  }
});

describe("rimuoviCommenti", () => {
  it("è idempotente", () => {
    for (const caso of casi) {
      expect(rimuoviCommenti(caso.atteso)).toBe(caso.atteso);
    }
  });
});

describe("intervalliCommenti", () => {
  it("nessun commento → nessun intervallo", () => {
    expect(intervalliCommenti("Ciao {{nome}}")).toEqual([]);
  });

  it("ritorna [from, to) del token intero", () => {
    const body = "a {{!-- x --}} b";
    expect(intervalliCommenti(body)).toEqual([{ from: 2, to: 14 }]);
    expect(body.slice(2, 14)).toBe("{{!-- x --}}");
  });

  it("intervalli multipli in ordine", () => {
    const r = intervalliCommenti("{{!-- a --}}\n{{!-- b --}}");
    expect(r).toEqual([
      { from: 0, to: 12 },
      { from: 13, to: 25 },
    ]);
  });

  it("commento non chiuso → nessun intervallo", () => {
    expect(intervalliCommenti("{{!-- aperto")).toEqual([]);
  });
});

describe("compila con commenti", () => {
  it("il commento non arriva nel testo compilato", () => {
    expect(
      compila("{{!-- tono B --}}\nCiao {{nome}}", { nome: "Anna" }),
    ).toBe("Ciao Anna");
  });

  it("un segnaposto citato solo nel commento non viene compilato né lasciato", () => {
    expect(compila("{{!-- {{x}} --}}\nok", { x: "NO" })).toBe("ok");
  });

  it("un segnaposto globale nel commento non viene compilato", () => {
    expect(compila("{{!-- {{global a}} --}}\nok", {}, { a: "NO" })).toBe(
      "ok",
    );
  });

  it("commento inline dopo un segnaposto", () => {
    expect(compila("Ciao {{nome}} {{!-- nota --}}", { nome: "Anna" })).toBe(
      "Ciao Anna ",
    );
  });
});
