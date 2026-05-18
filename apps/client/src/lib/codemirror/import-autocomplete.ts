/**
 * M4 PR-3 — Intellisense autocomplete per `{{import "..."}}`.
 *
 * Rileva il pattern `{{import "<prefix>` immediatamente prima del
 * cursore e suggerisce i prompt il cui titolo matcha il prefisso,
 * filtrati dal backend (`prompt_suggest_intellisense`).
 *
 * Esempio UX: l'utente digita `{{import "mark` -> dropdown CodeMirror
 * mostra "Marketing email", "marketing social", ecc. con folder path
 * come `detail`. Enter conferma: il prefisso viene esteso al titolo
 * completo (l'utente chiude le quote/braces manualmente).
 *
 * NOTA: usiamo il TITOLO del prompt come path d'import (non l'id),
 * coerente con `resolve_path` backend che cerca prima per
 * `Folders.Path + Title` poi per `Title` su root (vedi
 * prompt_componibili.rs:resolve_path).
 */

import {
  autocompletion,
  type CompletionContext,
  type CompletionResult,
  type Completion,
} from "@codemirror/autocomplete";
import type { Extension } from "@codemirror/state";
import { invoke } from "@tauri-apps/api/core";

interface PromptSuggerito {
  id: string;
  titolo: string;
  folder_path: string;
}

interface ImportAutocompleteOpts {
  /**
   * Callback chiamata al momento dell'invocazione per ottenere l'id
   * del prompt corrente nell'editor. Serve per escluderlo dai
   * suggerimenti (no self-import). Ritorna `null` se non disponibile.
   */
  getPromptId: () => string | null;
}

/**
 * Regex per detect del contesto `{{import "<prefix>` PRIMA del cursore.
 * Cattura il prefisso (eventualmente vuoto se cursore subito dopo `"`).
 * Non matcha se la quote è già stata chiusa (`"path"`) — perchè
 * l'utente ha già un valore valido.
 */
const RE_IMPORT_INCOMPLETO = /\{\{\s*import\s+"([^"]*)$/;

/**
 * Validator per `validFor`: caratteri tipici di un titolo prompt
 * (alfanumerici, spazi, trattini, slash per path, accenti italiani).
 * Quando l'utente continua a digitare e i nuovi char rientrano in
 * questa regex, CodeMirror riusa la stessa lista di completion
 * senza re-fetch — perf+UX.
 */
const RE_VALID_FOR = /^[\w\s\-/àèéìòùÀÈÉÌÒÙ]*$/i;

const FETCH_LIMIT = 20;
const LOOKBACK_BYTES = 200;

export function importAutocompletion(opts: ImportAutocompleteOpts): Extension {
  return autocompletion({
    override: [
      async (ctx: CompletionContext): Promise<CompletionResult | null> => {
        const start = Math.max(0, ctx.pos - LOOKBACK_BYTES);
        const before = ctx.state.sliceDoc(start, ctx.pos);
        const match = RE_IMPORT_INCOMPLETO.exec(before);
        if (!match) return null;

        const prefix = match[1];
        // Senza prefisso, attiva solo se utente preme Ctrl+Space (explicit)
        // per evitare popover invasivo subito dopo `"`.
        if (prefix.length === 0 && !ctx.explicit) return null;

        const fromPos = ctx.pos - prefix.length;
        const escludiId = opts.getPromptId();

        try {
          const sugg = await invoke<PromptSuggerito[]>(
            "prompt_suggest_intellisense",
            {
              prefix,
              limit: FETCH_LIMIT,
              escludiId,
            },
          );
          if (sugg.length === 0) return null;
          const options: Completion[] = sugg.map((s) => ({
            label: s.titolo,
            detail: s.folder_path || "(root)",
            apply: s.titolo,
          }));
          return {
            from: fromPos,
            options,
            validFor: RE_VALID_FOR,
          };
        } catch (err) {
          // Vault locked / backend non disponibile: silente, no popover
          // (la console mostra l'errore per debug).
          console.error("[import-autocomplete] suggest fallito", err);
          return null;
        }
      },
    ],
    activateOnTyping: true,
    closeOnBlur: true,
    defaultKeymap: true,
  });
}
