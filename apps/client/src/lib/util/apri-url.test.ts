import { describe, expect, it, vi } from "vitest";
import { apriUrlEsterno } from "./apri-url";

describe("apriUrlEsterno", () => {
  it("apre un URL https delegando alla funzione di apertura iniettata", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await apriUrlEsterno(
      "https://prompt-a-porter.app/docs",
      apri,
    );

    // Assert
    expect(risultato).toEqual({ ok: true });
    expect(apri).toHaveBeenCalledWith("https://prompt-a-porter.app/docs");
  });

  it("apre un URL http", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await apriUrlEsterno("http://example.com", apri);

    // Assert
    expect(risultato.ok).toBe(true);
    expect(apri).toHaveBeenCalledOnce();
  });

  it("rifiuta lo schema file: senza chiamare la funzione di apertura", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await apriUrlEsterno("file:///etc/passwd", apri);

    // Assert
    expect(risultato).toEqual({
      ok: false,
      motivo: "schema_non_ammesso",
      messaggio: expect.stringContaining("file:"),
    });
    expect(apri).not.toHaveBeenCalled();
  });

  it("rifiuta uno schema custom senza chiamare la funzione di apertura", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await apriUrlEsterno("ms-settings:privacy", apri);

    // Assert
    expect(risultato.ok).toBe(false);
    if (!risultato.ok) {
      expect(risultato.motivo).toBe("schema_non_ammesso");
    }
    expect(apri).not.toHaveBeenCalled();
  });

  it("rifiuta javascript: senza chiamare la funzione di apertura", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await apriUrlEsterno("javascript:alert(1)", apri);

    // Assert
    expect(risultato.ok).toBe(false);
    expect(apri).not.toHaveBeenCalled();
  });

  it("ritorna url_non_valido per una stringa non parseable come URL", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await apriUrlEsterno("non-un-url", apri);

    // Assert
    expect(risultato.ok).toBe(false);
    if (!risultato.ok) {
      expect(risultato.motivo).toBe("url_non_valido");
    }
    expect(apri).not.toHaveBeenCalled();
  });

  it("non lancia e riporta apertura_fallita quando il plugin rifiuta", async () => {
    // Arrange
    const apri = vi.fn().mockRejectedValue(new Error("plugin non disponibile"));

    // Act
    const risultato = await apriUrlEsterno("https://example.com", apri);

    // Assert
    expect(risultato).toEqual({
      ok: false,
      motivo: "apertura_fallita",
      messaggio: "plugin non disponibile",
    });
  });

  it("non propaga eccezioni al chiamante", async () => {
    // Arrange
    const apri = vi.fn().mockRejectedValue("errore non-Error");

    // Act / Assert
    await expect(apriUrlEsterno("https://example.com", apri)).resolves.not.toThrow();
  });
});
