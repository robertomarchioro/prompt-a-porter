<script lang="ts">
  import {
    Bold,
    Italic,
    Heading1,
    Heading2,
    List,
    ListOrdered,
    Quote,
    Code,
    Code2,
    Link,
    Minus,
    Variable,
    Globe,
    GitFork,
    Search,
  } from "lucide-svelte";
  import type { EditorView } from "@codemirror/view";
  import { openSearchPanel } from "@codemirror/search";

  interface Props {
    view: EditorView | null;
    onInserisciVariabile?: () => void;
    onInserisciImport?: () => void;
  }

  let { view, onInserisciVariabile, onInserisciImport }: Props = $props();

  function wrap(prefix: string, suffix: string = prefix): void {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    const selezione = view.state.doc.sliceString(from, to);
    const inserito = prefix + selezione + suffix;
    view.dispatch({
      changes: { from, to, insert: inserito },
      selection: {
        anchor: from + prefix.length,
        head: from + prefix.length + selezione.length,
      },
    });
    view.focus();
  }

  function prefissoRiga(prefisso: string): void {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    const lineaIniz = view.state.doc.lineAt(from);
    const lineaFine = view.state.doc.lineAt(to);
    const changes: { from: number; insert: string }[] = [];
    for (let n = lineaIniz.number; n <= lineaFine.number; n++) {
      const linea = view.state.doc.line(n);
      changes.push({ from: linea.from, insert: prefisso });
    }
    view.dispatch({ changes });
    view.focus();
  }

  function inserisciTesto(testo: string): void {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    view.dispatch({
      changes: { from, to, insert: testo },
      selection: { anchor: from + testo.length },
    });
    view.focus();
  }

  function inserisciHr(): void {
    inserisciTesto("\n\n---\n\n");
  }

  function inserisciCodeBlock(): void {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    const sel = view.state.doc.sliceString(from, to);
    const inserito = "```\n" + sel + "\n```";
    view.dispatch({
      changes: { from, to, insert: inserito },
      selection: { anchor: from + 4, head: from + 4 + sel.length },
    });
    view.focus();
  }

  function inserisciLink(): void {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    const sel = view.state.doc.sliceString(from, to) || "testo";
    const inserito = `[${sel}](url)`;
    view.dispatch({
      changes: { from, to, insert: inserito },
      // Seleziona "url" per editing immediato
      selection: { anchor: from + sel.length + 3, head: from + sel.length + 6 },
    });
    view.focus();
  }

  function aprireSearch(): void {
    if (view) openSearchPanel(view);
  }

  /**
   * Issue #159: inserisce `{{global <nome>}}` alla posizione del cursor.
   * Se c'è una selezione, la usa come nome del segnaposto; altrimenti usa
   * "nome" come placeholder e lo seleziona per editing immediato.
   */
  function inserisciVariabileGlobale(): void {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    const sel = view.state.doc.sliceString(from, to) || "nome";
    const inserito = `{{global ${sel}}}`;
    view.dispatch({
      changes: { from, to, insert: inserito },
      // "{{global " = 9 caratteri → seleziona <sel> per editing
      selection: { anchor: from + 9, head: from + 9 + sel.length },
    });
    view.focus();
  }
</script>

<div class="md-toolbar" role="toolbar" aria-label="Formattazione markdown">
  <button
    class="md-btn"
    type="button"
    title="Grassetto"
    aria-label="Grassetto"
    onclick={() => wrap("**")}
  >
    <Bold size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Corsivo"
    aria-label="Corsivo"
    onclick={() => wrap("*")}
  >
    <Italic size={14} strokeWidth={1.75} />
  </button>

  <span class="md-sep" aria-hidden="true"></span>

  <button
    class="md-btn"
    type="button"
    title="Titolo 1"
    aria-label="Titolo 1"
    onclick={() => prefissoRiga("# ")}
  >
    <Heading1 size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Titolo 2"
    aria-label="Titolo 2"
    onclick={() => prefissoRiga("## ")}
  >
    <Heading2 size={14} strokeWidth={1.75} />
  </button>

  <span class="md-sep" aria-hidden="true"></span>

  <button
    class="md-btn"
    type="button"
    title="Lista puntata"
    aria-label="Lista puntata"
    onclick={() => prefissoRiga("- ")}
  >
    <List size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Lista numerata"
    aria-label="Lista numerata"
    onclick={() => prefissoRiga("1. ")}
  >
    <ListOrdered size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Citazione"
    aria-label="Citazione"
    onclick={() => prefissoRiga("> ")}
  >
    <Quote size={14} strokeWidth={1.75} />
  </button>

  <span class="md-sep" aria-hidden="true"></span>

  <button
    class="md-btn"
    type="button"
    title="Codice inline"
    aria-label="Codice inline"
    onclick={() => wrap("`")}
  >
    <Code size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Blocco codice"
    aria-label="Blocco codice"
    onclick={inserisciCodeBlock}
  >
    <Code2 size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Link"
    aria-label="Link"
    onclick={inserisciLink}
  >
    <Link size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Riga orizzontale"
    aria-label="Riga orizzontale"
    onclick={inserisciHr}
  >
    <Minus size={14} strokeWidth={1.75} />
  </button>

  <span class="md-sep" aria-hidden="true"></span>

  <button
    class="md-btn"
    type="button"
    title="Inserisci variabile (segnaposto)"
    aria-label="Inserisci variabile"
    onclick={onInserisciVariabile}
  >
    <Variable size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title={"Inserisci segnaposto globale ({{global nome}})"}
    aria-label="Inserisci segnaposto globale"
    onclick={inserisciVariabileGlobale}
  >
    <Globe size={14} strokeWidth={1.75} />
  </button>
  <button
    class="md-btn"
    type="button"
    title="Inserisci direttiva di import (richiamo prompt componibile)"
    aria-label="Inserisci import"
    onclick={onInserisciImport}
  >
    <GitFork size={14} strokeWidth={1.75} />
  </button>

  <span class="md-sep" aria-hidden="true"></span>

  <button
    class="md-btn"
    type="button"
    title="Cerca nell'editor"
    aria-label="Cerca"
    onclick={aprireSearch}
  >
    <Search size={14} strokeWidth={1.75} />
  </button>
</div>

<style>
  .md-toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: var(--sp-1) var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    overflow-x: auto;
  }

  .md-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  .md-btn:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .md-sep {
    width: 1px;
    height: 18px;
    background: var(--border-subtle);
    margin: 0 var(--sp-1);
  }
</style>
