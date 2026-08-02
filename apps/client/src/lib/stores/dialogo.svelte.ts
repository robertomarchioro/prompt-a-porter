/**
 * Coda globale Svelte 5 per le conferme/avvisi in-app di `$lib/util/conferma.ts`.
 *
 * Perché esiste: su macOS `WKUIDelegate` di wry 0.55.1 non implementa
 * `runJavaScriptConfirmPanelWithMessage` né i pannelli alert/prompt (vedi
 * `src/wkwebview/class/wry_web_view_ui_delegate.rs`), quindi `window.confirm`
 * e `window.alert` non mostrano alcun dialogo — l'azione viene eseguita
 * come se l'utente avesse sempre confermato. Su macOS sostituiamo quei due
 * dialoghi nativi con una coda di richieste risolta da `DialogoHost.svelte`,
 * che riusa i componenti primitivi `Modale`/`Toast` già esistenti.
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
