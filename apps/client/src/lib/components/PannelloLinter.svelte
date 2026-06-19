<script lang="ts">
  /**
   * Pannello "Linter" (Impostazioni). Mostra il catalogo delle regole dal
   * backend (`prompt_lint_regole`, fonte di verità unica) raggruppate per
   * categoria, con toggle a granularità singola regola + toggle famiglia.
   * Un avviso è nascosto se la famiglia (prefisso) O la singola regola (code)
   * è disabilitata — stessa semantica del backend `filtra_disabilitate`.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Switch from "./Switch.svelte";
  import Toast from "./Toast.svelte";
  import {
    CATEGORIE_LINTER,
    ETICHETTE,
    type CategoriaLinter,
    leggiRegoleDisabilitate,
    salvaRegoleDisabilitate,
    toggleRegola,
  } from "$lib/preferenze-linter";

  interface RegolaMeta {
    code: string;
    categoria: CategoriaLinter;
    severita_default: "error" | "warning" | "info";
    titolo: string;
    descrizione: string;
    configurabile: boolean;
  }

  let catalogo = $state<RegolaMeta[]>([]);
  let disabilitate = $state<string[]>([]);
  let toastVisibile = $state(false);
  let toastTesto = $state("");
  let errore = $state(false);

  onMount(async () => {
    disabilitate = leggiRegoleDisabilitate();
    try {
      catalogo = await invoke<RegolaMeta[]>("prompt_lint_regole");
    } catch (e) {
      console.error("[linter] catalogo regole", e);
      errore = true;
    }
  });

  const perCategoria = $derived(
    CATEGORIE_LINTER.map((cat) => ({
      cat,
      regole: catalogo.filter((r) => r.categoria === cat),
    })).filter((g) => g.regole.length > 0),
  );

  function showToast(testo: string): void {
    toastTesto = testo;
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 1500);
  }

  // Salva + notifica DiagnosiTab di ri-lintare (altrimenti i risultati restano
  // stale finché l'utente non riedita il body).
  function salvaENotifica(): void {
    salvaRegoleDisabilitate(disabilitate);
    window.dispatchEvent(new CustomEvent("pap:linter-config-cambiata"));
  }

  function famigliaAttiva(cat: CategoriaLinter): boolean {
    return !disabilitate.includes(cat);
  }

  function regolaAttiva(code: string): boolean {
    return !disabilitate.includes(code);
  }

  function toggleFamiglia(cat: CategoriaLinter): void {
    disabilitate = toggleRegola(cat, disabilitate);
    salvaENotifica();
    showToast(
      `Famiglia ${cat} ${famigliaAttiva(cat) ? "attivata" : "disattivata"}`,
    );
  }

  function toggleSingola(r: RegolaMeta): void {
    disabilitate = toggleRegola(r.code, disabilitate);
    salvaENotifica();
    showToast(`${r.code} ${regolaAttiva(r.code) ? "attivata" : "disattivata"}`);
  }

  function riattivaTutte(): void {
    disabilitate = [];
    salvaENotifica();
    showToast("Tutte le regole riattivate");
  }
</script>

<div class="pannello">
  <h3 class="titolo">Linter — regole attive</h3>
  <p class="desc">
    Disattiva i singoli avvisi (o un'intera famiglia) che non vuoi vedere
    durante l'editing. Le impostazioni sono salvate nel browser locale (no sync
    server).
  </p>

  {#if errore}
    <p class="msg-err">
      Impossibile caricare il catalogo delle regole dal backend.
    </p>
  {/if}

  {#each perCategoria as gruppo (gruppo.cat)}
    {@const famAttiva = famigliaAttiva(gruppo.cat)}
    <section class="categoria">
      <header class="cat-head">
        <span class="cat-nome">{ETICHETTE[gruppo.cat]}</span>
        <Switch
          attivo={famAttiva}
          etichetta="Famiglia {ETICHETTE[gruppo.cat]}"
          onchange={() => toggleFamiglia(gruppo.cat)}
        />
      </header>

      <ul class="lista" class:famiglia-off={!famAttiva} role="list">
        {#each gruppo.regole as r (r.code)}
          <li class="regola">
            <div class="regola-info">
              <div class="regola-top">
                <span class="regola-titolo">{r.titolo}</span>
                <code class="regola-code">{r.code}</code>
                <span class="badge sev-{r.severita_default}"
                  >{r.severita_default}</span
                >
              </div>
              <div class="regola-desc">{r.descrizione}</div>
            </div>
            <Switch
              attivo={regolaAttiva(r.code)}
              disabled={!famAttiva}
              etichetta="Regola {r.code}"
              onchange={() => toggleSingola(r)}
            />
          </li>
        {/each}
      </ul>
      {#if !famAttiva}
        <p class="nota-fam">
          Intera famiglia disattivata: gli avvisi sono nascosti a prescindere
          dalle singole regole.
        </p>
      {/if}
    </section>
  {/each}

  {#if disabilitate.length > 0}
    <div class="actions">
      <button type="button" class="link-btn" onclick={riattivaTutte}>
        Riattiva tutto ({disabilitate.length} disattivate)
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
    color: var(--text-muted);
    font-size: var(--fs-sm);
    line-height: 1.5;
  }
  .categoria {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    overflow: hidden;
  }
  .cat-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
  }
  .cat-nome {
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    font-size: var(--fs-sm);
  }
  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }
  .lista.famiglia-off {
    opacity: 0.5;
  }
  .regola {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-top: 1px solid var(--border-subtle);
  }
  .regola:first-child {
    border-top: none;
  }
  .regola-info {
    flex: 1;
    min-width: 0;
  }
  .regola-top {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .regola-titolo {
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    font-size: var(--fs-sm);
  }
  .regola-code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    background: var(--bg-input);
    border-radius: 4px;
    padding: 1px 5px;
  }
  .regola-desc {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    margin-top: 3px;
    line-height: 1.4;
  }
  .badge {
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.02em;
    border-radius: 4px;
    padding: 0 5px;
    border: 1px solid transparent;
  }
  .sev-error {
    color: var(--danger, #e5484d);
    border-color: color-mix(in srgb, var(--danger, #e5484d) 40%, transparent);
  }
  .sev-warning {
    color: var(--warning, #f5a623);
    border-color: color-mix(in srgb, var(--warning, #f5a623) 40%, transparent);
  }
  .sev-info {
    color: var(--text-muted);
    border-color: var(--border-default);
  }
  .nota-fam {
    margin: 0;
    padding: 8px 14px;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    background: var(--bg-surface);
  }
  .msg-err {
    margin: 0;
    color: var(--danger, #e5484d);
    font-size: var(--fs-sm);
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
