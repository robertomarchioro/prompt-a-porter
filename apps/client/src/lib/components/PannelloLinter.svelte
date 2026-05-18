<script lang="ts">
  import Switch from "./Switch.svelte";
  import Toast from "./Toast.svelte";
  import {
    CATEGORIE_LINTER,
    DESCRIZIONI,
    ETICHETTE,
    type CategoriaLinter,
    leggiCategorieDisabilitate,
    salvaCategorieDisabilitate,
    toggleCategoria,
  } from "$lib/preferenze-linter";

  let disabilitate = $state<CategoriaLinter[]>([]);
  let toastVisibile = $state(false);
  let toastTesto = $state("");

  $effect(() => {
    disabilitate = leggiCategorieDisabilitate();
  });

  function showToast(testo: string) {
    toastTesto = testo;
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 1500);
  }

  function handleToggle(cat: CategoriaLinter) {
    disabilitate = toggleCategoria(cat, disabilitate);
    salvaCategorieDisabilitate(disabilitate);
    const attiva = !disabilitate.includes(cat);
    showToast(`${cat} ${attiva ? "attivata" : "disattivata"}`);
  }

  function attivaTutte() {
    disabilitate = [];
    salvaCategorieDisabilitate(disabilitate);
    showToast("Tutte le categorie attivate");
  }
</script>

<div class="pannello">
  <h3 class="titolo">Linter — categorie attive</h3>
  <p class="desc">
    Disattiva le categorie di avvisi che non vuoi vedere durante l'editing.
    Le impostazioni sono salvate nel browser locale (no sync server).
  </p>

  <ul class="lista" role="list">
    {#each CATEGORIE_LINTER as cat (cat)}
      {@const attiva = !disabilitate.includes(cat)}
      <li class="card">
        <div class="card-info">
          <div class="card-nome">{ETICHETTE[cat]}</div>
          <div class="card-desc">{DESCRIZIONI[cat]}</div>
        </div>
        <Switch
          attivo={attiva}
          etichetta="Categoria linter {ETICHETTE[cat]}"
          onchange={() => handleToggle(cat)}
        />
      </li>
    {/each}
  </ul>

  {#if disabilitate.length > 0}
    <div class="actions">
      <button type="button" class="link-btn" onclick={attivaTutte}>
        Riattiva tutte ({disabilitate.length} disattivate)
      </button>
    </div>
  {/if}
</div>

<Toast visibile={toastVisibile}>{toastTesto}</Toast>

<style>
  .pannello {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3, 1rem);
  }
  .titolo {
    margin: 0;
    font-size: var(--fs-base, 1.125rem);
    font-weight: var(--fw-semibold, 600);
  }
  .desc {
    margin: 0;
    color: var(--text-muted, #888);
    font-size: var(--fs-sm, 0.875rem);
    line-height: 1.5;
  }
  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2, 0.5rem);
  }
  .card {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    background: var(--bg-raised);
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .card-info {
    flex: 1;
    min-width: 0;
  }
  .card-nome {
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    font-size: var(--fs-sm);
  }
  .card-desc {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    margin-top: 2px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  .link-btn {
    background: none;
    border: none;
    color: var(--accent-team);
    font-size: var(--fs-xs);
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
  }
  .link-btn:hover {
    color: var(--accent-team-strong);
  }
</style>
