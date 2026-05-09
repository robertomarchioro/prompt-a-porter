<script lang="ts">
  type StatoSalvataggio = "salvato" | "dirty" | "salvando" | "errore";

  interface Props {
    statoSalvataggio: StatoSalvataggio;
    righe: number;
    colonna: number;
    chars: number;
  }

  let { statoSalvataggio, righe, colonna, chars }: Props = $props();

  const tok = $derived(Math.ceil(chars / 4));

  const labelStato = $derived.by(() => {
    switch (statoSalvataggio) {
      case "salvato":
        return "salvato";
      case "salvando":
        return "salvando…";
      case "dirty":
        return "modifiche non salvate";
      case "errore":
        return "errore salvataggio";
    }
  });
</script>

<div class="indicator" data-stato={statoSalvataggio}>
  <span class="seg stato">
    <span class="dot" aria-hidden="true"></span>
    <span>{labelStato}</span>
  </span>
  <span class="sep" aria-hidden="true"></span>
  <span class="seg mono">L:{righe}</span>
  <span class="seg mono">C:{colonna}</span>
  <span class="seg mono">{chars} char</span>
  <span class="seg mono">~{tok} tok</span>
</div>

<style>
  .indicator {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 24px;
    padding: 0 var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .seg {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .sep {
    width: 1px;
    height: 12px;
    background: var(--border-subtle);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--text-subtle);
  }

  .indicator[data-stato="salvato"] .dot {
    background: var(--success);
  }

  .indicator[data-stato="dirty"] .dot {
    background: var(--warning);
  }

  .indicator[data-stato="salvando"] .dot {
    background: var(--info);
    animation: pulse 1s ease-in-out infinite;
  }

  .indicator[data-stato="errore"] .dot {
    background: var(--danger);
  }

  .indicator[data-stato="errore"] .stato {
    color: var(--danger);
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }
</style>
