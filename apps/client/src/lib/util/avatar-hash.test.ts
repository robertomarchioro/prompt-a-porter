import { describe, it, expect, beforeEach } from "vitest";
import { avatarColorePerEmail, _resetCache } from "./avatar-hash";

describe("avatar-hash", () => {
  beforeEach(() => {
    _resetCache();
  });

  it("ritorna un colore deterministico per la stessa email", async () => {
    const a = await avatarColorePerEmail("alice@example.com");
    const b = await avatarColorePerEmail("alice@example.com");
    expect(a).toEqual(b);
  });

  it("background segue il formato hsl(<hue> 55% 58%) con hue in [0,360)", async () => {
    const c = await avatarColorePerEmail("bob@example.com");
    const m = /^hsl\((\d+) 55% 58%\)$/.exec(c.background);
    expect(m).not.toBeNull();
    const hue = Number(m![1]);
    expect(hue).toBeGreaterThanOrEqual(0);
    expect(hue).toBeLessThan(360);
  });

  it("foreground sempre bianco", async () => {
    const c = await avatarColorePerEmail("anything@example.com");
    expect(c.foreground).toBe("#fff");
  });

  it("normalizza case e whitespace (cache key uguale)", async () => {
    const a = await avatarColorePerEmail("  Foo@Example.com  ");
    const b = await avatarColorePerEmail("foo@example.com");
    expect(a).toEqual(b);
  });

  it("email diverse producono (di solito) hue diversi", async () => {
    const a = await avatarColorePerEmail("alice@example.com");
    const b = await avatarColorePerEmail("zorro@example.com");
    expect(a.background).not.toBe(b.background);
  });

  it("_resetCache permette ricalcolo deterministico identico", async () => {
    const prima = await avatarColorePerEmail("u@x.it");
    _resetCache();
    const dopo = await avatarColorePerEmail("u@x.it");
    expect(prima).toEqual(dopo);
  });
});
