<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, EmptyState, Toast } from "$lib/components";

  interface VersioneStorica {
    id: string;
    prompt_id: string;
    version: number;
    titolo: string;
    descrizione: string | null;
    body: string;
    visibilita: string | null;
    target_model: string | null;
    creato_a: string;
    creato_da_user_id: string;
  }

  interface Props {
    promptId: string;
    onchiudi: () => void;
    onrollback?: () => void;
  }

  let { promptId, onchiudi, onrollback }: Props = $props();

  let versioni = $state<VersioneStorica[]>([]);
  let versioneSelezionata = $state<VersioneStorica | null>(null);
  let caricamento = $state(true);
  let errore = $state("");
  let confermaRollback = $state<number | null>(null);
  let messaggioToast = $state("");

  $effect(() => {
    carica();
  });

  async function carica() {
    caricamento = true;
    try {
      versioni = await invoke<VersioneStorica[]>("prompt_get_history", { promptId });
      if (versioni.length > 0 && !versioneSelezionata) {
        versioneSelezionata = versioni[0];
      }
    } catch (e) {
      errore = String(e);
    } finally {
      caricamento = false;
    }
  }

  async function eseguiRollback() {
    if (confermaRollback === null) return;
    const target = confermaRollback;
    try {
      await invoke("prompt_rollback", {
        promptId,
        targetVersion: target,
      });
      messaggioToast = `Ripristinata versione v${target}`;
      confermaRollback = null;
      onrollback?.();
      versioneSelezionata = null;
      await carica();
    } catch (e) {
      errore = String(e);
    }
  }

  function tempoRelativo(iso: string): string {
    const ora = new Date();
    const past = new Date(iso.endsWith("Z") ? iso : iso + "Z");
    const diffSec = Math.floor((ora.getTime() - past.getTime()) / 1000);
    if (diffSec < 60) return "ora";
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m fa`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h fa`;
    if (diffSec < 86400 * 2) return "ieri";
    if (diffSec < 86400 * 30) return `${Math.floor(diffSec / 86400)}g fa`;
    return past.toLocaleDateString("it-IT", {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  }

  function gestisciTastiera(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (confermaRollback !== null) {
        confermaRollback = null;
      } else {
        onchiudi();
      }
    }
  }

  const isTesta = $derived(
    versioneSelezionata !== null &&
      versioni.length > 0 &&
      versioni[0].id === versioneSelezionata.id,
  );
</script>

<svelte:window onkeydown={gestisciTastiera} />

<div
  class="overlay"
  onclick={onchiudi}
  role="presentation"
>
  <div
    class="modale"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label="Cronologia versioni"
  >
    <header class="modale-header">
      <h2 class="modale-titolo">Cronologia versioni</h2>
      <button
        type="button"
        class="modale-chiudi"
        onclick={onchiudi}
        aria-label="Chiudi"
      >×</button>
    </header>

    <div class="modale-body">
      {#if caricamento}
        <p class="hint">Caricamento…</p>
      {:else if errore}
        <p class="errore">{errore}</p>
      {:else if versioni.length === 0}
        <div class="empty-wrap">
          <EmptyState
            titolo="Nessuna versione"
            descrizione="Questo prompt non ha ancora una storia di modifiche."
          />
        </div>
      {:else}
        <div class="split">
          <aside class="lista-versioni" aria-label="Lista versioni">
            {#each versioni as v (v.id)}
              <button
                type="button"
                class="versione-item"
                class:versione-item--attiva={versioneSelezionata?.id === v.id}
                aria-current={versioneSelezionata?.id === v.id}
                onclick={() => (versioneSelezionata = v)}
              >
                <div class="versione-header">
                  <span class="versione-numero">v{v.version}</span>
                  {#if versioni[0]?.id === v.id}
                    <span class="badge-corrente">testa</span>
                  {/if}
                </div>
                <div class="versione-meta">{tempoRelativo(v.creato_a)}</div>
                <div class="versione-titolo">{v.titolo}</div>
              </button>
            {/each}
          </aside>

          <main class="preview">
            {#if versioneSelezionata}
              <div class="preview-meta">
                <span class="preview-version">Versione {versioneSelezionata.version}</span>
                <span class="preview-data">{tempoRelativo(versioneSelezionata.creato_a)}</span>
              </div>
              <h3 class="preview-titolo">{versioneSelezionata.titolo}</h3>
              {#if versioneSelezionata.descrizione}
                <p class="preview-desc">{versioneSelezionata.descrizione}</p>
              {/if}
              <pre class="preview-body">{versioneSelezionata.body}</pre>
            {:else}
              <p class="hint">Seleziona una versione dalla lista</p>
            {/if}
          </main>
        </div>
      {/if}
    </div>

    <footer class="modale-footer">
      <Button variante="ghost" onclick={onchiudi}>Chiudi</Button>
      {#if versioneSelezionata && versioni.length > 1 && !isTesta}
        <Button
          variante="primary"
          onclick={() => (confermaRollback = versioneSelezionata?.version ?? null)}
        >
          Ripristina questa versione
        </Button>
      {/if}
    </footer>
  </div>

  {#if confermaRollback !== null}
    <div class="conferma" role="dialog" aria-modal="true">
      <div class="conferma-box">
        <h3>Ripristinare la versione v{confermaRollback}?</h3>
        <p>
          Il prompt corrente sarà sostituito con il contenuto della v{confermaRollback}.
          La versione attuale resterà nella storia ed è ripristinabile in qualsiasi momento.
        </p>
        <div class="conferma-azioni">
          <Button variante="ghost" onclick={() => (confermaRollback = null)}>Annulla</Button>
          <Button variante="primary" onclick={eseguiRollback}>
            Ripristina v{confermaRollback}
          </Button>
        </div>
      </div>
    </div>
  {/if}
</div>

{#if messaggioToast}
  <Toast
    messaggio={messaggioToast}
    variante="success"
    onchiudi={() => (messaggioToast = "")}
  />
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--bg-scrim);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal);
    font-family: var(--font-ui);
  }

  .modale {
    width: 880px;
    max-width: 95vw;
    height: 640px;
    max-height: 90vh;
    background: var(--bg-canvas);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modale-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-3) var(--sp-4);
    border-bottom: 1px solid var(--border-subtle);
  }

  .modale-titolo {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .modale-chiudi {
    background: transparent;
    border: none;
    font-size: 24px;
    line-height: 1;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0 var(--sp-1);
  }

  .modale-chiudi:hover {
    color: var(--text-strong);
  }

  .modale-body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .empty-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-4);
  }

  .split {
    flex: 1;
    display: grid;
    grid-template-columns: 280px 1fr;
    overflow: hidden;
  }

  .lista-versioni {
    border-right: 1px solid var(--border-subtle);
    overflow-y: auto;
    padding: var(--sp-2);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .versione-item {
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: var(--sp-2);
    cursor: pointer;
    color: var(--text-default);
    font-family: inherit;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  .versione-item:hover {
    background: var(--bg-overlay);
  }

  .versione-item--attiva {
    background: var(--accent-team-soft);
    border-color: var(--accent-team);
  }

  .versione-header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: 2px;
  }

  .versione-numero {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .badge-corrente {
    font-size: 10px;
    text-transform: uppercase;
    background: var(--accent-team);
    color: white;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    letter-spacing: 0.5px;
  }

  .versione-meta {
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .versione-titolo {
    font-size: var(--fs-sm);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: var(--sp-1);
  }

  .preview {
    overflow-y: auto;
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .preview-meta {
    display: flex;
    gap: var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .preview-version {
    font-family: var(--font-mono);
    color: var(--text-strong);
  }

  .preview-titolo {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .preview-desc {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .preview-body {
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
    flex: 1;
  }

  .modale-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    border-top: 1px solid var(--border-subtle);
  }

  .hint {
    color: var(--text-subtle);
    text-align: center;
    margin-top: var(--sp-6);
  }

  .errore {
    color: var(--danger);
    padding: var(--sp-4);
  }

  .conferma {
    position: fixed;
    inset: 0;
    background: var(--bg-scrim);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: calc(var(--z-modal) + 1);
  }

  .conferma-box {
    background: var(--bg-canvas);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: var(--sp-4);
    max-width: 480px;
    width: 90vw;
    box-shadow: var(--shadow-3);
  }

  .conferma-box h3 {
    margin: 0 0 var(--sp-2);
    font-size: var(--fs-lg);
    color: var(--text-strong);
  }

  .conferma-box p {
    margin: 0 0 var(--sp-3);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    line-height: var(--lh-relaxed);
  }

  .conferma-azioni {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-2);
  }
</style>
