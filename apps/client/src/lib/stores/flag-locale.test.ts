// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { caricaFlag, salvaFlag } from "./flag-locale";

/**
 * Flag booleani di UI persistiti in localStorage (#747): stato
 * compresso dell'intestazione del dettaglio e del pannello metadata.
 */

const CHIAVE = "pap.test.flag";

beforeEach(() => {
  localStorage.clear();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("caricaFlag", () => {
  it("ritorna il predefinito quando la chiave manca", () => {
    expect(caricaFlag(CHIAVE)).toBe(false);
    expect(caricaFlag(CHIAVE, true)).toBe(true);
  });

  it("legge '1' come true e '0' come false", () => {
    localStorage.setItem(CHIAVE, "1");
    expect(caricaFlag(CHIAVE)).toBe(true);
    localStorage.setItem(CHIAVE, "0");
    expect(caricaFlag(CHIAVE, true)).toBe(false);
  });

  it("ritorna il predefinito su un valore non riconosciuto", () => {
    localStorage.setItem(CHIAVE, "forse");
    expect(caricaFlag(CHIAVE)).toBe(false);
    expect(caricaFlag(CHIAVE, true)).toBe(true);
  });

  it("ritorna il predefinito se localStorage lancia", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("storage disabilitato");
    });
    expect(caricaFlag(CHIAVE, true)).toBe(true);
  });
});

describe("salvaFlag", () => {
  it("roundtrip true e false", () => {
    salvaFlag(CHIAVE, true);
    expect(localStorage.getItem(CHIAVE)).toBe("1");
    expect(caricaFlag(CHIAVE)).toBe(true);
    salvaFlag(CHIAVE, false);
    expect(localStorage.getItem(CHIAVE)).toBe("0");
    expect(caricaFlag(CHIAVE, true)).toBe(false);
  });

  it("non lancia se localStorage è pieno o disabilitato", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceededError");
    });
    expect(() => salvaFlag(CHIAVE, true)).not.toThrow();
  });

  it("chiavi diverse sono indipendenti", () => {
    salvaFlag(CHIAVE, true);
    expect(caricaFlag("pap.test.altro")).toBe(false);
  });
});
