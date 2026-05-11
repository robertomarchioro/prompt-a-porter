/**
 * Store reattivo Svelte 5 per preferenze tema/tono (V0.8 F0 Foundation).
 *
 * Wraps i comandi Tauri esistenti `preferenze_carica` / `preferenze_salva`
 * (vedi `apps/client/src-tauri/src/preferenze.rs`). Espone solo i campi
 * `tema` e `tono` reattivamente; gli altri campi della struct backend
 * `Preferenze` sono preservati at save time (read-modify-write).
 *
 * Nota: in Svelte 5, le runes ($state, $effect) sono disponibili solo in
 * file `.svelte` o `.svelte.ts`/`.svelte.js`. Da qui il suffisso `.svelte.ts`.
 *
 * Riferimenti:
 * - Blueprint: docs/roadmap/redesign-v08/blueprint-F0.md §6
 * - Decisione utente: theme="auto" risolto runtime via prefers-color-scheme
 */

import { invoke } from "@tauri-apps/api/core";

const TEMA_DEFAULT = "auto"; // "auto" risolto a "dark"/"light" da risolviTema
const TONO_DEFAULT = "zinc";

/**
 * Subset di `Preferenze` backend (apps/client/src-tauri/src/preferenze.rs).
 * Tutti i campi sono opzionali nel parsing per resilienza forward-compat:
 * se backend aggiunge campi nuovi, ignoriamo silenziosamente.
 */
interface Preferenze {
  profilo: string;
  hotkey: string;
  tema: string;
  tono: string;
  lingua: string;
  onboarding_completato: boolean;
  crea_prompt_esempio: boolean;
  sync_server_url: string;
  sync_email: string;
  sync_token: string;
  sync_intervallo_sec: number;
  sync_abilitato: boolean;
  ricerca_semantica_abilitata: boolean;
  ricerca_alpha: number;
  idle_unload_secondi: number;
  debug_log_abilitato: boolean;
}

class StatoTema {
  tema = $state(TEMA_DEFAULT);
  tono = $state(TONO_DEFAULT);
  caricato = $state(false);
}

export const statoTema = new StatoTema();

/**
 * Carica tema + tono dal backend e popola lo store. Chiamato una volta
 * al boot (App.svelte). Catch-all: se il backend non risponde (es. Tauri
 * non disponibile in test SSR), procedi con i default e segna `caricato`.
 */
export async function caricaTemaTono(): Promise<void> {
  try {
    const prefs = await invoke<Preferenze>("preferenze_carica");
    statoTema.tema = prefs.tema || TEMA_DEFAULT;
    statoTema.tono = prefs.tono || TONO_DEFAULT;
  } catch (err) {
    console.error("[preferenze] carica fallito, uso default", err);
  } finally {
    statoTema.caricato = true;
  }
}

/**
 * Salva tema + tono sul backend preservando gli altri campi.
 * Pattern read-modify-write: rilegge la struct corrente prima di scrivere
 * per non azzerare campi gestiti da altre superfici (sync, onboarding,
 * ricerca semantica, ecc.). Errore loggato ma NON propagato — perdere
 * un save è meglio di rompere il flusso utente.
 */
export async function salvaTemaTono(
  tema: string,
  tono: string,
): Promise<void> {
  try {
    const prefs = await invoke<Preferenze>("preferenze_carica");
    prefs.tema = tema;
    prefs.tono = tono;
    await invoke("preferenze_salva", { preferenze: prefs });
  } catch (err) {
    console.error("[preferenze] salva fallito", err);
  }
}

// Le funzioni pure `risolviTema` + `applicaThemeTone` vivono in
// `./preferenze-helpers.ts` per testabilità diretta via Vitest senza
// plugin svelte-vitest. Re-export per ergonomia importer.
export { risolviTema, applicaThemeTone } from "./preferenze-helpers";
