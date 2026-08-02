import { describe, it, expect } from "vitest";
import {
  compilaRegex,
  erroreRegex,
  filtraRighe,
  ordinaRecentiPrimi,
  formattaRigaRicaricaAvvio,
  formattaRigaRicaricaEsito,
  N_RIGHE_DEFAULT,
  N_RIGHE_MAX,
  OPZIONI_N_RIGHE,
  type RigaLog,
} from "./log-viewer-logic";

const RIGA = (over: Partial<RigaLog> = {}): RigaLog => ({
  timestamp: "2026-08-02 10:00:00",
  level: "INFO",
  target: "pap_lib::editor",
  message: "evento",
  raw: "[2026-08-02][10:00:00][pap_lib::editor][INFO] evento",
  ...over,
});

describe("log-viewer-logic — filtraRighe", () => {
  it('con livello "Tutti" (stringa vuota) non applica alcun filtro di livello', () => {
    // Arrange
    const righe = [
      RIGA({ level: "INFO", message: "uno" }),
      RIGA({ level: "WARN", message: "due" }),
      RIGA({ level: "ERROR", message: "tre" }),
    ];

    // Act
    const risultato = filtraRighe(righe, "", null);

    // Assert
    expect(risultato).toHaveLength(3);
    expect(risultato.map((r) => r.message)).toEqual(["uno", "due", "tre"]);
  });

  it("con un livello esplicito mostra solo le righe corrispondenti", () => {
    // Arrange
    const righe = [
      RIGA({ level: "INFO", message: "uno" }),
      RIGA({ level: "WARN", message: "due" }),
      RIGA({ level: "ERROR", message: "tre" }),
    ];

    // Act
    const risultato = filtraRighe(righe, "WARN", null);

    // Assert
    expect(risultato).toHaveLength(1);
    expect(risultato[0].message).toBe("due");
  });

  it("combina filtro livello e filtro regex", () => {
    // Arrange
    const righe = [
      RIGA({ level: "INFO", message: "salva prompt" }),
      RIGA({ level: "INFO", message: "carica vault" }),
      RIGA({ level: "ERROR", message: "salva fallita" }),
    ];

    // Act
    const risultato = filtraRighe(righe, "INFO", /salva/i);

    // Assert
    expect(risultato).toHaveLength(1);
    expect(risultato[0].message).toBe("salva prompt");
  });

  it("il filtro regex verifica anche target e raw, non solo message", () => {
    // Arrange
    const righe = [
      RIGA({ target: "pap_lib::sync", message: "evento generico" }),
      RIGA({ target: "pap_lib::editor", message: "evento generico" }),
    ];

    // Act
    const risultato = filtraRighe(righe, "", /sync/i);

    // Assert
    expect(risultato).toHaveLength(1);
    expect(risultato[0].target).toBe("pap_lib::sync");
  });

  it("non muta l'array in input", () => {
    // Arrange
    const righe = [RIGA({ level: "INFO" }), RIGA({ level: "WARN" })];
    const copiaOriginale = [...righe];

    // Act
    filtraRighe(righe, "WARN", null);

    // Assert
    expect(righe).toEqual(copiaOriginale);
  });

  it("senza alcun filtro attivo ritorna un array con lo stesso contenuto dell'input", () => {
    // Arrange
    const righe = [RIGA({ message: "a" }), RIGA({ message: "b" })];

    // Act
    const risultato = filtraRighe(righe, "", null);

    // Assert
    expect(risultato).toEqual(righe);
  });
});

describe("log-viewer-logic — ordinaRecentiPrimi", () => {
  it("inverte l'ordine così il più recente compare per primo", () => {
    // Arrange: il backend restituisce ordine cronologico crescente
    const righe = [
      RIGA({ message: "il più vecchio" }),
      RIGA({ message: "intermedio" }),
      RIGA({ message: "il più recente" }),
    ];

    // Act
    const risultato = ordinaRecentiPrimi(righe);

    // Assert
    expect(risultato.map((r) => r.message)).toEqual([
      "il più recente",
      "intermedio",
      "il più vecchio",
    ]);
  });

  it("non muta l'array sorgente (pattern immutabile)", () => {
    // Arrange
    const righe = [RIGA({ message: "a" }), RIGA({ message: "b" })];
    const riferimentoOriginale = righe;

    // Act
    const risultato = ordinaRecentiPrimi(righe);

    // Assert: l'array originale resta nello stesso ordine e non è lo
    // stesso riferimento di quello ritornato.
    expect(righe).toBe(riferimentoOriginale);
    expect(righe.map((r) => r.message)).toEqual(["a", "b"]);
    expect(risultato).not.toBe(righe);
  });

  it("gestisce l'array vuoto senza errori", () => {
    expect(ordinaRecentiPrimi([])).toEqual([]);
  });
});

