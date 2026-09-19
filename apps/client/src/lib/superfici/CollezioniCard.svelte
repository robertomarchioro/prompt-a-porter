<script lang="ts">
  /**
   * Card «Collezioni» (Impostazioni → Dati): alias della modale raggiungibile
   * dal footer della sidebar. Qui l'elenco si carica solo al click su
   * «Sfoglia»: aprire le impostazioni non deve toccare la rete.
   * L'elenco e le azioni sono in `CollezioniLista.svelte`.
   */
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import CollezioniLista from "./CollezioniLista.svelte";

  let lista = $state<CollezioniLista | undefined>();
  let sfogliato = $state(false);
  let occupato = $state(false);

  async function sfoglia(): Promise<void> {
    sfogliato = true;
    await lista?.sfoglia();
  }
</script>

<div class="card">
  <header class="card-h">
    <span class="card-title">
      Collezioni di prompt
      <AiutoLink chiave="collezioni" dimensione={16} />
    </span>
  </header>
  <p class="card-desc">
    Raccolte di prompt curate dal progetto, una per tipo di lavoro, scritte
    per mostrare segnaposti, import e varianti. Scaricate da GitHub solo
    quando premi <strong>Sfoglia</strong>; l'importazione non tocca mai i
    prompt che hai già, e <strong>Aggiorna</strong> riallinea solo quelli che
    non hai modificato. Finiscono nella cartella <code>Collezioni</code>.
    Le trovi anche dal pulsante <strong>Collezioni</strong> in fondo alla
    sidebar.
  </p>

  <button type="button" class="btn" onclick={sfoglia} disabled={occupato}>
    {sfogliato ? "Aggiorna elenco" : "Sfoglia collezioni…"}
  </button>

  <CollezioniLista bind:this={lista} onOccupato={(o) => (occupato = o)} />
</div>

<style>
  /* Stessi token delle card «dati-*» di ImpostazioniModal: il componente è
     separato per non far crescere ancora quel file, gli stili sono scoped. */
  .card {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    margin-bottom: var(--sp-3);
  }
  .card-h {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--sp-2);
  }
  .card-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }
  .card-desc {
    margin: 0 0 var(--sp-2) 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }
  .card-desc code {
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 0.9em;
  }
  .btn {
    padding: 6px 14px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    cursor: pointer;
  }
  .btn:hover:not(:disabled) {
    background: var(--bg-surface);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
