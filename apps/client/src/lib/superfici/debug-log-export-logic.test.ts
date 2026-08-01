/**
 * Test per l'export ZIP dei debug log (issue #558 punto 2).
 *
 * Fix HIGH (review avversariale pre-merge PR #570): il dialog di
 * salvataggio si apre ora lato Rust, dentro il comando
 * `debug_log_esporta_zip` — il frontend non gli passa più alcun percorso
 * di destinazione. Copre:
 * - annullamento del dialog (`null`) gestito senza errore
 * - invocazione del comando senza argomenti (nessun `destinazione`)
 * - successo ed errore reale
 */

import { describe, it, expect, vi } from "vitest";
import { eseguiEsportaLogZip } from "./debug-log-export-logic";

describe("eseguiEsportaLogZip", () => {
  it("invoca il comando senza passare alcun argomento di destinazione", async () => {
    // Arrange
    const invokeFn = vi.fn().mockResolvedValue("/home/utente/pap-debug-log.zip");

    // Act
    await eseguiEsportaLogZip(invokeFn);

    // Assert: un solo argomento (il nome del comando), niente `destinazione`
    // — il percorso non deve mai più attraversare il boundary IPC.
    expect(invokeFn).toHaveBeenCalledTimes(1);
    expect(invokeFn).toHaveBeenCalledWith("debug_log_esporta_zip");
  });

  it("annullamento del dialog nativo (null) non è un errore", async () => {
    // Arrange
    const invokeFn = vi.fn().mockResolvedValue(null);

    // Act
    const risultato = await eseguiEsportaLogZip(invokeFn);

    // Assert
    expect(risultato.ok).toBe(false);
    expect(risultato.annullato).toBe(true);
    expect(risultato.errore).toBeUndefined();
  });

  it("ritorna il path dello zip creato al successo", async () => {
    // Arrange
    const invokeFn = vi.fn().mockResolvedValue("/home/utente/pap-debug-log.zip");

    // Act
    const risultato = await eseguiEsportaLogZip(invokeFn);

    // Assert
    expect(risultato).toEqual({ ok: true, zipPath: "/home/utente/pap-debug-log.zip" });
  });

  it("un fallimento reale del comando produce un messaggio d'errore, non un annullamento silenzioso", async () => {
    // Arrange
    const invokeFn = vi.fn().mockRejectedValue(new Error("Creazione dell'archivio dei log non riuscita."));

    // Act
    const risultato = await eseguiEsportaLogZip(invokeFn);

    // Assert
    expect(risultato.ok).toBe(false);
    expect(risultato.annullato).toBeUndefined();
    expect(risultato.errore).toBe("Creazione dell'archivio dei log non riuscita.");
  });
});
