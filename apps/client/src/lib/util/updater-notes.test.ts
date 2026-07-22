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
});
