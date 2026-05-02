<script lang="ts">
  interface Props {
    attivo?: boolean;
    privato?: boolean;
    disabled?: boolean;
    onchange?: (attivo: boolean) => void;
  }

  let {
    attivo = $bindable(false),
    privato = false,
    disabled = false,
    onchange,
  }: Props = $props();

  function toggle() {
    if (disabled) return;
    attivo = !attivo;
    onchange?.(attivo);
  }
</script>

<button
  class="switch"
  class:switch--privato={privato}
  role="switch"
  aria-checked={attivo}
  {disabled}
  onclick={toggle}
  type="button"
></button>

<style>
  .switch {
    position: relative;
    width: 36px;
    height: 20px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: 999px;
    cursor: pointer;
    padding: 0;
    transition: background var(--motion-fast);
    flex-shrink: 0;
  }

  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    background: var(--text-muted);
    border-radius: 50%;
    transition:
      transform var(--motion-normal) var(--easing-standard),
      background var(--motion-fast);
  }

  .switch[aria-checked="true"] {
    background: var(--accent-team);
    border-color: transparent;
  }
  .switch[aria-checked="true"]::after {
    transform: translateX(16px);
    background: var(--accent-team-on);
  }

  .switch--privato[aria-checked="true"] {
    background: var(--accent-private);
  }
  .switch--privato[aria-checked="true"]::after {
    background: var(--accent-private-on);
  }

  .switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
