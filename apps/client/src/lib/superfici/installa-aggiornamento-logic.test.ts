/**
 * Test per `eseguiInstallaAggiornamento` (installa-aggiornamento-logic.ts).
 *
 * Copre gli esiti principali e, soprattutto, il caso che ha motivato
 * l'estrazione: se `conferma()` (o qualunque altro passo) lancia
 * un'eccezione, la funzione non deve mai rifiutare la Promise senza
 * loggare — deve tornare `{ kind: "errore" }` con `logErrore` chiamato.
 */

import { describe, it, expect, vi } from "vitest";
import { eseguiInstallaAggiornamento } from "./installa-aggiornamento-logic";

function creaDipendenze(
  overrides: Partial<{
    conferma: (messaggio: string) => Promise<boolean>;
    check: () => Promise<{ downloadAndInstall: () => Promise<void> } | null>;
    relaunch: () => Promise<void>;
  }> = {},
) {
  const logInfo = vi.fn();
  const logErrore = vi.fn();
  const onConfermato = vi.fn();
  const deps = {
    conferma: overrides.conferma ?? vi.fn().mockResolvedValue(true),
    check:
      overrides.check ??
      vi.fn().mockResolvedValue({
        downloadAndInstall: vi.fn().mockResolvedValue(undefined),
      }),
    relaunch: overrides.relaunch ?? vi.fn().mockResolvedValue(undefined),
    logInfo,
    logErrore,
    onConfermato,
  };
  return { deps, logInfo, logErrore, onConfermato };
}

describe("eseguiInstallaAggiornamento", () => {
  it("annulla senza toccare check/relaunch se la conferma è rifiutata", async () => {
    // Arrange
    const conferma = vi.fn().mockResolvedValue(false);
    const check = vi.fn();
    const { deps, onConfermato } = creaDipendenze({ conferma, check });

    // Act
    const risultato = await eseguiInstallaAggiornamento("1.2.3", deps);

    // Assert
    expect(risultato).toEqual({ kind: "annullato" });
    expect(check).not.toHaveBeenCalled();
    expect(onConfermato).not.toHaveBeenCalled();
  });

  it("percorso felice: conferma → check → downloadAndInstall → relaunch, tutto loggato", async () => {
    // Arrange
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    const check = vi.fn().mockResolvedValue({ downloadAndInstall });
    const relaunch = vi.fn().mockResolvedValue(undefined);
    const { deps, logInfo, onConfermato } = creaDipendenze({ check, relaunch });

    // Act
    const risultato = await eseguiInstallaAggiornamento("1.2.3", deps);

    // Assert
    expect(risultato).toEqual({ kind: "riavviato" });
    expect(onConfermato).toHaveBeenCalledOnce();
    expect(downloadAndInstall).toHaveBeenCalledOnce();
    expect(relaunch).toHaveBeenCalledOnce();
    // Ogni passo chiave ha lasciato una riga: ingresso, esito conferma,
    // check pre/post, download, relaunch.
    expect(logInfo.mock.calls.length).toBeGreaterThanOrEqual(6);
  });

  it("update non più disponibile al check pre-installazione", async () => {
    // Arrange
    const check = vi.fn().mockResolvedValue(null);
    const { deps } = creaDipendenze({ check });

    // Act
    const risultato = await eseguiInstallaAggiornamento("1.2.3", deps);

    // Assert
    expect(risultato).toEqual({ kind: "non_disponibile" });
  });

  it("relaunch() fallito produce riavvio_manuale, non un errore generico", async () => {
    // Arrange
    const relaunch = vi.fn().mockRejectedValue(new Error("exec fallita"));
    const { deps, logErrore } = creaDipendenze({ relaunch });

    // Act
    const risultato = await eseguiInstallaAggiornamento("1.2.3", deps);

    // Assert
    expect(risultato.kind).toBe("riavvio_manuale");
    expect(logErrore).toHaveBeenCalled();
  });

  it("un'eccezione nella conferma non propaga il reject: viene catturata e loggata", async () => {
    // Arrange: questo è il sintomo osservato dal vivo — window.confirm era
    // fuori dal try, un'eccezione qui sarebbe sparita senza log.
    const conferma = vi.fn().mockRejectedValue(new Error("boom"));
    const { deps, logErrore } = creaDipendenze({ conferma });

    // Act / Assert
    await expect(
      eseguiInstallaAggiornamento("1.2.3", deps),
    ).resolves.toEqual({ kind: "errore", messaggio: "Error: boom" });
    expect(logErrore).toHaveBeenCalledWith(
      expect.stringContaining("eccezione non gestita"),
    );
  });

  it("un'eccezione in downloadAndInstall() viene catturata e loggata", async () => {
    // Arrange
    const downloadAndInstall = vi.fn().mockRejectedValue(new Error("disco pieno"));
    const check = vi.fn().mockResolvedValue({ downloadAndInstall });
    const { deps, logErrore } = creaDipendenze({ check });

    // Act
    const risultato = await eseguiInstallaAggiornamento("1.2.3", deps);

    // Assert
    expect(risultato.kind).toBe("errore");
    expect(logErrore).toHaveBeenCalled();
  });
});
