// @vitest-environment jsdom
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/svelte";
import DiffViewer from "./DiffViewer.svelte";
// Import Vite `?raw` (sorgente come stringa), tipizzato via vite-env.d.ts.
import diffViewerSorgente from "./DiffViewer.svelte?raw";

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

  // #574 (QUARTA correzione): diff2html posiziona i numeri di riga
  // (`.d2h-code-linenumber`/`.d2h-code-side-linenumber`) con
  // `position:absolute` ma senza `top`/`left` e senza dichiarare
  // `position:relative` su nessun proprio antenato (`.d2h-file-wrapper`,
  // `.d2h-file-diff`, `.d2h-code-line` sono tutti `static` — verificato
  // in `diff2html/bundles/css/diff2html.min.css`). Il containing block
  // di un discendente `absolute` NON è creato da `overflow:auto` da
  // solo: senza un antenato posizionato dentro il blocco che scorre, il
  // gutter finisce ancorato al primo antenato positioned che trova —
  // nella modale è il `.backdrop` `position:fixed` di Modale.svelte, non
  // il contenuto che scorre. Da qui l'overlap riportato in #506/#514/#553.
  //
  // jsdom NON esegue layout (nessun box model reale, nessuno scroll):
  // questo test verifica SOLO che `.render` dichiari `position:relative`
  // nella regola CSS del componente, così che il containing block del
  // gutter sia sempre dentro l'antenato che scrolla davvero (`.render`
  // stesso in modalità "contenitore", `.diff-scroll` del consumer in
  // modalità "contenuto" — mai il backdrop fisso della modale). NON
  // dimostra l'assenza dell'overlap in WKWebView/WebView2: quella
  // verifica richiede la webview reale, non riproducibile in CI/jsdom —
  // esattamente la ragione delle tre recidive precedenti.
  it("'.render' dichiara position:relative, containing block del gutter numerico diff2html", () => {
    const { container } = render(DiffViewer, {
      props: { bodyA: "a", bodyB: "b" },
    });

    // Il nodo che ospita l'HTML iniettato via `{@html}` esiste.
    expect(container.querySelector(".render")).not.toBeNull();

    // jsdom non applica in modo affidabile la cascata CSS da <style>
    // scoped iniettati da Svelte (né esegue layout): leggiamo quindi il
    // sorgente del componente (`?raw`, Vite) e verifichiamo direttamente
    // la regola `.render` — non prova l'assenza dell'overlap dal vivo,
    // blocca solo la regressione della dichiarazione CSS.
    const bloccoStile = diffViewerSorgente.match(/<style>([\s\S]*)<\/style>/);
    expect(bloccoStile).not.toBeNull();

    // Rimuove i commenti CSS prima del match: possono contenere `{`/`}`
    // (es. l'inciso su `{@html}` qui sopra) che spezzerebbero altrimenti
    // il match "fino alla prima `}`".
    const stileSenzaCommenti = (bloccoStile?.[1] ?? "").replace(
      /\/\*[\s\S]*?\*\//g,
      "",
    );
    const regolaRender = stileSenzaCommenti.match(/\.render\s*\{[^}]*\}/);
    expect(regolaRender).not.toBeNull();
    expect(regolaRender?.[0]).toMatch(/position:\s*relative;/);
  });
});
