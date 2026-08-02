<script lang="ts">
  /**
   * Host globale per `conferma()`/`avvisa()` (`$lib/util/conferma.ts`),
   * montato una sola volta in App.svelte. Consuma la coda di
   * `$lib/stores/dialogo.svelte.ts` e mostra sempre e solo la richiesta in
   * testa, riusando i primitivi esistenti `Modale` (conferma) e `Toast`
   * (avviso) — nessun componente nuovo.
   *
   * Attivo su tutte le piattaforme ed è l'**unica** strada: da quando
   * `tauri-plugin-dialog` sostituisce `window.confirm` con una chiamata a
   * `plugin:dialog|confirm` — comando che la crate non registra —
   * `conferma()`/`avvisa()` non possono più delegare alla webview (vedi
   * `$lib/util/conferma.ts`). Montato in ogni finestra: se mancasse, una
   * conferma chiamata da lì resterebbe appesa per sempre.
   */
  import Modale from "./Modale.svelte";
  import Toast from "./Toast.svelte";
  import Button from "./Button.svelte";
  import { statoDialoghi, rimuoviDallaCoda } from "$lib/stores/dialogo.svelte";

  const AVVISO_DURATA_MS = 3200;

  const richiesta = $derived(statoDialoghi.coda[0] ?? null);

  function chiudiConferma(esito: boolean): void {
    if (!richiesta || richiesta.tipo !== "conferma") return;
    richiesta.risolvi(esito);
    rimuoviDallaCoda(richiesta.id);
  }

  // L'avviso si autochiude come i Toast già usati altrove nell'app (es.
  // CestinoVista), risolvendo la Promise di `avvisa()` allo scadere.
  $effect(() => {
    if (richiesta?.tipo !== "avviso") return;
    const { id, risolvi } = richiesta;
    const timer = setTimeout(() => {
      risolvi();
      rimuoviDallaCoda(id);
    }, AVVISO_DURATA_MS);
    return () => clearTimeout(timer);
  });
</script>

{#if richiesta?.tipo === "conferma"}
  <Modale
    titolo="Conferma"
    larghezza="sm"
    onChiudi={() => chiudiConferma(false)}
  >
    <p class="messaggio">{richiesta.messaggio}</p>
    {#snippet footer()}
      <Button variante="ghost" onclick={() => chiudiConferma(false)}>
        Annulla
      </Button>
      <Button variante="danger" onclick={() => chiudiConferma(true)}>
        Conferma
      </Button>
    {/snippet}
  </Modale>
{/if}

{#if richiesta?.tipo === "avviso"}
  <Toast variante="danger">{richiesta.messaggio}</Toast>
{/if}

<style>
  .messaggio {
    margin: 0;
    white-space: pre-line;
    color: var(--text-default);
    font-size: var(--fs-sm);
    line-height: 1.5;
  }
</style>
