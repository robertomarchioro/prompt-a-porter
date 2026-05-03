<script lang="ts">
  import { Button, Toast } from "$lib/components";
  import { syncLogin, syncConfigura } from "$lib/sync";

  interface Props {
    onchiudi: () => void;
    onconnesso: () => void;
    onresetpassword: () => void;
  }

  let { onchiudi, onconnesso, onresetpassword }: Props = $props();

  let serverUrl = $state("https://");
  let email = $state("");
  let password = $state("");
  let caricamento = $state(false);
  let errore = $state("");
  let toastVisibile = $state(false);
  let toastTesto = $state("");

  async function accedi() {
    errore = "";
    if (!serverUrl || !email || !password) {
      errore = "Tutti i campi sono obbligatori";
      return;
    }

    caricamento = true;
    try {
      const { token, user } = await syncLogin(serverUrl, email, password);

      await syncConfigura({
        serverUrl,
        email,
        token,
        intervalloSec: 60,
        abilitato: true,
      });

      toastTesto = "Connessione riuscita";
      toastVisibile = true;
      setTimeout(() => {
        toastVisibile = false;
        onconnesso();
      }, 1000);
    } catch (e) {
      errore = e instanceof Error ? e.message : "Errore di connessione";
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
  <div class="auth-card" role="dialog" aria-modal="true" aria-label="Login">
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
        <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
      <h2>Accedi al server sync</h2>
      <p class="auth-sub">Connettiti per sincronizzare i prompt del workspace team</p>
    </header>

    <form
      class="auth-form"
      onsubmit={(e) => {
        e.preventDefault();
        accedi();
      }}
    >
      <div class="campo">
        <label class="campo-label" for="server-url">URL del server</label>
        <input
          id="server-url"
          class="campo-input"
          type="url"
          bind:value={serverUrl}
          placeholder="https://sync.example.com"
        />
      </div>

      <div class="campo">
        <label class="campo-label" for="login-email">Email</label>
        <input
          id="login-email"
          class="campo-input"
          type="email"
          bind:value={email}
          placeholder="nome@azienda.com"
          autocomplete="email"
        />
      </div>

      <div class="campo">
        <label class="campo-label" for="login-password">Password</label>
        <input
          id="login-password"
          class="campo-input"
          type="password"
          bind:value={password}
          placeholder="Password"
          autocomplete="current-password"
        />
      </div>

      {#if errore}
        <p class="auth-errore">{errore}</p>
      {/if}

      <Button
        variante="primary"
        type="submit"
        disabled={caricamento || !email || !password || !serverUrl}
      >
        {caricamento ? "Connessione…" : "Accedi"}
      </Button>

      <div class="auth-link-row">
        <button
          class="auth-link"
          type="button"
          onclick={onresetpassword}
        >
          Password dimenticata?
        </button>
      </div>
    </form>

    <button class="auth-chiudi" type="button" onclick={onchiudi}>✕</button>
  </div>
</div>

<Toast variante="success" visibile={toastVisibile}>
  ✓ {toastTesto}
</Toast>

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
