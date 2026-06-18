import { describe, it, expect } from "vitest";
import { estraiSegnaposti, compila, contaCompilati } from "./template";

describe("estraiSegnaposti", () => {
  it("trova segnaposti semplici", () => {
    const r = estraiSegnaposti("Ciao {{nome}}, benvenuto!");
    expect(r).toEqual([{ nome: "nome", indice: 5, globale: false }]);
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

  // Issue #159: segnaposti globali
  it("riconosce segnaposto globale con prefisso 'global '", () => {
    const r = estraiSegnaposti("Autore: {{global autore}}");
    expect(r).toHaveLength(1);
    expect(r[0]).toMatchObject({ nome: "autore", globale: true });
  });

  it("globali e normali coesistono", () => {
    const r = estraiSegnaposti("{{global autore}} dice ciao a {{nome}}");
    expect(r).toHaveLength(2);
    expect(r[0]).toMatchObject({ nome: "autore", globale: true });
    expect(r[1]).toMatchObject({ nome: "nome", globale: false });
  });

  it("globale con whitespace extra", () => {
    const r = estraiSegnaposti("{{  global  autore  }}");
    expect(r).toHaveLength(1);
    expect(r[0]).toMatchObject({ nome: "autore", globale: true });
  });

  it("'globalautore' senza spazio è normale (nome=globalautore)", () => {
    const r = estraiSegnaposti("{{globalautore}}");
    expect(r).toHaveLength(1);
    expect(r[0]).toMatchObject({ nome: "globalautore", globale: false });
  });

  it("dedup separato per globali e normali con stesso nome", () => {
    const r = estraiSegnaposti("{{nome}} vs {{global nome}} vs {{nome}}");
    expect(r).toHaveLength(2);
    expect(r.find((s) => s.globale)).toBeDefined();
    expect(r.find((s) => !s.globale)).toBeDefined();
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

  // Issue #159: compila con valoriGlobali
  it("sostituisce segnaposto globale con valoriGlobali", () => {
    expect(
      compila("Firmato {{global autore}}", {}, { autore: "Roberto" }),
    ).toBe("Firmato Roberto");
  });

  it("mantiene segnaposto globale senza valore globale", () => {
    expect(compila("{{global autore}}", {})).toBe("{{global autore}}");
  });

  it("non confonde resolver normale e globale (stesso nome)", () => {
    const body = "{{nome}} firmato {{global nome}}";
    expect(compila(body, { nome: "Mario" }, { nome: "Roberto" })).toBe(
      "Mario firmato Roberto",
    );
  });

  it("globale preserva spazio normalizzato nel placeholder non risolto", () => {
    expect(compila("{{global  autore  }}", {})).toBe("{{global autore}}");
  });

  it("valori normali non leakano in globali", () => {
    expect(compila("{{global autore}}", { autore: "Mario" })).toBe(
      "{{global autore}}",
    );
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

  // Issue #159
  it("conta separatamente normali e globali", () => {
    const s = estraiSegnaposti("{{a}} {{global b}} {{global c}}");
    expect(contaCompilati(s, { a: "ok" }, { b: "ok" })).toBe(2);
  });

  it("globale non risolto da valori normali", () => {
    const s = estraiSegnaposti("{{global a}}");
    expect(contaCompilati(s, { a: "ok" })).toBe(0);
  });

  it("normale non risolto da valori globali", () => {
    const s = estraiSegnaposti("{{a}}");
    expect(contaCompilati(s, {}, { a: "ok" })).toBe(0);
  });
});
