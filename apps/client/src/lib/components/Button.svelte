<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface Props extends HTMLButtonAttributes {
    variante?: "default" | "primary" | "private" | "ghost" | "danger";
    dimensione?: "sm" | "default" | "lg";
    soloIcona?: boolean;
    children: Snippet;
  }

  let {
    variante = "default",
    dimensione = "default",
    soloIcona = false,
    children,
    class: classeExtra = "",
    ...rest
  }: Props = $props();

  const classi = $derived(
    [
      "btn",
      variante !== "default" && `btn--${variante}`,
      dimensione !== "default" && `btn--${dimensione}`,
      soloIcona && "btn--icon",
      classeExtra,
    ]
      .filter(Boolean)
      .join(" "),
  );
</script>

<button class={classi} {...rest}>
  {@render children()}
</button>

<style>
  .btn {
    --_h: 32px;
    --_px: 12px;
    --_bg: var(--bg-overlay);
    --_bg-hover: var(--border-default);
    --_fg: var(--text-strong);
    --_border: var(--border-default);
    --_ring: var(--accent-team);

    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: var(--_h);
    padding: 0 var(--_px);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    line-height: 1;
    color: var(--_fg);
    background: var(--_bg);
    border: 1px solid var(--_border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      background var(--motion-fast) var(--easing-standard),
      transform var(--motion-fast),
      box-shadow var(--motion-fast);
    white-space: nowrap;
  }

  .btn:hover {
    background: var(--_bg-hover);
  }
  .btn:active {
    transform: translateY(0.5px);
  }
  .btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--bg-canvas), 0 0 0 4px var(--_ring);
  }
  .btn:disabled {
    color: var(--text-disabled);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .btn--primary {
    --_bg: var(--accent-team);
    --_bg-hover: var(--accent-team-strong);
    --_fg: var(--accent-team-on);
    --_border: transparent;
  }
  .btn--private {
    --_bg: var(--accent-private);
    --_bg-hover: var(--accent-private-strong);
    --_fg: var(--accent-private-on);
    --_border: transparent;
    --_ring: var(--accent-private);
  }
  .btn--ghost {
    --_bg: transparent;
    --_bg-hover: var(--bg-overlay);
    --_border: transparent;
  }
  .btn--danger {
    --_bg: var(--danger);
    --_bg-hover: color-mix(in oklch, var(--danger) 85%, black);
    --_fg: oklch(0.99 0.005 25);
    --_border: transparent;
    --_ring: var(--danger);
  }

  .btn--sm {
    --_h: 26px;
    --_px: 8px;
    font-size: var(--fs-xs);
  }
  .btn--lg {
    --_h: 40px;
    --_px: 16px;
    font-size: var(--fs-base);
  }
  .btn--icon {
    --_px: 0;
    width: var(--_h);
    justify-content: center;
  }
</style>
