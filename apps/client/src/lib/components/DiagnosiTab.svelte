<script lang="ts">
  /**
   * Tab Diagnosi del DetailPane (F5 PR-B).
   *
   * Lista issue lint per il body corrente raggruppate per severità.
   * Click su issue dispatch CustomEvent "pap:goto-line" che il DetailPane
   * intercetta per switchare al tab Editor + scrollare alla riga.
   *
   * Riferimento blueprint: docs/roadmap/redesign-v08/blueprint-F5.md §2
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { AlertCircle, AlertTriangle, Info } from "lucide-svelte";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import { leggiConfig } from "$lib/preferenze-linter";

  type Severita = "Error" | "Warning" | "Info";

  interface Issue {
    code: string;
    severita: Severita;
    messaggio: string;
    linea?: number;
    colonna?: number;
  }

  interface Props {
    promptId: string;
    body: string;
    /** Callback opzionale per notificare il numero di issue (badge tab) */
    onConteggio?: (n: number) => void;
  }

  let { promptId, body, onConteggio }: Props = $props();

  let issues = $state<Issue[]>([]);
  let caricamento = $state(false);

  let timer: ReturnType<typeof setTimeout> | undefined;

  async function carica(): Promise<void> {
    caricamento = true;
    try {
      const res = await invoke<Issue[]>("prompt_lint", {
        body,
        promptId,
        config: leggiConfig(),
      });
      issues = res;
      onConteggio?.(res.length);
    } catch (e) {
      console.error("[diagnosi] prompt_lint", e);
      issues = [];
      onConteggio?.(0);
    } finally {
      caricamento = false;
    }
  }

  // Debounce 400ms su body change per non spammare il backend
  $effect(() => {
    void promptId;
    void body;
    if (timer) clearTimeout(timer);
    timer = setTimeout(carica, 400);
  });

  // Ri-lintaa quando l'utente cambia le regole attive in Impostazioni → Linter
  // (altrimenti i risultati resterebbero stale finché non si riedita il body).
  function onLinterCambiato(): void {
    void carica();
  }

  onMount(() => {
    window.addEventListener("pap:linter-config-cambiata", onLinterCambiato);
  });

  onDestroy(() => {
    if (timer) clearTimeout(timer);
    window.removeEventListener("pap:linter-config-cambiata", onLinterCambiato);
  });

  function categoria(code: string): string {
    const prefix = code.replace(/[0-9]+$/, "");
    switch (prefix) {
      case "LEN":
        return "Lunghezza";
      case "PH":
        return "Segnaposti";
      case "PII":
        return "Privacy";
      case "STY":
        return "Stile";
      case "IMP":
        return "Import";
      default:
        return prefix;
    }
  }

  function vaiAllaRiga(linea: number | undefined): void {
    if (typeof linea !== "number") return;
    window.dispatchEvent(
      new CustomEvent("pap:goto-line", { detail: linea }),
    );
  }

  // Raggruppa per severità (Error → Warning → Info), poi linea asc
  const ordinato = $derived.by(() => {
    const sev = (s: Severita) =>
      s === "Error" ? 0 : s === "Warning" ? 1 : 2;
    return [...issues].sort((a, b) => {
      const d = sev(a.severita) - sev(b.severita);
      if (d !== 0) return d;
      return (a.linea ?? 0) - (b.linea ?? 0);
    });
  });
</script>

<div class="diagnosi-tab">
  <header class="diagnosi-h">
    <span>Linter</span>
    <AiutoLink chiave="linting" dimensione={16} />
  </header>
  {#if caricamento && issues.length === 0}
    <div class="vuoto">
      <p>Analisi in corso…</p>
    </div>
  {:else if issues.length === 0}
    <div class="vuoto vuoto-ok">
      <p class="ok">Nessuna issue rilevata.</p>
      <p class="sub">Body pulito secondo le regole abilitate.</p>
    </div>
  {:else}
    <ul class="lista" role="list">
      {#each ordinato as it (it.code + (it.linea ?? 0) + it.messaggio)}
        <li>
          <button
            class="issue"
            type="button"
            data-sev={it.severita.toLowerCase()}
            disabled={it.linea === undefined}
            onclick={() => vaiAllaRiga(it.linea)}
            title={it.linea !== undefined
              ? `Vai alla riga ${it.linea}`
              : "Issue globale"}
          >
            <span class="ico" aria-hidden="true">
              {#if it.severita === "Error"}
                <AlertCircle size={14} />
              {:else if it.severita === "Warning"}
                <AlertTriangle size={14} />
              {:else}
                <Info size={14} />
              {/if}
            </span>
            <span class="meta">
              <span class="code">{it.code}</span>
              <span class="cat">· {categoria(it.code)}</span>
              {#if it.linea !== undefined}
                <span class="riga">· L:{it.linea}</span>
              {/if}
            </span>
            <span class="msg">{it.messaggio}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .diagnosi-tab {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2);
    background: var(--bg-canvas);
  }

  .diagnosi-h {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: var(--sp-2);
    font-size: var(--fs-xs);
    font-weight: var(--fw-medium);
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  .vuoto {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--text-muted);
    padding: var(--sp-4);
    gap: var(--sp-1);
  }

  .vuoto p {
    margin: 0;
    font-size: var(--fs-sm);
  }

  .vuoto-ok .ok {
    color: var(--success);
    font-weight: var(--fw-medium);
  }

  .vuoto .sub {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .issue {
    display: grid;
    grid-template-columns: 18px 1fr;
    grid-template-rows: auto auto;
    gap: 2px var(--sp-2);
    width: 100%;
    border: 0;
    border-left: 2px solid transparent;
    background: transparent;
    color: var(--text-default);
    text-align: left;
    cursor: pointer;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    transition: background var(--motion-fast);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }

  .issue:hover:not(:disabled) {
    background: var(--bg-overlay);
  }

  .issue:disabled {
    cursor: default;
    opacity: 0.7;
  }

  .issue[data-sev="error"] {
    border-left-color: var(--danger);
  }

  .issue[data-sev="error"] .ico {
    color: var(--danger);
  }

  .issue[data-sev="warning"] {
    border-left-color: var(--warning);
  }

  .issue[data-sev="warning"] .ico {
    color: var(--warning);
  }

  .issue[data-sev="info"] {
    border-left-color: var(--info);
  }

  .issue[data-sev="info"] .ico {
    color: var(--info);
  }

  .ico {
    grid-row: 1 / 3;
    grid-column: 1;
    display: inline-flex;
    align-items: start;
    justify-content: center;
    padding-top: 2px;
  }

  .meta {
    grid-row: 1;
    grid-column: 2;
    font-size: 11px;
    color: var(--text-subtle);
    font-family: var(--font-mono);
  }

  .code {
    color: var(--text-muted);
    font-weight: var(--fw-medium);
  }

  .cat,
  .riga {
    margin-left: 2px;
  }

  .msg {
    grid-row: 2;
    grid-column: 2;
    color: var(--text-default);
    font-size: var(--fs-sm);
    line-height: 1.45;
  }
</style>
