<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variante?: "default" | "success" | "warning" | "danger" | "info";
    children: Snippet;
  }

  let { variante = "default", children }: Props = $props();

  const classi = $derived(
    ["badge", variante !== "default" && `badge--${variante}`]
      .filter(Boolean)
      .join(" "),
  );
</script>

<span class={classi}>{@render children()}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 18px;
    padding: 0 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    letter-spacing: var(--tracking-wide);
  }

  .badge--success {
    color: var(--success);
    background: var(--success-soft);
  }
  .badge--warning {
    color: var(--warning);
    background: var(--warning-soft);
  }
  .badge--danger {
    color: var(--danger);
    background: var(--danger-soft);
  }
  .badge--info {
    color: var(--info);
    background: var(--info-soft);
  }
</style>
