import { describe, expect, test, vi } from "vitest";
import { eseguiRelaunchPostInstall } from "./updater-relaunch";

describe("eseguiRelaunchPostInstall", () => {
  test("ritorna relaunched quando relaunch() ha successo", async () => {
    // Arrange
    const relaunch = vi.fn().mockResolvedValue(undefined);

    // Act
    const risultato = await eseguiRelaunchPostInstall(relaunch);

    // Assert
    expect(risultato).toEqual({ kind: "relaunched" });
    expect(relaunch).toHaveBeenCalledOnce();
  });

  test("chiede riavvio manuale quando relaunch() fallisce (caso .deb)", async () => {
    // Arrange
    const relaunch = vi.fn().mockRejectedValue(new Error("exec fallita"));

    // Act
    const risultato = await eseguiRelaunchPostInstall(relaunch);

    // Assert
    expect(risultato.kind).toBe("riavvio_manuale_richiesto");
    if (risultato.kind === "riavvio_manuale_richiesto") {
      expect(risultato.messaggio).toContain("Riavvia manualmente");
    }
  });

  test("non propaga l'eccezione di relaunch() al chiamante", async () => {
    // Arrange
    const relaunch = vi.fn().mockRejectedValue("errore stringa qualsiasi");

    // Act / Assert
    await expect(eseguiRelaunchPostInstall(relaunch)).resolves.not.toThrow();
  });
});
