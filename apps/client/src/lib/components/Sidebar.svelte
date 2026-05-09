<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { ChevronsLeft, BarChart3, AlertTriangle } from "lucide-svelte";
  import NavGroup from "./NavGroup.svelte";
  import NavItem from "./NavItem.svelte";
  import WorkspaceSwitcher from "./WorkspaceSwitcher.svelte";

  interface ConteggiViste {
    tutti: number;
    preferiti: number;
    privati: number;
    team: number;
  }

  interface TagInfo {
    id: string;
    nome: string;
    colore: string;
  }

  interface Cartella {
    id: string;
    nome: string;
    path: string;
    parent_folder_id: string | null;
    conteggio_prompt: number;
  }

  interface GruppiState {
    viste: boolean;
    visibilita: boolean;
    cartelle: boolean;
    tag: boolean;
    modelTarget: boolean;
  }

  interface Props {
    vistaCorrente: string;
    folderSelezionato?: string | null;
    tagSelezionato?: string | null;
    modelTargetSelezionato?: string;
    gruppi: GruppiState;
    onSelezionaVista: (id: string) => void;
    onSelezionaFolder: (id: string | null) => void;
    onSelezionaTag: (id: string | null) => void;
    onSelezionaModelTarget: (model: string) => void;
    onApriCollapse?: () => void;
    onApriInsight?: () => void;
    onApriRegressioni?: () => void;
  }

  let {
    vistaCorrente,
    folderSelezionato = null,
    tagSelezionato = null,
    modelTargetSelezionato = "",
    gruppi = $bindable(),
    onSelezionaVista,
    onSelezionaFolder,
    onSelezionaTag,
    onSelezionaModelTarget,
    onApriCollapse,
    onApriInsight,
    onApriRegressioni,
  }: Props = $props();

  let conteggi = $state<ConteggiViste>({
    tutti: 0,
    preferiti: 0,
    privati: 0,
    team: 0,
  });
  let tags = $state<TagInfo[]>([]);
  let cartelle = $state<Cartella[]>([]);

  async function caricaDati(): Promise<void> {
    try {
      const [c, t, f] = await Promise.all([
        invoke<ConteggiViste>("libreria_conteggi"),
        invoke<TagInfo[]>("libreria_tag_lista"),
        invoke<Cartella[]>("folder_lista"),
      ]);
      conteggi = c;
      tags = t;
      cartelle = f;
    } catch (e) {
      console.error("[sidebar] caricamento dati fallito", e);
    }
  }

  onMount(() => {
    void caricaDati();
    window.addEventListener("pap:lista-mutata", caricaDati);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", caricaDati);
  });
</script>

<aside class="sidebar">
  <div class="sidebar-top">
    <WorkspaceSwitcher nome="Personale" />
    <button
      class="collapse-btn"
      type="button"
      aria-label="Comprimi sidebar"
      title="Comprimi"
      onclick={onApriCollapse}
    >
      <ChevronsLeft size={14} />
    </button>
  </div>

  <nav class="nav-list">
    <NavGroup titolo="VISTE" bind:collapsed={gruppi.viste}>
      <NavItem
        attivo={vistaCorrente === "recenti"}
        onclick={() => onSelezionaVista("recenti")}
      >
        Recenti
      </NavItem>
      <NavItem
        attivo={vistaCorrente === "preferiti"}
        conteggio={conteggi.preferiti}
        onclick={() => onSelezionaVista("preferiti")}
      >
        Preferiti
      </NavItem>
      <NavItem
        attivo={vistaCorrente === "tutti"}
        conteggio={conteggi.tutti}
        onclick={() => onSelezionaVista("tutti")}
      >
        Tutti i prompt
      </NavItem>
    </NavGroup>

    <NavGroup titolo="VISIBILITÀ" bind:collapsed={gruppi.visibilita}>
      <NavItem
        attivo={vistaCorrente === "privati"}
        conteggio={conteggi.privati}
        onclick={() => onSelezionaVista("privati")}
      >
        {#snippet icona()}
          <span class="dot dot-private" aria-hidden="true"></span>
        {/snippet}
        Privati
      </NavItem>
      <NavItem
        attivo={vistaCorrente === "team"}
        conteggio={conteggi.team}
        onclick={() => onSelezionaVista("team")}
      >
        {#snippet icona()}
          <span class="dot dot-team" aria-hidden="true"></span>
        {/snippet}
        Team
      </NavItem>
    </NavGroup>

    <NavGroup
      titolo="CARTELLE"
      conteggio={cartelle.length}
      bottonAggiungi
      bind:collapsed={gruppi.cartelle}
    >
      {#each cartelle as cart (cart.id)}
        <NavItem
          attivo={folderSelezionato === cart.id}
          conteggio={cart.conteggio_prompt}
          onclick={() => onSelezionaFolder(cart.id)}
        >
          {cart.nome}
        </NavItem>
      {/each}
    </NavGroup>

    <NavGroup
      titolo="TAG"
      conteggio={tags.length}
      bottonAggiungi
      bind:collapsed={gruppi.tag}
    >
      {#each tags as tag (tag.id)}
        <NavItem
          attivo={tagSelezionato === tag.id}
          onclick={() => onSelezionaTag(tag.id)}
        >
          {#snippet icona()}
            <span
              class="dot"
              style:background={tag.colore || "var(--text-subtle)"}
              aria-hidden="true"
            ></span>
          {/snippet}
          {tag.nome}
        </NavItem>
      {/each}
    </NavGroup>

    <NavGroup titolo="MODELLO TARGET" bind:collapsed={gruppi.modelTarget}>
      <NavItem
        attivo={!modelTargetSelezionato}
        onclick={() => onSelezionaModelTarget("")}
      >
        Tutti
      </NavItem>
    </NavGroup>
  </nav>

  <footer class="sidebar-footer">
    <button class="footer-link" type="button" onclick={onApriInsight}>
      <BarChart3 size={14} />
      <span>Insight</span>
    </button>
    <button class="footer-link" type="button" onclick={onApriRegressioni}>
      <AlertTriangle size={14} />
      <span>Regressioni</span>
    </button>
  </footer>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-surface);
  }

  .sidebar-top {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    padding: var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
  }

  .collapse-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: 0;
    background: transparent;
    color: var(--text-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .collapse-btn:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .nav-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2) 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-2);
    border-top: 1px solid var(--border-subtle);
  }

  .footer-link {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
  }

  .footer-link:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--text-subtle);
  }

  .dot-private {
    background: var(--accent-private);
  }

  .dot-team {
    background: var(--accent-team);
  }
</style>
