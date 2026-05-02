<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variante?: "default" | "success" | "danger";
    visibile?: boolean;
    children: Snippet;
  }

  let { variante = "default", visibile = true, children }: Props = $props();
</script>

{#if visibile}
  <div
    class="toast"
    class:toast--success={variante === "success"}
    class:toast--danger={variante === "danger"}
    role="status"
    aria-live="polite"
  >
    {@render children()}
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    bottom: var(--sp-5);
    left: 50%;
    transform: translateX(-50%);
    z-index: var(--z-toast);
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 10px 16px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-strong);
    font-size: var(--fs-sm);
    box-shadow: var(--shadow-md);
    animation: toastIn var(--motion-slow) var(--easing-emphasis);
  }

  .toast--success {
    border-color: color-mix(in oklch, var(--success) 40%, transparent);
  }
  .toast--danger {
    border-color: color-mix(in oklch, var(--danger) 40%, transparent);
  }

  @keyframes toastIn {
    from {
      opacity: 0;
      transform: translate(-50%, 8px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }
</style>
