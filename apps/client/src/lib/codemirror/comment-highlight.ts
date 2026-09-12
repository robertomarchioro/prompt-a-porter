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

/// Il commento che contiene il cursore (`[from, to)`, coerente con
/// `dentroCommento`: subito dopo `--}}` si è FUORI) oppure che copre per
/// intero la selezione non vuota.
function commentoIntorno(
  intervalli: IntervalloCommento[],
  from: number,
  to: number,
): IntervalloCommento | undefined {
  const vuota = from === to;
  return intervalli.find(
    (c) => from >= c.from && (vuota ? to < c.to : to <= c.to),
  );
}

/// Testo interno di un token `{{!-- … --}}`, senza (al più) uno spazio di
/// cornice per lato.
function scarta(token: string): string {
  let interno = token.slice(
    APERTURA_NUDA.length,
    token.length - CHIUSURA_NUDA.length,
  );
  if (interno.startsWith(" ")) interno = interno.slice(1);
  if (interno.endsWith(" ")) interno = interno.slice(0, -1);
  return interno;
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
    const interno = scarta(state.doc.sliceString(dentro.from, dentro.to));
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

  // La grammatica chiude al PRIMO `--}}`: un commento non può contenerne
  // un altro. Se la selezione tocca dei commenti, la si allarga a coprirli
  // e li si scarta (il loro testo resta, dentro il commento unico).
  const toccati = intervalli.filter((c) => c.from < to && c.to > from);
  const inizio = toccati.length > 0 ? Math.min(from, toccati[0].from) : from;
  const fine =
    toccati.length > 0
      ? Math.max(to, toccati[toccati.length - 1].to)
      : to;
  let selezione = "";
  let cursore = inizio;
  for (const c of toccati) {
    selezione += state.doc.sliceString(cursore, c.from);
    selezione += scarta(state.doc.sliceString(c.from, c.to));
    cursore = c.to;
  }
  selezione += state.doc.sliceString(cursore, fine);

  // Un `--}}` letterale nel testo chiuderebbe il commento a metà e la
  // coda partirebbe col prompt: meglio non fare nulla (ma il comando è
  // gestito, così Mod-/ non ricade sul toggleComment Markdown).
  if (selezione.includes(CHIUSURA_NUDA)) return true;

  const inserito = APERTURA + selezione + CHIUSURA;
  const iniziaInterno = inizio + APERTURA.length;
  dispatch(
    state.update({
      changes: { from: inizio, to: fine, insert: inserito },
      selection: EditorSelection.range(
        iniziaInterno,
        iniziaInterno + selezione.length,
      ),
      userEvent: "input",
    }),
  );
  return true;
};
