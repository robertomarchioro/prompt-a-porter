<script lang="ts">
  /**
   * Widget checklist "Primi passi" (Guida — Fase 3).
   *
   * Card compatta (in basso a destra, posizionata dal contenitore in Shell) che
   * traccia le 5 prime azioni chiave e dà senso di progresso. Dismissibile.
   * Lo stato vive in `primi-passi.svelte.ts` (flag localStorage per-azione).
   */
  import { Check, Circle, X, Sparkles } from "lucide-svelte";
  import {
    primiPassi,
    PASSI,
    chiudiPrimiPassi,
  } from "$lib/aiuto/primi-passi.svelte";
</script>

{#if !primiPassi.chiusa}
  <div class="primi-passi" role="status">
    <header>
      <span class="titolo">Primi passi</span>
      <span class="progresso">{primiPassi.completati}/{primiPassi.totale}</span>
      <button
        class="chiudi"
        aria-label="Chiudi i primi passi"
        onclick={chiudiPrimiPassi}
      >
        <X size={16} aria-hidden="true" />
      </button>
    </header>

    {#if primiPassi.tutti}
      <p class="fatto">
        <Sparkles size={14} aria-hidden="true" /> Hai imparato le basi. Buon lavoro!
      </p>
    {/if}

    <ul>
      {#each PASSI as p (p.id)}
        <li class:done={primiPassi.fatti[p.id]}>
          {#if primiPassi.fatti[p.id]}
            <Check size={15} class="ic-done" aria-hidden="true" />
          {:else}
            <Circle size={15} class="ic-todo" aria-hidden="true" />
          {/if}
          <span>{p.label}</span>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .primi-passi {
    width: 300px;
    max-width: calc(100vw - 40px);
    padding: 14px 16px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25));
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
  }

  .titolo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .progresso {
    font-size: var(--fs-xs);
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    margin-left: auto;
  }

  .fatto {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 0 10px;
    font-size: var(--fs-xs);
    color: var(--accent-private);
    font-weight: var(--fw-medium);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 7px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: var(--fs-sm);
    color: var(--text-default);
  }

  li.done span {
    color: var(--text-subtle);
    text-decoration: line-through;
  }

  li :global(.ic-done) {
    color: var(--accent-private);
    flex-shrink: 0;
  }

  li :global(.ic-todo) {
    color: var(--text-subtle);
    flex-shrink: 0;
  }

  .chiudi {
    width: 22px;
    height: 22px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-subtle);
    cursor: pointer;
  }

  .chiudi:hover {
    background: var(--bg-overlay);
    color: var(--text-strong);
  }

  .chiudi:focus-visible {
    outline: 2px solid var(--accent-private);
    outline-offset: 2px;
  }
</style>
