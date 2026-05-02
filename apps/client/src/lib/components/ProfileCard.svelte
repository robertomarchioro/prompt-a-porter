<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    titolo: string;
    descrizione: string;
    dettagli: string[];
    selezionato?: boolean;
    variante?: "private" | "team";
    onclick?: () => void;
    icona?: Snippet;
  }

  let {
    titolo,
    descrizione,
    dettagli,
    selezionato = false,
    variante = "private",
    onclick,
    icona,
  }: Props = $props();
</script>

<button
  class="card"
  class:card--selezionato={selezionato}
  class:card--team={variante === "team"}
  aria-pressed={selezionato}
  {onclick}
  type="button"
>
  <div class="card-header">
    {#if icona}
      <div class="card-icona">
        {@render icona()}
      </div>
    {/if}
    <div>
      <h3 class="card-titolo">{titolo}</h3>
      <p class="card-desc">{descrizione}</p>
    </div>
  </div>
  <ul class="card-dettagli">
    {#each dettagli as d}
      <li>{d}</li>
    {/each}
  </ul>
</button>

<style>
  .card {
    --_accent: var(--accent-private);
    --_accent-soft: var(--accent-private-soft);

    appearance: none;
    text-align: left;
    width: 100%;
    padding: var(--sp-4);
    background: var(--bg-input);
    border: 2px solid var(--border-default);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast),
      background var(--motion-fast);
    font-family: var(--font-ui);
    color: var(--text-default);
  }

  .card--team {
    --_accent: var(--accent-team);
    --_accent-soft: var(--accent-team-soft);
  }

  .card:hover {
    border-color: var(--border-strong);
    background: var(--bg-overlay);
  }

  .card--selezionato {
    border-color: var(--_accent);
    box-shadow: 0 0 0 3px var(--_accent-soft);
    background: var(--bg-overlay);
  }

  .card:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--bg-canvas), 0 0 0 4px var(--_accent);
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    gap: var(--sp-3);
    margin-bottom: var(--sp-3);
  }

  .card-icona {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--_accent-soft);
    border-radius: var(--radius-md);
    color: var(--_accent);
    flex-shrink: 0;
  }

  .card-titolo {
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    margin: 0 0 2px;
  }

  .card-desc {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .card-dettagli {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .card-dettagli li {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    padding-left: var(--sp-3);
    position: relative;
  }

  .card-dettagli li::before {
    content: "•";
    position: absolute;
    left: var(--sp-1);
    color: var(--_accent);
  }
</style>
