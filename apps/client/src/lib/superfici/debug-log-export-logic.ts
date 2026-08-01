/**
 * Logica pura per l'export ZIP dei debug log (issue #558 punto 2).
 *
 * Estratta da ImpostazioniModal.svelte per consentire test senza runtime
 * Svelte (stesso pattern `_logic.ts` di nuova-cartella-logic.ts).
 *
 * Fix HIGH (review avversariale pre-merge PR #570): il dialog di
 * salvataggio nativo ora si apre **dentro** il comando Rust
 * (`debug_log.rs::debug_log_esporta_zip`, `tauri-plugin-dialog`
 * `blocking_save_file`) — il percorso di destinazione non attraversa mai
 * più il boundary IPC come argomento del comando. Il frontend invoca il
 * comando senza parametri e distingue tre esiti:
 * - stringa: export riuscito, path del file creato;
 * - `null`: l'utente ha annullato il dialog — non è un errore;
 * - eccezione: l'export è fallito per un motivo effettivo.
 */

export interface RisultatoEsportaLogZip {
  ok: boolean;
  /** Path assoluto del file ZIP creato, presente solo se ok === true. */
  zipPath?: string;
  /** true se l'utente ha annullato il dialog di salvataggio nativo. */
  annullato?: boolean;
  /** Messaggio d'errore user-facing, presente solo per fallimenti reali. */
  errore?: string;
}

/**
 * Invoca `debug_log_esporta_zip` (nessun argomento: il percorso di
 * destinazione viene scelto dall'utente lato Rust, non passato dal
 * frontend) e normalizza i tre esiti possibili in `RisultatoEsportaLogZip`.
 */
export async function eseguiEsportaLogZip(
  invokeFn: (cmd: "debug_log_esporta_zip") => Promise<string | null>,
): Promise<RisultatoEsportaLogZip> {
  try {
    const zipPath = await invokeFn("debug_log_esporta_zip");
    if (zipPath === null) {
      return { ok: false, annullato: true };
    }
    return { ok: true, zipPath };
  } catch (err: unknown) {
    const msg =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err.replace(/^Error:\s*/i, "")
          : "Export ZIP fallito.";
    return { ok: false, errore: msg };
  }
}
