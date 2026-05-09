// @vitest-environment jsdom
import { describe, it, expect, beforeEach, afterAll, vi } from "vitest";

/**
 * Fix #134 — test fmtShortcut OS-aware.
 *
 * Mock di `navigator.platform` per testare entrambi i path mac/win
 * senza dipendere dalla piattaforma CI. Re-import dinamico del modulo
 * dopo ogni mock perché `isMac` è valutato al module load time.
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

afterAll(() => {
  if (ORIGINAL_PLATFORM) {
    Object.defineProperty(navigator, "platform", ORIGINAL_PLATFORM);
  }
});

describe("fmtShortcut su macOS", () => {
  beforeEach(() => {
    setPlatform("MacIntel");
    vi.resetModules();
  });

  it("singolo modifier + lettera", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("mod+k")).toBe("⌘K");
  });

  it("multipli modifier + lettera (ordine preservato)", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("ctrl+shift+p")).toBe("⌃⇧P");
  });

  it("modifier + virgola/punto", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("mod+,")).toBe("⌘,");
    expect(fmtShortcut("mod+.")).toBe("⌘.");
  });

  it("modifier + enter usa glifo ↵", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("ctrl+enter")).toBe("⌃↵");
  });

  it("frecce mantengono glifo Unicode", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("up")).toBe("↑");
    expect(fmtShortcut("down")).toBe("↓");
  });

  it("esc è universale", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("esc")).toBe("Esc");
  });
});

describe("fmtShortcut su Windows/Linux", () => {
  beforeEach(() => {
    setPlatform("Win32");
    vi.resetModules();
  });

  it("singolo modifier + lettera usa Ctrl+", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("mod+k")).toBe("Ctrl+K");
  });

  it("multipli modifier separati da +", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("ctrl+shift+p")).toBe("Ctrl+Shift+P");
  });

  it("ctrl+enter diventa Ctrl+Enter (no ↵)", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("ctrl+enter")).toBe("Ctrl+Enter");
  });

  it("frecce mantengono glifo Unicode anche su Win", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("up")).toBe("↑");
  });

  it("alt diventa Alt", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("alt+f4")).toBe("Alt+F4");
  });
});

describe("fmtShortcut edge cases", () => {
  beforeEach(() => {
    setPlatform("Linux x86_64");
    vi.resetModules();
  });

  it("stringa vuota ritorna stringa vuota", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("")).toBe("");
  });

  it("solo whitespace ritorna stringa vuota", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("  +  ")).toBe("");
  });

  it("trim su parti", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("  mod  +  k  ")).toBe("Ctrl+K");
  });

  it("case-insensitive", async () => {
    const { fmtShortcut } = await import("./shortcut");
    expect(fmtShortcut("MOD+K")).toBe("Ctrl+K");
    expect(fmtShortcut("Ctrl+Shift+P")).toBe("Ctrl+Shift+P");
  });
});
