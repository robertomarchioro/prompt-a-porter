<script lang="ts">
  /**
   * Primitivo "dumb" del menu contestuale: riceve voci + posizione dallo store
   * e le renderizza. Nessuna logica di dominio qui. Montato una volta in Shell.
   *
   * - Posizionamento manuale clampato al viewport (`posizionaMenu`, flip se sborda).
   * - Submenu a un livello: una voce con `figli` apre un menu figlio a destra
   *   (flip a sinistra se sborda). Hover o → per aprire, ← per chiudere.
   * - Tastiera: ↑/↓ navigano le voci abilitate (wrap), Home/End, Enter/Space
   *   attivano (nativo del <button>), Esc chiude tutto e restituisce il focus.
   * - Click fuori (backdrop) / resize / scroll → chiude.
   * - a11y: role=menu / menuitem, aria-haspopup/expanded sui parent, aria-disabled.
   */
  import { untrack } from "svelte";
  import { ChevronRight } from "lucide-svelte";
  import {
    menuContestuale,
    chiudiMenu,
    isSeparatore,
    type VoceMenu,
  } from "$lib/stores/menu-contestuale.svelte";
  import { posizionaMenu, type PuntoMenu } from "$lib/util/menu-posizione";
  import { fmtShortcut } from "$lib/util/shortcut";

  type Voce = Extract<VoceMenu, { id: string }>;

  let menuEl = $state<HTMLDivElement>();
  let subEl = $state<HTMLDivElement>();
  let pos = $state<PuntoMenu>({ left: 0, top: 0 });
  let subPos = $state<PuntoMenu>({ left: 0, top: 0 });
  let subApertoId = $state<string | null>(null);
  let origineFocus: HTMLElement | null = null;
  let eraAperto = false;
  // Generazione del submenu: invalida i rAF di un'apertura superata quando si
  // passa rapidamente da un genitore all'altro (evita posizioni "stale").
  let subGen = 0;

  function bottoneVoce(id: string): HTMLButtonElement | null | undefined {
    return menuEl?.querySelector<HTMLButtonElement>(
      `[data-id="${CSS.escape(id)}"]`,
    );
  }

  const stato = $derived(menuContestuale.stato);
  const vociSub = $derived.by<VoceMenu[]>(() => {
    if (!subApertoId) return [];
    const parent = stato.voci.find(
      (v): v is Voce => !isSeparatore(v) && v.id === subApertoId,
    );
    return parent?.figli ?? [];
  });

  function abilitate(container?: HTMLElement): HTMLButtonElement[] {
    if (!container) return [];
    return Array.from(
      container.querySelectorAll<HTMLButtonElement>(
        '[role="menuitem"]:not([aria-disabled="true"])',
      ),
    );
  }

  function muovi(container: HTMLElement | undefined, delta: number): void {
    const items = abilitate(container);
    if (items.length === 0) return;
    const idx = items.indexOf(document.activeElement as HTMLButtonElement);
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

  function chiudiSub(refocusParent = false): void {
    const id = subApertoId;
    subApertoId = null;
    if (refocusParent && id) {
      bottoneVoce(id)?.focus();
    }
  }

  function apriSub(voce: Voce, anchor: HTMLElement, focusFirst: boolean): void {
    const gen = ++subGen;
    const r = anchor.getBoundingClientRect();
    subApertoId = voce.id;
    // posizione provvisoria; raffinata dopo la misura del submenu.
    subPos = { left: r.right - 2, top: r.top };
    requestAnimationFrame(() => {
      if (!subEl || gen !== subGen) return;
      const sr = subEl.getBoundingClientRect();
      let left = r.right - 2;
      if (left + sr.width + 8 > window.innerWidth) {
        left = r.left - sr.width + 2; // apre a sinistra del genitore
      }
      let top = r.top;
      if (top + sr.height + 8 > window.innerHeight) {
        top = Math.max(8, window.innerHeight - sr.height - 8);
      }
      subPos = { left: Math.max(8, left), top: Math.max(8, top) };
      if (focusFirst) abilitate(subEl)[0]?.focus();
    });
  }

  async function attiva(voce: Voce): Promise<void> {
    if (voce.disabilitato) return;
    if (voce.figli) {
      // Voce-genitore: apre/chiude il submenu invece di agire.
      const anchor = bottoneVoce(voce.id);
      if (subApertoId === voce.id) chiudiSub(true);
      else if (anchor) apriSub(voce, anchor, true);
      return;
    }
    chiudiMenu();
    try {
      await voce.azione?.();
    } catch (e) {
      console.error("[menu-contestuale] azione fallita", e);
    }
  }

  function vocePerId(id: string | undefined): Voce | undefined {
    if (!id) return undefined;
    const v = stato.voci.find((x) => !isSeparatore(x) && x.id === id);
    return v && !isSeparatore(v) ? v : undefined;
  }

  function onKeydownRoot(e: KeyboardEvent): void {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        muovi(menuEl, 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        muovi(menuEl, -1);
        break;
      case "Home":
        e.preventDefault();
        abilitate(menuEl)[0]?.focus();
        break;
      case "End": {
        e.preventDefault();
        const v = abilitate(menuEl);
        v[v.length - 1]?.focus();
        break;
      }
      case "ArrowRight": {
        const voce = vocePerId(
          (document.activeElement as HTMLElement)?.dataset?.id,
        );
        if (voce?.figli) {
          e.preventDefault();
          apriSub(voce, document.activeElement as HTMLElement, true);
        }
        break;
      }
      case "Escape":
        e.preventDefault();
        chiudiERitorna();
        break;
    }
  }

  function onKeydownSub(e: KeyboardEvent): void {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        muovi(subEl, 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        muovi(subEl, -1);
        break;
      case "Home":
        e.preventDefault();
        abilitate(subEl)[0]?.focus();
        break;
      case "End": {
        e.preventDefault();
        const v = abilitate(subEl);
        v[v.length - 1]?.focus();
        break;
      }
      case "ArrowLeft":
        e.preventDefault();
        chiudiSub(true);
        break;
      case "Escape":
        e.preventDefault();
        chiudiERitorna();
        break;
    }
  }

  // Hover sui top-level: un genitore apre il suo submenu (sostituendo quello
  // eventualmente aperto). Passare su una voce semplice NON chiude il submenu
  // (così si può raggiungere senza che un fratello lo chiuda per strada: niente
  // hover-intent delay); il submenu si chiude su ←, Esc, selezione o click-fuori.
  function onHoverVoce(voce: Voce): void {
    if (voce.figli) {
      const anchor = bottoneVoce(voce.id);
      if (anchor && subApertoId !== voce.id) apriSub(voce, anchor, false);
    }
  }

  // Posiziona il menu radice dopo il render reale e focalizza la prima voce.
  $effect(() => {
    if (!stato.aperto) {
      eraAperto = false;
      subApertoId = null;
      return;
    }
    let raf = 0;
    untrack(() => {
      if (!eraAperto) {
        origineFocus = document.activeElement as HTMLElement | null;
        eraAperto = true;
      }
      subApertoId = null;
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
        abilitate(menuEl)[0]?.focus();
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

{#snippet voceBtn(voce: Voce, inSub: boolean)}
  <button
    type="button"
    role="menuitem"
    class="voce"
    class:pericolo={voce.pericolo}
    class:disabilitata={voce.disabilitato}
    tabindex="-1"
    id={`mc-${voce.id}`}
    data-id={voce.id}
    aria-disabled={voce.disabilitato}
    aria-haspopup={voce.figli ? "menu" : undefined}
    aria-expanded={voce.figli ? subApertoId === voce.id : undefined}
    title={voce.tooltip ??
      (voce.disabilitato ? "Disponibile prossimamente" : undefined)}
    onmouseenter={inSub ? undefined : () => onHoverVoce(voce)}
    onclick={() => attiva(voce)}
  >
    <span class="ico" aria-hidden="true">
      {#if voce.icona}
        {@const Ico = voce.icona}
        <Ico size={15} />
      {/if}
    </span>
    <span class="label">{voce.label}</span>
    {#if voce.figli}
      <ChevronRight class="chevron" size={14} aria-hidden="true" />
    {:else if voce.scorciatoia}
      <kbd class="scorciatoia">{fmtShortcut(voce.scorciatoia)}</kbd>
    {/if}
  </button>
{/snippet}

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
    onkeydown={onKeydownRoot}
  >
    {#each stato.voci as voce, i (isSeparatore(voce) ? `sep-${i}` : voce.id)}
      {#if isSeparatore(voce)}
        <div class="separatore" role="separator"></div>
      {:else}
        {@render voceBtn(voce, false)}
      {/if}
    {/each}
  </div>

  {#if subApertoId}
    <div
      bind:this={subEl}
      class="menu submenu"
      role="menu"
      aria-labelledby={subApertoId ? `mc-${subApertoId}` : undefined}
      tabindex="-1"
      style="left: {subPos.left}px; top: {subPos.top}px;"
      onkeydown={onKeydownSub}
    >
      {#each vociSub as voce, i (isSeparatore(voce) ? `s-${i}` : voce.id)}
        {#if isSeparatore(voce)}
          <div class="separatore" role="separator"></div>
        {:else}
          {@render voceBtn(voce, true)}
        {/if}
      {/each}
    </div>
  {/if}
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

  .submenu {
    z-index: 3002;
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

  .voce :global(.chevron) {
    color: var(--text-muted);
    flex-shrink: 0;
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
