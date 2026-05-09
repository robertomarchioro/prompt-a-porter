import { describe, it, expect } from "vitest";
import { hashDjb2, coloreAvatar } from "./avatar-color";

describe("hashDjb2", () => {
  it("è deterministico (stessa stringa = stesso hash)", () => {
    expect(hashDjb2("Personale")).toBe(hashDjb2("Personale"));
    expect(hashDjb2("Team Marketing")).toBe(hashDjb2("Team Marketing"));
  });

  it("ritorna hash diverso per stringhe diverse (campione)", () => {
    const inputs = [
      "Personale",
      "Team",
      "Marketing",
      "Engineering",
      "ws-personale",
      "ws-team",
      "Roberto",
      "Maria",
    ];
    const hashes = new Set(inputs.map(hashDjb2));
    expect(hashes.size).toBe(inputs.length);
  });

  it("non lancia su stringa vuota", () => {
    expect(() => hashDjb2("")).not.toThrow();
    expect(hashDjb2("")).toBe(5381);
  });

  it("ritorna intero positivo", () => {
    const inputs = ["a", "z", "lorem ipsum dolor", "🚀"];
    for (const i of inputs) {
      const h = hashDjb2(i);
      expect(Number.isInteger(h)).toBe(true);
      expect(h).toBeGreaterThanOrEqual(0);
    }
  });
});

describe("coloreAvatar", () => {
  it("ritorna formato HSL parseable", () => {
    const c = coloreAvatar("Personale");
    expect(c.background).toMatch(/^hsl\(\d+ 55% 58%\)$/);
  });

  it("hue compreso in [0, 359]", () => {
    const inputs = ["a", "Personale", "very long string that we hash"];
    for (const i of inputs) {
      const c = coloreAvatar(i);
      const match = c.background.match(/^hsl\((\d+) 55% 58%\)$/);
      expect(match).not.toBeNull();
      const hue = Number(match![1]);
      expect(hue).toBeGreaterThanOrEqual(0);
      expect(hue).toBeLessThanOrEqual(359);
    }
  });

  it("foreground è '#000' o '#fff'", () => {
    const c = coloreAvatar("Personale");
    expect(["#fff", "#000"]).toContain(c.foreground);
  });

  it("è deterministico", () => {
    const a = coloreAvatar("Personale");
    const b = coloreAvatar("Personale");
    expect(a.background).toBe(b.background);
    expect(a.foreground).toBe(b.foreground);
  });

  it("non lancia su stringa vuota", () => {
    expect(() => coloreAvatar("")).not.toThrow();
  });
});
