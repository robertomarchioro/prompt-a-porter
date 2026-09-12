/**
 * Test per `puoSalvare` (issue #642).
 *
 * La logica pura è estratta in salvataggio-logic.ts e importata qui per
 * evitare drift tra test e implementazione reale (_pure/_impl pattern, già
 * usato per nuova-cartella-logic.ts / crea-variante-logic.ts).
 *
 * Copre:
 * - solo titolo compilato → true (era il bug: il salvataggio si bloccava)
 * - solo body compilato, titolo vuoto → false
 * - entrambi vuoti → false
 * - titolo composto solo da spazi → false
 * - entrambi compilati → true
 */

import { describe, it, expect } from "vitest";
import { puoSalvare } from "./salvataggio-logic";

describe("puoSalvare", () => {
  it("ritorna true con solo il titolo compilato (#642)", () => {
    expect(puoSalvare("Il mio prompt", "")).toBe(true);
  });

  it("ritorna false con solo il body compilato e titolo vuoto", () => {
    expect(puoSalvare("", "Un body qualsiasi")).toBe(false);
  });

  it("ritorna false con titolo e body entrambi vuoti", () => {
    expect(puoSalvare("", "")).toBe(false);
  });

  it("ritorna false con titolo composto solo da spazi", () => {
    expect(puoSalvare("   ", "Un body")).toBe(false);
  });

  it("ritorna true con titolo e body entrambi compilati", () => {
    expect(puoSalvare("Titolo", "Body")).toBe(true);
  });
});
