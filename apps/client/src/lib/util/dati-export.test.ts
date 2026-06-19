import { describe, it, expect } from "vitest";
import { nomeFileExport, slugFile } from "./dati-export";

describe("slugFile", () => {
  it("minuscolo + spazi in trattini", () => {
    expect(slugFile("Email Professionale")).toBe("email-professionale");
  });

  it("rimuove caratteri speciali e trattini doppi/bordo", () => {
    expect(slugFile("  Ciao, {{nome}}!  ")).toBe("ciao-nome");
  });

  it("fallback 'prompt' se resta vuoto", () => {
    expect(slugFile("***")).toBe("prompt");
    expect(slugFile("")).toBe("prompt");
  });
});

describe("nomeFileExport", () => {
  it("usa solo la parte data (YYYY-MM-DD) della ISO string", () => {
    expect(nomeFileExport("json", "2026-06-03T10:30:00.000Z")).toBe(
      "prompt-a-porter-export-2026-06-03.json",
    );
  });

  it("normalizza l'estensione con punto iniziale", () => {
    expect(nomeFileExport(".zip", "2026-01-09T00:00:00Z")).toBe(
      "prompt-a-porter-export-2026-01-09.zip",
    );
  });

  it("accetta l'estensione senza punto", () => {
    expect(nomeFileExport("zip", "2026-12-31T23:59:59Z")).toBe(
      "prompt-a-porter-export-2026-12-31.zip",
    );
  });
});
