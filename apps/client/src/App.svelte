<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import CommandPalette from "$lib/superfici/CommandPalette.svelte";
  import DemoComponenti from "$lib/superfici/DemoComponenti.svelte";
  import Libreria from "$lib/superfici/Libreria.svelte";
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

</script>

{#if mostraDemo}
  <DemoComponenti />
{:else if etichetta === "palette"}
  <CommandPalette />
{:else if onboardingCompletato === null}
  <main class="libreria-root">
    <div class="benvenuto">
      <p class="hint subtle">Caricamento…</p>
    </div>
  </main>
{:else if !onboardingCompletato}
  <OnboardingWizard oncompletato={() => (onboardingCompletato = true)} />
{:else}
  <Libreria />
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

  .hint {
    font-size: var(--fs-sm);
    color: var(--text-subtle);
    margin: 0 0 var(--sp-2);
  }
</style>
