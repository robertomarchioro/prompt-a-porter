// Fix #443: dopo `update.downloadAndInstall()` l'updater Tauri v2 riavvia
// da solo l'app SOLO su Windows (l'installer NSIS chiude il processo
// corrente e ne avvia uno nuovo). Su Linux (AppImage/.deb) e macOS il
// binario viene sostituito in-place ma il processo corrente resta vivo
// con il vecchio codice caricato in memoria: serve chiamare esplicitamente
// `relaunch()` da `@tauri-apps/plugin-process`.
//
// Logica estratta in un modulo puro (niente Svelte/Tauri runtime diretto)
// per essere testabile senza dover montare l'intero ImpostazioniModal.

export type RisultatoRelaunch =
  | { kind: "relaunched" }
  | { kind: "riavvio_manuale_richiesto"; messaggio: string };

const MESSAGGIO_RIAVVIO_MANUALE =
  "Aggiornamento installato. Riavvia manualmente Prompt a Porter per applicarlo.";

/**
 * Esegue `relaunch()` dopo un `downloadAndInstall()` riuscito.
 *
 * Su `.deb` un self-relaunch del binario appena sostituito in-place può
 * fallire (permessi, exec del file appena scritto, ecc.). In quel caso
 * NON propaghiamo l'errore verso l'alto: l'update è comunque installato,
 * chiediamo solo all'utente di riavviare manualmente invece di far
 * crashare l'app o mostrare un errore fuorviante.
 */
export async function eseguiRelaunchPostInstall(
  relaunch: () => Promise<void>,
): Promise<RisultatoRelaunch> {
  try {
    await relaunch();
    // Se relaunch() ha successo, sulla maggior parte delle piattaforme il
    // processo termina prima che questo return venga osservato dal chiamante.
    return { kind: "relaunched" };
  } catch {
    return {
      kind: "riavvio_manuale_richiesto",
      messaggio: MESSAGGIO_RIAVVIO_MANUALE,
    };
  }
}
