<script lang="ts">
  /**
   * Modale per creare una nuova cartella (issue #301).
   *
   * Chiama `folder_crea` con { nome, parent_folder_id: null } per creare
   * una cartella radice. Al successo emette `pap:lista-mutata` (pattern
   * già usato da ListPane) così la Sidebar si aggiorna automaticamente,
   * poi chiude la modale.
   *
   * Focus trap e chiusura ESC gestiti dalla primitive Modale (F10 PR-B).
   */
  import { invoke } from "@tauri-apps/api/core";
  import Modale from "$lib/components/Modale.svelte";
  import { nomeValido, eseguiCreaCartella } from "./nuova-cartella-logic";

  interface Props {
    onChiudi: () => void;
  }

  let { onChiudi }: Props = $props();

  let nome = $state("");
  let errore = $state<string | null>(null);
  let invio = $state(false);

  async function onSubmit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (!nomeValido(nome) || invio) return;

    errore = null;
    invio = true;

    const risultato = await eseguiCreaCartella(
      nome,
      invoke as (
        cmd: string,
        args: { dati: { nome: string; parent_folder_id: null } },
      ) => Promise<string>,
      () => window.dispatchEvent(new CustomEvent("pap:lista-mutata")),
      onChiudi,
    );

    if (!risultato.ok) {
      errore = risultato.errore ?? "Impossibile creare la cartella.";
    }

    invio = false;
  }
</script>

<Modale
  titolo="Nuova cartella"
  sottotitolo="Dai un nome alla cartella da aggiungere alla libreria."
  larghezza="sm"
  {onChiudi}
>
  <form class="form" onsubmit={onSubmit}>
    <label class="campo" for="nome-cartella">
      <span class="etichetta">Nome</span>
      <input
        id="nome-cartella"
        class="input"
        type="text"
        placeholder="es. Marketing"
        bind:value={nome}
        autocomplete="off"
        maxlength="100"
        required
      />
    </label>

    {#if errore}
      <p class="errore" role="alert">{errore}</p>
    {/if}

    <div class="azioni">
      <button
        class="btn-secondario"
        type="button"
        onclick={onChiudi}
      >
        Annulla
      </button>
      <button
        class="btn-primario"
        type="submit"
        disabled={!nomeValido(nome) || invio}
      >
        {invio ? "Creazione…" : "Crea cartella"}
      </button>
    </div>
  </form>
</Modale>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .etichetta {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }

  .input {
    width: 100%;
    padding: var(--sp-2) var(--sp-2);
    background: var(--bg-canvas);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-md);
    font-family: var(--font-ui);
    box-sizing: border-box;
  }

  .input:focus {
    outline: 2px solid var(--accent-team);
    outline-offset: -1px;
  }

  .input::placeholder {
    color: var(--text-subtle);
  }

  .errore {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--color-danger, #e53e3e);
  }

  .azioni {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-2);
  }

  .btn-secondario {
    padding: var(--sp-1) var(--sp-3);
    border: 1px solid var(--border-default);
    background: transparent;
    color: var(--text-default);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
    cursor: pointer;
  }

  .btn-secondario:hover {
    background: var(--bg-overlay);
  }

  .btn-primario {
    padding: var(--sp-1) var(--sp-3);
    border: 0;
    background: var(--accent-team);
    color: #fff;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
    font-weight: var(--fw-medium);
    cursor: pointer;
  }

  .btn-primario:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-primario:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
