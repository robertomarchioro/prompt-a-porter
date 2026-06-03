import { describe, it, expect } from "vitest";
import { nomeFileExport } from "./dati-export";

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
