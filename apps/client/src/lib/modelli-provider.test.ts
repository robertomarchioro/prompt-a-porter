import { describe, it, expect } from "vitest";
import {
  modelliNoti,
  providerHaModelliNoti,
  opzioniModello,
} from "./modelli-provider";

describe("modelli-provider", () => {
  it("i provider pubblici hanno modelli noti", () => {
    expect(providerHaModelliNoti("anthropic")).toBe(true);
    expect(providerHaModelliNoti("openai")).toBe(true);
    expect(providerHaModelliNoti("gemini")).toBe(true);
  });

  it("i provider non pubblici non hanno modelli noti (→ testo libero)", () => {
    expect(providerHaModelliNoti("ollama")).toBe(false);
    expect(providerHaModelliNoti("openai-compat")).toBe(false);
    expect(providerHaModelliNoti("")).toBe(false);
  });

  it("modelliNoti ritorna una lista non vuota per anthropic", () => {
    expect(modelliNoti("anthropic")).toContain("claude-sonnet-4-6");
  });

  it("opzioniModello include il valore corrente se non già in lista", () => {
    const opz = opzioniModello("anthropic", "claude-custom-x");
    expect(opz).toContain("claude-custom-x");
    expect(opz).toContain("claude-sonnet-4-6");
  });

  it("opzioniModello non duplica un valore già presente", () => {
    const opz = opzioniModello("anthropic", "claude-sonnet-4-6");
    const occorrenze = opz.filter((m) => m === "claude-sonnet-4-6").length;
    expect(occorrenze).toBe(1);
  });

  it("opzioniModello ignora un valore corrente vuoto", () => {
    expect(opzioniModello("openai", "")).toEqual(modelliNoti("openai"));
  });
});
