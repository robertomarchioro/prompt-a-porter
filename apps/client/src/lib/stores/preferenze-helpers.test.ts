// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { risolviTema, applicaThemeTone } from "./preferenze-helpers";

const matchMediaOriginale = window.matchMedia;

function mockPrefersColorScheme(prefersDark: boolean): void {
  window.matchMedia = vi.fn().mockImplementation(() => ({
    matches: prefersDark,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })) as unknown as typeof window.matchMedia;
}

beforeEach(() => {
  window.matchMedia = matchMediaOriginale;
  document.documentElement.removeAttribute("data-theme");
  document.documentElement.removeAttribute("data-tone");
});

describe("risolviTema", () => {
  it("ritorna 'dark' quando input è 'dark'", () => {
    expect(risolviTema("dark")).toBe("dark");
  });

  it("ritorna 'light' quando input è 'light'", () => {
    expect(risolviTema("light")).toBe("light");
  });

  it("ritorna 'dark' per 'auto' quando sistema preferisce dark", () => {
    mockPrefersColorScheme(true);
    expect(risolviTema("auto")).toBe("dark");
  });

  it("ritorna 'light' per 'auto' quando sistema preferisce light", () => {
    mockPrefersColorScheme(false);
    expect(risolviTema("auto")).toBe("light");
  });

  it("fallback su 'dark' per stringhe sconosciute (sistema dark)", () => {
    mockPrefersColorScheme(true);
    expect(risolviTema("blu-vintage")).toBe("dark");
  });

  it("fallback su 'light' per stringhe sconosciute (sistema light)", () => {
    mockPrefersColorScheme(false);
    expect(risolviTema("blu-vintage")).toBe("light");
  });
});

describe("applicaThemeTone", () => {
  it("setta data-theme e data-tone su <html>", () => {
    applicaThemeTone("dark", "zinc");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    expect(document.documentElement.getAttribute("data-tone")).toBe("zinc");
  });

  it("aggiorna gli attributi su chiamate successive", () => {
    applicaThemeTone("dark", "zinc");
    applicaThemeTone("light", "stone");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    expect(document.documentElement.getAttribute("data-tone")).toBe("stone");
  });

  it("risolve 'auto' a dark/light prima del setAttribute", () => {
    mockPrefersColorScheme(false);
    applicaThemeTone("auto", "slate");
    // 'auto' non leakato come valore: deve essere risolto a 'light' qui.
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    expect(document.documentElement.getAttribute("data-tone")).toBe("slate");
  });

  it("risolve 'auto' a 'dark' quando sistema preferisce dark", () => {
    mockPrefersColorScheme(true);
    applicaThemeTone("auto", "zinc");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });
});
