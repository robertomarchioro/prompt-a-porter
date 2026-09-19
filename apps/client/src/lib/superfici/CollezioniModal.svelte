<script lang="ts">
  /**
   * Modale «Collezioni di prompt», aperta dal footer della sidebar
   * (`apriModale({ tipo: "collezioni" })`). È il punto di menù da cui
   * scaricare e aggiornare le collezioni curate; la card in Impostazioni →
   * Dati è un alias. Aprirla è l'azione esplicita: l'elenco parte subito.
   */
  import Modale from "$lib/components/Modale.svelte";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import CollezioniLista from "./CollezioniLista.svelte";

  interface Props {
    onChiudi: () => void;
  }

  let { onChiudi }: Props = $props();
</script>

<Modale
  titolo="Collezioni di prompt"
  sottotitolo="Raccolte curate dal progetto, una per tipo di lavoro. Importa quelle che ti servono; quando una cambia, Aggiorna riallinea solo i prompt che non hai modificato."
  larghezza="md"
  {onChiudi}
>
  <div class="intestazione">
    <span class="nota">
      Scaricate da GitHub e importate nella cartella <code>Collezioni</code>;
      i tuoi prompt non vengono mai sovrascritti.
    </span>
    <AiutoLink chiave="collezioni" dimensione={16} />
  </div>
  <CollezioniLista caricaSubito />
</Modale>

<style>
  .intestazione {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }
  .nota {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }
  .nota code {
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 0.9em;
  }
</style>
