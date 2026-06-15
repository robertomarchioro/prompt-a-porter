import { describe, expect, it } from "vitest";
import { _findImportAt } from "./import-tokens";

describe("import-tokens / _findImportAt", () => {
  it("ritorna null se nessun token import nel doc", () => {
    const doc = "Lorem ipsum {{var}} dolor sit amet.";
    expect(_findImportAt(doc, 5)).toBeNull();
  });

  it("trova il token quando pos è all'inizio del match", () => {
    const doc = 'before {{import "marketing/email"}} after';
    const found = _findImportAt(doc, 7);
    expect(found).not.toBeNull();
    expect(found!.path).toBe("marketing/email");
  });

  it("trova il token quando pos è in mezzo al match", () => {
    const doc = 'a {{import "x"}} b';
    const found = _findImportAt(doc, 6);
    expect(found?.path).toBe("x");
  });

  it("trova il token quando pos è alla fine del match", () => {
    const doc = '{{import "x"}}END';
    const found = _findImportAt(doc, 14);
    expect(found?.path).toBe("x");
  });

  it("ritorna null se pos è prima del token", () => {
    const doc = 'XX {{import "x"}}';
    expect(_findImportAt(doc, 0)).toBeNull();
  });

  it("ritorna null se pos è dopo il token", () => {
    const doc = '{{import "x"}} YY';
    const found = _findImportAt(doc, 16);
    expect(found).toBeNull();
  });

  it("trova il token corretto fra più import nel doc", () => {
    const doc = '{{import "first"}} bla bla {{import "second"}}';
    const f1 = _findImportAt(doc, 5);
    const f2 = _findImportAt(doc, 30);
    expect(f1?.path).toBe("first");
    expect(f2?.path).toBe("second");
  });

  it("supporta path con sotto-cartelle", () => {
    const doc = '{{import "foo/bar/baz"}}';
    expect(_findImportAt(doc, 10)?.path).toBe("foo/bar/baz");
  });

  it("supporta whitespace dentro il token", () => {
    const doc = '{{ import   "x"  }}';
    expect(_findImportAt(doc, 5)?.path).toBe("x");
  });

  it("ritorna from/to coerenti col match", () => {
    const doc = 'aaa {{import "x"}} bbb';
    const found = _findImportAt(doc, 10)!;
    expect(found.from).toBe(4);
    expect(found.to).toBe(18);
    expect(doc.slice(found.from, found.to)).toBe('{{import "x"}}');
  });

  // Parametrized imports (M4 syntax — issue #304)
  it("trova token con modificatore `with k=v` e ritorna path corretto", () => {
    const doc = '{{import "marketing/email" with tone=formal}}';
    const found = _findImportAt(doc, 10);
    expect(found).not.toBeNull();
    expect(found!.path).toBe("marketing/email");
    expect(found!.from).toBe(0);
    expect(found!.to).toBe(doc.length);
  });

  it("trova token con modificatore `version=N` e ritorna path corretto", () => {
    const doc = 'pre {{import "intro" version=2}} post';
    const found = _findImportAt(doc, 10);
    expect(found).not.toBeNull();
    expect(found!.path).toBe("intro");
    expect(doc.slice(found!.from, found!.to)).toBe(
      '{{import "intro" version=2}}',
    );
  });

  it("nessuna regressione: plain `{{import \"x\"}}` ancora riconosciuto", () => {
    const doc = '{{import "plain"}}';
    expect(_findImportAt(doc, 5)?.path).toBe("plain");
  });

  it("due token parametrizzati adiacenti: [^}]*? non li fonde", () => {
    const doc = '{{import "a" with k=v}}{{import "b" version=2}}';
    const f1 = _findImportAt(doc, 5);
    const f2 = _findImportAt(doc, 26);
    expect(f1?.path).toBe("a");
    expect(f2?.path).toBe("b");
  });
});
