import { describe, it, expect } from "vitest";
import {
  importFallito,
  MESSAGGIO_IMPORT_ANNULLATO,
  type ImportReport,
} from "./import-report";

const OK: ImportReport = {
  nuovi: 3,
  aggiornati: 0,
  conflitti: 1,
  errori: [],
  annullato: false,
};

describe("importFallito", () => {
  it("è falso per un import riuscito, anche con conflitti", () => {
    expect(importFallito(OK)).toBe(false);
  });

  it("è vero se l'import è stato annullato", () => {
    expect(
      importFallito({
        ...OK,
        nuovi: 0,
        conflitti: 0,
        errori: ["Prompt p: importazione non riuscita."],
        annullato: true,
      }),
    ).toBe(true);
  });

  it("è vero anche con errori e flag assente: mai un successo con errori", () => {
    expect(importFallito({ ...OK, errori: ["x"] })).toBe(true);
  });

  it("espone il messaggio unico di annullamento", () => {
    expect(MESSAGGIO_IMPORT_ANNULLATO).toBe(
      "Importazione annullata: nessuna modifica è stata salvata.",
    );
  });
});
