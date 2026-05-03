<script lang="ts">
  import { Button } from "$lib/components";

  interface ConflittoItem {
    entityType: string;
    entityId: string;
    titolo: string;
    versioneLocale: number;
    versioneServer: number;
    aggiornatoLocale: string;
    aggiornatoServer: string;
  }

  interface Props {
    conflitti: ConflittoItem[];
    onrisolvi: (entityId: string, scelta: "locale" | "server") => void;
    onchiudi: () => void;
  }

  let { conflitti, onrisolvi, onchiudi }: Props = $props();
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onchiudi();
  }}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="scrim"
  onmousedown={(e) => {
    if (e.target === e.currentTarget) onchiudi();
  }}
>
  <div
    class="conflitto-card"
    role="dialog"
    aria-modal="true"
    aria-label="Conflitti sync"
  >
    <header class="conf-header">
      <div class="conf-header-row">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
          />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <h2>Conflitti di sincronizzazione</h2>
      </div>
      <p class="conf-sub">
        {conflitti.length}
        {conflitti.length === 1 ? "elemento ha" : "elementi hanno"} versioni
        diverse tra locale e server. Scegli quale mantenere.
      </p>
    </header>

    <div class="conf-lista">
      {#each conflitti as c}
        <div class="conf-item">
          <div class="conf-info">
            <span class="conf-tipo">{c.entityType}</span>
            <span class="conf-titolo">{c.titolo}</span>
            <div class="conf-meta">
              <span>Locale: v{c.versioneLocale} ({c.aggiornatoLocale})</span>
              <span>Server: v{c.versioneServer} ({c.aggiornatoServer})</span>
            </div>
          </div>
          <div class="conf-azioni">
            <Button
              dimensione="sm"
              variante="ghost"
              onclick={() => onrisolvi(c.entityId, "locale")}
            >
              Tieni locale
            </Button>
            <Button
              dimensione="sm"
              variante="primary"
              onclick={() => onrisolvi(c.entityId, "server")}
            >
              Usa server
            </Button>
          </div>
        </div>
      {/each}
    </div>

    <footer class="conf-footer">
      <Button variante="ghost" dimensione="sm" onclick={onchiudi}>
        Chiudi
      </Button>
    </footer>
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
  }

  .conflitto-card {
    width: min(600px, 94vw);
    max-height: min(500px, 80vh);
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .conf-header {
    padding: var(--sp-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .conf-header-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    color: var(--warning);
  }

  .conf-header h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .conf-sub {
    margin: var(--sp-2) 0 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .conf-lista {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .conf-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .conf-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .conf-tipo {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .conf-titolo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .conf-meta {
    display: flex;
    gap: var(--sp-3);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .conf-azioni {
    display: flex;
    gap: var(--sp-2);
    flex-shrink: 0;
  }

  .conf-footer {
    display: flex;
    justify-content: flex-end;
    padding: var(--sp-3) var(--sp-5);
    border-top: 1px solid var(--border-subtle);
  }
</style>
