<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import CommandPalette from "$lib/superfici/CommandPalette.svelte";
  import DemoComponenti from "$lib/superfici/DemoComponenti.svelte";
  import Libreria from "$lib/superfici/Libreria.svelte";
  import OnboardingWizard from "$lib/superfici/OnboardingWizard.svelte";
  import {
    statoTema,
    caricaTemaTono,
    applicaThemeTone,
  } from "$lib/stores/preferenze.svelte";

  const finestra = getCurrentWindow();
  const etichetta = finestra.label;
  const mostraDemo = new URLSearchParams(window.location.search).has("demo");

  let onboardingCompletato = $state<boolean | null>(null);

  interface Preferenze {
    onboarding_completato: boolean;
  }

  async function caricaPreferenze() {
    try {
      const [prefs, vaultEsiste] = await Promise.all([
        invoke<Preferenze>("preferenze_carica"),
        invoke<boolean>("vault_esiste"),
      ]);
      // Se il vault esiste su disco, l'onboarding è già stato eseguito
      // almeno una volta. vault_esiste serve da fallback robusto per i
      // casi in cui preferenze.json non si è persistito (es. EDR che
      // blocca la scrittura selettivamente). Vedi issue #4.
      onboardingCompletato = prefs.onboarding_completato || vaultEsiste;
    } catch {
      onboardingCompletato = false;
    }
  }

  $effect(() => {
    if (etichetta === "libreria" && !mostraDemo) {
      caricaPreferenze();
    }
  });

  // F0 PR-B: cascade reattiva tema/tono su <html> per tutte le finestre
  // (libreria, palette, demo). Carica una volta dal backend al mount,
  // applica ad ogni cambio nello store, e ascolta prefers-color-scheme
  // quando tema === "auto" per adeguarsi al sistema in tempo reale.
  onMount(() => {
    void caricaTemaTono();

    let mq: MediaQueryList | null = null;
    let onSystemChange: ((e: MediaQueryListEvent) => void) | null = null;
    if (typeof window !== "undefined" && window.matchMedia) {
      mq = window.matchMedia("(prefers-color-scheme: dark)");
      onSystemChange = () => {
        if (statoTema.tema === "auto") {
          applicaThemeTone(statoTema.tema, statoTema.tono);
        }
      };
      mq.addEventListener("change", onSystemChange);
    }
    return () => {
      if (mq && onSystemChange) {
        mq.removeEventListener("change", onSystemChange);
      }
    };
  });

  $effect(() => {
    if (statoTema.caricato) {
      applicaThemeTone(statoTema.tema, statoTema.tono);
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
