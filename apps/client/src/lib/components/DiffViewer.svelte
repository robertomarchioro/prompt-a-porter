<script lang="ts">
  /**
   * Diff viewer side-by-side / unified (V0.8 F5 PR-D).
   *
   * Genera un unified diff con jsdiff `createTwoFilesPatch`, poi delega
   * a diff2html per il render. Toggle persistito in localStorage per-utente
   * (decisione designer #9). Fallback automatico unified sotto 900px viewport.
   *
   * Riferimenti:
   * - Decisione designer #9
   * - Blueprint F5 PR-D §4
   */
  import { createTwoFilesPatch } from "diff";
  import { html as diff2htmlHtml } from "diff2html";
  import "diff2html/bundles/css/diff2html.min.css";
  import { Columns, AlignLeft } from "lucide-svelte";

  type Modalita = "side-by-side" | "line-by-line";

  const STORAGE_KEY = "pap.diff.modalita";
  const RESPONSIVE_PX = 900;

  function caricaModalita(): Modalita {
    try {
      const v = localStorage.getItem(STORAGE_KEY);
      return v === "line-by-line" || v === "side-by-side"
        ? v
        : "side-by-side";
    } catch {
      return "side-by-side";
    }
  }
  function salvaModalita(m: Modalita): void {
    try {
      localStorage.setItem(STORAGE_KEY, m);
    } catch {
      /* ignore */
    }
  }

  interface Props {
    bodyA: string;
    bodyB: string;
    etichettaA?: string;
    etichettaB?: string;
  }

  let {
    bodyA,
    bodyB,
    etichettaA = "Versione A",
    etichettaB = "Versione B",
  }: Props = $props();

  let preferenzaUtente = $state<Modalita>(caricaModalita());
  let larghezza = $state(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );

  function onResize(): void {
    larghezza = window.innerWidth;
  }

  $effect(() => {
    if (typeof window === "undefined") return;
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  function setModalita(m: Modalita): void {
    preferenzaUtente = m;
    salvaModalita(m);
  }

  // Modalità effettiva: forza unified sotto 900px (decisione #9 responsive)
  const modalitaEffettiva = $derived<Modalita>(
    larghezza < RESPONSIVE_PX ? "line-by-line" : preferenzaUtente,
  );

  const renderHtml = $derived.by(() => {
    if (bodyA === bodyB) return "";
    const patch = createTwoFilesPatch(
      etichettaA,
      etichettaB,
      bodyA,
      bodyB,
      "",
      "",
      { context: 3 },
    );
    return diff2htmlHtml(patch, {
      drawFileList: false,
      outputFormat: modalitaEffettiva,
      matching: "lines",
      diffStyle: "word",
    });
  });
</script>

<div class="diff-viewer">
  <header class="header">
    <span class="lbl">Diff</span>
    <div class="toggle" role="group" aria-label="Modalità diff">
      <button
        type="button"
        class="t-btn"
        data-attivo={preferenzaUtente === "side-by-side" || undefined}
        disabled={larghezza < RESPONSIVE_PX}
        onclick={() => setModalita("side-by-side")}
        title={larghezza < RESPONSIVE_PX
          ? "Side-by-side disabilitato sotto 900px"
          : "Side-by-side"}
      >
        <Columns size={12} />
        <span>Side-by-side</span>
      </button>
      <button
        type="button"
        class="t-btn"
        data-attivo={preferenzaUtente === "line-by-line" || undefined}
        onclick={() => setModalita("line-by-line")}
        title="Unified"
      >
        <AlignLeft size={12} />
        <span>Unified</span>
      </button>
    </div>
  </header>

  <div class="render">
    {#if bodyA === bodyB}
      <p class="vuoto">Nessuna differenza tra le due versioni.</p>
    {:else}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html renderHtml}
    {/if}
  </div>
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-canvas);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  .lbl {
    font-size: 10px;
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    font-weight: var(--fw-semibold);
  }

  .toggle {
    margin-left: auto;
    display: inline-flex;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .t-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 2px 8px;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    font-family: var(--font-ui);
  }

  .t-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .t-btn[data-attivo] {
    background: var(--bg-surface);
    color: var(--text-default);
  }

  .render {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
  }

  .vuoto {
    color: var(--text-subtle);
    text-align: center;
    padding: var(--sp-4);
    margin: 0;
  }

  :global(.diff-viewer .d2h-wrapper) {
    background: var(--bg-canvas);
  }

  :global(.diff-viewer .d2h-file-header) {
    display: none;
  }

  :global(.diff-viewer .d2h-code-line),
  :global(.diff-viewer .d2h-code-side-line) {
    font-family: var(--font-mono);
  }

  :global(.diff-viewer .d2h-ins) {
    background: var(--success-soft);
    color: var(--success);
  }

  :global(.diff-viewer .d2h-del) {
    background: var(--danger-soft);
    color: var(--danger);
  }

  :global(.diff-viewer .d2h-info) {
    background: var(--bg-overlay);
    color: var(--text-subtle);
  }
</style>
