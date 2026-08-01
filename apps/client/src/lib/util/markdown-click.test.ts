// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { gestisciClickLinkMarkdown } from "./markdown-click";

function creaClickSuLink(href: string | null): { evento: MouseEvent; contenitore: HTMLElement } {
  const contenitore = document.createElement("div");
  const link = document.createElement("a");
  if (href !== null) link.setAttribute("href", href);
  link.textContent = "vai";
  contenitore.appendChild(link);
  document.body.appendChild(contenitore);

  const evento = new MouseEvent("click", { bubbles: true, cancelable: true });
  Object.defineProperty(evento, "target", { value: link, enumerable: true });
  return { evento, contenitore };
}

describe("gestisciClickLinkMarkdown", () => {
  it("instrada il click su un link http verso apriUrlEsterno", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue({ ok: true });
    const { evento } = creaClickSuLink("https://example.com/changelog");

    // Act
    const risultato = await gestisciClickLinkMarkdown(evento, apri);

    // Assert
    expect(apri).toHaveBeenCalledWith("https://example.com/changelog");
    expect(risultato).toEqual({ ok: true });
  });

  it("chiama preventDefault sul click intercettato", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue({ ok: true });
    const { evento } = creaClickSuLink("https://example.com");
    const spy = vi.spyOn(evento, "preventDefault");

    // Act
    await gestisciClickLinkMarkdown(evento, apri);

    // Assert
    expect(spy).toHaveBeenCalledOnce();
  });

  it("ignora un click che non è su un link (nessuna chiamata, nessun preventDefault)", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue({ ok: true });
    const contenitore = document.createElement("div");
    const span = document.createElement("span");
    contenitore.appendChild(span);
    const evento = new MouseEvent("click", { bubbles: true, cancelable: true });
    Object.defineProperty(evento, "target", { value: span, enumerable: true });
    const spy = vi.spyOn(evento, "preventDefault");

    // Act
    const risultato = await gestisciClickLinkMarkdown(evento, apri);

    // Assert
    expect(risultato).toBeNull();
    expect(apri).not.toHaveBeenCalled();
    expect(spy).not.toHaveBeenCalled();
  });

  it("ignora un <a> senza href", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue({ ok: true });
    const { evento } = creaClickSuLink(null);

    // Act
    const risultato = await gestisciClickLinkMarkdown(evento, apri);

    // Assert
    expect(risultato).toBeNull();
    expect(apri).not.toHaveBeenCalled();
  });

  it("delega la validazione dello schema ad apriUrlEsterno (es. rifiuta javascript:)", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue({
      ok: false,
      motivo: "schema_non_ammesso",
      messaggio: "Schema non ammesso",
    });
    const { evento } = creaClickSuLink("javascript:alert(1)");

    // Act
    const risultato = await gestisciClickLinkMarkdown(evento, apri);

    // Assert
    expect(apri).toHaveBeenCalledWith("javascript:alert(1)");
    expect(risultato).toEqual({
      ok: false,
      motivo: "schema_non_ammesso",
      messaggio: "Schema non ammesso",
    });
  });

  it("trova il link anche cliccando su un figlio annidato (event delegation via closest)", async () => {
    // Arrange
    const apri = vi.fn().mockResolvedValue({ ok: true });
    const contenitore = document.createElement("div");
    const link = document.createElement("a");
    link.setAttribute("href", "https://example.com/nested");
    const strong = document.createElement("strong");
    strong.textContent = "testo enfatizzato";
    link.appendChild(strong);
    contenitore.appendChild(link);
    document.body.appendChild(contenitore);

    const evento = new MouseEvent("click", { bubbles: true, cancelable: true });
    Object.defineProperty(evento, "target", { value: strong, enumerable: true });

    // Act
    const risultato = await gestisciClickLinkMarkdown(evento, apri);

    // Assert
    expect(apri).toHaveBeenCalledWith("https://example.com/nested");
    expect(risultato).toEqual({ ok: true });
  });
});
