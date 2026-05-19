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
} from "./preferenze-linter";

const KEY = "pap.linter.categorie_disabilitate";

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
