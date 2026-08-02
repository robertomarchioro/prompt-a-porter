/**
 * Logica pura per l'installazione di un aggiornamento (Impostazioni →
 * Sviluppo → Aggiornamenti).
 *
 * Estratta da ImpostazioniModal.svelte per essere testabile senza runtime
 * Svelte (stesso pattern `_logic.ts` di debug-log-export-logic.ts), e per
 * rendere diagnosticabile un caso osservato dal vivo: l'utente ha premuto
 * «Installa e riavvia» su macOS con un aggiornamento disponibile e nel file
 * di log ci sono stati trenta secondi senza una sola riga, poi l'app si è
 * riavviata restando alla versione precedente. Nessun percorso di questo
 * codice registrava alcunché, quindi non è stato possibile stabilire la
 * causa: questa funzione logga ogni passo (ingresso, esito della conferma,
 * prima/dopo `check()`, prima/dopo `downloadAndInstall()`, prima di
 * `relaunch()`, ed eventuali eccezioni) così un prossimo tentativo sia
 * diagnosticabile. Non chiude la causa del mancato aggiornamento, che
 * resta ignota.
 *
 * Nota sul `catch`: nella versione precedente la conferma (`window.confirm`)
 * era chiamata FUORI dal try — se avesse sollevato un'eccezione la Promise
 * sarebbe stata rifiutata senza che nulla la catturasse (nessun messaggio,
 * nessun log, app invariata: esattamente il sintomo osservato). Qui
 * l'intera sequenza, conferma inclusa, è dentro un solo try/catch.
 */

import { eseguiRelaunchPostInstall } from "../updater-relaunch";

/** Sottoinsieme di `Update` (tauri-plugin-updater) usato da questa funzione. */
export interface AggiornamentoDisponibile {
  downloadAndInstall: () => Promise<void>;
}

export interface DipendenzeInstallaAggiornamento {
  /** Tipicamente `conferma()` di `$lib/util/conferma.ts`. */
  conferma: (messaggio: string) => Promise<boolean>;
  /** Tipicamente `check` di `@tauri-apps/plugin-updater`. */
  check: () => Promise<AggiornamentoDisponibile | null>;
  /** Tipicamente `relaunch` di `@tauri-apps/plugin-process`. */
  relaunch: () => Promise<void>;
  logInfo: (messaggio: string) => void;
  logErrore: (messaggio: string) => void;
  /** Chiamato subito dopo una conferma accettata, prima di check()/download. */
  onConfermato?: () => void;
}

export type RisultatoInstallaAggiornamento =
  | { kind: "annullato" }
  | { kind: "non_disponibile" }
  | { kind: "riavviato" }
  | { kind: "riavvio_manuale"; messaggio: string }
  | { kind: "errore"; messaggio: string };

export async function eseguiInstallaAggiornamento(
  versione: string,
  deps: DipendenzeInstallaAggiornamento,
): Promise<RisultatoInstallaAggiornamento> {
  deps.logInfo(
    `[updater] installaAggiornamento: ingresso (versione ${versione})`,
  );
  try {
    const ok = await deps.conferma(
      `Installare la versione ${versione}?\n\nL'app verrà chiusa e riavviata. I tuoi dati restano intatti.`,
    );
    deps.logInfo(
      `[updater] conferma utente: ${ok ? "accettata" : "annullata"}`,
    );
    if (!ok) return { kind: "annullato" };
    deps.onConfermato?.();

    deps.logInfo("[updater] check() pre-installazione: avvio");
    const update = await deps.check();
    deps.logInfo(
      `[updater] check() pre-installazione: ${update ? "aggiornamento confermato disponibile" : "non più disponibile"}`,
    );
    if (!update) {
      return { kind: "non_disponibile" };
    }

    deps.logInfo("[updater] downloadAndInstall(): avvio");
    await update.downloadAndInstall();
    deps.logInfo("[updater] downloadAndInstall(): completato");

    deps.logInfo("[updater] relaunch(): avvio");
    const risultatoRelaunch = await eseguiRelaunchPostInstall(deps.relaunch);
    if (risultatoRelaunch.kind === "riavvio_manuale_richiesto") {
      deps.logErrore(
        `[updater] relaunch(): fallito, riavvio manuale richiesto — ${risultatoRelaunch.messaggio}`,
      );
      return {
        kind: "riavvio_manuale",
        messaggio: risultatoRelaunch.messaggio,
      };
    }
    // Sulla maggior parte delle piattaforme il processo termina dentro
    // relaunch() prima che questo log venga scritto/osservato.
    deps.logInfo("[updater] relaunch(): completato");
    return { kind: "riavviato" };
  } catch (e) {
    const messaggio = String(e);
    deps.logErrore(
      `[updater] installaAggiornamento: eccezione non gestita — ${messaggio}`,
    );
    return { kind: "errore", messaggio };
  }
}
