/**
 * Coda globale Svelte 5 per le conferme/avvisi in-app di `$lib/util/conferma.ts`.
 *
 * Perché esiste, in due tempi. Nata per macOS: `WKUIDelegate` di wry 0.55.1
 * non implementa `runJavaScriptConfirmPanelWithMessage` né i pannelli
 * alert/prompt, quindi lì `window.confirm` non mostrava alcun dialogo e
 * l'azione veniva eseguita come se l'utente avesse sempre confermato.
 * Estesa poi a **tutte** le piattaforme: `tauri-plugin-dialog` (dalla #570,
 * rilasciata in v0.8.44) sostituisce `window.confirm` con una chiamata a
 * `plugin:dialog|confirm`, comando che la crate non registra — quindi fuori
 * da macOS ogni conferma falliva con «not allowed by ACL». Vedi
 * `$lib/util/conferma.ts` per i riferimenti al sorgente della crate.
 * La coda è risolta da `DialogoHost.svelte`, che riusa i componenti
 * primitivi `Modale`/`Toast` già esistenti.
 *
 * Una coda (non un solo slot) perché nulla vieta a due chiamate di
 * accodarsi quasi in contemporanea (es. due `catch` che scattano in rapida
 * successione): l'host mostra sempre e solo `coda[0]`, le altre attendono.
 */

export interface RichiestaConferma {
  tipo: "conferma";
  id: number;
  messaggio: string;
  risolvi: (esito: boolean) => void;
}

export interface RichiestaAvviso {
  tipo: "avviso";
  id: number;
  messaggio: string;
  risolvi: () => void;
}

export type RichiestaDialogo = RichiestaConferma | RichiestaAvviso;

class StatoDialoghi {
  coda = $state<RichiestaDialogo[]>([]);
}

export const statoDialoghi = new StatoDialoghi();

let prossimoId = 0;

/** Accoda una richiesta di conferma; la Promise si risolve quando l'host chiama `risolvi`. */
export function accodaConferma(messaggio: string): Promise<boolean> {
  return new Promise<boolean>((risolvi) => {
    statoDialoghi.coda = [
      ...statoDialoghi.coda,
      { tipo: "conferma", id: prossimoId++, messaggio, risolvi },
    ];
  });
}

/** Accoda una richiesta di avviso; la Promise si risolve quando l'host chiama `risolvi`. */
export function accodaAvviso(messaggio: string): Promise<void> {
  return new Promise<void>((risolvi) => {
    statoDialoghi.coda = [
      ...statoDialoghi.coda,
      { tipo: "avviso", id: prossimoId++, messaggio, risolvi },
    ];
  });
}

/** Rimuove dalla coda la richiesta con l'id indicato (già risolta dall'host). */
export function rimuoviDallaCoda(id: number): void {
  statoDialoghi.coda = statoDialoghi.coda.filter((r) => r.id !== id);
}
