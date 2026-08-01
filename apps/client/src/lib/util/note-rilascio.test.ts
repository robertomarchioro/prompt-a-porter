import { describe, expect, it, vi } from "vitest";
import { recuperaNoteRilascio } from "./note-rilascio";

describe("recuperaNoteRilascio", () => {
  it("usa la sezione del changelog quando il recupero riesce", async () => {
    // Arrange
    const invocaChangelog = vi.fn().mockResolvedValue("## v0.8.44 — Titolo\n\nCorpo.");

    // Act
    const risultato = await recuperaNoteRilascio({
      versione: "0.8.44",
      corpoRelease: "Corpo release GitHub (template fisso)",
      invocaChangelog,
    });

    // Assert
    expect(risultato).toEqual({
      testo: "## v0.8.44 — Titolo\n\nCorpo.",
      fonte: "changelog",
    });
    expect(invocaChangelog).toHaveBeenCalledWith("0.8.44");
  });

  it("ricade sul corpo della release se il changelog remoto fallisce (rete assente)", async () => {
    // Arrange
    const invocaChangelog = vi.fn().mockRejectedValue(new Error("rete assente"));

    // Act
    const risultato = await recuperaNoteRilascio({
      versione: "0.8.44",
      corpoRelease: "Corpo release GitHub",
      invocaChangelog,
    });

    // Assert
    expect(risultato).toEqual({ testo: "Corpo release GitHub", fonte: "release" });
  });

  it("ricade sul corpo della release se la sezione del changelog è vuota", async () => {
    // Arrange
    const invocaChangelog = vi.fn().mockResolvedValue("   ");

    // Act
    const risultato = await recuperaNoteRilascio({
      versione: "0.8.44",
      corpoRelease: "Corpo release GitHub",
      invocaChangelog,
    });

    // Assert
    expect(risultato.fonte).toBe("release");
  });

  it("ritorna fonte 'nessuna' se sia il changelog che il corpo release sono assenti", async () => {
    // Arrange
    const invocaChangelog = vi.fn().mockRejectedValue(new Error("404"));

    // Act
    const risultato = await recuperaNoteRilascio({
      versione: "0.8.44",
      corpoRelease: "",
      invocaChangelog,
    });

    // Assert
    expect(risultato).toEqual({ testo: "", fonte: "nessuna" });
  });

  it("non propaga mai l'eccezione del recupero changelog al chiamante", async () => {
    // Arrange
    const invocaChangelog = vi.fn().mockRejectedValue(new Error("sezione non trovata"));

    // Act / Assert
    await expect(
      recuperaNoteRilascio({
        versione: "0.8.44",
        corpoRelease: "fallback",
        invocaChangelog,
      }),
    ).resolves.not.toThrow();
  });
});
