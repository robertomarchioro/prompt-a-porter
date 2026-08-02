// @vitest-environment jsdom
import { describe, it, expect, vi, afterEach } from "vitest";

/**
 * Test per `conferma()`/`avvisa()` (`conferma.ts`), che dalla correzione del
 * blocco dell'aggiornamento automatico usano la coda in-app **su tutte le
 * piattaforme**.
 *
 * Il test più importante di questo file è la guardia anti-regressione:
 * `window.confirm`/`window.alert` non devono MAI essere chiamate. Quelle
 * funzioni globali sono sostituite da `tauri-plugin-dialog` con invocazioni
 * di `plugin:dialog|confirm`, comando che la crate non registra: ogni
 * chiamata fallisce con «not allowed by ACL». È la causa unica di #585,
 * #572, #584 e del blocco su «Installa e riavvia». Se qualcuno
 * reintroducesse quella dipendenza, questi test devono diventare rossi.
 *
 * Copre inoltre:
 * - la Promise si risolve solo quando l'host risolve la richiesta in coda,
 *   esattamente come farebbe `DialogoHost.svelte` al click dell'utente;
 * - il vincolo di privacy: la riga di log non contiene mai il testo del
 *   messaggio, che tipicamente include il titolo di un prompt.
 *
 * `vi.resetModules()` + import dinamico dopo aver mockato
 * `navigator.platform`: stessa strategia di os.test.ts, qui usata per
 * verificare che il comportamento sia identico su Windows, Linux e macOS.
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

/**
 * Simula `DialogoHost.svelte`: attende che la richiesta compaia in coda e la
 * risolve con l'esito indicato, come farebbe il click dell'utente.
 */
async function rispondiAllaCoda(
  dialogo: typeof import("$lib/stores/dialogo.svelte"),
  esito: boolean,
): Promise<void> {
  for (let i = 0; i < 20 && dialogo.statoDialoghi.coda.length === 0; i++) {
    await Promise.resolve();
  }
  const richiesta = dialogo.statoDialoghi.coda[0];
  expect(richiesta, "la richiesta deve essere stata accodata").toBeDefined();
  if (richiesta.tipo === "conferma") richiesta.risolvi(esito);
  else richiesta.risolvi();
  dialogo.rimuoviDallaCoda(richiesta.id);
}

afterEach(() => {
  vi.restoreAllMocks();
  infoSpy.mockClear();
  if (ORIGINAL_PLATFORM) {
    Object.defineProperty(navigator, "platform", ORIGINAL_PLATFORM);
  }
});

describe("conferma() — coda in-app su ogni piattaforma", () => {
  for (const [nome, platform] of [
    ["Windows", "Win32"],
    ["Linux", "Linux x86_64"],
    ["macOS", "MacIntel"],
  ] as const) {
    it(`su ${nome} accoda invece di chiamare window.confirm`, async () => {
      const { conferma, dialogo } = await importConPiattaforma(platform);
      const confirmSpy = vi.spyOn(window, "confirm");

      const promessa = conferma("Eliminare il prompt?", "elimina-prompt");
      await rispondiAllaCoda(dialogo, true);

      expect(await promessa).toBe(true);
      expect(confirmSpy).not.toHaveBeenCalled();
    });
  }

  it("ritorna false quando l'host risolve con un rifiuto", async () => {
    const { conferma, dialogo } = await importConPiattaforma("Win32");
    const promessa = conferma("Eliminare il prompt?", "elimina-prompt");
    await rispondiAllaCoda(dialogo, false);
    expect(await promessa).toBe(false);
  });

  it("il messaggio accodato è quello passato dal chiamante", async () => {
    const { conferma, dialogo } = await importConPiattaforma("Win32");
    const promessa = conferma("Eliminare il prompt?", "elimina-prompt");
    for (let i = 0; i < 20 && dialogo.statoDialoghi.coda.length === 0; i++) {
      await Promise.resolve();
    }
    expect(dialogo.statoDialoghi.coda[0].messaggio).toBe(
      "Eliminare il prompt?",
    );
    await rispondiAllaCoda(dialogo, true);
    await promessa;
  });
});

describe("avvisa() — Toast in-app su ogni piattaforma", () => {
  it("accoda invece di chiamare window.alert", async () => {
    const { avvisa, dialogo } = await importConPiattaforma("Win32");
    const alertSpy = vi.spyOn(window, "alert");

    const promessa = avvisa("Errore durante l'eliminazione", "elimina-prompt");
    await rispondiAllaCoda(dialogo, true);

    await promessa;
    expect(alertSpy).not.toHaveBeenCalled();
  });
});

describe("strumentazione diagnostica (#585 #572 #584)", () => {
  it("logga azione, ramo e lunghezza, mai il testo del messaggio", async () => {
    const { conferma, dialogo } = await importConPiattaforma("Win32");
    const titoloSensibile = 'Eliminare il prompt "Preventivo Rossi Spa"?';

    const promessa = conferma(titoloSensibile, "elimina-prompt");
    await rispondiAllaCoda(dialogo, true);
    await promessa;

    const riga = infoSpy.mock.calls[0][0] as string;
    expect(riga).toContain("azione=elimina-prompt");
    expect(riga).toContain("ramo=in-app");
    expect(riga).toContain(`lunghezzaMessaggio=${titoloSensibile.length}`);
    // Il vincolo che conta: nessun frammento del titolo nel log.
    expect(riga).not.toContain("Preventivo Rossi Spa");
    expect(riga).not.toContain(titoloSensibile);
  });

  it("avvisa() logga senza far trapelare il messaggio", async () => {
    const { avvisa, dialogo } = await importConPiattaforma("Win32");
    const messaggio =
      'Errore sul prompt "Preventivo Rossi Spa": accesso negato';

    const promessa = avvisa(messaggio, "elimina-prompt");
    await rispondiAllaCoda(dialogo, true);
    await promessa;

    const riga = infoSpy.mock.calls[0][0] as string;
    expect(riga).toContain("[avvisa]");
    expect(riga).toContain("ramo=in-app");
    expect(riga).not.toContain("Preventivo Rossi Spa");
  });
});
