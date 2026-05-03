import { describe, it, expect } from "vitest";
import { estraiSegnaposti, compila, contaCompilati } from "./template";

describe("estraiSegnaposti", () => {
  it("trova segnaposti semplici", () => {
    const r = estraiSegnaposti("Ciao {{nome}}, benvenuto!");
    expect(r).toEqual([{ nome: "nome", indice: 5 }]);
  });

  it("trova segnaposti multipli", () => {
    const r = estraiSegnaposti("{{saluto}} {{nome}}, sei {{ruolo}}");
    expect(r).toHaveLength(3);
    expect(r.map((s) => s.nome)).toEqual(["saluto", "nome", "ruolo"]);
  });

  it("deduplica segnaposti ripetuti", () => {
    const r = estraiSegnaposti("{{nome}} e ancora {{nome}}");
    expect(r).toHaveLength(1);
  });

  it("gestisce spazi dentro le graffe", () => {
    const r = estraiSegnaposti("{{ nome }}");
    expect(r).toHaveLength(1);
    expect(r[0].nome).toBe("nome");
  });

  it("restituisce vuoto senza segnaposti", () => {
    expect(estraiSegnaposti("Nessun segnaposto")).toEqual([]);
  });

  it("restituisce vuoto per stringa vuota", () => {
    expect(estraiSegnaposti("")).toEqual([]);
  });

  it("gestisce underscore nei nomi", () => {
    const r = estraiSegnaposti("{{nome_completo}}");
    expect(r[0].nome).toBe("nome_completo");
  });

  it("ignora graffe singole o triple", () => {
    expect(estraiSegnaposti("{nome}")).toEqual([]);
    expect(estraiSegnaposti("{{{nome}}}")).toHaveLength(1);
  });
});

describe("compila", () => {
  it("sostituisce segnaposti con valori", () => {
    expect(compila("Ciao {{nome}}!", { nome: "Roberto" })).toBe(
      "Ciao Roberto!",
    );
  });

  it("mantiene segnaposti senza valore", () => {
    expect(compila("{{saluto}} {{nome}}", { saluto: "Ciao" })).toBe(
      "Ciao {{nome}}",
    );
  });

  it("trimma i valori", () => {
    expect(compila("{{nome}}", { nome: "  Roberto  " })).toBe("Roberto");
  });

  it("mantiene segnaposti per valori vuoti", () => {
    expect(compila("{{nome}}", { nome: "" })).toBe("{{nome}}");
  });

  it("mantiene segnaposti per valori solo spazi", () => {
    expect(compila("{{nome}}", { nome: "   " })).toBe("{{nome}}");
  });

  it("sostituisce tutte le occorrenze", () => {
    expect(compila("{{x}} e {{x}}", { x: "ok" })).toBe("ok e ok");
  });

  it("non modifica testo senza segnaposti", () => {
    expect(compila("Testo normale", {})).toBe("Testo normale");
  });

  it("gestisce body multilinea", () => {
    const body = "Riga 1: {{a}}\nRiga 2: {{b}}";
    expect(compila(body, { a: "X", b: "Y" })).toBe("Riga 1: X\nRiga 2: Y");
  });
});

describe("contaCompilati", () => {
  it("conta segnaposti compilati", () => {
    const s = estraiSegnaposti("{{a}} {{b}} {{c}}");
    expect(contaCompilati(s, { a: "ok", b: "", c: "ok" })).toBe(2);
  });

  it("zero per nessun valore", () => {
    const s = estraiSegnaposti("{{a}} {{b}}");
    expect(contaCompilati(s, {})).toBe(0);
  });

  it("ignora valori solo spazi", () => {
    const s = estraiSegnaposti("{{a}}");
    expect(contaCompilati(s, { a: "  " })).toBe(0);
  });

  it("tutti compilati", () => {
    const s = estraiSegnaposti("{{a}} {{b}}");
    expect(contaCompilati(s, { a: "x", b: "y" })).toBe(2);
  });

  it("zero per array vuoto", () => {
    expect(contaCompilati([], { a: "x" })).toBe(0);
  });
});
