<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import {
    EditorView,
    lineNumbers,
    highlightActiveLine,
    keymap,
  } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { indentUnit } from "@codemirror/language";
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
  import { importAutocompletion } from "$lib/codemirror/import-autocomplete";
  import { statoEditor } from "$lib/stores/preferenze.svelte";

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
  // Snapshot iniziale di `body` (prop reattiva). untrack() evita di
  // catturare body come dipendenza reattiva del top-level script:
  // bodyInterno traccia lo stato interno CodeMirror, sync con prop
  // body avviene nell'$effect dedicato, non automaticamente.
  let bodyInterno = untrack(() => body);
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
    // M10 — preferenze editor (rimonto su cambio prefs via $effect sotto;
    // il pattern Compartment darebbe granularita' migliore ma il rimonto
    // e' semplice e i cambi sono rari).
    const indentString = " ".repeat(statoEditor.indentSize);
    const fontTheme = EditorView.theme({
      "&": { fontSize: `${statoEditor.fontSize}px` },
    });
    const updateListener = EditorView.updateListener.of((u) => {
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
    });
    const extensions = [
      history(),
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
        indentWithTab,
      ]),
      markdown(),
      indentUnit.of(indentString),
      fontTheme,
      importTokens({ onapri: onApriPrompt }),
      importTheme,
      // M4 PR-3: intellisense autocomplete `{{import "...`. Callback
      // legge promptId al momento dell'invocazione per escludere self.
      importAutocompletion({ getPromptId: () => promptId }),
      segnapostoHighlight,
      segnapostoTheme,
      updateListener,
    ];
    if (statoEditor.showLineNumbers) extensions.push(lineNumbers());
    if (statoEditor.lineWrapping) extensions.push(EditorView.lineWrapping);
    if (statoEditor.highlightActiveLine) extensions.push(highlightActiveLine());
    const state = EditorState.create({ doc: initial, extensions });
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

  // M10 — Rimonta l'editor quando cambia una preferenza editor. Le
  // dipendenze reattive sono lette dentro $effect cosi' Svelte 5 le
  // traccia correttamente. Salta il primo run (montaggio gia' fatto
  // da onMount) controllando che `view` esista e che i valori siano
  // diversi da quelli usati al monta.
  //
  // Issue #275 (catastrofico, eco di #170): `prefsSnapshot` NON deve
  // essere un `$state`. L'$effect leggeva `prefsSnapshot[k]` (dipendenza
  // reattiva) e poi lo riscriveva incondizionatamente (`prefsSnapshot =
  // curr`): siccome la riscrittura assegna un nuovo oggetto ad ogni run,
  // l'effect dipendeva dalla propria scrittura → loop reattivo infinito
  // che congelava tutta l'UI appena montato l'EditorTab (cioe' appena un
  // prompt veniva selezionato; "persiste a riavvio" = ricaricando lo
  // stesso prompt si ri-monta l'EditorTab e si ri-triggera il loop).
  // Come semplice variabile locale (non reattiva) il confronto resta
  // valido ma non genera dipendenza reattiva sulla propria scrittura.
  let prefsSnapshot = {
    autosaveDelayMs: statoEditor.autosaveDelayMs,
    lineWrapping: statoEditor.lineWrapping,
    indentSize: statoEditor.indentSize,
    fontSize: statoEditor.fontSize,
    showLineNumbers: statoEditor.showLineNumbers,
    highlightActiveLine: statoEditor.highlightActiveLine,
  };
  $effect(() => {
    const curr = {
      autosaveDelayMs: statoEditor.autosaveDelayMs,
      lineWrapping: statoEditor.lineWrapping,
      indentSize: statoEditor.indentSize,
      fontSize: statoEditor.fontSize,
      showLineNumbers: statoEditor.showLineNumbers,
      highlightActiveLine: statoEditor.highlightActiveLine,
    };
    // autosaveDelayMs e' gestito da DetailPane (non rimontare l'editor).
    const rilevanti = [
      "lineWrapping",
      "indentSize",
      "fontSize",
      "showLineNumbers",
      "highlightActiveLine",
    ] as const;
    const cambiato = rilevanti.some(
      (k) => curr[k] !== prefsSnapshot[k],
    );
    if (cambiato && view) {
      const corrente = view.state.doc.toString();
      bodyInterno = corrente;
      montaEditor(corrente);
    }
    prefsSnapshot = curr;
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
