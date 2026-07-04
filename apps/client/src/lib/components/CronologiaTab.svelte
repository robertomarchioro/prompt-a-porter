<script lang="ts">
  /**
   * Tab Cronologia del DetailPane (V0.8 F5 PR-D).
   *
   * Lista versioni storiche del prompt con avatar autore deterministico
   * (SHA1+HSL #13). Click su versione → diff side-by-side via DiffViewer.
   * Bottone rollback ripristina la versione selezionata come testa.
   *
   * Riferimenti:
   * - Decisione designer #9 (diff side-by-side default + toggle unified)
   * - Decisione designer #13 (avatar SHA1+HSL deterministico)
   * - V014 backend extension (autore_display_name + autore_email)
   * - Blueprint F5 PR-D §4
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import { History, RotateCcw } from "lucide-svelte";
  import {
    avatarColorePerEmail,
    type ColoreAvatar,
  } from "$lib/util/avatar-hash";
  import DiffViewer from "./DiffViewer.svelte";

  interface VersioneStorica {
    id: string;
    prompt_id: string;
    version: number;
    titolo: string;
    descrizione: string | null;
    body: string;
    visibilita: string | null;
    target_model: string | null;
    creato_a: string;
    creato_da_user_id: string;
    autore_display_name: string;
    autore_email: string | null;
  }

  interface Props {
    promptId: string;
    onConteggio?: (n: number) => void;
  }

  let { promptId, onConteggio }: Props = $props();

  let versioni = $state<VersioneStorica[]>([]);
  let caricamento = $state(false);
  let selezionata = $state<string | null>(null);
  let coloriAutore = $state<Record<string, ColoreAvatar>>({});

  async function carica(): Promise<void> {
    caricamento = true;
    try {
      versioni = await invoke<VersioneStorica[]>("prompt_get_history", {
        promptId,
      });
      onConteggio?.(versioni.length);
      if (versioni.length > 0 && !selezionata) {
        selezionata = versioni[0].id;
      }
      void caricaColoriAutori();
    } catch (e) {
      console.error("[cronologia] prompt_get_history", e);
      versioni = [];
      onConteggio?.(0);
    } finally {
      caricamento = false;
    }
  }

  async function caricaColoriAutori(): Promise<void> {
    const next: Record<string, ColoreAvatar> = {};
    for (const v of versioni) {
      const key = v.autore_email ?? v.autore_display_name;
      if (next[key]) continue;
      next[key] = await avatarColorePerEmail(key);
    }
    coloriAutore = next;
  }

  $effect(() => {
    void promptId;
    selezionata = null;
    void carica();
  });

  function gestListaMutata(): void {
    void carica();
  }

  onMount(() => {
    window.addEventListener("pap:lista-mutata", gestListaMutata);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", gestListaMutata);
  });

  async function rollback(version: number, etichetta: string): Promise<void> {
    if (
      !confirm(
        `Ripristinare la versione ${etichetta}? Verrà creata una nuova versione con questo contenuto.`,
      )
    )
      return;
    try {
      await invoke("prompt_rollback", {
        promptId,
        targetVersion: version,
      });
      // Evento dedicato: il DetailPane ricarica il contenuto del prompt aperto
      // così l'editor mostra subito la versione ripristinata (#425). Distinto
      // da `pap:lista-mutata` (che lo dispatcha anche l'autosave dell'editor).
      window.dispatchEvent(
        new CustomEvent("pap:prompt-ripristinato", { detail: { promptId } }),
      );
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      console.error("[cronologia] prompt_rollback", e);
    }
  }

  function tempoRelativo(iso: string): string {
    if (!iso) return "";
    try {
      const sec = Math.max(
        0,
        Math.floor((Date.now() - new Date(iso).getTime()) / 1000),
      );
      if (sec < 60) return "ora";
      const min = Math.floor(sec / 60);
      if (min < 60) return `${min}m fa`;
      const h = Math.floor(min / 60);
      if (h < 24) return `${h}h fa`;
      const g = Math.floor(h / 24);
      return `${g}g fa`;
    } catch {
      return "";
    }
  }

  function inizialeAutore(nome: string): string {
    return (nome.charAt(0) || "?").toUpperCase();
  }

  // versioni è ordinato dal più recente. Diff confronta selezionata con
  // la versione IMMEDIATAMENTE più vecchia. Se è la più vecchia (idx ultimo)
  // confronta con stringa vuota (creazione iniziale).
  const versione = $derived(
    versioni.find((v) => v.id === selezionata) ?? null,
  );

  const bodyA = $derived.by(() => {
    if (!versione) return "";
    const idx = versioni.findIndex((v) => v.id === versione.id);
    if (idx < 0 || idx >= versioni.length - 1) return "";
    return versioni[idx + 1].body;
  });
  const bodyB = $derived(versione?.body ?? "");
  const labelA = $derived.by(() => {
    if (!versione) return "v?";
    const idx = versioni.findIndex((v) => v.id === versione.id);
    return idx >= versioni.length - 1
      ? "(vuoto)"
      : `v${versioni[idx + 1].version}`;
  });
  const labelB = $derived(versione ? `v${versione.version}` : "v?");
</script>

<div class="cronologia-tab">
  <aside class="lista-versioni">
    <header class="header">
      <History size={14} />
      <span class="titolo">Cronologia</span>
      <span class="conteggio">{versioni.length}</span>
    </header>

    {#if caricamento && versioni.length === 0}
      <p class="vuoto">Caricamento…</p>
    {:else if versioni.length === 0}
      <p class="vuoto">Nessuna versione storica.</p>
    {:else}
      <ul class="lista" role="list">
        {#each versioni as v, i (v.id)}
          {@const colore =
            coloriAutore[v.autore_email ?? v.autore_display_name]}
          <li>
            <div
              class="versione"
              role="button"
              tabindex="0"
              data-attivo={selezionata === v.id || undefined}
              onclick={() => (selezionata = v.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  selezionata = v.id;
                }
              }}
            >
              <span
                class="avatar"
                style:background={colore?.background ?? "var(--text-subtle)"}
                style:color={colore?.foreground ?? "#fff"}
                aria-hidden="true"
              >
                {inizialeAutore(v.autore_display_name)}
              </span>
              <span class="meta">
                <span class="riga-1">
                  <span class="versione-num">v{v.version}</span>
                  <span class="autore">{v.autore_display_name}</span>
                </span>
                <span class="riga-2">
                  <span class="quando">{tempoRelativo(v.creato_a)}</span>
                  <span class="titolo-rev">· {v.titolo}</span>
                </span>
              </span>
              {#if i > 0}
                <button
                  class="rollback-btn"
                  type="button"
                  title="Ripristina questa versione"
                  aria-label="Ripristina v{v.version}"
                  onclick={(e) => {
                    e.stopPropagation();
                    rollback(v.version, `v${v.version}`);
                  }}
                >
                  <RotateCcw size={12} />
                </button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </aside>

  <div class="diff-area">
    {#if !versione}
      <p class="placeholder">Seleziona una versione per vedere il diff.</p>
    {:else}
      <DiffViewer
        {bodyA}
        {bodyB}
        etichettaA={labelA}
        etichettaB={labelB}
      />
    {/if}
  </div>
</div>

<style>
  .cronologia-tab {
    flex: 1;
    display: grid;
    grid-template-columns: 280px 1fr;
    overflow: hidden;
    background: var(--bg-canvas);
  }

  .lista-versioni {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-subtle);
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-subtle);
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .titolo {
    color: var(--text-muted);
  }

  .conteggio {
    margin-left: auto;
    background: var(--bg-overlay);
    color: var(--text-default);
    padding: 1px 6px;
    border-radius: var(--radius-full);
    text-transform: none;
    letter-spacing: 0;
    font-size: 11px;
    font-weight: var(--fw-regular);
  }

  .vuoto,
  .placeholder {
    color: var(--text-subtle);
    text-align: center;
    padding: var(--sp-4);
    margin: 0;
    font-size: var(--fs-sm);
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .lista {
    list-style: none;
    margin: 0;
    padding: var(--sp-1);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .versione {
    display: grid;
    grid-template-columns: 24px 1fr auto;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    border-left: 2px solid transparent;
    background: transparent;
    color: var(--text-default);
    cursor: pointer;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    transition: background var(--motion-fast);
  }

  .versione:hover {
    background: var(--bg-overlay);
  }

  .versione[data-attivo] {
    background: var(--bg-overlay);
    border-left-color: var(--accent-team);
  }

  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    font-weight: var(--fw-semibold);
    font-size: 11px;
    font-family: var(--font-ui);
  }

  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .riga-1 {
    display: inline-flex;
    align-items: baseline;
    gap: var(--sp-1);
    font-size: var(--fs-sm);
  }

  .versione-num {
    font-family: var(--font-mono);
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }

  .autore {
    color: var(--text-default);
    font-weight: var(--fw-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .riga-2 {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    font-size: 11px;
    color: var(--text-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .titolo-rev {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-muted);
  }

  .rollback-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: 0;
    background: transparent;
    color: var(--text-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--motion-fast);
  }

  .versione:hover .rollback-btn {
    opacity: 1;
  }

  .rollback-btn:hover {
    background: var(--bg-canvas);
    color: var(--text-default);
  }

  .diff-area {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
