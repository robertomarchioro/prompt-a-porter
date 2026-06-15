<script lang="ts">
  /**
   * Vista Cestino (#302): elenca i prompt soft-deleted e permette di
   * ripristinarli o eliminarli definitivamente. Sostituisce ListPane quando
   * la vista corrente è "cestino".
   *
   * Backend: cestino_lista / prompt_ripristina / prompt_elimina_definitivo /
   * cestino_svuota (modulo cestino.rs). Dopo ogni mutazione dispatcha
   * `pap:lista-mutata` così Sidebar (conteggi) e altre viste si rinfrescano.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { RotateCcw, Trash2, Link2 } from "lucide-svelte";
  import Toast from "./Toast.svelte";

  interface PromptCancellato {
    id: string;
    titolo: string;
    eliminato_il: string;
    importato_da: number;
  }

  let prompts = $state<PromptCancellato[]>([]);
  let caricamento = $state(true);
  let toastVisibile = $state(false);
  let toastTesto = $state("");

  function showToast(testo: string): void {
    toastTesto = testo;
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 1800);
  }

  async function carica(): Promise<void> {
    caricamento = true;
    try {
      prompts = await invoke<PromptCancellato[]>("cestino_lista");
    } catch (e) {
      console.error("[cestino] carica", e);
      prompts = [];
    } finally {
      caricamento = false;
    }
  }

  function notificaMutazione(): void {
    window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
  }

  async function ripristina(p: PromptCancellato): Promise<void> {
    try {
      await invoke("prompt_ripristina", { id: p.id });
      notificaMutazione();
      await carica();
      showToast(`"${p.titolo || "(senza titolo)"}" ripristinato`);
    } catch (e) {
      console.error("[cestino] ripristina", e);
      showToast("Errore nel ripristino");
    }
  }

  async function eliminaDefinitivo(p: PromptCancellato): Promise<void> {
    const titolo = p.titolo || "(senza titolo)";
    const ok = window.confirm(
      `Eliminare definitivamente "${titolo}"?\n\nQuesta operazione NON è reversibile: il prompt e la sua cronologia verranno rimossi per sempre.`,
    );
    if (!ok) return;
    try {
      await invoke("prompt_elimina_definitivo", { id: p.id });
      notificaMutazione();
      await carica();
      showToast(`"${titolo}" eliminato definitivamente`);
    } catch (e) {
      console.error("[cestino] elimina definitivo", e);
      showToast("Errore nell'eliminazione");
    }
  }

  async function svuota(): Promise<void> {
    if (prompts.length === 0) return;
    const ok = window.confirm(
      `Svuotare il cestino?\n\n${prompts.length} prompt verranno eliminati definitivamente. L'operazione NON è reversibile.`,
    );
    if (!ok) return;
    try {
      const n = await invoke<number>("cestino_svuota");
      notificaMutazione();
      await carica();
      showToast(`Cestino svuotato (${n} prompt)`);
    } catch (e) {
      console.error("[cestino] svuota", e);
      showToast("Errore nello svuotamento");
    }
  }

  function dataLeggibile(iso: string): string {
    // DeletedAt è "YYYY-MM-DD HH:MM:SS" (datetime SQLite, UTC).
    const d = new Date(iso.replace(" ", "T") + "Z");
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleString(undefined, {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  onMount(() => {
    void carica();
    window.addEventListener("pap:lista-mutata", carica);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", carica);
  });
</script>

<div class="cestino">
  <header class="cestino-header">
    <div class="titolo-wrap">
      <h2 class="titolo">Cestino</h2>
      {#if prompts.length > 0}
        <span class="conteggio">{prompts.length}</span>
      {/if}
    </div>
    {#if prompts.length > 0}
      <button type="button" class="svuota-btn" onclick={svuota}>
        Svuota cestino
      </button>
    {/if}
  </header>

  {#if caricamento}
    <div class="vuoto"><p>Caricamento…</p></div>
  {:else if prompts.length === 0}
    <div class="vuoto">
      <p class="vuoto-titolo">Il cestino è vuoto</p>
      <p class="vuoto-sub">I prompt eliminati compaiono qui e possono essere ripristinati.</p>
    </div>
  {:else}
    <ul class="lista" role="list">
      {#each prompts as p (p.id)}
        <li class="riga">
          <div class="info">
            <div class="nome">{p.titolo || "(senza titolo)"}</div>
            <div class="meta">
              <span>Eliminato il {dataLeggibile(p.eliminato_il)}</span>
              {#if p.importato_da > 0}
                <span class="badge-import" title="Prompt vivi che lo importano: ripristinandolo ricuci import oggi rotti">
                  <Link2 size={11} />
                  {p.importato_da}
                </span>
              {/if}
            </div>
          </div>
          <div class="azioni">
            <button
              type="button"
              class="azione ripristina"
              title="Ripristina"
              aria-label="Ripristina {p.titolo || 'prompt'}"
              onclick={() => ripristina(p)}
            >
              <RotateCcw size={14} />
            </button>
            <button
              type="button"
              class="azione elimina"
              title="Elimina definitivamente"
              aria-label="Elimina definitivamente {p.titolo || 'prompt'}"
              onclick={() => eliminaDefinitivo(p)}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<Toast visibile={toastVisibile}>{toastTesto}</Toast>

<style>
  .cestino {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-canvas);
    overflow: hidden;
  }

  .cestino-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .titolo-wrap {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .titolo {
    margin: 0;
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .conteggio {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    background: var(--bg-overlay);
    border-radius: var(--radius-full);
    padding: 0 6px;
    line-height: 1.6;
  }

  .svuota-btn {
    border: 1px solid var(--border-default);
    background: transparent;
    color: var(--danger);
    font-size: var(--fs-xs);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .svuota-btn:hover {
    background: var(--danger);
    color: var(--bg-canvas);
    border-color: var(--danger);
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

  .vuoto-titolo {
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }

  .vuoto-sub {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .lista {
    list-style: none;
    margin: 0;
    padding: var(--sp-2);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .riga {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 8px 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
  }

  .info {
    flex: 1;
    min-width: 0;
  }

  .nome {
    font-size: var(--fs-sm);
    color: var(--text-default);
    font-weight: var(--fw-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-top: 2px;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .badge-import {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    color: var(--warning);
  }

  .azioni {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .azione {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--motion-fast);
  }

  .azione.ripristina:hover {
    background: var(--bg-overlay);
    color: var(--success);
  }

  .azione.elimina:hover {
    background: var(--bg-overlay);
    color: var(--danger);
  }
</style>
