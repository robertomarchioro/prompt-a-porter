import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { intervalliCommenti, rimuoviCommenti } from "./commenti.js";

interface CasoConformita {
  nome: string;
  body: string;
  atteso: string;
}

// #643: la fixture è la fonte di verità condivisa con client, Rust e Go.
const FIXTURE = fileURLToPath(
  new URL("../fixtures/commenti-conformita.json", import.meta.url),
);
const { casi } = JSON.parse(readFileSync(FIXTURE, "utf8")) as {
  casi: CasoConformita[];
};

describe("rimuoviCommenti — conformità (#643)", () => {
  it("la fixture non è vuota", () => {
    expect(casi.length).toBeGreaterThan(10);
  });

  for (const caso of casi) {
    it(caso.nome, () => {
      expect(rimuoviCommenti(caso.body)).toBe(caso.atteso);
    });
  }

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
    expect(intervalliCommenti("a {{!-- x --}} b")).toEqual([
      { from: 2, to: 14 },
    ]);
  });

  it("commento non chiuso → nessun intervallo", () => {
    expect(intervalliCommenti("{{!-- aperto")).toEqual([]);
  });
});
