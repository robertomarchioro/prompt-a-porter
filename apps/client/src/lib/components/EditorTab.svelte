<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorView, lineNumbers, keymap } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import {
    defaultKeymap,
    history,
    historyKeymap,
    indentWithTab,
  } from "@codemirror/commands";
  import { searchKeymap } from "@codemirror/search";
  import { markdown } from "@codemirror/lang-markdown";
  import {
    importTokens,
    importTheme,
  } from "$lib/codemirror/import-tokens";
  import {
    segnapostoHighlight,
    segnapostoTheme,
  } from "$lib/codemirror/placeholder-highlight";

  interface Props {
    body: string;
    onChangeBody: (newBody: string) => void;
    onSelectionChange?: (info: {
      righe: number;
      colonna: number;
      chars: number;
    }) => void;
    promptId: string | null;
    onApriPrompt?: (id: string) => void;
    editorView?: EditorView | null;
  }

  let {
    body,
    onChangeBody,
    onSelectionChange,
    promptId,
    onApriPrompt,
    editorView = $bindable(null),
  }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let bodyInterno = body;
  // Issue #167: distingue dispatch programmatico (switch prompt → sync
  // contenuto) da input utente. Settato a true prima del dispatch
  // programmatico nell'$effect su `promptId`; l'updateListener salta
  // `onChangeBody` se il flag è alto. Senza questo, switchare prompt
  // emette un finto change che setta dirty=true in DetailPane → trigger
  // salvataggio fantasma del nuovo prompt con body del vecchio.
  let ignoraProssimoCambio = false;

  function montaEditor(initial: string): void {
    if (!container) return;
    smontaEditor();
    const state = EditorState.create({
      doc: initial,
      extensions: [
        lineNumbers(),
        history(),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
          indentWithTab,
        ]),
        markdown(),
        importTokens({ onapri: onApriPrompt }),
        importTheme,
        segnapostoHighlight,
        segnapostoTheme,
        EditorView.lineWrapping,
        EditorView.updateListener.of((u) => {
          if (u.docChanged) {
            const text = u.state.doc.toString();
            bodyInterno = text;
            if (ignoraProssimoCambio) {
              // Cambio programmatico (switch prompt): non propagare a
              // DetailPane, altrimenti dirty=true fantasma. Issue #167.
              ignoraProssimoCambio = false;
            } else {
              onChangeBody(text);
            }
          }
          if ((u.docChanged || u.selectionSet) && onSelectionChange) {
            const sel = u.state.selection.main;
            const linea = u.state.doc.lineAt(sel.head);
            onSelectionChange({
              righe: u.state.doc.lines,
              colonna: sel.head - linea.from + 1,
              chars: u.state.doc.length,
            });
          }
        }),
      ],
    });
    view = new EditorView({ state, parent: container });
    editorView = view;
    if (onSelectionChange) {
      onSelectionChange({
        righe: state.doc.lines,
        colonna: 1,
        chars: state.doc.length,
      });
    }
  }

  function smontaEditor(): void {
    if (view) {
      view.destroy();
      view = null;
      editorView = null;
    }
  }

  onMount(() => {
    montaEditor(body);
  });

  onDestroy(() => {
    smontaEditor();
  });

  // Quando cambia promptId esternamente, sincronizza il contenuto.
  // Issue #167: marca il dispatch come programmatico → updateListener
  // NON chiama onChangeBody (evita dirty=true fantasma in DetailPane).
  $effect(() => {
    void promptId;
    if (view && body !== bodyInterno) {
      ignoraProssimoCambio = true;
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: body },
      });
      bodyInterno = body;
    }
  });
</script>

<div class="editor-tab" bind:this={container}></div>

<style>
  .editor-tab {
    flex: 1;
    overflow: hidden;
    background: var(--bg-canvas);
  }

  :global(.editor-tab .cm-editor) {
    height: 100%;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.65;
    background: var(--bg-canvas);
    color: var(--text-default);
  }

  :global(.editor-tab .cm-editor.cm-focused) {
    outline: none;
  }

  :global(.editor-tab .cm-gutters) {
    background: var(--bg-surface);
    border-right: 1px solid var(--border-subtle);
    color: var(--text-subtle);
  }

  :global(.editor-tab .cm-content) {
    padding: var(--sp-3);
  }

  :global(.editor-tab .cm-line) {
    padding: 0 var(--sp-1);
  }

  :global(.editor-tab .cm-cursor) {
    border-left-color: var(--accent-team);
    border-left-width: 2px;
  }

  :global(.editor-tab .cm-selectionBackground) {
    background: var(--accent-team-soft) !important;
  }

  :global(.editor-tab .cm-activeLine) {
    background: var(--bg-overlay);
  }

  :global(.editor-tab .cm-activeLineGutter) {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
</style>
