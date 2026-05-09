<script lang="ts">
  import {
    ChevronsRight,
    Clock,
    Star,
    LayoutList,
    Lock,
    Users,
    Folder,
    Tag,
    Cpu,
    BarChart3,
    AlertTriangle,
  } from "lucide-svelte";

  interface Props {
    onApriExpand: () => void;
    onApriInsight?: () => void;
    onApriRegressioni?: () => void;
  }

  let { onApriExpand, onApriInsight, onApriRegressioni }: Props = $props();
</script>

<aside class="sidebar-mini">
  <button
    class="ico expand"
    type="button"
    onclick={onApriExpand}
    title="Espandi sidebar"
    aria-label="Espandi sidebar"
  >
    <ChevronsRight size={14} />
  </button>
  <div class="separatore" aria-hidden="true"></div>
  <button class="ico" type="button" title="Recenti" aria-label="Recenti">
    <Clock size={14} />
  </button>
  <button class="ico" type="button" title="Preferiti" aria-label="Preferiti">
    <Star size={14} />
  </button>
  <button
    class="ico"
    type="button"
    title="Tutti i prompt"
    aria-label="Tutti i prompt"
  >
    <LayoutList size={14} />
  </button>
  <div class="separatore" aria-hidden="true"></div>
  <button class="ico" type="button" title="Privati" aria-label="Privati">
    <Lock size={14} />
  </button>
  <button class="ico" type="button" title="Team" aria-label="Team">
    <Users size={14} />
  </button>
  <div class="separatore" aria-hidden="true"></div>
  <button class="ico" type="button" title="Cartelle" aria-label="Cartelle">
    <Folder size={14} />
  </button>
  <button class="ico" type="button" title="Tag" aria-label="Tag">
    <Tag size={14} />
  </button>
  <button
    class="ico"
    type="button"
    title="Modello target"
    aria-label="Modello target"
  >
    <Cpu size={14} />
  </button>
  <div class="footer-spacer"></div>
  <button
    class="ico"
    type="button"
    title="Insight"
    aria-label="Insight"
    onclick={onApriInsight}
  >
    <BarChart3 size={14} />
  </button>
  <button
    class="ico"
    type="button"
    title="Regressioni"
    aria-label="Regressioni"
    onclick={onApriRegressioni}
  >
    <AlertTriangle size={14} />
  </button>
</aside>

<style>
  .sidebar-mini {
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    width: 100%;
    padding: var(--sp-2) 0;
    gap: var(--sp-1);
    background: var(--bg-surface);
  }

  .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  .ico:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .expand {
    color: var(--text-default);
  }

  .separatore {
    width: 24px;
    height: 1px;
    background: var(--border-subtle);
    margin: var(--sp-1) 0;
  }

  .footer-spacer {
    flex: 1;
  }
</style>