describe("log-viewer-logic — compilaRegex / erroreRegex", () => {
  it("ritorna null per input vuoto", () => {
    expect(compilaRegex("")).toBeNull();
    expect(compilaRegex("   ")).toBeNull();
    expect(erroreRegex("")).toBe("");
  });

  it("compila un pattern valido case-insensitive", () => {
    const re = compilaRegex("Salva");
    expect(re).not.toBeNull();
    expect(re?.test("salva prompt")).toBe(true);
    expect(erroreRegex("Salva")).toBe("");
  });

  it("ritorna null e un messaggio d'errore per un pattern non valido", () => {
    expect(compilaRegex("(")).toBeNull();
    expect(erroreRegex("(")).not.toBe("");
  });
});

describe("log-viewer-logic — strumentazione diagnostica ricarica() (#558 punto 1)", () => {
  it("formattaRigaRicaricaAvvio riporta origine, filtro livello e stato aperto", () => {
    const riga = formattaRigaRicaricaAvvio({
      origine: "apertura-pannello",
      livelloFiltro: "WARN",
      aperto: true,
    });

    expect(riga).toContain("origine=apertura-pannello");
    expect(riga).toContain('livelloFiltro="WARN"');
    expect(riga).toContain("aperto=true");
  });

  it('formattaRigaRicaricaAvvio mostra "Tutti" quando il filtro livello è la stringa vuota', () => {
    const riga = formattaRigaRicaricaAvvio({
      origine: "mount",
      livelloFiltro: "",
      aperto: false,
    });

    expect(riga).toContain('livelloFiltro="Tutti"');
  });

  it("formattaRigaRicaricaEsito riporta la coppia di conteggi backend/filtrate", () => {
    const riga = formattaRigaRicaricaEsito({
      origine: "manuale",
      nRigheBackend: 1000,
      nRigheFiltrate: 0,
      errore: "",
    });

    expect(riga).toContain("nRigheBackend=1000");
    expect(riga).toContain("nRigheFiltrate=0");
    expect(riga).not.toContain("errore=");
  });

  it("formattaRigaRicaricaEsito include l'errore quando presente", () => {
    const riga = formattaRigaRicaricaEsito({
      origine: "cambio-righe",
      nRigheBackend: 0,
      nRigheFiltrate: 0,
      errore: "connessione al backend fallita",
    });

    expect(riga).toContain('errore="connessione al backend fallita"');
  });

  it("non include mai il contenuto delle righe di log (solo conteggi)", () => {
    // Le funzioni non accettano nemmeno le righe come parametro: per
    // costruzione non possono farne trapelare il contenuto.
    const riga = formattaRigaRicaricaEsito({
      origine: "mount",
      nRigheBackend: 3,
      nRigheFiltrate: 3,
      errore: "",
    });

    expect(riga).not.toMatch(/message|target|timestamp/i);
  });
});

describe("log-viewer-logic — costanti righe", () => {
  it("N_RIGHE_DEFAULT è maggiore del vecchio default hardcoded (200)", () => {
    expect(N_RIGHE_DEFAULT).toBeGreaterThan(200);
  });

  it("N_RIGHE_DEFAULT non supera il massimo consentito dal backend", () => {
    expect(N_RIGHE_DEFAULT).toBeLessThanOrEqual(N_RIGHE_MAX);
  });

  it("le opzioni selezionabili includono il default e rispettano il massimo", () => {
    expect(OPZIONI_N_RIGHE).toContain(N_RIGHE_DEFAULT);
    for (const opzione of OPZIONI_N_RIGHE) {
      expect(opzione).toBeLessThanOrEqual(N_RIGHE_MAX);
      expect(opzione).toBeGreaterThan(0);
    }
  });
});
