import {
  ViewPlugin,
  Decoration,
  type DecorationSet,
  EditorView,
  type ViewUpdate,
} from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";

/// Regex che riconosce sia `{{nome}}` (segnaposto semplice) sia
/// `{{globale nome}}` (segnaposto globale, Issue #353).
/// Non interferisce con `{{import "..."}}` gestito da import-tokens.ts.
const RE = /\{\{\s*(?:globale\s+\w+|\w+)\s*\}\}/g;

export interface SegnapostoMatch {
  from: number;
  to: number;
  globale: boolean;
}

/// Helper puro esportato per i test: trova tutti i segnaposti nel testo.
/// Ritorna i match con la loro posizione e se sono globali.
export function _matchSegnaposti(text: string): SegnapostoMatch[] {
  const re = new RegExp(RE.source, "g");
  const results: SegnapostoMatch[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    const inner = m[0];
    // Determina se è globale: la parola chiave "globale" seguita da spazio
    // e un altro identificatore dentro le graffe.
    const isGlobale = /\{\{\s*globale\s+\w+/.test(inner);
    results.push({ from: m.index, to: m.index + inner.length, globale: isGlobale });
  }
  return results;
}

const segnapostoMark = Decoration.mark({ class: "cm-segnaposto" });
const segnapostoGlobaleMark = Decoration.mark({
  class: "cm-segnaposto cm-segnaposto-globale",
});

function costruisciDecorazioni(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  for (const { from, to } of view.visibleRanges) {
    const testo = view.state.doc.sliceString(from, to);
    const matches = _matchSegnaposti(testo);
    for (const match of matches) {
      const mark = match.globale ? segnapostoGlobaleMark : segnapostoMark;
      builder.add(from + match.from, from + match.to, mark);
    }
  }
  return builder.finish();
}

export const segnapostoHighlight = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = costruisciDecorazioni(view);
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = costruisciDecorazioni(update.view);
      }
    }
  },
  { decorations: (v) => v.decorations },
);

export const segnapostoTheme = EditorView.baseTheme({
  ".cm-segnaposto": {
    color: "var(--accent-private)",
    background: "var(--accent-private-soft)",
    borderRadius: "var(--radius-sm)",
    padding: "1px 4px",
    fontWeight: "500",
  },
  ".cm-segnaposto-globale": {
    color: "var(--accent-team)",
    background: "var(--accent-team-soft)",
  },
});
