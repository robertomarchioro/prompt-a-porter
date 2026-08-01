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

  // #506/#514/#553: il layout del diff dipende dalla modalità `altezza`.
  // "contenitore" (default) è usata da CronologiaTab dentro un pannello ad
  // altezza fissa e NON deve regredire; "contenuto" è usata da
  // RitoccoModal, che delega lo scroll a un antenato con solo `max-height`.
  // Il componente espone la modalità attiva via `data-altezza` sul nodo
  // radice, cosa che il CSS usa per decidere fra flex:1/overflow:hidden
  // (contenitore) e flex:none/overflow:visible (contenuto) — vedi lo
  // <style> del componente per il dettaglio del perché.
  it("usa la modalità 'contenitore' di default (comportamento storico di CronologiaTab)", () => {
    const { container } = render(DiffViewer, {
      props: { bodyA: "a", bodyB: "b" },
    });

    const radice = container.querySelector(".diff-viewer");
    expect(radice?.getAttribute("data-altezza")).toBe("contenitore");
  });

  it("passa a 'contenuto' quando richiesto esplicitamente (usato da RitoccoModal)", () => {
    const { container } = render(DiffViewer, {
      props: { bodyA: "a", bodyB: "b", altezza: "contenuto" },
    });

    const radice = container.querySelector(".diff-viewer");
    expect(radice?.getAttribute("data-altezza")).toBe("contenuto");
  });

  it("renderizza il diff in entrambe le modalità senza perdere contenuto", () => {
    const bodyA = "riga uno\nriga due\nriga tre";
    const bodyB = "riga uno\nriga due modificata\nriga tre";

    for (const altezza of ["contenitore", "contenuto"] as const) {
      const { container, unmount } = render(DiffViewer, {
        props: { bodyA, bodyB, altezza },
      });

      expect(container.querySelector(".render")?.textContent).toContain(
        "riga due modificata",
      );

      unmount();
    }
  });
});
