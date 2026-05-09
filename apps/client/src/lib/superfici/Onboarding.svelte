<script lang="ts">
  /**
   * F9 PR-A — Onboarding consolidato.
   *
   * Wrapper che gestisce 3 stati per il flusso pre-Shell:
   *
   *   caricamento ─► sblocco (vault esiste cifrato) ─► oncompletato()
   *               ─► setup   (vault non esiste)     ─► oncompletato()
   *               ─► oncompletato() (vault già aperto)
   *
   * Lo step "setup" delega a `<OnboardingWizard />` esistente (533 righe,
   * gestisce profilo + master pwd + hotkey + creazione vault).
   * Lo step "sblocco" è nuovo — assorbe la logica `stato === "blocco"` di
   * Libreria.svelte (linee 700+) che sarà cancellata in F9 PR-C.
   *
   * Le superfici legacy `AuthLogin/Recupera/Reset` NON vengono assorbite
   * qui: gestiscono la config del sync server remoto (accessibile da
   * Impostazioni → Avanzate → Sync, F8 PR-D2), non il flusso primary auth.
   *
   * Riferimenti:
   * - Blueprint: docs/roadmap/redesign-v08/blueprint-F9.md §1
   * - Cmd backend: src-tauri/src/vault.rs (vault_esiste/aperto/cifrato/unlock)
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Lock } from "lucide-svelte";
  import OnboardingWizard from "$lib/superfici/OnboardingWizard.svelte";

  interface PreferenzeMin {
    onboarding_completato: boolean;
  }

  interface Props {
    oncompletato: () => void;
  }

  let { oncompletato }: Props = $props();

  type Stato = "caricamento" | "setup" | "sblocco" | "errore";

  let stato = $state<Stato>("caricamento");
  let password = $state("");
  let errore = $state("");
  let sbloccoInCorso = $state(false);

  async function rilevaStatoIniziale(): Promise<void> {
    try {
      const [aperto, esiste, prefs] = await Promise.all([
        invoke<boolean>("vault_aperto"),
        invoke<boolean>("vault_esiste"),
        invoke<PreferenzeMin>("preferenze_carica").catch(() => ({
          onboarding_completato: false,
        })),
      ]);

      if (aperto) {
        oncompletato();
        return;
      }

      if (esiste) {
        // Vault sul disco ma non aperto: deduco se è cifrato.
        try {
          const cifrato = await invoke<boolean>("vault_cifrato");
          if (cifrato) {
            stato = "sblocco";
          } else {
            // Vault esistente non cifrato: prova a sbloccare con password
            // vuota (vault_unlock con stringa vuota su vault non cifrato
            // è il pattern adottato dal legacy Libreria.svelte).
            await invoke("vault_unlock", { password: "" });
            oncompletato();
          }
        } catch (e) {
          errore = String(e);
          stato = "errore";
        }
        return;
      }

      // Vault non esiste su disco: setup richiesto, indipendentemente
      // dal flag onboarding_completato delle preferenze (potrebbe essere
      // un'installazione nuova o un vault eliminato manualmente).
      void prefs; // ack lettura per parallelizzare il fetch
      stato = "setup";
    } catch (e) {
      errore = String(e);
      stato = "errore";
    }
  }

  async function sbloccaVault(): Promise<void> {
    if (!password) {
      errore = "Inserisci la master password.";
      return;
    }
    sbloccoInCorso = true;
    errore = "";
    try {
      await invoke("vault_unlock", { password });
      password = "";
      oncompletato();
    } catch (e) {
      errore = "Password errata.";
      void e;
    } finally {
      sbloccoInCorso = false;
    }
  }

  onMount(() => {
    void rilevaStatoIniziale();
  });
</script>

{#if stato === "caricamento"}
  <main class="root">
    <div class="card center">
      <p class="hint">Caricamento…</p>
    </div>
  </main>
{:else if stato === "setup"}
  <OnboardingWizard {oncompletato} />
{:else if stato === "sblocco"}
  <main class="root">
    <div class="card sblocco">
      <header class="testa">
        <span class="icona-cerchio">
          <Lock size={20} />
        </span>
        <h1>Sblocca il vault</h1>
        <p class="hint">
          Inserisci la master password per accedere ai tuoi prompt.
        </p>
      </header>

      <form
        class="form"
        onsubmit={(e) => {
          e.preventDefault();
          void sbloccaVault();
        }}
      >
        <label for="vault-pwd" class="campo-label">Password</label>
        <input
          id="vault-pwd"
          type="password"
          bind:value={password}
          placeholder="Master password"
          autocomplete="current-password"
          autofocus
          disabled={sbloccoInCorso}
        />

        {#if errore}
          <p class="msg-err">{errore}</p>
        {/if}

        <button
          type="submit"
          class="btn-primary"
          disabled={sbloccoInCorso || !password}
        >
          {sbloccoInCorso ? "Sblocco…" : "Sblocca"}
        </button>
      </form>

      <footer class="footer-hint">
        La password non viene mai trasmessa né recuperabile. Se l'hai persa
        dovrai ripartire da un nuovo vault.
      </footer>
    </div>
  </main>
{:else if stato === "errore"}
  <main class="root">
    <div class="card sblocco">
      <header class="testa">
        <h1>Errore</h1>
        <p class="msg-err">{errore || "Errore sconosciuto."}</p>
      </header>
      <button
        type="button"
        class="btn-primary"
        onclick={() => {
          errore = "";
          stato = "caricamento";
          void rilevaStatoIniziale();
        }}
      >
        Riprova
      </button>
    </div>
  </main>
{/if}

<style>
  .root {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .card {
    width: min(440px, 92vw);
    background: var(--bg-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: var(--sp-5);
    box-shadow: var(--shadow-1);
  }

  .center {
    text-align: center;
  }

  .testa {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: var(--sp-4);
    text-align: center;
  }

  .icona-cerchio {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--bg-input);
    color: var(--text-muted);
  }

  .testa h1 {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .hint {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .campo-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .form input {
    padding: var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-md);
  }

  .form input:focus {
    outline: 2px solid var(--accent-team);
    outline-offset: 0;
  }

  .msg-err {
    margin: 0;
    color: var(--accent-danger, #d9534f);
    font-size: var(--fs-sm);
  }

  .btn-primary {
    padding: var(--sp-2) var(--sp-3);
    background: var(--accent-team);
    color: var(--accent-team-on);
    border: 0;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    cursor: pointer;
    margin-top: var(--sp-2);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .footer-hint {
    margin-top: var(--sp-4);
    padding-top: var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-align: center;
  }
</style>
