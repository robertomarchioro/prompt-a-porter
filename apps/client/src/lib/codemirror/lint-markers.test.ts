import { describe, expect, it } from "vitest";
import { Text } from "@codemirror/state";
import {
  _decorazioniPerTest,
  type LintIssue,
} from "./lint-markers";

function contaRange(set: ReturnType<typeof _decorazioniPerTest>): number {
  let n = 0;
  const iter = set.iter();
  while (iter.value !== null) {
    n++;
    iter.next();
  }
  return n;
}

describe("lint-markers", () => {
  it("ignora issue senza posizione (linea/colonna null)", () => {
    const doc = Text.of(["riga uno", "riga due"]);
    const issues: LintIssue[] = [
      {
        code: "LEN001",
        severita: "warning",
        messaggio: "body troppo lungo",
        linea: null,
        colonna: null,
      },
    ];
    const set = _decorazioniPerTest(doc, issues);
    expect(contaRange(set)).toBe(0);
  });

  it("crea una decoration per issue con posizione valida", () => {
    const doc = Text.of(["ciao {nome}", "altra riga"]);
    const issues: LintIssue[] = [
      {
        code: "PH001",
        severita: "error",
        messaggio: "segnaposto malformato",
        linea: 1,
        colonna: 6, // posizione di '{'
      },
    ];
    const set = _decorazioniPerTest(doc, issues);
    expect(contaRange(set)).toBe(1);
  });

  it("skippa issue su linea fuori range del doc", () => {
    const doc = Text.of(["unica riga"]);
    const issues: LintIssue[] = [
      {
        code: "PH003",
        severita: "warning",
        messaggio: "x",
        linea: 999,
        colonna: 1,
      },
    ];
    const set = _decorazioniPerTest(doc, issues);
    expect(contaRange(set)).toBe(0);
  });

  it("skippa issue su linea < 1 (input invalido difensivo)", () => {
    const doc = Text.of(["riga"]);
    const issues: LintIssue[] = [
      {
        code: "X",
        severita: "info",
        messaggio: "x",
        linea: 0,
        colonna: 1,
      },
    ];
    const set = _decorazioniPerTest(doc, issues);
    expect(contaRange(set)).toBe(0);
  });

  it("ordina i range per offset crescente come richiesto da CodeMirror", () => {
    const doc = Text.of(["aaaa bbbb cccc"]);
    // Issue volutamente non ordinati: colonna 10, poi 1, poi 6.
    const issues: LintIssue[] = [
      { code: "A", severita: "info", messaggio: "x", linea: 1, colonna: 10 },
      { code: "B", severita: "warning", messaggio: "y", linea: 1, colonna: 1 },
      { code: "C", severita: "error", messaggio: "z", linea: 1, colonna: 6 },
    ];
    const set = _decorazioniPerTest(doc, issues);
    const offsets: number[] = [];
    const iter = set.iter();
    while (iter.value !== null) {
      offsets.push(iter.from);
      iter.next();
    }
    const sorted = [...offsets].sort((a, b) => a - b);
    expect(offsets).toEqual(sorted);
    expect(offsets).toHaveLength(3);
  });

  it("supporta issue su linee multiple", () => {
    const doc = Text.of(["riga uno", "riga due", "riga tre"]);
    const issues: LintIssue[] = [
      { code: "A", severita: "warning", messaggio: "x", linea: 1, colonna: 1 },
      { code: "B", severita: "error", messaggio: "y", linea: 3, colonna: 1 },
    ];
    const set = _decorazioniPerTest(doc, issues);
    expect(contaRange(set)).toBe(2);
  });

  it("clamp colonna oltre la fine della linea senza crash", () => {
    const doc = Text.of(["xy"]);
    const issues: LintIssue[] = [
      {
        code: "X",
        severita: "info",
        messaggio: "x",
        linea: 1,
        colonna: 999,
      },
    ];
    expect(() => _decorazioniPerTest(doc, issues)).not.toThrow();
  });
});
