/// CodeMirror 6: commenti inline `{{!-- … --}}` (issue #643).
/// - Highlight attenuato del token intero (il commento non parte col
///   prompt: deve leggersi come "non testo")
/// - Comando `toggleCommento` per toolbar, `Mod-/` e menu contestuale
///
/// La grammatica è quella di `$lib/commenti` (unica fonte). Gli altri
/// plugin (`placeholder-highlight`, `import-tokens`) saltano i match che
/// cadono dentro questi intervalli, così un `{{nome}}` commentato resta
/// grigio e non viola.

import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";
import {
  EditorSelection,
  RangeSetBuilder,
  type StateCommand,
} from "@codemirror/state";
import { intervalliCommenti, type IntervalloCommento } from "$lib/commenti";

const APERTURA = "{{!-- ";
const CHIUSURA = " --}}";
const APERTURA_NUDA = "{{!--";
const CHIUSURA_NUDA = "--}}";

/// Helper puro esportato per i test: intervalli [from, to) dei commenti.
export function _matchCommenti(testo: string): IntervalloCommento[] {
  return intervalliCommenti(testo);
}

const commentoMark = Decoration.mark({ class: "cm-commento" });

function costruisciDecorazioni(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  // Sul documento intero, non sui visibleRanges: un commento multiriga
  // può iniziare sopra la viewport e finire dentro.
  for (const { from, to } of intervalliCommenti(view.state.doc.toString())) {
    builder.add(from, to, commentoMark);
  }
  return builder.finish();
}

export const commentoHighlight = ViewPlugin.fromClass(
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

export const commentoTheme = EditorView.baseTheme({
  ".cm-commento": {
    color: "var(--text-muted)",
    fontStyle: "italic",
    opacity: "0.85",
  },
});

/// Il commento che contiene `pos`, oppure quello toccato dalla selezione
/// [from, to] se non vuota.
function commentoIntorno(
  intervalli: IntervalloCommento[],
  from: number,
  to: number,
): IntervalloCommento | undefined {
  return intervalli.find((c) => from >= c.from && to <= c.to);
}

/// Toggle: se la selezione (o il cursore) sta in un commento, lo
/// scommenta lasciando il testo interno selezionato; altrimenti avvolge la
/// selezione in `{{!-- … --}}` (o inserisce un commento vuoto col cursore
/// in mezzo) e seleziona il testo interno. Toglie al più UNO spazio di
/// cornice per lato, così `{{!--   x   --}}` non perde gli spazi voluti.
export const toggleCommento: StateCommand = ({ state, dispatch }) => {
  const { from, to } = state.selection.main;
  const intervalli = intervalliCommenti(state.doc.toString());
  const dentro = commentoIntorno(intervalli, from, to);

  if (dentro) {
    const grezzo = state.doc.sliceString(dentro.from, dentro.to);
    let interno = grezzo.slice(
      APERTURA_NUDA.length,
      grezzo.length - CHIUSURA_NUDA.length,
    );
    if (interno.startsWith(" ")) interno = interno.slice(1);
    if (interno.endsWith(" ")) interno = interno.slice(0, -1);
    dispatch(
      state.update({
        changes: { from: dentro.from, to: dentro.to, insert: interno },
        selection: EditorSelection.range(
          dentro.from,
          dentro.from + interno.length,
        ),
        userEvent: "delete",
      }),
    );
    return true;
  }

  const selezione = state.doc.sliceString(from, to);
  const inserito = APERTURA + selezione + CHIUSURA;
  const iniziaInterno = from + APERTURA.length;
  dispatch(
    state.update({
      changes: { from, to, insert: inserito },
      selection: EditorSelection.range(
        iniziaInterno,
        iniziaInterno + selezione.length,
      ),
      userEvent: "input",
    }),
  );
  return true;
};
