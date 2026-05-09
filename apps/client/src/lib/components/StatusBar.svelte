<script lang="ts">
  // F1: placeholder. F7 lo wire-uppa con tooltip vault dinamico,
  // dot save status reattivo, nome prompt corrente, ecc.
</script>

<footer class="statusbar">
  <div class="seg">
    <span class="dot dot-ok" aria-hidden="true"></span>
    <span>vault locale</span>
  </div>
  <span class="sep" aria-hidden="true"></span>
  <div class="seg">
    <span>(nessun prompt selezionato)</span>
  </div>
  <div class="right">
    <span class="seg">
      <kbd>⌃⇧P</kbd>
      <span class="muted">cerca</span>
    </span>
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: var(--statusbar-h);
    padding: 0 var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .seg {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--text-muted);
  }

  .dot-ok {
    background: var(--success);
  }

  .sep {
    width: 1px;
    height: 14px;
    background: var(--border-subtle);
  }

  .right {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-3);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px var(--sp-1);
    border-radius: var(--radius-sm);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    color: var(--text-default);
  }

  .muted {
    color: var(--text-subtle);
  }
</style>
