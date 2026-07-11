// @vitest-environment jsdom
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/svelte";
import DiffViewer from "./DiffViewer.svelte";

// #462 (security review, LOW): DiffViewer usa `{@html renderHtml}` per
// mostrare l'output di diff2html. Oggi diff2html esegue l'escape
// dell'HTML del contenuto diffato, quindi è sicuro — ma un futuro bump
// della libreria potrebbe silenziosamente cambiare comportamento.
// Questo test da regressione fallisce se un body con `<script>` finisce
// nel DOM come nodo eseguibile invece che come testo innocuo.
describe("DiffViewer", () => {
  it("un body con <script> viene renderizzato come testo, non come nodo script", () => {
    const bodyA = "riga sicura invariata";
    const bodyB = "riga sicura invariata\n<script>alert(1)</script>";

    const { container } = render(DiffViewer, {
      props: { bodyA, bodyB },
    });

    // Nessun elemento <script> deve comparire nel DOM renderizzato.
    expect(container.querySelectorAll("script")).toHaveLength(0);

    // Il markup iniettato deve comparire come testo visibile (escapato),
    // non essere silenziosamente scartato o interpretato.
    expect(container.textContent).toContain("<script>alert(1)</script>");
  });
});
