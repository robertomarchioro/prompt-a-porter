// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import {
  caricaStato,
  salvaStato,
  DEFAULTS,
  COL_SIDEBAR_MIN,
  COL_SIDEBAR_MAX,
  COL_LIST_MIN,
  COL_LIST_MAX,
} from "./shell-layout";

const KEY = "pap.shell.layout";

describe("shell-layout", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("caricaStato ritorna DEFAULTS quando localStorage vuoto", () => {
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("caricaStato ritorna DEFAULTS su JSON malformato", () => {
    localStorage.setItem(KEY, "{{not json");
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("caricaStato clampa valori sotto il minimo", () => {
    localStorage.setItem(
      KEY,
      JSON.stringify({ colSidebar: 10, colList: 50 }),
    );
    const s = caricaStato();
    expect(s.colSidebar).toBe(COL_SIDEBAR_MIN);
    expect(s.colList).toBe(COL_LIST_MIN);
  });

  it("caricaStato clampa valori sopra il massimo", () => {
    localStorage.setItem(
      KEY,
      JSON.stringify({ colSidebar: 9999, colList: 9999 }),
    );
    const s = caricaStato();
    expect(s.colSidebar).toBe(COL_SIDEBAR_MAX);
    expect(s.colList).toBe(COL_LIST_MAX);
  });

  it("caricaStato fallback a DEFAULTS per valori non numerici", () => {
    localStorage.setItem(
      KEY,
      JSON.stringify({ colSidebar: "abc", colList: null }),
    );
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("caricaStato fallback a DEFAULTS per NaN/Infinity (JSON.stringify -> null)", () => {
    localStorage.setItem(
      KEY,
      JSON.stringify({ colSidebar: Number.NaN, colList: 1e400 }),
    );
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("salvaStato persiste e arrotonda al pixel", () => {
    salvaStato({ colSidebar: 250.7, colList: 330.4 });
    const raw = localStorage.getItem(KEY);
    expect(JSON.parse(raw!)).toEqual({ colSidebar: 251, colList: 330 });
  });

  it("salvaStato clampa i valori fuori range prima di scrivere", () => {
    salvaStato({ colSidebar: 5, colList: 9999 });
    expect(JSON.parse(localStorage.getItem(KEY)!)).toEqual({
      colSidebar: COL_SIDEBAR_MIN,
      colList: COL_LIST_MAX,
    });
  });

  it("round trip salva->carica preserva valori validi", () => {
    salvaStato({ colSidebar: 300, colList: 400 });
    expect(caricaStato()).toEqual({ colSidebar: 300, colList: 400 });
  });
});
