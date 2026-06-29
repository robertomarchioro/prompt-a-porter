<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import {
    ChevronsLeft,
    BarChart3,
    AlertTriangle,
    Trash2,
    FilePlus,
    FolderPlus,
    Pencil,
  } from "lucide-svelte";
  import NavGroup from "./NavGroup.svelte";
  import NavItem from "./NavItem.svelte";
  import WorkspaceSwitcher from "./WorkspaceSwitcher.svelte";
  import { MODELLI_TARGET } from "$lib/modelli-target";
  import { statoVault } from "$lib/stores/preferenze.svelte";
  import {
    apriMenu,
    type VoceMenu,
  } from "$lib/stores/menu-contestuale.svelte";

  interface ConteggiViste {
    tutti: number;
    preferiti: number;
    privati: number;
    team: number;
    cestino: number;
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
    onAggiungiCartella?: () => void;
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
    onAggiungiCartella,
  }: Props = $props();

  let conteggi = $state<ConteggiViste>({
    tutti: 0,
    preferiti: 0,
    privati: 0,
    team: 0,
    cestino: 0,
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

  // ─── Menu contestuale cartelle (blueprint menu-contestuale §6.2) ───
  let rinominandoId = $state<string | null>(null);
  let nomeModifica = $state("");

  function focusSelect(node: HTMLInputElement): void {
    node.focus();
    node.select();
  }

  async function nuovoPromptInCartella(folderId: string): Promise<void> {
    try {
      const id = await invoke<string>("prompt_crea", {
        dati: {
          titolo: "Nuovo prompt",
          descrizione: "",
          body: "",
          visibilita: "private",
          tag_nomi: [],
          target_model: null,
          folder_id: folderId,
        },
      });
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
      window.dispatchEvent(new CustomEvent("pap:apri-prompt", { detail: id }));
    } catch (e) {
      console.error("[sidebar] nuovo prompt in cartella", e);
    }
  }

  async function nuovaSottocartella(parentId: string): Promise<void> {
    try {
      const id = await invoke<string>("folder_crea", {
        dati: { nome: "Nuova cartella", parent_folder_id: parentId },
      });
      await caricaDati();
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
      avviaRinomina(id, "Nuova cartella");
    } catch (e) {
      console.error("[sidebar] nuova sottocartella", e);
    }
  }

  function avviaRinomina(id: string, nome: string): void {
    rinominandoId = id;
    nomeModifica = nome;
  }

  function annullaRinomina(): void {
    rinominandoId = null;
  }

  async function salvaRinomina(): Promise<void> {
    const id = rinominandoId;
    const nome = nomeModifica.trim();
    rinominandoId = null;
    if (!id || !nome) return;
    try {
      await invoke<void>("folder_rinomina", {
        dati: { id, nuovo_nome: nome },
      });
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      console.error("[sidebar] rinomina cartella", e);
    }
  }

  async function eliminaCartella(cart: Cartella): Promise<void> {
    if (
      !confirm(
        `Eliminare la cartella "${cart.nome}"? I prompt al suo interno torneranno alla libreria principale.`,
      )
    ) {
      return;
    }
    try {
      await invoke<void>("folder_elimina", { id: cart.id });
      if (folderSelezionato === cart.id) onSelezionaFolder(null);
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      console.error("[sidebar] elimina cartella", e);
    }
  }

  function vociCartella(cart: Cartella): VoceMenu[] {
    return [
      {
        id: "nuovo-prompt",
        label: "Nuovo prompt qui",
        icona: FilePlus,
        azione: () => nuovoPromptInCartella(cart.id),
      },
      {
        id: "nuova-sub",
        label: "Nuova sottocartella",
        icona: FolderPlus,
        azione: () => nuovaSottocartella(cart.id),
      },
      {
        id: "rinomina",
        label: "Rinomina",
        icona: Pencil,
        azione: () => avviaRinomina(cart.id, cart.nome),
      },
      { separatore: true },
      {
        id: "elimina",
        label: "Elimina cartella",
        icona: Trash2,
        pericolo: true,
        azione: () => eliminaCartella(cart),
      },
    ];
  }

  function onContextCartella(e: MouseEvent, cart: Cartella): void {
    e.preventDefault();
    apriMenu(e.clientX, e.clientY, vociCartella(cart));
  }

  onMount(() => {
    void caricaDati();
    window.addEventListener("pap:lista-mutata", caricaDati);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", caricaDati);
  });
</script>

<aside class="sidebar" data-tour="sidebar">
  <div class="sidebar-top">
    <WorkspaceSwitcher nome={statoVault.nome} />
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
      <NavItem
        attivo={vistaCorrente === "cestino"}
        conteggio={conteggi.cestino}
        onclick={() => onSelezionaVista("cestino")}
      >
        {#snippet icona()}
          <Trash2 size={13} aria-hidden="true" />
        {/snippet}
        Cestino
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
      onAggiungi={onAggiungiCartella}
      bind:collapsed={gruppi.cartelle}
    >
      {#each cartelle as cart (cart.id)}
        {#if rinominandoId === cart.id}
          <input
            class="rinomina-cartella"
            aria-label="Nuovo nome cartella"
            bind:value={nomeModifica}
            use:focusSelect
            onkeydown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                void salvaRinomina();
              } else if (e.key === "Escape") {
                e.preventDefault();
                annullaRinomina();
              }
            }}
            onblur={salvaRinomina}
          />
        {:else}
          <NavItem
            attivo={folderSelezionato === cart.id}
            conteggio={cart.conteggio_prompt}
            onclick={() => onSelezionaFolder(cart.id)}
            oncontextmenu={(e) => onContextCartella(e, cart)}
          >
            {cart.nome}
          </NavItem>
        {/if}
      {/each}
    </NavGroup>

    <!-- Nessun "+" qui: i tag non si creano stand-alone, nascono
         assegnandoli durante la creazione/modifica di un prompt (#307). -->
    <NavGroup
      titolo="TAG"
      conteggio={tags.length}
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
      {#each MODELLI_TARGET as m (m.value)}
        <NavItem
          attivo={modelTargetSelezionato === m.value}
          onclick={() => onSelezionaModelTarget(m.value)}
        >
          {m.label}
        </NavItem>
      {/each}
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

  .rinomina-cartella {
    width: 100%;
    height: 32px;
    padding: 0 var(--sp-3);
    border: 1px solid var(--accent-private);
    border-radius: var(--radius-sm);
    background: var(--bg-input);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    outline: none;
  }
</style>
