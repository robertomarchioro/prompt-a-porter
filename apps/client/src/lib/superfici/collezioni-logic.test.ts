/**
 * Test per collezioni-logic.ts: transizioni di stato della card
 * «Collezioni» e formattazione del riepilogo, senza runtime Svelte né rete.
 */

import { describe, it, expect, vi } from "vitest";
import {
  eseguiAggiorna,
  eseguiElenca,
  eseguiImporta,
  formattaDataImport,
  formattaRiepilogo,
  formattaRiepilogoAggiorna,
  messaggioErrore,
  statoCollezione,
  type CollezioneVoce,
} from "./collezioni-logic";

const COLLEZIONE: CollezioneVoce = {
  slug: "sviluppatore",
  titolo: "Sviluppatore",
  descrizione: "Code review, test, refactoring.",
  prompt: 13,
  sha256: "a".repeat(64),
  importata_sha256: null,
  importata_a: null,
};

describe("statoCollezione", () => {
  it("è «nuova» se mai importata", () => {
    expect(statoCollezione(COLLEZIONE)).toBe("nuova");
  });

  it("è «aggiornata» se l'impronta importata coincide con quella remota", () => {
    expect(
      statoCollezione({
        ...COLLEZIONE,
        importata_sha256: "a".repeat(64),
        importata_a: "2026-09-19 08:00:00",
      }),
    ).toBe("aggiornata");
  });

  it("è «aggiornabile» se il file remoto è cambiato dopo l'importazione", () => {
    expect(
      statoCollezione({
        ...COLLEZIONE,
        importata_sha256: "b".repeat(64),
        importata_a: "2026-09-19 08:00:00",
      }),
    ).toBe("aggiornabile");
  });
});

describe("formattaRiepilogoAggiorna", () => {
  it("elenca aggiornati, nuovi e conservati", () => {
    expect(
      formattaRiepilogoAggiorna({
        nuovi: 1,
        aggiornati: 3,
        conflitti: 2,
        errori: [],
      }),
    ).toBe("3 aggiornati · 1 nuovo · 2 conservati perché modificati da te");
  });

  it("dice che è già allineata quando non cambia nulla", () => {
    expect(
      formattaRiepilogoAggiorna({ nuovi: 0, aggiornati: 0, conflitti: 0, errori: [] }),
    ).toBe("Già allineata: nessuna modifica");
  });

  it("usa il singolare", () => {
    expect(
      formattaRiepilogoAggiorna({ nuovi: 0, aggiornati: 1, conflitti: 1, errori: ["x"] }),
    ).toBe("1 aggiornato · 1 conservato perché modificato da te · 1 non aggiornato");
  });
});

describe("formattaDataImport", () => {
  it("formatta la data SQLite «YYYY-MM-DD HH:MM:SS» come UTC", () => {
    expect(formattaDataImport("2026-09-19 08:00:00")).toMatch(/19 set 2026/);
  });

  it("torna stringa vuota per null o data non valida", () => {
    expect(formattaDataImport(null)).toBe("");
    expect(formattaDataImport("boh")).toBe("");
  });
});

describe("eseguiAggiorna", () => {
  it("chiama il comando aggiorna e notifica se ha scritto", async () => {
    const aggiorna = vi
      .fn()
      .mockResolvedValue({ nuovi: 0, aggiornati: 2, conflitti: 1, errori: [] });
    const notificaListaMutata = vi.fn();

    const esito = await eseguiAggiorna("sviluppatore", { aggiorna, notificaListaMutata });

    expect(aggiorna).toHaveBeenCalledWith("sviluppatore");
    expect(notificaListaMutata).toHaveBeenCalledTimes(1);
    expect(esito).toEqual({
      ok: true,
      esito: {
        slug: "sviluppatore",
        riepilogo: "2 aggiornati · 1 conservato perché modificato da te",
        errori: [],
      },
    });
  });

  it("non notifica se l'aggiornamento non ha scritto nulla", async () => {
    const aggiorna = vi
      .fn()
      .mockResolvedValue({ nuovi: 0, aggiornati: 0, conflitti: 0, errori: [] });
    const notificaListaMutata = vi.fn();

    const esito = await eseguiAggiorna("sviluppatore", { aggiorna, notificaListaMutata });

    expect(notificaListaMutata).not.toHaveBeenCalled();
    expect(esito.ok && esito.esito.riepilogo).toBe("Già allineata: nessuna modifica");
  });
});

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
