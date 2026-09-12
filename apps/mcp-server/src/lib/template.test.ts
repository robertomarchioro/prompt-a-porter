import { describe, it, expect } from "vitest";
import { compila, estraiSegnaposti } from "./template.js";

describe("compila", () => {
  it("sostituisce un singolo segnaposto", () => {
    expect(compila("Ciao {{nome}}!", { nome: "Mario" })).toBe("Ciao Mario!");
  });

  it("sostituisce piu segnaposti distinti", () => {
    expect(
      compila("{{saluto}} {{nome}}, ho {{eta}} anni", {
        saluto: "Salve",
        nome: "Luca",
        eta: "30",
      }),
    ).toBe("Salve Luca, ho 30 anni");
  });

  it("ripete la stessa sostituzione per occorrenze multiple", () => {
    expect(compila("{{x}} e {{x}}", { x: "ok" })).toBe("ok e ok");
  });

  it("trimma il valore prima della sostituzione", () => {
    expect(compila("[{{a}}]", { a: "   pulito   " })).toBe("[pulito]");
  });

  it("lascia inalterato il segnaposto se valore mancante", () => {
    expect(compila("Ciao {{nome}}", {})).toBe("Ciao {{nome}}");
  });

  it("lascia inalterato il segnaposto se valore vuoto o whitespace", () => {
    expect(compila("[{{a}}]", { a: "" })).toBe("[{{a}}]");
    expect(compila("[{{a}}]", { a: "   " })).toBe("[{{a}}]");
  });

  it("tollera whitespace interno {{ nome }}", () => {
    expect(compila("Ciao {{   nome   }}", { nome: "Anna" })).toBe("Ciao Anna");
  });

  it("non sostituisce segnaposti con identificatore invalido (spazi/punteggiatura)", () => {
    expect(compila("{{con punteggio!}}", { "con punteggio!": "x" })).toBe(
      "{{con punteggio!}}",
    );
  });
});

describe("estraiSegnaposti", () => {
  it("ritorna array vuoto su body senza segnaposti", () => {
    expect(estraiSegnaposti("solo testo")).toEqual([]);
  });

  it("estrae un singolo nome", () => {
    expect(estraiSegnaposti("Ciao {{nome}}")).toEqual(["nome"]);
  });

  it("dedup mantenendo l'ordine di prima apparizione", () => {
    expect(estraiSegnaposti("{{a}} {{b}} {{a}} {{c}} {{b}}")).toEqual([
      "a",
      "b",
      "c",
    ]);
  });

  it("ignora segnaposti malformati", () => {
    expect(estraiSegnaposti("{{ }} {{!}} {{ok}}")).toEqual(["ok"]);
  });

  it("e idempotente su chiamate successive (reset lastIndex)", () => {
    const body = "{{x}} {{y}}";
    expect(estraiSegnaposti(body)).toEqual(["x", "y"]);
    expect(estraiSegnaposti(body)).toEqual(["x", "y"]);
  });
});

// #643: conformità con la fixture condivisa (client, Rust, Go CLI).
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

interface CasoConformita {
  nome: string;
  body: string;
  atteso: string;
  segnaposti: string[];
}

const FIXTURE = fileURLToPath(
  new URL(
    "../../../../packages/shared-schema/fixtures/commenti-conformita.json",
    import.meta.url,
  ),
);
const { casi } = JSON.parse(readFileSync(FIXTURE, "utf8")) as {
  casi: CasoConformita[];
};

describe("commenti {{!-- --}} — conformità (#643)", () => {
  for (const caso of casi) {
    it(caso.nome, () => {
      // Senza valori, compila = rimozione dei soli commenti.
      expect(compila(caso.body, {})).toBe(caso.atteso);
      expect(estraiSegnaposti(caso.body)).toEqual(caso.segnaposti);
    });
  }

  it("un segnaposto citato solo nel commento non viene compilato", () => {
    expect(compila("{{!-- {{x}} --}}\nCiao {{nome}}", { x: "NO", nome: "Anna" }))
      .toBe("Ciao Anna");
  });
});
