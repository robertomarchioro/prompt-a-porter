import { describe, it, expect, vi, afterEach } from "vitest";

/**
 * Test per il wrapper condiviso di logging (`log-app.ts`), stesso
 * meccanismo già in uso in `installa-aggiornamento-logic.ts` e
 * `ImpostazioniModal.svelte`: scrive via `tauri-plugin-log`, con fallback
 * su `console.*` quando il comando IPC non è disponibile (nessun contesto
 * Tauri, es. in test).
 */

const infoMock = vi.fn();
const erroreMock = vi.fn();

vi.mock("@tauri-apps/plugin-log", () => ({
  info: (msg: string) => infoMock(msg),
  error: (msg: string) => erroreMock(msg),
}));

afterEach(() => {
  vi.restoreAllMocks();
  infoMock.mockReset();
  erroreMock.mockReset();
});

describe("logInfoApp", () => {
  it("scrive il messaggio via il plugin quando l'IPC ha successo", async () => {
    infoMock.mockResolvedValue(undefined);
    const { logInfoApp } = await import("./log-app");

    logInfoApp("[test] riga di prova");
    await Promise.resolve();

    expect(infoMock).toHaveBeenCalledWith("[test] riga di prova");
  });

  it("ricade su console.info senza lanciare se l'IPC non è disponibile", async () => {
    infoMock.mockRejectedValue(new Error("no tauri context"));
    const consoleSpy = vi.spyOn(console, "info").mockImplementation(() => {});
    const { logInfoApp } = await import("./log-app");

    expect(() => logInfoApp("[test] fallback")).not.toThrow();
    await Promise.resolve();
    await Promise.resolve();

    expect(consoleSpy).toHaveBeenCalledWith("[test] fallback");
  });
});

describe("logErroreApp", () => {
  it("scrive il messaggio via il plugin quando l'IPC ha successo", async () => {
    erroreMock.mockResolvedValue(undefined);
    const { logErroreApp } = await import("./log-app");

    logErroreApp("[test] errore di prova");
    await Promise.resolve();

    expect(erroreMock).toHaveBeenCalledWith("[test] errore di prova");
  });

  it("ricade su console.error senza lanciare se l'IPC non è disponibile", async () => {
    erroreMock.mockRejectedValue(new Error("no tauri context"));
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const { logErroreApp } = await import("./log-app");

    expect(() => logErroreApp("[test] fallback errore")).not.toThrow();
    await Promise.resolve();
    await Promise.resolve();

    expect(consoleSpy).toHaveBeenCalledWith("[test] fallback errore");
  });
});
