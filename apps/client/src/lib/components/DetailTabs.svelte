<script lang="ts" module>
  export type TabId =
    | "editor"
    | "anteprima"
    | "diagnosi"
    | "golden"
    | "cronologia"
    | "import-var";
</script>

<script lang="ts">
  interface Badge {
    diagnosi?: number;
    golden?: number;
    cronologia?: number;
    importVar?: number;
  }

  interface Props {
    tabAttivo: TabId;
    badge?: Badge;
    onSeleziona: (tab: TabId) => void;
  }

  let { tabAttivo, badge = {}, onSeleziona }: Props = $props();

  type TabDef = { id: TabId; label: string; badge?: number };

  const tabs: TabDef[] = $derived([
    { id: "editor", label: "Editor" },
    { id: "anteprima", label: "Anteprima" },
    { id: "diagnosi", label: "Diagnosi", badge: badge.diagnosi },
    { id: "golden", label: "Test golden", badge: badge.golden },
    { id: "cronologia", label: "Cronologia", badge: badge.cronologia },
    { id: "import-var", label: "Import & Var.", badge: badge.importVar },
  ]);
</script>

<div class="tab-strip" role="tablist" aria-label="Sezioni del prompt">
  {#each tabs as tab (tab.id)}
    <button
      class="tab"
      role="tab"
      aria-selected={tab.id === tabAttivo}
      data-attivo={tab.id === tabAttivo || undefined}
      type="button"
      onclick={() => onSeleziona(tab.id)}
    >
      <span class="label">{tab.label}</span>
      {#if tab.badge !== undefined && tab.badge > 0}
        <span class="count" aria-label="{tab.badge} elementi">{tab.badge}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .tab-strip {
    display: flex;
    align-items: center;
    gap: 0;
    overflow-x: auto;
    border-bottom: 1px solid var(--border-subtle);
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 8px 12px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    white-space: nowrap;
  }

  .tab:hover {
    color: var(--text-default);
  }

  .tab[data-attivo] {
    color: var(--text-default);
    border-bottom-color: var(--accent-team);
    font-weight: var(--fw-medium);
  }

  .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 6px;
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
    color: var(--text-subtle);
    font-size: 11px;
    font-weight: var(--fw-medium);
  }

  .tab[data-attivo] .count {
    background: var(--accent-team-soft);
    color: var(--accent-team-strong);
  }
</style>
