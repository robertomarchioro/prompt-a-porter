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
///
/// Il gruppo di cattura 1 è presente solo quando il ramo `globale <nome>` viene
/// selezionato dall'alternazione; `m[1] === 'globale'` è l'unica sorgente di
/// verità per distinguere segnaposti globali da quelli normali.
///
/// Nota: `{{globale}}` senza nome — la parola chiave sola — non soddisfa il
/// primo ramo (manca `\s+\w+`) e ricade nel secondo (`\w+`), quindi viene
/// trattato come segnaposto normale (globale = false). Comportamento intenzionale.
///
/// La costante è usata solo come `.source` per costruire RegExp istanza-locali;
/// il flag /g qui è assente perché non serve a livello di modulo.
const RE = /\{\{\s*(?:(globale)\s+\w+|\w+)\s*\}\}/;

export interface SegnapostoMatch {
  from: number;
  to: number;
  globale: boolean;
}

/// Helper puro esportato per i test: trova tutti i segnaposti nel testo.
/// Ritorna i match con la loro posizione e se sono globali.
/// La globalità è rilevata tramite il gruppo di cattura 1 della RE — nessuna
/// seconda regex separata.
export function _matchSegnaposti(text: string): SegnapostoMatch[] {
  const re = new RegExp(RE.source, "g");
  const results: SegnapostoMatch[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    // m[1] è definito ('globale') solo quando il ramo globale ha fatto match.
    const isGlobale = m[1] === "globale";
    results.push({ from: m.index, to: m.index + m[0].length, globale: isGlobale });
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
  // Applicata insieme a .cm-segnaposto, quindi eredita border-radius,
  // padding e font-weight da quella classe; qui si sovrascrivono solo i colori.
  ".cm-segnaposto-globale": {
    color: "var(--accent-team)",
    background: "var(--accent-team-soft)",
  },
});
