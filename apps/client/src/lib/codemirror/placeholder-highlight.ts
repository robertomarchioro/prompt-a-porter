import {
  ViewPlugin,
  Decoration,
  type DecorationSet,
  EditorView,
  type ViewUpdate,
} from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";

const RE = /\{\{\s*\w+\s*\}\}/g;

const segnapostoMark = Decoration.mark({ class: "cm-segnaposto" });

function costruisciDecorazioni(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  for (const { from, to } of view.visibleRanges) {
    const testo = view.state.doc.sliceString(from, to);
    let match;
    RE.lastIndex = 0;
    while ((match = RE.exec(testo)) !== null) {
      builder.add(
        from + match.index,
        from + match.index + match[0].length,
        segnapostoMark,
      );
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
});
