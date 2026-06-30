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
  import { invoke } from "@tauri-apps/api/core";
  import CommandPalette from "$lib/superfici/CommandPalette.svelte";
  import Onboarding from "$lib/superfici/Onboarding.svelte";
  import Shell from "$lib/superfici/Shell.svelte";
  import {
    statoTema,
    caricaTemaTono,
    caricaEditor,
    caricaVault,
    applicaThemeTone,
  } from "$lib/stores/preferenze.svelte";

  const finestra = getCurrentWindow();
  const etichetta = finestra.label;

  let authCompleted = $state(false);

  // Issue #268: al primissimo avvio (subito dopo l'installazione) il
  // webview può chiamare i comandi vault PRIMA che il backend abbia
  // completato `app.manage(VaultState)` nel hook `.setup()`. In quella
  // finestra l'invoke viene rifiutato e Onboarding mostrava un dialog
  // "Errore / Riprova" spurio (premendo Riprova proseguiva perché nel
  // frattempo lo state era pronto). Qui facciamo un probe idempotente di
  // `vault_aperto` (comando infallibile by-design: ritorna sempre bool)
  // con qualche retry prima di montare Onboarding, così l'errore non
  // appare mai su un primo avvio legittimo.
  let backendPronto = $state(false);

  const PROBE_MAX_TENTATIVI = 20;
  const PROBE_DELAY_MS = 100;

  async function attendiBackendPronto(): Promise<void> {
    for (let i = 0; i < PROBE_MAX_TENTATIVI; i++) {
      try {
        await invoke<boolean>("vault_aperto");
        backendPronto = true;
        return;
      } catch {
        // State non ancora gestito / comando non ancora registrato:
        // attendi e riprova.
        await new Promise((r) => setTimeout(r, PROBE_DELAY_MS));
      }
    }
    // Esauriti i tentativi: monta comunque Onboarding, che gestisce
    // internamente eventuali errori reali (file rotto, ecc.).
    backendPronto = true;
  }

  // Issue #273/#274: quando il vault viene bloccato o eliminato dalle
  // Impostazioni, l'app deve tornare allo stato pre-Shell. ImpostazioniModal
  // emette `pap:vault-bloccato`; qui resettiamo authCompleted così
  // Onboarding viene rimontato e rileva il nuovo stato del vault
  // (cifrato esistente → schermata di sblocco; inesistente → setup).
  function onVaultBloccato(): void {
    authCompleted = false;
  }

  // F0 PR-B: cascade reattiva tema/tono su <html> per tutte le finestre.
  // Carica una volta dal backend al mount, applica ad ogni cambio nello
  // store, e ascolta prefers-color-scheme quando tema === "auto" per
  // adeguarsi al sistema in tempo reale.
  onMount(() => {
    void caricaTemaTono();
    void caricaEditor();
    void caricaVault();
    // La palette è una finestra separata che non passa dal flusso vault.
    if (etichetta !== "palette") {
      void attendiBackendPronto();
    }
    window.addEventListener("pap:vault-bloccato", onVaultBloccato);

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
      window.removeEventListener("pap:vault-bloccato", onVaultBloccato);
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
{:else if !backendPronto}
  <!-- Issue #268: attesa breve che il backend sia pronto, evita il
       dialog d'errore spurio al primo avvio. -->
  <main class="boot-attesa">
    <p>Avvio…</p>
  </main>
{:else if !authCompleted}
  <Onboarding
    oncompletato={() => {
      authCompleted = true;
      // #404: l'onboarding ha appena salvato `nome_vault`; ricarica lo
      // store così lo switcher mostra il nome scelto e non il default
      // "Personale" letto al boot (prima che il file esistesse).
      void caricaVault();
    }}
  />
{:else}
  <Shell />
{/if}

<style>
  .boot-attesa {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-muted);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }
</style>
