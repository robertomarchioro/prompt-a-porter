<script lang="ts">
  import { Button } from "$lib/components";

  interface Props {
    onchiudi: () => void;
    ontornaLogin: () => void;
    serverUrl?: string;
  }

  let { onchiudi, ontornaLogin, serverUrl = "" }: Props = $props();

  let email = $state("");
  let inviato = $state(false);
  let caricamento = $state(false);
  let errore = $state("");

  async function inviaReset() {
    errore = "";
    if (!email) {
      errore = "Inserisci la tua email";
      return;
    }

    caricamento = true;
    try {
      const url = serverUrl.replace(/\/+$/, "");
      await fetch(`${url}/auth/reset-password`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });

      // Mostra sempre "inviato" per non rivelare se l'email esiste
      inviato = true;
    } catch {
      inviato = true;
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
    aria-label="Reset password"
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
        <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
      </svg>
      <h2>Reimposta password</h2>
      <p class="auth-sub">
        Inserisci la tua email per ricevere il link di reset
      </p>
    </header>

    {#if !inviato}
      <form
        class="auth-form"
        onsubmit={(e) => {
          e.preventDefault();
          inviaReset();
        }}
      >
        <div class="avviso">
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
            <path
              d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
            />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
          <span
            >I prompt cifrati con la vecchia password andranno persi</span
          >
        </div>

        <div class="campo">
          <label class="campo-label" for="reset-email">Email</label>
          <input
            id="reset-email"
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
          {caricamento ? "Invio…" : "Invia link di reset"}
        </Button>

        <div class="auth-link-row">
          <button class="auth-link" type="button" onclick={ontornaLogin}>
            ← Torna al login
          </button>
        </div>
      </form>
    {:else}
      <div class="auth-risultato">
        <div class="risultato-icona risultato-icona--ok">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
        </div>
        <p class="risultato-testo">
          Link inviato! Controlla la tua email.
        </p>
        <Button dimensione="sm" onclick={ontornaLogin}>
          Torna al login
        </Button>
      </div>
    {/if}

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
    color: var(--accent-private);
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

  .avviso {
    display: flex;
    align-items: flex-start;
    gap: var(--sp-2);
    padding: var(--sp-3);
    background: color-mix(in oklch, var(--warning) 10%, transparent);
    border: 1px solid color-mix(in oklch, var(--warning) 30%, transparent);
    border-radius: var(--radius-md);
    font-size: var(--fs-xs);
    color: var(--warning);
    line-height: var(--lh-snug);
  }

  .avviso svg {
    flex-shrink: 0;
    margin-top: 1px;
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

  .auth-link-row {
    display: flex;
    justify-content: center;
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

  .auth-risultato {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-4);
    padding: var(--sp-4) 0;
  }

  .risultato-icona {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .risultato-icona--ok {
    background: color-mix(in oklch, var(--success) 15%, transparent);
    color: var(--success);
  }

  .risultato-testo {
    font-size: var(--fs-sm);
    color: var(--text-default);
    text-align: center;
    margin: 0;
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
