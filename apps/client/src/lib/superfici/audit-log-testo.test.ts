import { describe, expect, test } from "vitest";
import {
  AUDIT_A_COSA_SERVE,
  AUDIT_COME,
  AUDIT_COSA,
  AUDIT_DOVE,
  AUDIT_LABEL_SIDEBAR,
  AUDIT_TITOLO,
} from "./audit-log-testo";

describe("testi sezione Registro attività (issue #583)", () => {
  test("l'etichetta di sidebar non menziona l'AI in modo fuorviante", () => {
    expect(AUDIT_LABEL_SIDEBAR.toLowerCase()).not.toContain("ai");
  });

  test("il titolo resta riconoscibile come audit log per la ricerca", () => {
    expect(AUDIT_TITOLO.toLowerCase()).toContain("audit log");
  });

  test("«cosa» dichiara esplicitamente cosa NON viene registrato", () => {
    expect(AUDIT_COSA).toContain("Non registra il contenuto dei prompt");
    expect(AUDIT_COSA.toLowerCase()).toContain("richieste inviate ai modelli ai");
  });

  test("«cosa» descrive azioni su vault/prompt, non conversazioni AI", () => {
    expect(AUDIT_COSA.toLowerCase()).toContain("vault");
    expect(AUDIT_COSA.toLowerCase()).toContain("prompt");
  });

  test("«cosa» avvisa che titoli ed etichette finiscono nei metadati e nel CSV (#587)", () => {
    expect(AUDIT_COSA.toLowerCase()).toContain("titolo");
    expect(AUDIT_COSA.toLowerCase()).toContain("etichetta");
    expect(AUDIT_COSA.toLowerCase()).toContain("csv");
  });

  test("«cosa» segnala il percorso di cartella come il più rivelatore (#587)", () => {
    expect(AUDIT_COSA.toLowerCase()).toContain("percorso completo di una cartella");
    expect(AUDIT_COSA.toLowerCase()).toContain("variante");
  });

  test("«dove» chiarisce che i dati restano in locale", () => {
    expect(AUDIT_DOVE.toLowerCase()).toContain("locale");
    expect(AUDIT_DOVE.toLowerCase()).toContain("computer");
  });

  test("«come» dichiara che oggi esiste solo l'export CSV", () => {
    expect(AUDIT_COME.toLowerCase()).toContain("csv");
    expect(AUDIT_COME.toLowerCase()).toContain("non esiste ancora");
  });

  test("«a cosa serve» spiega la finalità di ricostruzione storica", () => {
    expect(AUDIT_A_COSA_SERVE.toLowerCase()).toContain("cosa è stato fatto");
  });

  test("nessun testo è vuoto", () => {
    for (const testo of [
      AUDIT_LABEL_SIDEBAR,
      AUDIT_TITOLO,
      AUDIT_COSA,
      AUDIT_DOVE,
      AUDIT_COME,
      AUDIT_A_COSA_SERVE,
    ]) {
      expect(testo.trim().length).toBeGreaterThan(0);
    }
  });
});
