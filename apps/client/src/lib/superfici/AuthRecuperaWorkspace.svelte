<script lang="ts">
  import { Button } from "$lib/components";

  interface Props {
    onchiudi: () => void;
    ontornaLogin: () => void;
    onseleziona: (serverUrl: string) => void;
  }

  let { onchiudi, ontornaLogin, onseleziona }: Props = $props();

  interface WorkspaceInfo {
    id: string;
    nome: string;
    serverUrl: string;
    tipo: string;
  }

  let email = $state("");
  let caricamento = $state(false);
  let cercato = $state(false);
  let risultati = $state<WorkspaceInfo[]>([]);
  let errore = $state("");

  async function cercaWorkspace() {
    errore = "";
    if (!email) {
      errore = "Inserisci la tua email";
      return;
    }

    caricamento = true;
    cercato = false;
    risultati = [];

    try {
      // In Fase 1 non c'è un endpoint di discovery server-side,
      // quindi mostriamo solo la possibilità di inserire manualmente l'URL.
      // Il flusso reale sarà: l'utente conosce il server URL dal suo admin.
      cercato = true;
    } catch {
      errore = "Errore nella ricerca";
    } finally {
      caricamento = false;
    }
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onchiudi();
  }}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="scrim"
  onmousedown={(e) => {
    if (e.target === e.currentTarget) onchiudi();
  }}
>
  <div
    class="auth-card"
    role="dialog"
    aria-modal="true"
    aria-label="Recupera workspace"
  >
    <header class="auth-header">
      <svg
        width="32"
        height="32"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"
        />
        <circle cx="9" cy="7" r="4" />
        <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
      <h2>Recupera workspace</h2>
      <p class="auth-sub">
        Inserisci la tua email per trovare i workspace associati
      </p>
    </header>

    <form
      class="auth-form"
      onsubmit={(e) => {
        e.preventDefault();
        cercaWorkspace();
      }}
    >
      <div class="campo">
        <label class="campo-label" for="recupera-email">Email</label>
        <input
          id="recupera-email"
          class="campo-input"
          type="email"
          bind:value={email}
          placeholder="nome@azienda.com"
          autocomplete="email"
          autofocus
        />
      </div>

      {#if errore}
        <p class="auth-errore">{errore}</p>
      {/if}

      <Button
        variante="primary"
        type="submit"
        disabled={caricamento || !email}
      >
        {caricamento ? "Ricerca…" : "Cerca workspace"}
      </Button>
    </form>

    {#if cercato}
      <div class="risultati">
        {#if risultati.length > 0}
          {#each risultati as ws}
            <button
              class="ws-item"
              type="button"
              onclick={() => onseleziona(ws.serverUrl)}
            >
              <div class="ws-avatar">
                {ws.nome.charAt(0).toUpperCase()}
              </div>
              <div class="ws-info">
                <span class="ws-nome">{ws.nome}</span>
                <span class="ws-url">{ws.serverUrl}</span>
              </div>
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
          {/each}
        {:else}
          <div class="nessuno">
            <p>Nessun workspace trovato per questa email.</p>
            <p class="nessuno-hint">
              Chiedi al tuo admin l'URL del server sync e usa il login
              diretto.
            </p>
          </div>
        {/if}
      </div>
    {/if}

    <div class="auth-footer">
      <button class="auth-link" type="button" onclick={ontornaLogin}>
        ← Torna al login
      </button>
    </div>

    <button class="auth-chiudi" type="button" onclick={onchiudi}
      >✕</button
    >
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
  }

  .auth-card {
    position: relative;
    width: min(440px, 94vw);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    padding: var(--sp-6, 24px) var(--sp-6, 24px) var(--sp-5, 20px);
  }

  .auth-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-2);
    text-align: center;
    margin-bottom: var(--sp-5);
    color: var(--accent-team);
  }

  .auth-header h2 {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .auth-sub {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    max-width: 30ch;
  }

  .auth-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .campo-label {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    font-weight: var(--fw-medium);
  }

  .campo-input {
    height: 38px;
    padding: 0 var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    outline: none;
    transition: border-color var(--motion-fast);
  }
  .campo-input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }
  .campo-input::placeholder {
    color: var(--text-subtle);
  }

  .auth-errore {
    font-size: var(--fs-xs);
    color: var(--danger);
    margin: 0;
    text-align: center;
  }

  .risultati {
    margin-top: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .ws-item {
    appearance: none;
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    width: 100%;
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-family: var(--font-ui);
    color: var(--text-default);
    text-align: left;
    transition: all var(--motion-fast);
  }
  .ws-item:hover {
    border-color: var(--accent-team);
    background: var(--bg-overlay);
  }

  .ws-avatar {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    background: linear-gradient(
      135deg,
      var(--accent-team),
      var(--accent-team-strong)
    );
    color: var(--accent-team-on);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--fs-base);
    font-weight: var(--fw-bold);
    flex-shrink: 0;
  }

  .ws-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .ws-nome {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
  }

  .ws-url {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nessuno {
    text-align: center;
    padding: var(--sp-4);
  }

  .nessuno p {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .nessuno-hint {
    margin-top: var(--sp-2) !important;
    font-size: var(--fs-xs) !important;
    color: var(--text-subtle) !important;
  }

  .auth-footer {
    display: flex;
    justify-content: center;
    margin-top: var(--sp-4);
  }

  .auth-link {
    appearance: none;
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--accent-team);
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .auth-link:hover {
    color: var(--accent-team-strong);
  }

  .auth-chiudi {
    position: absolute;
    top: var(--sp-3);
    right: var(--sp-3);
    appearance: none;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    transition: background var(--motion-fast);
  }
  .auth-chiudi:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
</style>
