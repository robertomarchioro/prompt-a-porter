import { describe, expect, it } from "vitest";
import {
  diffMarkdown,
  diffParole,
  diffSideBySide,
  statiDiff,
} from "./index";

describe("diffParole", () => {
  it("ritorna un solo segmento uguale se i testi sono identici", () => {
    const r = diffParole("ciao mondo", "ciao mondo");
    expect(r).toHaveLength(1);
    expect(r[0].tipo).toBe("uguale");
    expect(r[0].testo).toBe("ciao mondo");
  });

  it("rileva una parola aggiunta in mezzo", () => {
    const r = diffParole("ciao mondo", "ciao bel mondo");
    expect(r.some((s) => s.tipo === "aggiunto" && s.testo.includes("bel"))).toBe(true);
  });

  it("rileva una parola rimossa", () => {
    const r = diffParole("ciao bel mondo", "ciao mondo");
    expect(r.some((s) => s.tipo === "rimosso" && s.testo.includes("bel"))).toBe(true);
  });

  it("preserva i segnaposti {{nome}} come token unitari", () => {
    const a = "Saluta {{nome}} con tono formale";
    const b = "Saluta {{nome}} con tono informale";
    const r = diffParole(a, b);
    // Il segnaposto deve apparire intero in almeno un segmento "uguale".
    const segUguali = r.filter((s) => s.tipo === "uguale").map((s) => s.testo).join("");
    expect(segUguali).toContain("{{nome}}");
  });

  it("differenzia testo completamente diverso", () => {
    const r = diffParole("alpha", "beta");
    expect(r.some((s) => s.tipo === "rimosso")).toBe(true);
    expect(r.some((s) => s.tipo === "aggiunto")).toBe(true);
  });
});

describe("diffSideBySide", () => {
  it("identici producono N righe `uguale`", () => {
    const r = diffSideBySide("riga 1\nriga 2\nriga 3", "riga 1\nriga 2\nriga 3");
    expect(r).toHaveLength(3);
    expect(r.every((x) => x.stato === "uguale")).toBe(true);
    expect(r[0].numeroA).toBe(1);
    expect(r[0].numeroB).toBe(1);
  });

  it("aggiunta in coda ⇒ riga 'aggiunta' senza pair A", () => {
    const r = diffSideBySide("a\nb", "a\nb\nc");
    const aggiunta = r.find((x) => x.stato === "aggiunta");
    expect(aggiunta).toBeDefined();
    expect(aggiunta!.testoB).toBe("c");
    expect(aggiunta!.testoA).toBeNull();
    expect(aggiunta!.numeroA).toBeNull();
    expect(aggiunta!.numeroB).toBe(3);
  });

  it("rimozione in mezzo ⇒ riga 'rimossa' senza pair B", () => {
    const r = diffSideBySide("a\nb\nc", "a\nc");
    const rimossa = r.find((x) => x.stato === "rimossa");
    expect(rimossa).toBeDefined();
    expect(rimossa!.testoA).toBe("b");
    expect(rimossa!.testoB).toBeNull();
  });

  it("modifica di una riga ⇒ riga 'cambiata' con entrambi i lati", () => {
    const r = diffSideBySide("alpha", "alpha modificato");
    const cambiata = r.find((x) => x.stato === "cambiata");
    expect(cambiata).toBeDefined();
    expect(cambiata!.testoA).toBe("alpha");
    expect(cambiata!.testoB).toBe("alpha modificato");
    expect(cambiata!.numeroA).toBe(1);
    expect(cambiata!.numeroB).toBe(1);
  });

  it("numerazione consistente con righe identiche prima/dopo modifica", () => {
    const r = diffSideBySide("a\nb\nc", "a\nB-mod\nc");
    // Riga 1 ('a') uguale, 2 ('b' → 'B-mod') cambiata, 3 ('c') uguale.
    expect(r).toHaveLength(3);
    expect(r[0].stato).toBe("uguale");
    expect(r[1].stato).toBe("cambiata");
    expect(r[2].stato).toBe("uguale");
    expect(r[2].numeroA).toBe(3);
    expect(r[2].numeroB).toBe(3);
  });
});

describe("diffMarkdown", () => {
  it("produce un blocco ```diff con prefissi +/-/space", () => {
    const md = diffMarkdown("alpha\nbeta", "alpha\ngamma");
    expect(md).toContain("```diff");
    expect(md).toContain(" alpha");
    expect(md).toContain("-beta");
    expect(md).toContain("+gamma");
  });

  it("include header quando passi le etichette", () => {
    const md = diffMarkdown("a", "b", { etichettaA: "v1", etichettaB: "v2" });
    expect(md).toMatch(/^--- v1\n\+\+\+ v2/);
  });

  it("nessun header se etichette omesse", () => {
    const md = diffMarkdown("a", "b");
    expect(md.startsWith("```diff")).toBe(true);
  });
});

describe("statiDiff", () => {
  it("conta correttamente righe aggiunte/rimosse", () => {
    const r = statiDiff("a\nb\nc", "a\nx\ny\nz\nc");
    expect(r.rimosse).toBeGreaterThanOrEqual(1);
    expect(r.aggiunte).toBeGreaterThanOrEqual(2);
  });

  it("zero su testi identici", () => {
    expect(statiDiff("alpha", "alpha")).toEqual({ aggiunte: 0, rimosse: 0 });
  });

  it("conta aggiunte quando b estende a (con newline trailing)", () => {
    // Newline trailing su `a` evita l'ambiguità: la riga `a` resta
    // identica e si aggiungono solo `b` e `c`.
    const r = statiDiff("a\n", "a\nb\nc\n");
    expect(r.aggiunte).toBeGreaterThanOrEqual(2);
    expect(r.rimosse).toBe(0);
  });
});
