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
  updater_abilitato: boolean;
  // M10 — preferenze editor (vedi preferenze.rs)
  autosave_delay_ms: number;
  line_wrapping: boolean;
  indent_size: number;
  font_size: number;
  show_line_numbers: boolean;
  highlight_active_line: boolean;
}

// M10 — Default editor (allineati ai default Rust in preferenze.rs)
export const AUTOSAVE_DELAY_DEFAULT = 2000;
export const LINE_WRAPPING_DEFAULT = true;
export const INDENT_SIZE_DEFAULT = 2;
export const FONT_SIZE_DEFAULT = 13;
export const SHOW_LINE_NUMBERS_DEFAULT = true;
export const HIGHLIGHT_ACTIVE_LINE_DEFAULT = false;

// M10 — Range UI clamp
export const AUTOSAVE_DELAY_MIN = 500;
export const AUTOSAVE_DELAY_MAX = 5000;
export const FONT_SIZE_MIN = 12;
export const FONT_SIZE_MAX = 20;

class StatoTema {
  tema = $state(TEMA_DEFAULT);
  tono = $state(TONO_DEFAULT);
  caricato = $state(false);
}

export const statoTema = new StatoTema();

/**
 * M10 — Store reattivo per le 6 preferenze editor configurabili.
 * Caricato al boot (App.svelte) come `statoTema`. Modifiche dall'UI
 * Impostazioni passano per `salvaEditor()` con read-modify-write.
 */
class StatoEditor {
  autosaveDelayMs = $state(AUTOSAVE_DELAY_DEFAULT);
  lineWrapping = $state(LINE_WRAPPING_DEFAULT);
  indentSize = $state(INDENT_SIZE_DEFAULT);
  fontSize = $state(FONT_SIZE_DEFAULT);
  showLineNumbers = $state(SHOW_LINE_NUMBERS_DEFAULT);
  highlightActiveLine = $state(HIGHLIGHT_ACTIVE_LINE_DEFAULT);
  caricato = $state(false);
}

export const statoEditor = new StatoEditor();

function clampInt(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(v)));
}

/**
 * Carica le preferenze editor dal backend e popola lo store. Idempotente.
 */
export async function caricaEditor(): Promise<void> {
  try {
    const prefs = await invoke<Preferenze>("preferenze_carica");
    statoEditor.autosaveDelayMs = clampInt(
      prefs.autosave_delay_ms ?? AUTOSAVE_DELAY_DEFAULT,
      AUTOSAVE_DELAY_MIN,
      AUTOSAVE_DELAY_MAX,
    );
    statoEditor.lineWrapping = prefs.line_wrapping ?? LINE_WRAPPING_DEFAULT;
    statoEditor.indentSize = prefs.indent_size === 4 ? 4 : INDENT_SIZE_DEFAULT;
    statoEditor.fontSize = clampInt(
      prefs.font_size ?? FONT_SIZE_DEFAULT,
      FONT_SIZE_MIN,
      FONT_SIZE_MAX,
    );
    statoEditor.showLineNumbers =
      prefs.show_line_numbers ?? SHOW_LINE_NUMBERS_DEFAULT;
    statoEditor.highlightActiveLine =
      prefs.highlight_active_line ?? HIGHLIGHT_ACTIVE_LINE_DEFAULT;
  } catch (err) {
    console.error("[preferenze] carica editor fallito, uso default", err);
  } finally {
    statoEditor.caricato = true;
  }
}

interface EditorPrefsPatch {
  autosaveDelayMs?: number;
  lineWrapping?: boolean;
  indentSize?: number;
  fontSize?: number;
  showLineNumbers?: boolean;
  highlightActiveLine?: boolean;
}

/**
 * Salva i campi editor sul backend con read-modify-write (preserva gli
 * altri campi di `Preferenze` gestiti da altre superfici). Aggiorna
 * anche lo store in-memory cosi' i componenti reagiscono subito.
 */
export async function salvaEditor(patch: EditorPrefsPatch): Promise<void> {
  try {
    const prefs = await invoke<Preferenze>("preferenze_carica");
    if (patch.autosaveDelayMs !== undefined) {
      const v = clampInt(
        patch.autosaveDelayMs,
        AUTOSAVE_DELAY_MIN,
        AUTOSAVE_DELAY_MAX,
      );
      prefs.autosave_delay_ms = v;
      statoEditor.autosaveDelayMs = v;
    }
    if (patch.lineWrapping !== undefined) {
      prefs.line_wrapping = patch.lineWrapping;
      statoEditor.lineWrapping = patch.lineWrapping;
    }
    if (patch.indentSize !== undefined) {
      const v = patch.indentSize === 4 ? 4 : 2;
      prefs.indent_size = v;
      statoEditor.indentSize = v;
    }
    if (patch.fontSize !== undefined) {
      const v = clampInt(patch.fontSize, FONT_SIZE_MIN, FONT_SIZE_MAX);
      prefs.font_size = v;
      statoEditor.fontSize = v;
    }
    if (patch.showLineNumbers !== undefined) {
      prefs.show_line_numbers = patch.showLineNumbers;
      statoEditor.showLineNumbers = patch.showLineNumbers;
    }
    if (patch.highlightActiveLine !== undefined) {
      prefs.highlight_active_line = patch.highlightActiveLine;
      statoEditor.highlightActiveLine = patch.highlightActiveLine;
    }
    await invoke("preferenze_salva", { preferenze: prefs });
  } catch (err) {
    console.error("[preferenze] salva editor fallito", err);
  }
}

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
