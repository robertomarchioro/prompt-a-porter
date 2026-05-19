import { describe, it, expect } from "vitest";
import {
  MODELLI_TARGET,
  VALORI_VALIDI,
  etichettaPerValore,
} from "./modelli-target";

describe("modelli-target", () => {
  it("MODELLI_TARGET non e vuoto e include almeno una entry per famiglia anthropic", () => {
    expect(MODELLI_TARGET.length).toBeGreaterThan(0);
    expect(MODELLI_TARGET.some((m) => m.famiglia === "anthropic")).toBe(true);
  });

  it("VALORI_VALIDI contiene tutti i value di MODELLI_TARGET", () => {
    for (const m of MODELLI_TARGET) {
      expect(VALORI_VALIDI.has(m.value)).toBe(true);
    }
  });

  it("etichettaPerValore ritorna stringa vuota per null/undefined/empty", () => {
    expect(etichettaPerValore(null)).toBe("");
    expect(etichettaPerValore(undefined)).toBe("");
    expect(etichettaPerValore("")).toBe("");
  });

  it("etichettaPerValore mappa value noto al label", () => {
    expect(etichettaPerValore("claude-opus")).toBe("Claude Opus");
    expect(etichettaPerValore("gpt-4")).toBe("GPT-4");
    expect(etichettaPerValore("generic")).toBe("Generico");
  });

  it("etichettaPerValore fallback al value se sconosciuto", () => {
    expect(etichettaPerValore("modello-non-esistente")).toBe(
      "modello-non-esistente",
    );
  });
});
