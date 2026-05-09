<script lang="ts">
  /**
   * Primitive Modale per il redesign v0.8 (F8 PR-A).
   *
   * Backdrop full-screen + container centrale con header (titolo + ✕)
   * + slot children + slot footer opzionale. Chiusura via:
   * - Click su backdrop (click-outside)
   * - Tasto ESC
   * - Bottone ✕ in header
   *
   * Riferimenti: docs/roadmap/redesign-v08/blueprint-F8.md §1
   */
  import type { Snippet } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import { X } from "lucide-svelte";

  type Larghezza = "sm" | "md" | "lg" | "xl";

  interface Props {
    titolo: string;
    sottotitolo?: string;
    onChiudi: () => void;
    larghezza?: Larghezza;
    children: Snippet;
    footer?: Snippet;
  }

  let {
    titolo,
    sottotitolo,
    onChiudi,
    larghezza = "md",
    children,
    footer,
  }: Props = $props();

  function onKeydown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      onChiudi();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", onKeydown);
    document.body.style.overflow = "hidden";
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKeydown);
    document.body.style.overflow = "";
  });
</script>

<div
  class="backdrop"
  role="presentation"
  onclick={onChiudi}
>
  <div
    class="container"
    data-w={larghezza}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modale-titolo"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <header class="header">
      <div class="titoli">
        <h2 class="titolo" id="modale-titolo">{titolo}</h2>
        {#if sottotitolo}<p class="sottotitolo">{sottotitolo}</p>{/if}
      </div>
      <button
        class="chiudi"
        type="button"
        onclick={onChiudi}
        aria-label="Chiudi"
        title="Chiudi (Esc)"
      >
        <X size={16} />
      </button>
    </header>
    <div class="contenuto">
      {@render children()}
    </div>
    {#if footer}
      <footer class="footer">
        {@render footer()}
      </footer>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-4);
    background: var(--bg-scrim);
    animation: fade-in var(--motion-fast) var(--easing-standard);
  }

  .container {
    background: var(--bg-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    max-height: 90vh;
    width: 100%;
    overflow: hidden;
    animation: pop-in var(--motion-normal) var(--easing-emphasis);
  }

  .container[data-w="sm"] {
    max-width: 420px;
  }
  .container[data-w="md"] {
    max-width: 640px;
  }
  .container[data-w="lg"] {
    max-width: 960px;
  }
  .container[data-w="xl"] {
    max-width: min(1200px, 92vw);
  }

  .header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-3) var(--sp-3) var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .titoli {
    flex: 1;
    min-width: 0;
  }

  .titolo {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    line-height: 1.3;
  }

  .sottotitolo {
    margin: 4px 0 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .chiudi {
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
    flex-shrink: 0;
  }

  .chiudi:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .contenuto {
    flex: 1;
    overflow: auto;
    padding: var(--sp-3);
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
