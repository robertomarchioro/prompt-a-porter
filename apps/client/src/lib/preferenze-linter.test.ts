// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import {
  CATEGORIE_LINTER,
  ETICHETTE,
  DESCRIZIONI,
  leggiCategorieDisabilitate,
  salvaCategorieDisabilitate,
  categoriaAttiva,
  toggleCategoria,
  leggiRegoleDisabilitate,
  salvaRegoleDisabilitate,
  toggleRegola,
} from "./preferenze-linter";

const KEY = "pap.linter.categorie_disabilitate";
const KEY_REGOLE = "pap.linter.regole_disabilitate";

describe("preferenze-linter", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("CATEGORIE_LINTER ha 5 elementi e match con ETICHETTE/DESCRIZIONI", () => {
    expect(CATEGORIE_LINTER).toHaveLength(5);
    for (const c of CATEGORIE_LINTER) {
      expect(ETICHETTE[c]).toBeTruthy();
      expect(DESCRIZIONI[c]).toBeTruthy();
    }
  });

  it("leggiCategorieDisabilitate ritorna [] se localStorage vuoto", () => {
    expect(leggiCategorieDisabilitate()).toEqual([]);
  });

  it("leggiCategorieDisabilitate filtra valori non validi", () => {
    localStorage.setItem(KEY, JSON.stringify(["LEN", "FAKE", "PH", 42]));
    expect(leggiCategorieDisabilitate()).toEqual(["LEN", "PH"]);
  });

  it("leggiCategorieDisabilitate ritorna [] su JSON malformato", () => {
    localStorage.setItem(KEY, "not-json {{");
    expect(leggiCategorieDisabilitate()).toEqual([]);
  });

  it("leggiCategorieDisabilitate ritorna [] se valore non e array", () => {
    localStorage.setItem(KEY, JSON.stringify({ a: 1 }));
    expect(leggiCategorieDisabilitate()).toEqual([]);
  });

  it("salvaCategorieDisabilitate scrive solo valori validi", () => {
    salvaCategorieDisabilitate(["LEN", "PH", "X" as unknown as "LEN"]);
    const raw = localStorage.getItem(KEY);
    expect(JSON.parse(raw!)).toEqual(["LEN", "PH"]);
  });

  it("categoriaAttiva true quando non disabilitata", () => {
    expect(categoriaAttiva("LEN", [])).toBe(true);
    expect(categoriaAttiva("LEN", ["PH"])).toBe(true);
  });

  it("categoriaAttiva false quando disabilitata", () => {
    expect(categoriaAttiva("PH", ["PH", "PII"])).toBe(false);
  });

  it("toggleCategoria aggiunge se assente", () => {
    expect(toggleCategoria("LEN", [])).toEqual(["LEN"]);
    expect(toggleCategoria("PH", ["LEN"])).toEqual(["LEN", "PH"]);
  });

  it("toggleCategoria rimuove se presente e non muta input", () => {
    const input = ["LEN", "PH"] as const;
    const out = toggleCategoria("LEN", input);
    expect(out).toEqual(["PH"]);
    expect(input).toEqual(["LEN", "PH"]);
  });

  it("round trip salva->leggi", () => {
    salvaCategorieDisabilitate(["STY", "IMP"]);
    expect(leggiCategorieDisabilitate()).toEqual(["STY", "IMP"]);
  });
});

describe("preferenze-linter — per-regola (Fase 1)", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("leggiRegoleDisabilitate ritorna [] se tutto vuoto", () => {
    expect(leggiRegoleDisabilitate()).toEqual([]);
  });

  it("migra one-shot dalla vecchia key categorie se la nuova è assente", () => {
    localStorage.setItem(KEY, JSON.stringify(["PII", "IMP"]));
    expect(leggiRegoleDisabilitate()).toEqual(["PII", "IMP"]);
    // La migrazione scrive anche la nuova key.
    expect(JSON.parse(localStorage.getItem(KEY_REGOLE)!)).toEqual(["PII", "IMP"]);
  });

  it("la nuova key ha precedenza sulla vecchia (niente migrazione)", () => {
    localStorage.setItem(KEY, JSON.stringify(["PII"]));
    localStorage.setItem(KEY_REGOLE, JSON.stringify(["LEN001"]));
    expect(leggiRegoleDisabilitate()).toEqual(["LEN001"]);
  });

  it("accetta code completi e prefissi, filtra non-stringhe", () => {
    localStorage.setItem(KEY_REGOLE, JSON.stringify(["PII001", "LEN", 7, ""]));
    expect(leggiRegoleDisabilitate()).toEqual(["PII001", "LEN"]);
  });

  it("leggiRegoleDisabilitate ritorna [] su JSON malformato", () => {
    localStorage.setItem(KEY_REGOLE, "{{ not json");
    expect(leggiRegoleDisabilitate()).toEqual([]);
  });

  it("salvaRegoleDisabilitate scrive solo stringhe non vuote", () => {
    salvaRegoleDisabilitate(["PII001", "", "STY"]);
    expect(JSON.parse(localStorage.getItem(KEY_REGOLE)!)).toEqual([
      "PII001",
      "STY",
    ]);
  });

  it("toggleRegola aggiunge/rimuove un code e non muta l'input", () => {
    expect(toggleRegola("PII001", [])).toEqual(["PII001"]);
    const input = ["PII001", "LEN"] as const;
    const out = toggleRegola("PII001", input);
    expect(out).toEqual(["LEN"]);
    expect(input).toEqual(["PII001", "LEN"]);
  });
});
