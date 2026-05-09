// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import { caricaStato, salvaStato, DEFAULTS } from "./sidebar-collapsed";

/**
 * F11 PR-B — test pure logic dello store sidebar-collapsed (F2).
 * Stesso pattern di densita.test.ts: parse/save resilient su
 * localStorage chiave `pap.sidebar.collapsed` con fallback DEFAULTS.
 */

const STORAGE_KEY = "pap.sidebar.collapsed";

beforeEach(() => {
  localStorage.clear();
});

describe("caricaStato", () => {
  it("ritorna DEFAULTS quando localStorage è vuoto", () => {
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("ritorna copia (structuredClone) di DEFAULTS, non la stessa reference", () => {
    const a = caricaStato();
    const b = caricaStato();
    expect(a).toEqual(b);
    expect(a).not.toBe(b);
    expect(a.gruppi).not.toBe(b.gruppi);
  });

  it("parsing valido di tutti i campi", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        sidebarCollapsed: true,
        gruppi: {
          viste: true,
          visibilita: true,
          cartelle: false,
          tag: true,
          modelTarget: false,
        },
      }),
    );
    expect(caricaStato()).toEqual({
      sidebarCollapsed: true,
      gruppi: {
        viste: true,
        visibilita: true,
        cartelle: false,
        tag: true,
        modelTarget: false,
      },
    });
  });

  it("usa DEFAULTS per campi mancanti", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ sidebarCollapsed: true }),
    );
    expect(caricaStato()).toEqual({
      sidebarCollapsed: true,
      gruppi: DEFAULTS.gruppi,
    });
  });

  it("ignora valori di tipo errato e fallback a DEFAULTS", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        sidebarCollapsed: "yes",
        gruppi: { viste: 1, modelTarget: null },
      }),
    );
    const stato = caricaStato();
    expect(stato.sidebarCollapsed).toBe(DEFAULTS.sidebarCollapsed);
    expect(stato.gruppi.viste).toBe(DEFAULTS.gruppi.viste);
    expect(stato.gruppi.modelTarget).toBe(DEFAULTS.gruppi.modelTarget);
  });

  it("ritorna DEFAULTS su JSON malformato", () => {
    localStorage.setItem(STORAGE_KEY, "{not json");
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("modelTarget ha default true (collassato come da prototipo)", () => {
    expect(DEFAULTS.gruppi.modelTarget).toBe(true);
  });
});

describe("salvaStato", () => {
  it("scrive lo stato come JSON in localStorage", () => {
    const stato = {
      sidebarCollapsed: true,
      gruppi: {
        viste: false,
        visibilita: true,
        cartelle: false,
        tag: false,
        modelTarget: true,
      },
    };
    salvaStato(stato);
    const raw = localStorage.getItem(STORAGE_KEY);
    expect(raw).toBeTruthy();
    expect(JSON.parse(raw!)).toEqual(stato);
  });

  it("round-trip salva → carica preserva lo stato", () => {
    const stato = {
      sidebarCollapsed: true,
      gruppi: {
        viste: true,
        visibilita: false,
        cartelle: true,
        tag: false,
        modelTarget: false,
      },
    };
    salvaStato(stato);
    expect(caricaStato()).toEqual(stato);
  });

  it("non lancia su localStorage pieno (silently ignore)", () => {
    const original = Storage.prototype.setItem;
    Storage.prototype.setItem = () => {
      throw new Error("QuotaExceededError");
    };
    expect(() => salvaStato(DEFAULTS)).not.toThrow();
    Storage.prototype.setItem = original;
  });
});
