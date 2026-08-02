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
 * - strumentazione diagnostica (#585/#572/#584): la riga di log scritta a
 *   ogni chiamata non contiene mai il testo del messaggio dell'utente.
 *
 * `vi.resetModules()` + import dinamico dopo aver mockato
 * `navigator.platform`: stessa strategia di os.test.ts, necessaria perché
 * `sistemaOperativo` (da cui dipende conferma.ts) è calcolato una volta al
 * module load.
 */

// `@tauri-apps/plugin-log` chiama `invoke()` di `@tauri-apps/api/core`, che
// fuori da un contesto Tauri rifiuta la Promise (nessun
// `window.__TAURI_INTERNALS__`): mockato qui per osservare le righe scritte
// invece di limitarsi a verificare il fallback silenzioso su `console.*`.
const infoSpy = vi.fn().mockResolvedValue(undefined);
vi.mock("@tauri-apps/plugin-log", () => ({
  info: (msg: string) => infoSpy(msg),
  error: () => Promise.resolve(),
}));

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
  infoSpy.mockClear();
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

describe("conferma()/avvisa() — strumentazione diagnostica (#585 #572 #584)", () => {
  it("logga una riga di diagnostica ad ogni conferma(), senza testo utente", async () => {
    const { conferma } = await importConPiattaforma("Win32");
    vi.spyOn(window, "confirm").mockReturnValue(true);

    const titoloSensibile = 'Eliminare il prompt "Preventivo Rossi Spa"?';
    await conferma(titoloSensibile, "elimina-prompt");

    expect(infoSpy).toHaveBeenCalledTimes(1);
    const riga = infoSpy.mock.calls[0][0] as string;
    expect(riga).toContain("azione=elimina-prompt");
    expect(riga).toContain("ramo=nativo");
    expect(riga).toContain("esitoTipo=boolean esitoValore=true");
    expect(riga).not.toContain("Preventivo Rossi Spa");
    expect(riga).not.toContain(titoloSensibile);
  });

  it("azione è opzionale: i chiamanti esistenti senza terzo argomento non si rompono", async () => {
    const { conferma } = await importConPiattaforma("Win32");
    vi.spyOn(window, "confirm").mockReturnValue(false);

    await expect(conferma("Sicuro?")).resolves.toBe(false);

    const riga = infoSpy.mock.calls[0][0] as string;
    expect(riga).toContain("azione=(non specificata)");
  });

  it("distingue nella riga il ramo macos da quello nativo", async () => {
    const { conferma, dialogo } = await importConPiattaforma("MacIntel");

    const promessa = conferma("Promuovere questa variante?", "promuovi-variante");
    const richiesta = dialogo.statoDialoghi.coda[0];
    if (richiesta.tipo !== "conferma") throw new Error("tipo inatteso");
    richiesta.risolvi(true);
    await promessa;

    const riga = infoSpy.mock.calls[0][0] as string;
    expect(riga).toContain("ramo=macos");
    expect(riga).toContain("azione=promuovi-variante");
  });

  it("avvisa() logga azione e lunghezza, mai il testo del messaggio", async () => {
    const { avvisa } = await importConPiattaforma("Win32");
    vi.spyOn(window, "alert").mockImplementation(() => {});

    const messaggioSensibile = 'Errore durante la promozione di "Preventivo Rossi Spa"';
    await avvisa(messaggioSensibile, "promuovi-variante");

    const riga = infoSpy.mock.calls[0][0] as string;
    expect(riga).toContain("azione=promuovi-variante");
    expect(riga).toContain(`lunghezzaMessaggio=${messaggioSensibile.length}`);
    expect(riga).not.toContain("Preventivo Rossi Spa");
  });
});
