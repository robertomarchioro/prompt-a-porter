<script lang="ts">
  import { Star } from "lucide-svelte";
  import type { Densita } from "$lib/stores/densita";

  interface TagInfo {
    id: string;
    nome: string;
    colore: string;
  }

  interface Props {
    id: string;
    titolo: string;
    descrizione: string;
    visibilita: string;
    preferito: boolean;
    usoCount: number;
    aggiornatoA: string;
    tags: TagInfo[];
    attivo?: boolean;
    densita?: Densita;
    righePreview?: number; // F3 PR-B
    bodyPreview?: string; // F3 PR-B
    onclick?: () => void;
    onToggleFavorite?: () => void;
  }

  let {
    titolo,
    descrizione,
    visibilita,
    preferito,
    usoCount,
    aggiornatoA,
    tags,
    attivo = false,
    densita = "comoda",
    righePreview = 3,
    bodyPreview,
    onclick,
    onToggleFavorite,
  }: Props = $props();

  function gestFav(e: MouseEvent): void {
    e.stopPropagation();
    onToggleFavorite?.();
  }

  function tempoRelativo(iso: string): string {
    if (!iso) return "";
    try {
      const t = new Date(iso).getTime();
      const ora = Date.now();
      const sec = Math.max(0, Math.floor((ora - t) / 1000));
      if (sec < 60) return "ora";
      const min = Math.floor(sec / 60);
      if (min < 60) return `${min}m fa`;
      const h = Math.floor(min / 60);
      if (h < 24) return `${h}h fa`;
      const g = Math.floor(h / 24);
      if (g < 30) return `${g}g fa`;
      const m = Math.floor(g / 30);
      if (m < 12) return `${m}M fa`;
      return `${Math.floor(m / 12)}a fa`;
    } catch {
      return "";
    }
  }
</script>

<div
  class="prompt-card"
  data-densita={densita}
  data-attivo={attivo || undefined}
  role="button"
  tabindex="0"
  onclick={onclick}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onclick?.();
    }
  }}
>
  <span
    class="vis-dot"
    class:vis-private={visibilita === "private"}
    class:vis-team={visibilita === "workspace"}
    aria-hidden="true"
  ></span>

  <div class="contenuto">
    <div class="title-row">
      <span class="titolo">{titolo}</span>
      <button
        class="fav"
        class:fav-on={preferito}
        type="button"
        aria-label={preferito ? "Rimuovi dai preferiti" : "Aggiungi ai preferiti"}
        onclick={gestFav}
      >
        <Star size={12} fill={preferito ? "currentColor" : "transparent"} />
      </button>
    </div>

    {#if densita === "comoda" || densita === "anteprima"}
      {#if descrizione}
        <p class="desc">{descrizione}</p>
      {/if}
      {#if tags.length > 0}
        <div class="tags">
          {#each tags.slice(0, 3) as t (t.id)}
            <span class="tag" style:--tag-c={t.colore || "var(--text-subtle)"}>
              {t.nome}
            </span>
          {/each}
          {#if tags.length > 3}
            <span class="tag-extra">+{tags.length - 3}</span>
          {/if}
        </div>
      {/if}
    {/if}

    {#if densita === "anteprima" && bodyPreview}
      <pre
        class="preview"
        style:--preview-lines={righePreview}>{bodyPreview}</pre>
    {/if}
  </div>

  <div class="meta">
    {#if densita === "compatta"}
      <span class="meta-time">{tempoRelativo(aggiornatoA)}</span>
    {:else}
      <span class="meta-uso">{usoCount}×</span>
    {/if}
  </div>
</div>

<style>
  .prompt-card {
    display: grid;
    grid-template-columns: 8px 1fr auto;
    align-items: start;
    gap: var(--sp-2);
    width: 100%;
    padding: 8px 12px;
    border: 0;
    border-left: 2px solid transparent;
    background: transparent;
    color: var(--text-default);
    text-align: left;
    cursor: pointer;
    transition: background var(--motion-fast) var(--easing-standard);
    font-family: var(--font-ui);
    position: relative;
  }

  .prompt-card:hover {
    background: var(--bg-overlay);
  }

  .prompt-card[data-attivo] {
    background: var(--bg-overlay);
    border-left-color: var(--accent-team);
  }

  .prompt-card[data-densita="compatta"] {
    align-items: center;
    padding: 6px 12px;
  }

  .vis-dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    margin-top: 7px;
    background: var(--text-subtle);
  }

  .prompt-card[data-densita="compatta"] .vis-dot {
    margin-top: 0;
  }

  .vis-private {
    background: var(--accent-private);
  }

  .vis-team {
    background: var(--accent-team);
  }

  .contenuto {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .titolo {
    flex: 1;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fav {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: 0;
    background: transparent;
    color: var(--text-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--motion-fast);
  }

  .prompt-card:hover .fav,
  .fav-on {
    opacity: 1;
  }

  .fav-on {
    color: var(--warning);
  }

  .desc {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
  }

  .tags {
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-1);
    margin-top: 2px;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted);
    padding: 0 6px;
    height: 18px;
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
  }

  .tag::before {
    content: "";
    width: 5px;
    height: 5px;
    border-radius: var(--radius-full);
    background: var(--tag-c, var(--text-subtle));
  }

  .tag-extra {
    font-size: 11px;
    color: var(--text-subtle);
  }

  .preview {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    display: -webkit-box;
    -webkit-line-clamp: var(--preview-lines, 3);
    line-clamp: var(--preview-lines, 3);
    -webkit-box-orient: vertical;
    overflow: hidden;
    white-space: pre-wrap;
    margin: var(--sp-1) 0 0;
    background: var(--bg-canvas);
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .meta {
    display: inline-flex;
    align-items: center;
    color: var(--text-subtle);
    font-size: var(--fs-xs);
  }

  .meta-uso {
    font-variant-numeric: tabular-nums;
  }
</style>
