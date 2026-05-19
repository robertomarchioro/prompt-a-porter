<script lang="ts">
  /**
   * App root del redesign v0.8 (F9 routing/cleanup).
   *
   * Routing semplificato a 3 stati:
   * - window label "palette" → CommandPalette (window legacy, rimossa
   *   solo in F11; PaletteModal F8 PR-E è la versione interna preferita)
   * - !authCompleted → Onboarding (gestisce internamente caricamento /
   *   setup wizard / sblocco vault cifrato)
   * - authCompleted → Shell (la nuova UI redesign)
   *
   * Non più gestiti qui (PR-B):
   * - flag `?demo` (DemoComponenti.svelte cancellato)
   * - flag `?redesign-shell` (Shell è il default, non più opt-in)
   * - <Libreria /> come default (sostituito da Shell)
   *
   * Riferimenti:
   * - Blueprint F9: docs/roadmap/redesign-v08/blueprint-F9.md §2
   */
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import CommandPalette from "$lib/superfici/CommandPalette.svelte";
  import Onboarding from "$lib/superfici/Onboarding.svelte";
  import Shell from "$lib/superfici/Shell.svelte";
  import {
    statoTema,
    caricaTemaTono,
    caricaEditor,
    applicaThemeTone,
  } from "$lib/stores/preferenze.svelte";

  const finestra = getCurrentWindow();
  const etichetta = finestra.label;

  let authCompleted = $state(false);

  // F0 PR-B: cascade reattiva tema/tono su <html> per tutte le finestre.
  // Carica una volta dal backend al mount, applica ad ogni cambio nello
  // store, e ascolta prefers-color-scheme quando tema === "auto" per
  // adeguarsi al sistema in tempo reale.
  onMount(() => {
    void caricaTemaTono();
    void caricaEditor();

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

{#if etichetta === "palette"}
  <CommandPalette />
{:else if !authCompleted}
  <Onboarding oncompletato={() => (authCompleted = true)} />
{:else}
  <Shell />
{/if}
