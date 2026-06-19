<script lang="ts">
  /**
   * Primitivo "dumb" del menu contestuale: riceve voci + posizione dallo store
   * e le renderizza. Nessuna logica di dominio qui. Montato una volta in Shell.
   *
   * - Posizionamento manuale clampato al viewport (`posizionaMenu`, flip se sborda).
   * - Tastiera: ↑/↓ navigano le voci abilitate (wrap), Home/End, Enter/Space
   *   attivano (nativo del <button>), Esc chiude e restituisce il focus.
   * - Click fuori / resize / scroll → chiude.
   * - a11y: role=menu / menuitem, aria-disabled, focus-trap leggero.
   */
  import { untrack } from "svelte";
  import {
    menuContestuale,
    chiudiMenu,
    isSeparatore,
    type VoceMenu,
  } from "$lib/stores/menu-contestuale.svelte";
  import { posizionaMenu, type PuntoMenu } from "$lib/util/menu-posizione";
  import { fmtShortcut } from "$lib/util/shortcut";

  let menuEl = $state<HTMLDivElement>();
  let pos = $state<PuntoMenu>({ left: 0, top: 0 });
  let origineFocus: HTMLElement | null = null;

  const stato = $derived(menuContestuale.stato);

  function vociAbilitate(): HTMLButtonElement[] {
    if (!menuEl) return [];
    return Array.from(
      menuEl.querySelectorAll<HTMLButtonElement>(
        '[role="menuitem"]:not([aria-disabled="true"])',
      ),
    );
  }

  function muovi(delta: number): void {
    const items = vociAbilitate();
    if (items.length === 0) return;
    const idx = items.indexOf(document.activeElement as HTMLButtonElement);
    // Da "nessun focus" (idx -1): ↓ va al primo, ↑ all'ultimo (WAI-ARIA menu).
    const next =
      idx === -1
        ? delta > 0
          ? 0
          : items.length - 1
        : (idx + delta + items.length) % items.length;
    items[next]?.focus();
  }

  function chiudiERitorna(): void {
    chiudiMenu();
    origineFocus?.focus?.();
  }

  function onKeydown(e: KeyboardEvent): void {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        muovi(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        muovi(-1);
        break;
      case "Home":
        e.preventDefault();
        vociAbilitate()[0]?.focus();
        break;
      case "End": {
        e.preventDefault();
        const v = vociAbilitate();
        v[v.length - 1]?.focus();
        break;
      }
      case "Escape":
        e.preventDefault();
        chiudiERitorna();
        break;
    }
  }

  async function attiva(voce: Extract<VoceMenu, { id: string }>): Promise<void> {
    if (voce.disabilitato) return;
    // Chiude e smonta il menu; il focus lo gestisce l'azione (apre una modale,
    // naviga, ecc.). NON ripristiniamo il focus qui per non interferire col
    // focus-trap di un'eventuale modale aperta dall'azione.
    chiudiMenu();
    try {
      await voce.azione?.();
    } catch (e) {
      console.error("[menu-contestuale] azione fallita", e);
    }
  }

  // True tra l'apertura e la chiusura: evita di ri-catturare `origineFocus`
  // quando `apriMenu` viene richiamato a menu già aperto (es. tasto destro su
  // un'altra card) — altrimenti l'origine diventerebbe una voce del menu stesso.
  let eraAperto = false;

  // Posiziona dopo il render (misura reale) e focalizza la prima voce.
  $effect(() => {
    if (!stato.aperto) {
      eraAperto = false;
      return;
    }
    let raf = 0;
    untrack(() => {
      if (!eraAperto) {
        origineFocus = document.activeElement as HTMLElement | null;
        eraAperto = true;
      }
      // Posiziona subito al punto di click (niente flash a 0,0); poi raffina
      // dopo la misura reale.
      pos = { left: stato.x, top: stato.y };
      raf = requestAnimationFrame(() => {
        if (!menuEl) return;
        const r = menuEl.getBoundingClientRect();
        if (r.width === 0 && r.height === 0) return;
        pos = posizionaMenu(
          stato.x,
          stato.y,
          r.width,
          r.height,
          window.innerWidth,
          window.innerHeight,
        );
        vociAbilitate()[0]?.focus();
      });
    });
    const chiudiSuEvento = (): void => chiudiMenu();
    window.addEventListener("resize", chiudiSuEvento);
    window.addEventListener("scroll", chiudiSuEvento, true);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", chiudiSuEvento);
      window.removeEventListener("scroll", chiudiSuEvento, true);
    };
  });
</script>

{#if stato.aperto}
  <!-- Backdrop trasparente che assorbe il click-fuori. È un <button> (non un
       div) così è interattivo e non viola le regole a11y; nascosto agli screen
       reader (il menu è l'elemento accessibile) e fuori dal tab order. La
       chiusura da tastiera è Esc, gestita sul menu. -->
  <button
    type="button"
    class="backdrop"
    aria-hidden="true"
    tabindex="-1"
    onclick={chiudiMenu}
    oncontextmenu={(e) => {
      e.preventDefault();
      chiudiMenu();
    }}
  ></button>

  <div
    bind:this={menuEl}
    class="menu"
    role="menu"
    aria-label="Azioni"
    tabindex="-1"
    style="left: {pos.left}px; top: {pos.top}px;"
    onkeydown={onKeydown}
  >
    {#each stato.voci as voce, i (isSeparatore(voce) ? `sep-${i}` : voce.id)}
      {#if isSeparatore(voce)}
        <div class="separatore" role="separator"></div>
      {:else}
        <button
          type="button"
          role="menuitem"
          class="voce"
          class:pericolo={voce.pericolo}
          class:disabilitata={voce.disabilitato}
          tabindex="-1"
          aria-disabled={voce.disabilitato}
          title={voce.disabilitato ? "Disponibile prossimamente" : undefined}
          onclick={() => attiva(voce)}
        >
          <span class="ico" aria-hidden="true">
            {#if voce.icona}
              {@const Ico = voce.icona}
              <Ico size={15} />
            {/if}
          </span>
          <span class="label">{voce.label}</span>
          {#if voce.scorciatoia}
            <kbd class="scorciatoia">{fmtShortcut(voce.scorciatoia)}</kbd>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 3000;
    background: transparent;
    border: none;
    padding: 0;
    cursor: default;
  }

  .menu {
    position: fixed;
    z-index: 3001;
    min-width: 200px;
    max-width: 320px;
    max-height: calc(100vh - 24px);
    overflow-y: auto;
    padding: 4px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25));
    font-family: var(--font-ui);
    color: var(--text-default);
  }

  .voce {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-default);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
  }

  .voce:hover:not(.disabilitata),
  .voce:focus-visible {
    background: var(--bg-overlay);
    outline: none;
  }

  .voce:focus-visible {
    box-shadow: inset 0 0 0 2px var(--accent-private);
  }

  .voce.disabilitata {
    color: var(--text-subtle);
    cursor: default;
    pointer-events: none;
  }

  .voce.pericolo {
    color: var(--danger, #e5484d);
  }

  .voce.pericolo:hover:not(.disabilitata),
  .voce.pericolo:focus-visible {
    background: color-mix(in srgb, var(--danger, #e5484d) 12%, transparent);
  }

  .ico {
    display: grid;
    place-items: center;
    width: 16px;
    flex-shrink: 0;
  }

  .label {
    flex: 1;
  }

  .scorciatoia {
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 5px;
  }

  .separatore {
    height: 1px;
    margin: 4px 6px;
    background: var(--border-subtle);
  }
</style>
