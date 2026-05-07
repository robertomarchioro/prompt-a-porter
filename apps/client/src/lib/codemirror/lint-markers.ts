/// CodeMirror 6 extension per evidenziare inline gli issue del linter
/// (PH001, PH003, PII*, IMP*, ecc.). Mostra una decoration sottolineata
/// colorata per severità (error/warning/info) con tooltip nativo del
/// browser che riporta `code: messaggio`.
///
/// v0.6.0 Step 3: complemento al pannello Diagnosi (oggi unica
/// rappresentazione delle issue). Riferimento:
/// `docs/roadmap/rinvii.md` § Da Fase 3 Step 5.

import {
  Decoration,
  type DecorationSet,
  EditorView,
} from "@codemirror/view";
import { StateEffect, StateField } from "@codemirror/state";

export type LintSeverita = "error" | "warning" | "info";

export interface LintIssue {
  code: string;
  severita: LintSeverita;
  messaggio: string;
  /// 1-based, `null` quando l'issue non è localizzabile (es. LEN001).
  linea: number | null;
  colonna: number | null;
}

/// Effect dispatchato dal componente quando la lista issues cambia.
export const setLintIssues = StateEffect.define<LintIssue[]>();

function markPerSeverita(issue: LintIssue): Decoration {
  const cls = `cm-lint-${issue.severita}`;
  // Tooltip nativo browser via attributo title.
  // Formato: "PH001: messaggio dettagliato"
  return Decoration.mark({
    class: cls,
    attributes: { title: `${issue.code}: ${issue.messaggio}` },
  });
}

/// Costruisce le decoration dalle issue. Skippa quelle senza posizione
/// (linea null) — restano visibili solo nel pannello Diagnosi.
/// Posizione fuori range del doc → skip silenzioso (auto-recovery se
/// il body cambia tra lint e dispatch).
function decorazioniDaDoc(
  doc: import("@codemirror/state").Text,
  issues: LintIssue[],
): DecorationSet {
  const ranges: { from: number; to: number; deco: Decoration }[] = [];
  for (const issue of issues) {
    if (issue.linea === null || issue.colonna === null) continue;
    if (issue.linea < 1 || issue.linea > doc.lines) continue;
    const lineInfo = doc.line(issue.linea);
    const offsetCol = Math.max(0, issue.colonna - 1);
    const from = Math.min(lineInfo.from + offsetCol, lineInfo.to);
    // Marker da 1 carattere: visibile come underline puntuale, sufficiente
    // per ancorare il tooltip. Estendere a token = scope futuro.
    const to = Math.min(from + 1, lineInfo.to);
    if (from >= to) continue;
    ranges.push({ from, to, deco: markPerSeverita(issue) });
  }
  // CodeMirror richiede ranges ordinati per `from` ASC.
  ranges.sort((a, b) => a.from - b.from || a.to - b.to);
  return Decoration.set(
    ranges.map((r) => r.deco.range(r.from, r.to)),
    /* sort */ true,
  );
}

/// StateField che mantiene il DecorationSet corrente. Si aggiorna
/// quando arriva un `setLintIssues` effect, oppure mappa le posizioni
/// quando il doc cambia (le decoration più "vecchie" del prossimo lint
/// asincrono restano valide finché il body non viene re-analizzato).
export const lintMarkers = StateField.define<DecorationSet>({
  create() {
    return Decoration.none;
  },
  update(deco, tr) {
    let nuove = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (effect.is(setLintIssues)) {
        nuove = decorazioniDaDoc(tr.state.doc, effect.value);
      }
    }
    return nuove;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/// Theme di default per i 3 livelli di severità. Sottolineatura wavy
/// stile spell-checker, colori coerenti con la palette `--danger`,
/// `--warning`, `--info`.
export const lintMarkersTheme = EditorView.baseTheme({
  ".cm-lint-error": {
    textDecoration: "underline wavy var(--danger)",
    textDecorationSkipInk: "none",
    cursor: "help",
  },
  ".cm-lint-warning": {
    textDecoration: "underline wavy var(--warning)",
    textDecorationSkipInk: "none",
    cursor: "help",
  },
  ".cm-lint-info": {
    textDecoration: "underline wavy var(--info)",
    textDecorationSkipInk: "none",
    cursor: "help",
  },
});

/// Helper esportato per i test: costruisce il DecorationSet a partire
/// da un Text doc e una lista di issue, senza richiedere uno
/// StateField/EditorView reale.
export function _decorazioniPerTest(
  doc: import("@codemirror/state").Text,
  issues: LintIssue[],
): DecorationSet {
  return decorazioniDaDoc(doc, issues);
}
