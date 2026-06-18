<script lang="ts">
  /**
   * Invito non invadente al tour di benvenuto (Guida — Fase 1 PR-2).
   *
   * Compare la prima volta che si raggiunge la libreria senza aver mai visto il
   * tour (`!tourBenvenutoVisto()`), tipicamente subito dopo l'onboarding. È
   * *offerto, mai forzato*: una card in basso a destra con "Avvia il tour" /
   * "Non ora". In entrambi i casi il flag "visto" viene segnato, così non si
   * ripropone a ogni avvio; il tour resta sempre rilanciabile da
   * Impostazioni → Guida e aiuto.
   */
  import { Compass, X } from "lucide-svelte";
  import {
    tourBenvenutoVisto,
    segnaTourBenvenutoVisto,
    richiediTourBenvenuto,
  } from "$lib/aiuto/tour.svelte";

  let mostra = $state(!tourBenvenutoVisto());

  function avvia(): void {
    // Nasconde l'invito subito, poi richiede il tour: lo Shell lo esegue in
    // modo asincrono (doppio rAF). Il flag "visto" lo segna `onDestroyed` del
    // tour, così se l'app si chiude prima dell'avvio l'invito si ripropone.
    mostra = false;
    richiediTourBenvenuto();
  }

  function rifiuta(): void {
    // "Non ora"/chiudi: segna come visto così non si ripropone. Il tour resta
    // sempre rilanciabile dall'hub Guida e aiuto.
    segnaTourBenvenutoVisto();
    mostra = false;
  }
</script>

{#if mostra}
  <!--
    role="status" + aria-live="polite": è un invito toast non invadente, NON
    una dialog modale. Non cattura il focus (sarebbe ostile subito dopo
    l'onboarding) e viene annunciato cortesemente dagli screen reader quando
    compare. La chiusura avviene via i pulsanti, non via ESC.
  -->
  <div class="tour-invito" role="status" aria-live="polite">
    <button class="chiudi" aria-label="Chiudi invito" onclick={rifiuta}>
      <X size={16} aria-hidden="true" />
    </button>
    <div class="testo">
      <Compass class="icona" size={20} aria-hidden="true" />
      <div>
        <p class="titolo">Primo giro da queste parti?</p>
        <p class="sotto">Ti mostro l'interfaccia in un breve tour guidato.</p>
      </div>
    </div>
    <div class="azioni">
      <button class="non-ora" onclick={rifiuta}>Non ora</button>
      <button class="avvia" onclick={avvia}>Avvia il tour</button>
    </div>
  </div>
{/if}

<style>
  .tour-invito {
    position: relative;
    width: 320px;
    max-width: calc(100vw - 40px);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25));
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .testo {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .testo :global(.icona) {
    flex-shrink: 0;
    margin-top: 2px;
    color: var(--accent-private);
  }

  .titolo {
    margin: 0 0 2px;
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .sotto {
    margin: 0;
    font-size: var(--fs-xs);
    color: var(--text-muted);
    line-height: 1.4;
  }

  .azioni {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .azioni button {
    padding: 6px 14px;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: 1px solid transparent;
  }

  .non-ora {
    background: transparent;
    border-color: var(--border-default);
    color: var(--text-muted);
  }

  .non-ora:hover {
    background: var(--bg-overlay);
    color: var(--text-strong);
  }

  .avvia {
    background: var(--accent-private);
    color: var(--accent-private-on);
  }

  .avvia:hover {
    filter: brightness(1.05);
  }

  .azioni button:focus-visible,
  .chiudi:focus-visible {
    outline: 2px solid var(--accent-private);
    outline-offset: 2px;
  }

  .chiudi {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 24px;
    height: 24px;
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
</style>
