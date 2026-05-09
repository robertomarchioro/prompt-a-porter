<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { Lock, Users } from "lucide-svelte";

  type StatoSalvataggio = "salvato" | "dirty" | "salvando" | "errore";

  interface PromptCorrente {
    id: string;
    titolo: string;
    visibilita: string;
  }

  let vaultPath = $state("");
  let vaultAperto = $state(false);
  let promptCorrente = $state<PromptCorrente | null>(null);
  let statoSalvataggio = $state<StatoSalvataggio | null>(null);
  let salvatoA = $state<string | null>(null);
  let tickRelative = $state(0);

  async function caricaVaultInfo(): Promise<void> {
    try {
      const [path, aperto] = await Promise.all([
        invoke<string>("vault_percorso"),
        invoke<boolean>("vault_aperto"),
      ]);
      vaultPath = path;
      vaultAperto = aperto;
    } catch (e) {
      console.error("[statusbar] vault info", e);
    }
  }

  function onPromptCorrente(e: Event): void {
    const detail = (e as CustomEvent<PromptCorrente | null>).detail;
    promptCorrente = detail;
  }

  function onSaveStato(e: Event): void {
    const detail = (e as CustomEvent<{
      stato: StatoSalvataggio;
      salvatoA?: string;
    }>).detail;
    statoSalvataggio = detail.stato;
    if (detail.salvatoA) salvatoA = detail.salvatoA;
  }

  let timerTick: ReturnType<typeof setInterval> | undefined;

  onMount(() => {
    void caricaVaultInfo();
    window.addEventListener("pap:prompt-corrente", onPromptCorrente);
    window.addEventListener("pap:save-stato", onSaveStato);
    timerTick = setInterval(() => (tickRelative += 1), 10_000);
  });

  onDestroy(() => {
    window.removeEventListener("pap:prompt-corrente", onPromptCorrente);
    window.removeEventListener("pap:save-stato", onSaveStato);
    if (timerTick) clearInterval(timerTick);
  });

  function tempoRelativoBreve(iso: string | null, _tick: number): string {
    if (!iso) return "";
    try {
      const sec = Math.max(
        0,
        Math.floor((Date.now() - new Date(iso).getTime()) / 1000),
      );
      if (sec < 5) return "ora";
      if (sec < 60) return `${sec}s fa`;
      if (sec < 3600) return `${Math.floor(sec / 60)}m fa`;
      return `${Math.floor(sec / 3600)}h fa`;
    } catch {
      return "";
    }
  }

  const labelSalvataggio = $derived.by(() => {
    if (statoSalvataggio === null) return "";
    if (statoSalvataggio === "salvando") return "salvando…";
    if (statoSalvataggio === "dirty") return "modifiche non salvate";
    if (statoSalvataggio === "errore") return "errore salvataggio";
    const rel = tempoRelativoBreve(salvatoA, tickRelative);
    return rel ? `salvato ${rel}` : "salvato";
  });

  const tooltipVault = $derived(
    vaultAperto
      ? vaultPath
        ? `Vault locale: ${vaultPath}`
        : "Vault locale (caricamento…)"
      : "Vault chiuso",
  );
</script>

<footer class="statusbar">
  <div class="seg" title={tooltipVault}>
    <span
      class="dot"
      class:dot-ok={vaultAperto}
      class:dot-err={!vaultAperto}
      aria-hidden="true"
    ></span>
    <span>vault {vaultAperto ? "locale" : "chiuso"}</span>
  </div>

  <span class="sep" aria-hidden="true"></span>

  <div class="seg seg-prompt">
    {#if promptCorrente}
      <span class="ico-vis" aria-hidden="true">
        {#if promptCorrente.visibilita === "private"}
          <Lock size={11} />
        {:else}
          <Users size={11} />
        {/if}
      </span>
      <span class="titolo">{promptCorrente.titolo}</span>
    {:else}
      <span class="muted">(nessun prompt selezionato)</span>
    {/if}
  </div>

  <div class="right">
    {#if labelSalvataggio}
      <span class="seg save-stato" data-stato={statoSalvataggio ?? ""}>
        <span class="dot" aria-hidden="true"></span>
        <span>{labelSalvataggio}</span>
      </span>
    {/if}
    <button
      class="seg clickable"
      type="button"
      onclick={() => console.log("F8 palette")}
      title="Apri command palette (F8)"
    >
      <kbd>⌃⇧P</kbd>
      <span class="muted">cerca</span>
    </button>
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: var(--statusbar-h);
    padding: 0 var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .seg {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .seg-prompt {
    min-width: 0;
    flex: 0 1 auto;
  }

  .titolo {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 280px;
    color: var(--text-default);
  }

  .ico-vis {
    display: inline-flex;
    align-items: center;
    color: var(--text-subtle);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--text-muted);
  }

  .dot-ok {
    background: var(--success);
  }

  .dot-err {
    background: var(--danger);
  }

  .save-stato[data-stato="salvato"] .dot {
    background: var(--success);
  }

  .save-stato[data-stato="dirty"] .dot {
    background: var(--warning);
  }

  .save-stato[data-stato="salvando"] .dot {
    background: var(--info);
    animation: pulse 1s ease-in-out infinite;
  }

  .save-stato[data-stato="errore"] .dot {
    background: var(--danger);
  }

  .save-stato[data-stato="errore"] {
    color: var(--danger);
  }

  .sep {
    width: 1px;
    height: 14px;
    background: var(--border-subtle);
  }

  .right {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .clickable {
    border: 0;
    background: transparent;
    color: inherit;
    font-size: inherit;
    font-family: inherit;
    cursor: pointer;
    padding: 0 var(--sp-1);
    border-radius: var(--radius-sm);
    transition: background var(--motion-fast);
  }

  .clickable:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px var(--sp-1);
    border-radius: var(--radius-sm);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    color: var(--text-default);
  }

  .muted {
    color: var(--text-subtle);
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }
</style>
