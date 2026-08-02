import { describe, it, expect } from "vitest";
import {
  formattaRigaConferma,
  formattaRigaAvviso,
  dialogoProbabilmenteNonMostrato,
  SOGLIA_DIALOGO_NON_MOSTRATO_MS,
} from "./conferma-log";

describe("dialogoProbabilmenteNonMostrato", () => {
  it("è true su ramo nativo con durata sotto soglia", () => {
    expect(dialogoProbabilmenteNonMostrato("nativo", 0.4)).toBe(true);
  });

  it("è false su ramo nativo con durata sopra soglia (dialogo reale)", () => {
    expect(
      dialogoProbabilmenteNonMostrato("nativo", SOGLIA_DIALOGO_NON_MOSTRATO_MS + 50),
    ).toBe(false);
  });

  it("è sempre false su macOS, indipendentemente dalla durata", () => {
    expect(dialogoProbabilmenteNonMostrato("macos", 0)).toBe(false);
  });
});

describe("formattaRigaConferma", () => {
  const base = {
    azione: "elimina-prompt",
    ramo: "nativo" as const,
    durataMs: 1234.5678,
    lunghezzaMessaggio: 42,
    tipoEsito: "boolean",
    valoreEsito: true,
    platform: "Win32",
    piattaformaClassificata: "windows",
  };

  it("include azione, ramo, durata, lunghezza, esito e piattaforma", () => {
    const riga = formattaRigaConferma(base);

    expect(riga).toContain("azione=elimina-prompt");
    expect(riga).toContain("ramo=nativo");
    expect(riga).toContain("durataMs=1234.6");
    expect(riga).toContain("lunghezzaMessaggio=42");
    expect(riga).toContain("esitoTipo=boolean");
    expect(riga).toContain("esitoValore=true");
    expect(riga).toContain('platform="Win32"');
    expect(riga).toContain("piattaformaClassificata=windows");
  });

  it("distingue false da undefined nel valore grezzo dell'esito", () => {
    const rigaFalse = formattaRigaConferma({ ...base, tipoEsito: "boolean", valoreEsito: false });
    const rigaUndefined = formattaRigaConferma({
      ...base,
      tipoEsito: "undefined",
      valoreEsito: undefined,
    });

    expect(rigaFalse).toContain("esitoTipo=boolean esitoValore=false");
    expect(rigaUndefined).toContain("esitoTipo=undefined esitoValore=undefined");
    expect(rigaFalse).not.toBe(rigaUndefined);
  });

  it("segnala il sospetto quando il dialogo nativo probabilmente non è apparso", () => {
    const riga = formattaRigaConferma({ ...base, durataMs: 0.2 });

    expect(riga).toContain("sospetto=dialogo_non_mostrato");
  });

  it("non segnala alcun sospetto quando la durata è coerente con un dialogo reale", () => {
    const riga = formattaRigaConferma({ ...base, durataMs: 3000 });

    expect(riga).not.toContain("sospetto");
  });

  it("non segnala alcun sospetto sul ramo macOS anche a durata quasi nulla", () => {
    const riga = formattaRigaConferma({ ...base, ramo: "macos", durataMs: 0 });

    expect(riga).not.toContain("sospetto");
  });

  it("non contiene mai il testo del messaggio dell'utente (vincolo di privacy)", () => {
    const titoloSensibile = 'Eliminare il prompt "Preventivo Rossi Spa"?';
    // La funzione non riceve affatto il messaggio: solo la sua lunghezza.
    const riga = formattaRigaConferma({
      ...base,
      lunghezzaMessaggio: titoloSensibile.length,
    });

    expect(riga).not.toContain("Preventivo Rossi Spa");
    expect(riga).not.toContain(titoloSensibile);
    expect(riga).toContain(`lunghezzaMessaggio=${titoloSensibile.length}`);
  });
});

describe("formattaRigaAvviso", () => {
  it("include azione, ramo e lunghezza del messaggio", () => {
    const riga = formattaRigaAvviso({
      azione: "pulisci-log",
      ramo: "nativo",
      lunghezzaMessaggio: 17,
    });

    expect(riga).toContain("azione=pulisci-log");
    expect(riga).toContain("ramo=nativo");
    expect(riga).toContain("lunghezzaMessaggio=17");
  });

  it("non contiene mai il testo del messaggio dell'utente (vincolo di privacy)", () => {
    const titoloSensibile = "Errore nell'eliminazione del prompt \"Preventivo Rossi Spa\"";
    const riga = formattaRigaAvviso({
      azione: "elimina-prompt",
      ramo: "nativo",
      lunghezzaMessaggio: titoloSensibile.length,
    });

    expect(riga).not.toContain("Preventivo Rossi Spa");
    expect(riga).not.toContain(titoloSensibile);
  });
});
