import { describe, expect, it } from "vitest";
import { EditorState, EditorSelection } from "@codemirror/state";
import { _matchCommenti, toggleCommento } from "./comment-highlight";

/// Esegue un StateCommand su uno stato sintetico e ritorna lo stato
/// risultante (o lo stesso se il comando ha rifiutato).
function esegui(
  doc: string,
  anchor: number,
  head: number = anchor,
): { doc: string; from: number; to: number; eseguito: boolean } {
  let state = EditorState.create({
    doc,
    selection: EditorSelection.single(anchor, head),
  });
  const eseguito = toggleCommento({
    state,
    dispatch: (tr) => {
      state = tr.state;
    },
  });
  const sel = state.selection.main;
  return { doc: state.doc.toString(), from: sel.from, to: sel.to, eseguito };
}

describe("comment-highlight / _matchCommenti", () => {
  it("nessun commento → vuoto", () => {
    expect(_matchCommenti("Ciao {{nome}}")).toEqual([]);
  });

  it("ritorna [from, to) del token intero, anche multiriga", () => {
    const m = _matchCommenti("a {{!-- x\ny --}} b");
    expect(m).toEqual([{ from: 2, to: 16 }]);
  });

  it("la forma breve {{! }} non è un commento", () => {
    expect(_matchCommenti("{{! x }}")).toEqual([]);
  });
});

describe("toggleCommento", () => {
  it("senza selezione inserisce un commento vuoto col cursore dentro", () => {
    const r = esegui("Ciao", 4);
    expect(r.doc).toBe("Ciao{{!--  --}}");
    // Cursore fra i due spazi: "Ciao{{!-- " = 10 caratteri.
    expect(r.from).toBe(10);
    expect(r.to).toBe(10);
    expect(r.eseguito).toBe(true);
  });

  it("con selezione la avvolge e seleziona il testo interno", () => {
    const r = esegui("Ciao nota fine", 5, 9);
    expect(r.doc).toBe("Ciao {{!-- nota --}} fine");
    expect(r.doc.slice(r.from, r.to)).toBe("nota");
  });

  it("con selezione multiriga avvolge tutto in un solo commento", () => {
    const r = esegui("a\nb\nc", 0, 5);
    expect(r.doc).toBe("{{!-- a\nb\nc --}}");
  });

  it("col cursore dentro un commento lo scommenta", () => {
    const r = esegui("x {{!-- nota --}} y", 8);
    expect(r.doc).toBe("x nota y");
    expect(r.doc.slice(r.from, r.to)).toBe("nota");
  });

  it("con selezione che copre esattamente un commento lo scommenta", () => {
    const doc = "x {{!-- nota --}} y";
    const r = esegui(doc, 2, 17);
    expect(r.doc).toBe("x nota y");
  });

  it("scommentare toglie un solo spazio di cornice per lato, non di più", () => {
    const r = esegui("{{!--   larga   --}}", 6);
    expect(r.doc).toBe("  larga  ");
  });

  it("scommentare un commento senza spazi di cornice", () => {
    const r = esegui("{{!--stretto--}}", 6);
    expect(r.doc).toBe("stretto");
  });

  it("commentare e scommentare è un roundtrip", () => {
    const avvolto = esegui("uno due tre", 4, 7);
    expect(avvolto.doc).toBe("uno {{!-- due --}} tre");
    const tolto = esegui(avvolto.doc, avvolto.from, avvolto.to);
    expect(tolto.doc).toBe("uno due tre");
  });

  it("la selezione dentro un commento (non esatta) lo scommenta comunque", () => {
    const r = esegui("{{!-- abc def --}}", 7, 9);
    expect(r.doc).toBe("abc def");
  });

  // Rilievi review PR-2.
  it("col cursore subito dopo --}} inserisce un commento nuovo, non scommenta", () => {
    const r = esegui("{{!-- nota --}}", 15);
    expect(r.doc).toBe("{{!-- nota --}}{{!--  --}}");
  });

  it("col cursore subito prima di {{!-- scommenta (confine incluso)", () => {
    const r = esegui("{{!-- nota --}}", 0);
    expect(r.doc).toBe("nota");
  });

  it("selezione che contiene un commento: lo assorbe in un commento unico", () => {
    const doc = "x {{!-- a --}} y";
    const r = esegui(doc, 0, doc.length);
    expect(r.doc).toBe("{{!-- x a y --}}");
    expect(r.doc.slice(r.from, r.to)).toBe("x a y");
  });

  it("selezione che taglia un commento a metà: si allarga a coprirlo", () => {
    const doc = "{{!-- abc --}} def";
    // "c --}} de"
    const r = esegui(doc, 8, 17);
    expect(r.doc).toBe("{{!-- abc de --}}f");
  });

  it("testo con --}} letterale: nessuna modifica ma comando gestito", () => {
    const doc = "a --}} b";
    const r = esegui(doc, 0, doc.length);
    expect(r.doc).toBe(doc);
    expect(r.eseguito).toBe(true);
  });
});
