<script lang="ts">
  import {
    diffParole,
    diffSideBySide,
    type Segment,
    type RigaSideBySide,
  } from "$lib/diff";

  type Modalita = "unified" | "side-by-side";

  interface Props {
    /// Testo "vecchio" (lato sinistro nel side-by-side, contesto delle
    /// rimozioni nell'unified).
    a: string;
    /// Testo "nuovo".
    b: string;
    /// Modalità di rendering del diff.
    modalita: Modalita;
    /// Etichette per i due testi (mostrate nei titoli colonna o
    /// nell'header). Default "v1"/"v2".
    etichettaA?: string;
    etichettaB?: string;
  }

  let { a, b, modalita, etichettaA = "v1", etichettaB = "v2" }: Props = $props();

  const segmenti = $derived<Segment[]>(
    modalita === "unified" ? diffParole(a, b) : [],
  );
  const righe = $derived<RigaSideBySide[]>(
    modalita === "side-by-side" ? diffSideBySide(a, b) : [],
  );
</script>

{#if modalita === "unified"}
  <pre class="vd-unified">{#each segmenti as seg, i (i)}<span
        class="vd-seg vd-seg--{seg.tipo}">{seg.testo}</span
      >{/each}</pre>
{:else}
  <div class="vd-sbs" role="table" aria-label="Confronto fianco a fianco">
    <div class="vd-sbs-header">
      <div class="vd-sbs-col vd-sbs-col--left">{etichettaA}</div>
      <div class="vd-sbs-col vd-sbs-col--right">{etichettaB}</div>
    </div>
    {#each righe as r, i (i)}
      <div class="vd-sbs-row vd-sbs-row--{r.stato}" role="row">
        <div class="vd-sbs-side vd-sbs-side--left">
          <span class="vd-num">{r.numeroA ?? ""}</span>
          <span class="vd-text">{r.testoA ?? ""}</span>
        </div>
        <div class="vd-sbs-side vd-sbs-side--right">
          <span class="vd-num">{r.numeroB ?? ""}</span>
          <span class="vd-text">{r.testoB ?? ""}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .vd-unified {
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: var(--lh-relaxed);
    color: var(--text-default);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    overflow: auto;
  }

  /* .vd-seg base: nessuna decorazione (parte invariata fra versioni).
     Le varianti aggiunto/rimosso sotto applicano highlight semantici. */

  .vd-seg--aggiunto {
    background: rgba(108, 184, 108, 0.18);
    color: var(--accent-success, #2c8a2c);
    text-decoration: underline;
    text-decoration-color: rgba(108, 184, 108, 0.6);
  }

  .vd-seg--rimosso {
    background: rgba(217, 83, 79, 0.18);
    color: var(--accent-danger, #b73c38);
    text-decoration: line-through;
    text-decoration-color: rgba(217, 83, 79, 0.6);
  }

  .vd-sbs {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    background: var(--bg-overlay);
  }

  .vd-sbs-header {
    display: grid;
    grid-template-columns: 1fr 1fr;
    background: var(--bg-canvas);
    border-bottom: 1px solid var(--border-subtle);
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .vd-sbs-col {
    padding: var(--sp-2) var(--sp-3);
  }

  .vd-sbs-col--right {
    border-left: 1px solid var(--border-subtle);
  }

  .vd-sbs-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    border-bottom: 1px solid var(--border-subtle);
  }

  .vd-sbs-row:last-child {
    border-bottom: none;
  }

  .vd-sbs-row--cambiata {
    background: rgba(217, 162, 95, 0.08);
  }

  .vd-sbs-row--aggiunta {
    background: rgba(108, 184, 108, 0.10);
  }

  .vd-sbs-row--rimossa {
    background: rgba(217, 83, 79, 0.10);
  }

  .vd-sbs-side {
    display: flex;
    align-items: flex-start;
    gap: var(--sp-2);
    padding: 4px var(--sp-3);
    min-height: 1.5em;
  }

  .vd-sbs-side--right {
    border-left: 1px solid var(--border-subtle);
  }

  .vd-num {
    flex-shrink: 0;
    width: 32px;
    text-align: right;
    color: var(--text-subtle);
    font-size: var(--fs-xs);
    user-select: none;
    padding-top: 2px;
  }

  .vd-text {
    flex: 1;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
