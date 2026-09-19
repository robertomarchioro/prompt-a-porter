/**
 * Test per collezioni-logic.ts: transizioni di stato della card
 * «Collezioni» e formattazione del riepilogo, senza runtime Svelte né rete.
 */

import { describe, it, expect, vi } from "vitest";
import {
  eseguiElenca,
  eseguiImporta,
  formattaRiepilogo,
  messaggioErrore,
  type CollezioneInfo,
} from "./collezioni-logic";

const COLLEZIONE: CollezioneInfo = {
  slug: "sviluppatore",
  titolo: "Sviluppatore",
  descrizione: "Code review, test, refactoring.",
  prompt: 13,
  sha256: "a".repeat(64),
};

describe("formattaRiepilogo", () => {
  it("mostra solo gli aggiunti quando non ci sono conflitti né errori", () => {
    expect(
      formattaRiepilogo({ nuovi: 12, aggiornati: 0, conflitti: 0, errori: [] }),
    ).toBe("12 elementi aggiunti");
  });

  it("usa il singolare per un solo elemento", () => {
    expect(
      formattaRiepilogo({ nuovi: 1, aggiornati: 0, conflitti: 0, errori: [] }),
    ).toBe("1 elemento aggiunto");
  });

  it("aggiunge i già presenti e i non importati", () => {
    expect(
      formattaRiepilogo({
        nuovi: 0,
        aggiornati: 0,
        conflitti: 20,
        errori: ["Tag x: importazione non riuscita."],
      }),
    ).toBe("0 elementi aggiunti · 20 già presenti · 1 non importato");
  });
});

describe("messaggioErrore", () => {
  it("toglie il prefisso «Error: » degli errori Tauri", () => {
    expect(messaggioErrore(new Error("Verifica la connessione."))).toBe(
      "Verifica la connessione.",
    );
    expect(messaggioErrore("Download non riuscito.")).toBe(
      "Download non riuscito.",
    );
  });
});

describe("eseguiElenca", () => {
  it("restituisce le collezioni quando il comando riesce", async () => {
    // Arrange
    const elenca = vi.fn().mockResolvedValue([COLLEZIONE]);

    // Act
    const esito = await eseguiElenca({ elenca });

    // Assert
    expect(esito).toEqual({ ok: true, collezioni: [COLLEZIONE] });
  });

  it("restituisce l'errore ripulito quando il comando fallisce", async () => {
    const elenca = vi
      .fn()
      .mockRejectedValue(new Error("Download dell'indice non riuscito."));

    const esito = await eseguiElenca({ elenca });

    expect(esito).toEqual({
      ok: false,
      errore: "Download dell'indice non riuscito.",
    });
  });
});

describe("eseguiImporta", () => {
  it("notifica la libreria e riassume quando importa prompt nuovi", async () => {
    // Arrange
    const importa = vi
      .fn()
      .mockResolvedValue({ nuovi: 13, aggiornati: 0, conflitti: 2, errori: [] });
    const notificaListaMutata = vi.fn();

    // Act
    const esito = await eseguiImporta("sviluppatore", {
      importa,
      notificaListaMutata,
    });

    // Assert
    expect(importa).toHaveBeenCalledWith("sviluppatore");
    expect(notificaListaMutata).toHaveBeenCalledTimes(1);
    expect(esito).toEqual({
      ok: true,
      esito: {
        slug: "sviluppatore",
        riepilogo: "13 elementi aggiunti · 2 già presenti",
        errori: [],
      },
    });
  });

  it("NON notifica la libreria se il re-import non ha scritto nulla", async () => {
    const importa = vi
      .fn()
      .mockResolvedValue({ nuovi: 0, aggiornati: 0, conflitti: 22, errori: [] });
    const notificaListaMutata = vi.fn();

    const esito = await eseguiImporta("sviluppatore", {
      importa,
      notificaListaMutata,
    });

    expect(notificaListaMutata).not.toHaveBeenCalled();
    expect(esito.ok).toBe(true);
  });

  it("restituisce l'errore ripulito e non notifica quando il comando fallisce", async () => {
    const importa = vi
      .fn()
      .mockRejectedValue(
        new Error("Il file della collezione non corrisponde all'indice."),
      );
    const notificaListaMutata = vi.fn();

    const esito = await eseguiImporta("sviluppatore", {
      importa,
      notificaListaMutata,
    });

    expect(notificaListaMutata).not.toHaveBeenCalled();
    expect(esito).toEqual({
      ok: false,
      errore: "Il file della collezione non corrisponde all'indice.",
    });
  });
});
