import { describe, it, expect } from "vitest";
import { argsTroppoGrandi, clampLimit, LIMIT_MAX, LIMIT_MIN, MAX_ARGS_JSON_LENGTH } from "./limits.js";

describe("clampLimit", () => {
  it("lascia invariato un valore già nell'intervallo", () => {
    expect(clampLimit(5)).toBe(5);
  });

  it("blocca a LIMIT_MAX un valore troppo alto (999 -> 50)", () => {
    expect(clampLimit(999)).toBe(LIMIT_MAX);
  });

  it("blocca a LIMIT_MIN un valore pari a zero (0 -> 1)", () => {
    expect(clampLimit(0)).toBe(LIMIT_MIN);
  });

  it("blocca a LIMIT_MIN un valore negativo", () => {
    expect(clampLimit(-100)).toBe(LIMIT_MIN);
  });

  it("accetta i due estremi dell'intervallo senza modificarli", () => {
    expect(clampLimit(LIMIT_MIN)).toBe(LIMIT_MIN);
    expect(clampLimit(LIMIT_MAX)).toBe(LIMIT_MAX);
  });
});

describe("argsTroppoGrandi", () => {
  it("ritorna false per argomenti piccoli", () => {
    expect(argsTroppoGrandi({ query: "hello" })).toBe(false);
  });

  it("ritorna false per argomenti assenti (undefined)", () => {
    expect(argsTroppoGrandi(undefined)).toBe(false);
  });

  it("ritorna true per un payload con moltissime chiavi (simula vars/tags oversize)", () => {
    // Costruisce direttamente un numero di chiavi sufficiente a superare
    // la soglia, senza ricalcolare JSON.stringify ad ogni iterazione
    // (eviterebbe un costo O(n^2) nel test stesso).
    const vars: Record<string, string> = {};
    const entryCount = Math.ceil(MAX_ARGS_JSON_LENGTH / 10) + 1000;
    for (let i = 0; i < entryCount; i++) vars[`k${i}`] = "v";
    expect(argsTroppoGrandi({ vars })).toBe(true);
  });

  it("ritorna false esattamente al limite e true appena sopra", () => {
    // Costruisce una stringa la cui rappresentazione JSON è nota.
    const base = "a".repeat(MAX_ARGS_JSON_LENGTH - 10);
    expect(argsTroppoGrandi({ q: base }).valueOf()).toBe(false);
    const oltre = "a".repeat(MAX_ARGS_JSON_LENGTH + 10);
    expect(argsTroppoGrandi({ q: oltre })).toBe(true);
  });
});
