<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import DemoComponenti from "$lib/superfici/DemoComponenti.svelte";
  import OnboardingWizard from "$lib/superfici/OnboardingWizard.svelte";

  const finestra = getCurrentWindow();
  const etichetta = finestra.label;
  const mostraDemo = new URLSearchParams(window.location.search).has("demo");

  let onboardingCompletato = $state<boolean | null>(null);

  interface Preferenze {
    onboarding_completato: boolean;
  }

  async function caricaPreferenze() {
    try {
      const prefs = await invoke<Preferenze>("preferenze_carica");
      onboardingCompletato = prefs.onboarding_completato;
    } catch {
      onboardingCompletato = false;
    }
  }

  $effect(() => {
    if (etichetta === "libreria" && !mostraDemo) {
      caricaPreferenze();
    }
  });

  $effect(() => {
    if (etichetta === "palette") {
      function nascondiFinestraPalette(e: KeyboardEvent) {
        if (e.key === "Escape") finestra.hide();
      }
      window.addEventListener("keydown", nascondiFinestraPalette);
      return () => window.removeEventListener("keydown", nascondiFinestraPalette);
    }
  });
</script>

{#if mostraDemo}
  <DemoComponenti />
{:else if etichetta === "palette"}
  <main class="palette-root">
    <p class="placeholder">Command Palette — in arrivo (Step 6)</p>
  </main>
{:else if onboardingCompletato === null}
  <main class="libreria-root">
    <div class="benvenuto">
      <p class="hint subtle">Caricamento…</p>
    </div>
  </main>
{:else if !onboardingCompletato}
  <OnboardingWizard oncompletato={() => (onboardingCompletato = true)} />
{:else}
  <main class="libreria-root">
    <div class="benvenuto">
      <h1>Prompt a Porter</h1>
      <p class="sottotitolo">Libreria locale per prompt AI</p>
      <p class="hint">
        Premi <kbd>Ctrl+Shift+P</kbd> per aprire la command palette
      </p>
      <p class="hint subtle">
        Apri <a href="/?demo">?demo</a> per la galleria componenti
      </p>
    </div>
  </main>
{/if}

<style>
  .libreria-root {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .benvenuto {
    text-align: center;
  }

  h1 {
    font-size: var(--fs-3xl);
    font-weight: var(--fw-bold);
    color: var(--text-strong);
    margin: 0 0 var(--sp-2);
    letter-spacing: var(--tracking-tight);
  }

  .sottotitolo {
    font-size: var(--fs-lg);
    color: var(--text-muted);
    margin: 0 0 var(--sp-5);
  }

  .hint {
    font-size: var(--fs-sm);
    color: var(--text-subtle);
    margin: 0 0 var(--sp-2);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    background: var(--bg-overlay);
    border: var(--border-thin) solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: var(--sp-1) var(--sp-2);
  }

  .palette-root {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-raised);
    color: var(--text-muted);
    font-family: var(--font-ui);
  }

  .placeholder {
    font-size: var(--fs-base);
  }
</style>
