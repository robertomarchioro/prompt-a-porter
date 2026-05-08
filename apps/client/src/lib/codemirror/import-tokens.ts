/// CodeMirror 6 extension per token `{{import "path"}}`:
/// - Highlight (sottolineato + cursor pointer al Ctrl/Cmd)
/// - Hover tooltip nativo CodeMirror con titolo + snippet del prompt
///   importato (via comando Tauri `prompt_resolve_import_preview`)
/// - Ctrl/Cmd+click: callback `onapri(promptId)` per navigare al prompt
///
/// v0.7.0 Step 4. Riferimento parser backend: `prompt_componibili.rs`.

import {
  Decoration,
  type DecorationSet,
  EditorView,
  hoverTooltip,
  type Tooltip,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";
import { type Extension, RangeSetBuilder } from "@codemirror/state";
import { invoke } from "@tauri-apps/api/core";

/// Stesso pattern del backend (`prompt_componibili::re_import`).
const RE_IMPORT = /\{\{\s*import\s+"([^"]+)"\s*\}\}/g;

const importMark = Decoration.mark({ class: "cm-import" });

function buildDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  for (const { from, to } of view.visibleRanges) {
    const text = view.state.doc.sliceString(from, to);
    let m: RegExpExecArray | null;
    RE_IMPORT.lastIndex = 0;
    while ((m = RE_IMPORT.exec(text)) !== null) {
      builder.add(from + m.index, from + m.index + m[0].length, importMark);
    }
  }
  return builder.finish();
}

const importHighlight = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }
    update(u: ViewUpdate) {
      if (u.docChanged || u.viewportChanged) {
        this.decorations = buildDecorations(u.view);
      }
    }
  },
  { decorations: (v) => v.decorations },
);

interface ImportPreview {
  id: string;
  titolo: string;
  body: string;
}

/// Trova il match `{{import "..."}}` che contiene la posizione `pos`.
/// Ritorna `{from, to, path}` se trovato, `null` altrimenti.
export function _findImportAt(
  doc: string,
  pos: number,
): { from: number; to: number; path: string } | null {
  const re = new RegExp(RE_IMPORT.source, "g");
  let m: RegExpExecArray | null;
  while ((m = re.exec(doc)) !== null) {
    if (pos >= m.index && pos <= m.index + m[0].length) {
      return { from: m.index, to: m.index + m[0].length, path: m[1] };
    }
  }
  return null;
}

function makeTooltip(view: EditorView, pos: number): Promise<Tooltip | null> {
  const doc = view.state.doc.toString();
  const found = _findImportAt(doc, pos);
  if (!found) return Promise.resolve(null);

  return invoke<ImportPreview | null>("prompt_resolve_import_preview", {
    path: found.path,
  })
    .then((preview) => {
      if (!preview) {
        return {
          pos: found.from,
          end: found.to,
          above: true,
          create: () => {
            const dom = document.createElement("div");
            dom.className = "cm-import-tooltip cm-import-tooltip--missing";
            dom.textContent = `Import non risolto: "${found.path}"`;
            return { dom };
          },
        };
      }
      return {
        pos: found.from,
        end: found.to,
        above: true,
        create: () => {
          const dom = document.createElement("div");
          dom.className = "cm-import-tooltip";

          const title = document.createElement("div");
          title.className = "cm-import-tooltip-title";
          title.textContent = preview.titolo;
          dom.appendChild(title);

          const snippet = document.createElement("div");
          snippet.className = "cm-import-tooltip-body";
          const truncated =
            preview.body.length > 240
              ? preview.body.slice(0, 240) + "…"
              : preview.body;
          snippet.textContent = truncated;
          dom.appendChild(snippet);

          const hint = document.createElement("div");
          hint.className = "cm-import-tooltip-hint";
          hint.textContent = "Ctrl/Cmd+click per aprire";
          dom.appendChild(hint);

          return { dom };
        },
      };
    })
    .catch(() => null);
}

const importTooltip = hoverTooltip(makeTooltip);

interface ImportTokensOptions {
  onapri?: (promptId: string) => void;
}

function clickHandler(opts: ImportTokensOptions) {
  return EditorView.domEventHandlers({
    click(event, view) {
      if (!(event.ctrlKey || event.metaKey)) return false;
      if (!opts.onapri) return false;
      const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
      if (pos === null) return false;
      const doc = view.state.doc.toString();
      const found = _findImportAt(doc, pos);
      if (!found) return false;

      // Risolvi async, poi callback. Click handler ritorna sincrono.
      invoke<ImportPreview | null>("prompt_resolve_import_preview", {
        path: found.path,
      })
        .then((preview) => {
          if (preview && opts.onapri) {
            opts.onapri(preview.id);
          }
        })
        .catch(() => {});

      event.preventDefault();
      return true;
    },
  });
}

export const importTheme = EditorView.baseTheme({
  ".cm-import": {
    color: "var(--accent-team)",
    textDecoration: "underline dotted var(--accent-team)",
    textUnderlineOffset: "3px",
  },
  ".cm-import-tooltip": {
    background: "var(--bg-raised)",
    border: "1px solid var(--border-default)",
    borderRadius: "var(--radius-md)",
    padding: "8px 12px",
    maxWidth: "320px",
    fontFamily: "var(--font-ui)",
    fontSize: "var(--fs-sm)",
    color: "var(--text-strong)",
    boxShadow: "var(--shadow-md)",
  },
  ".cm-import-tooltip-title": {
    fontWeight: "var(--fw-semibold)",
    marginBottom: "4px",
  },
  ".cm-import-tooltip-body": {
    fontFamily: "var(--font-mono)",
    fontSize: "12px",
    color: "var(--text-muted)",
    whiteSpace: "pre-wrap",
    maxHeight: "120px",
    overflow: "hidden",
  },
  ".cm-import-tooltip-hint": {
    marginTop: "6px",
    fontSize: "11px",
    color: "var(--text-subtle)",
    fontStyle: "italic",
  },
  ".cm-import-tooltip--missing": {
    color: "var(--danger)",
  },
});

export function importTokens(opts: ImportTokensOptions = {}): Extension {
  return [importHighlight, importTooltip, clickHandler(opts), importTheme];
}
