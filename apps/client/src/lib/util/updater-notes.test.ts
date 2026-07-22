// @vitest-environment jsdom
import { describe, it, expect } from "vitest";
import { renderNotesHtml } from "./updater-notes";

describe("renderNotesHtml", () => {
  it("converte titoli, elenchi e grassetto in HTML", () => {
    // Arrange
    const markdown = "# Novità\n\n- **Importante**: fix vault\n- Altro punto";

    // Act
    const html = renderNotesHtml(markdown);

    // Assert
    expect(html).toContain("<h1>Novità</h1>");
    expect(html).toContain("<li>");
    expect(html).toContain("<strong>Importante</strong>");
  });

  it("rimuove i tag <script> (XSS via markdown/HTML inline)", () => {
    // Arrange
    const payload = "Testo normale\n\n<script>alert(1)</script>";

    // Act
    const html = renderNotesHtml(payload);

    // Assert
    expect(html).not.toContain("<script");
    expect(html).not.toContain("alert(1)");
  });

  it("rimuove gli handler inline come onerror da un <img>", () => {
    // Arrange
    const payload = '<img src="x" onerror="alert(1)">';

    // Act
    const html = renderNotesHtml(payload);

    // Assert
    expect(html).not.toContain("onerror");
  });

  it("apre i link markdown in una nuova scheda (target=_blank, rel noopener)", () => {
    // Arrange
    const markdown = "Vedi [changelog](https://example.com/CHANGELOG.md)";

    // Act
    const html = renderNotesHtml(markdown);

    // Assert
    expect(html).toContain('target="_blank"');
    expect(html).toMatch(/rel="[^"]*noopener[^"]*"/);
  });

  it("neutralizza un link javascript: (XSS via href)", () => {
    // Arrange
    const markdown = "[clicca qui](javascript:alert(1))";

    // Act
    const html = renderNotesHtml(markdown);

    // Assert
    expect(html).not.toContain("javascript:");
  });

  it("rimuove il tag <style> e l'attributo style", () => {
    // Arrange
    const payload =
      '<style>body{display:none}</style><p style="color:red">testo</p>';

    // Act
    const html = renderNotesHtml(payload);

    // Assert
    expect(html).not.toContain("<style");
    expect(html).not.toContain("style=");
  });
});
