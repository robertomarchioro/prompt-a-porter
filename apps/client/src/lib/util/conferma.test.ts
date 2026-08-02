// @vitest-environment jsdom
import { describe, it, expect, vi, afterEach } from "vitest";

/**
 * Test per i sostituti macOS di `window.confirm`/`window.alert`
 * (`conferma.ts`). Copre:
 * - fuori da macOS: delega invariata a `window.confirm`/`window.alert`
 * - su macOS: la richiesta viene accodata in `dialogo.svelte.ts` (nessun
 *   dialogo nativo), e la Promise si risolve solo quando l'host chiama
 *   `risolvi` sulla richiesta in coda — esattamente come farebbe
 *   `DialogoHost.svelte` in risposta al click dell'utente.
 *
 * `vi.resetModules()` + import dinamico dopo aver mockato
 * `navigator.platform`: stessa strategia di os.test.ts, necessaria perché
 * `sistemaOperativo` (da cui dipende conferma.ts) è calcolato una volta al
 * module load.
 */

const ORIGINAL_PLATFORM = Object.getOwnPropertyDescriptor(
  globalThis.navigator,
  "platform",
);

function setPlatform(value: string): void {
  Object.defineProperty(navigator, "platform", {
    configurable: true,
    get: () => value,
  });
}

async function importConPiattaforma(platform: string) {
  vi.resetModules();
  setPlatform(platform);
  const conferma = await import("./conferma");
  const dialogo = await import("$lib/stores/dialogo.svelte");
  return { ...conferma, dialogo };
}

afterEach(() => {
  vi.restoreAllMocks();
  if (ORIGINAL_PLATFORM) {
    Object.defineProperty(navigator, "platform", ORIGINAL_PLATFORM);
  }
});

describe("conferma() fuori da macOS", () => {
  it("delega a window.confirm e ne ritorna il valore", async () => {
    const { conferma } = await importConPiattaforma("Win32");
    const spy = vi.spyOn(window, "confirm").mockReturnValue(true);

    await expect(conferma("Sicuro?")).resolves.toBe(true);

    expect(spy).toHaveBeenCalledWith("Sicuro?");
  });

  it("propaga l'annullamento (false) di window.confirm", async () => {
    const { conferma } = await importConPiattaforma("Linux x86_64");
    vi.spyOn(window, "confirm").mockReturnValue(false);

    await expect(conferma("Sicuro?")).resolves.toBe(false);
  });
});

describe("avvisa() fuori da macOS", () => {
  it("delega a window.alert con lo stesso messaggio", async () => {
    const { avvisa } = await importConPiattaforma("Win32");
    const spy = vi.spyOn(window, "alert").mockImplementation(() => {});

    await avvisa("Errore nel salvataggio");

    expect(spy).toHaveBeenCalledWith("Errore nel salvataggio");
  });
});

describe("conferma() su macOS", () => {
  it("accoda la richiesta invece di chiamare window.confirm", async () => {
    const { conferma, dialogo } = await importConPiattaforma("MacIntel");
    const spy = vi.spyOn(window, "confirm");

    const promessa = conferma("Eliminare il prompt?");

    expect(spy).not.toHaveBeenCalled();
    expect(dialogo.statoDialoghi.coda).toHaveLength(1);
    expect(dialogo.statoDialoghi.coda[0].messaggio).toBe(
      "Eliminare il prompt?",
    );

    const richiesta = dialogo.statoDialoghi.coda[0];
    if (richiesta.tipo !== "conferma") throw new Error("tipo inatteso");
    richiesta.risolvi(true);

    await expect(promessa).resolves.toBe(true);
  });

  it("risolve false quando l'host annulla (es. Esc/click su Annulla)", async () => {
    const { conferma, dialogo } = await importConPiattaforma("MacARM");

    const promessa = conferma("Svuotare il cestino?");
    const richiesta = dialogo.statoDialoghi.coda[0];
    if (richiesta.tipo !== "conferma") throw new Error("tipo inatteso");
    richiesta.risolvi(false);

    await expect(promessa).resolves.toBe(false);
  });
});

describe("avvisa() su macOS", () => {
  it("accoda un avviso invece di chiamare window.alert", async () => {
    const { avvisa, dialogo } = await importConPiattaforma("MacIntel");
    const spy = vi.spyOn(window, "alert");

    const promessa = avvisa("Errore nell'eliminazione");

    expect(spy).not.toHaveBeenCalled();
    expect(dialogo.statoDialoghi.coda).toHaveLength(1);

    const richiesta = dialogo.statoDialoghi.coda[0];
    if (richiesta.tipo !== "avviso") throw new Error("tipo inatteso");
    richiesta.risolvi();

    await expect(promessa).resolves.toBeUndefined();
  });
});
