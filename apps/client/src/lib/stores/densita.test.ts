// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import { caricaStato, salvaStato, DEFAULTS } from "./densita";

const STORAGE_KEY = "pap.lista.densita";

beforeEach(() => {
  localStorage.clear();
});

describe("caricaStato", () => {
  it("ritorna DEFAULTS quando localStorage è vuoto", () => {
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("parsing valido di tutti i campi", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        densita: "compatta",
        righePreview: 5,
        ordine: "alfabetico",
      }),
    );
    expect(caricaStato()).toEqual({
      densita: "compatta",
      righePreview: 5,
      ordine: "alfabetico",
    });
  });

  it("JSON corrotto → DEFAULTS", () => {
    localStorage.setItem(STORAGE_KEY, "{not valid json");
    expect(caricaStato()).toEqual(DEFAULTS);
  });

  it("densità sconosciuta → fallback default", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ densita: "extra-large", righePreview: 3, ordine: "recente" }),
    );
    expect(caricaStato().densita).toBe(DEFAULTS.densita);
  });

  it("ordine sconosciuto → fallback default", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ densita: "comoda", righePreview: 3, ordine: "random" }),
    );
    expect(caricaStato().ordine).toBe(DEFAULTS.ordine);
  });

  it("righePreview clampato in [1, 8]", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ densita: "anteprima", righePreview: 99, ordine: "recente" }),
    );
    expect(caricaStato().righePreview).toBe(8);

    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ densita: "anteprima", righePreview: -3, ordine: "recente" }),
    );
    expect(caricaStato().righePreview).toBe(1);

    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ densita: "anteprima", righePreview: 4.7, ordine: "recente" }),
    );
    expect(caricaStato().righePreview).toBe(5); // round
  });
});

describe("salvaStato", () => {
  it("non lancia se localStorage è disponibile", () => {
    expect(() =>
      salvaStato({ densita: "comoda", righePreview: 3, ordine: "recente" }),
    ).not.toThrow();
  });

  it("sanitizza valori non validi prima di salvare", () => {
    salvaStato({
      densita: "fake" as never,
      righePreview: 999,
      ordine: "fake" as never,
    });
    const letto = caricaStato();
    expect(letto.densita).toBe(DEFAULTS.densita);
    expect(letto.ordine).toBe(DEFAULTS.ordine);
    expect(letto.righePreview).toBe(8);
  });

  it("round-trip: save + load preserva i valori validi", () => {
    salvaStato({ densita: "compatta", righePreview: 4, ordine: "qualita" });
    expect(caricaStato()).toEqual({
      densita: "compatta",
      righePreview: 4,
      ordine: "qualita",
    });
  });
});
