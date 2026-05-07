<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, EmptyState, Tag } from "$lib/components";
  import VersionDiff from "$lib/components/VersionDiff.svelte";
  import { statiDiff } from "$lib/diff";
  import { etichettaPerValore } from "$lib/modelli-target";

  interface TagInfoFE {
    id: string;
    nome: string;
    colore: string;
  }

  interface PromptDettaglio {
    id: string;
    titolo: string;
    descrizione: string | null;
    body: string;
    visibilita: string;
    target_model: string | null;
    folder_id: string | null;
    preferito: boolean;
    uso_count: number;
    creato_a: string;
    aggiornato_a: string;
    tags: TagInfoFE[];
  }

  interface Props {
    /// Id dei prompt da confrontare (2 in MVP). Se vuoto/<2 il modale
    /// mostra un EmptyState.
    promptIds: string[];
    onchiudi: () => void;
  }

  let { promptIds, onchiudi }: Props = $props();

  let dettagli = $state<(PromptDettaglio | null)[]>([]);
  let caricamento = $state(true);
  let errore = $state<string | null>(null);
  let modalitaDiff = $state<"none" | "side-by-side">("none");

  $effect(() => {
    void promptIds;
    carica();
  });

  async function carica() {
    caricamento = true;
    errore = null;
    try {
      const promesse = promptIds.map((id) =>
        invoke<PromptDettaglio>("libreria_dettaglio", { id }).catch(
          () => null,
        ),
      );
      dettagli = await Promise.all(promesse);
    } catch (e) {
      errore = String(e);
    } finally {
      caricamento = false;
    }
  }

  function gestisciTastiera(e: KeyboardEvent) {
    if (e.key === "Escape") onchiudi();
  }

  function tempoRelativo(iso: string): string {
    const ora = new Date();
    const past = new Date(iso.endsWith("Z") ? iso : iso + "Z");
    const diffSec = Math.floor((ora.getTime() - past.getTime()) / 1000);
    if (diffSec < 60) return "ora";
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m fa`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h fa`;
    if (diffSec < 86400 * 30) return `${Math.floor(diffSec / 86400)}g fa`;
    return past.toLocaleDateString("it-IT", {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  }

  // Diff body fra il primo e il secondo prompt (MVP 2 colonne).
  const dettagliValidi = $derived(
    dettagli.filter((d): d is PromptDettaglio => d !== null),
  );
  const diffStats = $derived(
    dettagliValidi.length >= 2
      ? statiDiff(dettagliValidi[0].body, dettagliValidi[1].body)
      : { aggiunte: 0, rimosse: 0 },
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
    aria-label="Confronto prompt"
  >
    <header class="modale-header">
      <h2 class="modale-titolo">Confronto fianco-a-fianco</h2>
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
      {:else if dettagliValidi.length < 2}
        <div class="empty-wrap">
          <EmptyState
            titolo="Selezione insufficiente"
            hint="Seleziona almeno 2 prompt dalla libreria (Cmd/Ctrl+click) per confrontarli."
          />
        </div>
      {:else}
        <div class="vista-controlli">
          <div class="seg-control" role="tablist" aria-label="Modalità diff">
            <button
              type="button"
              class="seg-btn"
              class:seg-btn--attivo={modalitaDiff === "none"}
              onclick={() => (modalitaDiff = "none")}
              role="tab"
              aria-selected={modalitaDiff === "none"}>Body affiancato</button
            >
            <button
              type="button"
              class="seg-btn"
              class:seg-btn--attivo={modalitaDiff === "side-by-side"}
              onclick={() => (modalitaDiff = "side-by-side")}
              role="tab"
              aria-selected={modalitaDiff === "side-by-side"}>Diff colorato</button
            >
          </div>
          <span class="diff-badges">
            <span class="badge-aggiunte">+{diffStats.aggiunte}</span>
            <span class="badge-rimosse">−{diffStats.rimosse}</span>
          </span>
        </div>

        {#if modalitaDiff === "none"}
          <div
            class="cmp-grid"
            style="grid-template-columns: repeat({dettagliValidi.length}, 1fr);"
          >
            {#each dettagliValidi as p (p.id)}
              <article class="cmp-col" aria-label={p.titolo}>
                <header class="cmp-head">
                  <h3 class="cmp-titolo">{p.titolo}</h3>
                  {#if p.descrizione}
                    <p class="cmp-desc">{p.descrizione}</p>
                  {/if}
                  <div class="cmp-meta">
                    <span class="cmp-vis"
                      >{p.visibilita === "private" ? "Privato" : "Team"}</span
                    >
                    {#if p.target_model}
                      <span class="cmp-target"
                        >{etichettaPerValore(p.target_model)}</span
                      >
                    {/if}
                    {#if p.uso_count > 0}
                      <span class="cmp-uso">{p.uso_count} usi</span>
                    {/if}
                    <span class="cmp-data">{tempoRelativo(p.aggiornato_a)}</span>
                  </div>
                  {#if p.tags.length > 0}
                    <div class="cmp-tags">
                      {#each p.tags as tag}
                        <Tag colore={tag.colore}>{tag.nome}</Tag>
                      {/each}
                    </div>
                  {/if}
                </header>
                <pre class="cmp-body">{p.body}</pre>
              </article>
            {/each}
          </div>
        {:else}
          <div class="cmp-diff-wrap">
            <VersionDiff
              a={dettagliValidi[0].body}
              b={dettagliValidi[1].body}
              modalita="side-by-side"
              etichettaA={dettagliValidi[0].titolo}
              etichettaB={dettagliValidi[1].titolo}
            />
          </div>
        {/if}
      {/if}
    </div>

    <footer class="modale-footer">
      <Button variante="ghost" onclick={onchiudi}>Chiudi</Button>
    </footer>
  </div>
</div>

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
    width: 1200px;
    max-width: 95vw;
    height: 720px;
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
    overflow: auto;
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .empty-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
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

  .vista-controlli {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .seg-control {
    display: inline-flex;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-canvas);
  }

  .seg-btn {
    padding: var(--sp-1) var(--sp-3);
    background: transparent;
    border: none;
    color: var(--text-default);
    font-family: inherit;
    font-size: var(--fs-sm);
    cursor: pointer;
    border-right: 1px solid var(--border-subtle);
  }

  .seg-btn:last-child {
    border-right: none;
  }

  .seg-btn--attivo {
    background: var(--accent-team-soft);
    color: var(--text-strong);
  }

  .diff-badges {
    display: inline-flex;
    gap: var(--sp-1);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }

  .badge-aggiunte {
    color: var(--accent-success, #2c8a2c);
    background: rgba(108, 184, 108, 0.18);
    padding: 2px 6px;
    border-radius: var(--radius-md);
  }

  .badge-rimosse {
    color: var(--accent-danger, #b73c38);
    background: rgba(217, 83, 79, 0.18);
    padding: 2px 6px;
    border-radius: var(--radius-md);
  }

  .cmp-grid {
    flex: 1;
    display: grid;
    gap: var(--sp-3);
    overflow: hidden;
  }

  .cmp-col {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-overlay);
  }

  .cmp-head {
    padding: var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .cmp-titolo {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .cmp-desc {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .cmp-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .cmp-tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
  }

  .cmp-body {
    flex: 1;
    margin: 0;
    padding: var(--sp-3);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: var(--lh-relaxed);
    color: var(--text-default);
    white-space: pre-wrap;
    word-break: break-word;
    overflow: auto;
  }

  .cmp-diff-wrap {
    flex: 1;
    overflow: auto;
  }

  .modale-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    border-top: 1px solid var(--border-subtle);
  }
</style>
