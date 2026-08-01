// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte";
import AiutoLink from "./AiutoLink.svelte";
import { urlDoc } from "./docs-links";

// Fix #555: senza un plugin opener registrato un semplice
// `<a target="_blank">` non naviga in una webview Tauri. Questo test
// verifica che il click sia intercettato e delegato al plugin opener
// (qui mockato) invece di lasciare fare al browser/webview il default.
const apriUrlEsternoMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: apriUrlEsternoMock,
}));

describe("AiutoLink", () => {
  beforeEach(() => {
    apriUrlEsternoMock.mockReset();
    apriUrlEsternoMock.mockResolvedValue(undefined);
  });

  afterEach(cleanup);

  it("intercetta il click e apre l'URL della guida via plugin opener", async () => {
    // Arrange
    const { getByRole } = render(AiutoLink, {
      props: { chiave: "getting-started" },
    });
    const link = getByRole("link");

    // Act
    const evento = await fireEvent.click(link);

    // Assert: preventDefault chiamato (niente navigazione webview) e
    // apertura delegata al plugin opener con lo stesso URL dell'href.
    expect(evento).toBe(false); // fireEvent ritorna false se defaultPrevented
    await vi.waitFor(() => {
      expect(apriUrlEsternoMock).toHaveBeenCalledWith(
        urlDoc("getting-started"),
      );
    });
  });

  it("il link espone comunque l'href reale per accessibilità/copia-link", () => {
    // Arrange / Act
    const { getByRole } = render(AiutoLink, {
      props: { chiave: "getting-started" },
    });
    const link = getByRole("link") as HTMLAnchorElement;

    // Assert
    expect(link.getAttribute("href")).toBe(urlDoc("getting-started"));
  });
});
