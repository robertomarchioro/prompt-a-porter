import { describe, it, expect } from "vitest";
import {
  REGISTRO,
  etichettaModello,
  modelliDelProvider,
  modelliNoti,
  modelloPredefinito,
  opzioniModello,
  providerHaModelliNoti,
  type ModelloAI,
} from "./modelli-provider";

const finto = (p: Partial<ModelloAI>): ModelloAI => ({
  id: "x",
  provider: "anthropic",
  etichetta: "X",
  obsoleto: false,
  anteprima: false,
  visto_il: "2026-08-01",
  contesto: null,
  prezzo: null,
  ...p,
});

describe("registro modelli", () => {
  it("i provider pubblici hanno modelli noti", () => {
    expect(providerHaModelliNoti("anthropic")).toBe(true);
    expect(providerHaModelliNoti("openai")).toBe(true);
    expect(providerHaModelliNoti("gemini")).toBe(true);
  });

  it("i provider senza listino non hanno modelli noti (→ testo libero)", () => {
    expect(providerHaModelliNoti("ollama")).toBe(false);
    expect(providerHaModelliNoti("openai-compat")).toBe(false);
    expect(providerHaModelliNoti("")).toBe(false);
  });

  it("ogni voce del registro ha id, provider ed etichetta non vuoti", () => {
    expect(REGISTRO.modelli.length).toBeGreaterThan(0);
    for (const m of REGISTRO.modelli) {
      expect(m.id).toBeTruthy();
      expect(m.etichetta).toBeTruthy();
      expect(["anthropic", "openai", "gemini"]).toContain(m.provider);
    }
  });

  it("non ci sono id duplicati", () => {
    const id = REGISTRO.modelli.map((m) => m.id);
    expect(new Set(id).size).toBe(id.length);
  });

  it("ogni provider pubblico ha almeno un modello attivo e non in anteprima", () => {
    for (const p of ["anthropic", "openai", "gemini"]) {
      expect(modelloPredefinito(p)).not.toBe("");
    }
  });
});

describe("ordinamento e obsoleti", () => {
  it("modelliDelProvider mette gli obsoleti in fondo", () => {
    const attivi = modelliDelProvider("anthropic").filter((m) => !m.obsoleto).length;
    const ordinati = modelliDelProvider("anthropic");
    // Nessun attivo compare dopo un obsoleto.
    const primoObsoleto = ordinati.findIndex((m) => m.obsoleto);
    if (primoObsoleto !== -1) expect(primoObsoleto).toBe(attivi);
  });

  it("etichettaModello segnala anteprima e obsoleto", () => {
    expect(etichettaModello(finto({ etichetta: "A" }))).toBe("A");
    expect(etichettaModello(finto({ etichetta: "A", anteprima: true }))).toBe(
      "A (anteprima)",
    );
    expect(etichettaModello(finto({ etichetta: "A", obsoleto: true }))).toBe(
      "A (obsoleto)",
    );
    expect(
      etichettaModello(finto({ etichetta: "A", anteprima: true, obsoleto: true })),
    ).toBe("A (anteprima, obsoleto)");
  });

  it("modelloPredefinito salta anteprime e obsoleti", () => {
    // Gemini ha anteprime in registro: il predefinito non deve esserlo.
    const scelto = REGISTRO.modelli.find((m) => m.id === modelloPredefinito("gemini"));
    expect(scelto?.anteprima).toBe(false);
    expect(scelto?.obsoleto).toBe(false);
  });
});

describe("opzioniModello", () => {
  it("restituisce valore ed etichetta per ogni voce", () => {
    const opz = opzioniModello("anthropic", "");
    expect(opz.length).toBe(modelliNoti("anthropic").length);
    for (const o of opz) {
      expect(o.value).toBeTruthy();
      expect(o.etichetta).toBeTruthy();
    }
  });

  it("aggiunge in coda il valore corrente se non è in registro", () => {
    const opz = opzioniModello("anthropic", "claude-custom-x");
    expect(opz.at(-1)).toEqual({
      value: "claude-custom-x",
      etichetta: "claude-custom-x (non in elenco)",
    });
  });

  it("non duplica un valore già in registro", () => {
    const presente = modelliNoti("anthropic")[0];
    const occorrenze = opzioniModello("anthropic", presente).filter(
      (o) => o.value === presente,
    ).length;
    expect(occorrenze).toBe(1);
  });

  it("ignora un valore corrente vuoto", () => {
    expect(opzioniModello("openai", "").map((o) => o.value)).toEqual(
      modelliNoti("openai"),
    );
  });

  it("per un provider a campo libero non offre opzioni", () => {
    expect(opzioniModello("ollama", "")).toEqual([]);
  });
});
