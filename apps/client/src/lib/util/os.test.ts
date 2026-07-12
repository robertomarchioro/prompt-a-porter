// @vitest-environment jsdom
import { describe, it, expect, afterAll, vi } from "vitest";

/**
 * Test rilevamento OS per etichette UI (os.ts).
 *
 * `classificaPiattaforma` è pura, quindi testabile direttamente. Il
 * blocco finale mocka `navigator.platform` e re-importa il modulo
 * (come shortcut.test.ts) per verificare le costanti module-level.
 */

import { classificaPiattaforma } from "./os";

describe("classificaPiattaforma", () => {
  it("riconosce Windows da Win32", () => {
    expect(classificaPiattaforma("Win32")).toBe("windows");
  });

  it("riconosce macOS da MacIntel e Mac ARM", () => {
    expect(classificaPiattaforma("MacIntel")).toBe("macos");
    expect(classificaPiattaforma("MacARM")).toBe("macos");
  });

  it("riconosce Linux da Linux x86_64", () => {
    expect(classificaPiattaforma("Linux x86_64")).toBe("linux");
  });

  it("stringa vuota o sconosciuta ricade su linux", () => {
    expect(classificaPiattaforma("")).toBe("linux");
    expect(classificaPiattaforma("FreeBSD amd64")).toBe("linux");
  });
});

describe("costanti module-level", () => {
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

  afterAll(() => {
    if (ORIGINAL_PLATFORM) {
      Object.defineProperty(navigator, "platform", ORIGINAL_PLATFORM);
    }
  });

  it("nomeOS riflette la piattaforma corrente", async () => {
    setPlatform("MacIntel");
    vi.resetModules();
    const { nomeOS, sistemaOperativo } = await import("./os");
    expect(sistemaOperativo).toBe("macos");
    expect(nomeOS).toBe("macOS");
  });

  it("nomeOS su Windows", async () => {
    setPlatform("Win32");
    vi.resetModules();
    const { nomeOS } = await import("./os");
    expect(nomeOS).toBe("Windows");
  });
});
