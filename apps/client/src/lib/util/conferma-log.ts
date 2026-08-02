/**
 * Formattazione pura delle righe di diagnostica per `conferma()`/`avvisa()`
 * (`$lib/util/conferma.ts`) — strumentazione di sola diagnosi per #585
 * (cancellazione prompt non fa nulla su Windows), #572 (promozione variante
 * non fa nulla) e #584 («Pulisci log» non pulisce, Windows).
 *
 * VINCOLO DI PRIVACY (non negoziabile): il messaggio passato a
 * `conferma()`/`avvisa()` contiene spesso il titolo di un prompt (es.
 * `Eliminare il prompt "Preventivo Rossi Spa"?`). Queste funzioni non
 * ricevono mai il testo del messaggio, solo la sua lunghezza — per
 * costruzione non possono farlo trapelare nella riga di log. Vedi
 * `conferma-log.test.ts`: un test dimostra esplicitamente che un titolo
 * riconoscibile passato a `conferma()` non compare nella riga prodotta.
 *
 * Perché la durata resta utile: misura quanto passa fra la richiesta e la
 * risposta dell'utente. Una durata di pochi millisecondi indica che nessuno
 * ha risposto davvero, cioè che il dialogo non è stato mostrato o è stato
 * risolto da codice — utile a distinguere «l'utente ha annullato» da «la
 * conferma non è mai arrivata all'utente».
 *
 * Il ramo `"nativo"` (`window.confirm`/`window.alert`) NON è più usato: le
 * funzioni globali della webview sono sostituite da `tauri-plugin-dialog`
 * con chiamate a un comando inesistente, quindi sollevavano sempre
 * un'eccezione fuori da macOS (vedi il commento di `conferma.ts` per i
 * riferimenti al sorgente della crate). Il valore resta nel tipo perché
 * compare nelle righe di log raccolte **prima** della correzione, e chi le
 * rilegge deve poterle interpretare.
 */

export type RamoConferma = "macos" | "nativo" | "in-app";

/**
 * Sotto questa soglia, su ramo "nativo", `window.confirm` non può aver
 * mostrato davvero un dialogo bloccante in attesa di risposta dell'utente:
 * è tornato subito. Valore scelto ben al di sotto delle decine di
 * millisecondi minime di un'interazione umana reale, per non generare falsi
 * positivi su macchine lente.
 */
export const SOGLIA_DIALOGO_NON_MOSTRATO_MS = 5;

export interface MisuraConferma {
  /** Identificativo dell'azione (es. "elimina-prompt"), MAI testo utente. */
  azione: string;
  ramo: RamoConferma;
  durataMs: number;
  /** Lunghezza del messaggio, MAI il messaggio stesso. */
  lunghezzaMessaggio: number;
  /** `typeof` del valore restituito: distingue `false` da `undefined`. */
  tipoEsito: string;
  valoreEsito: unknown;
  platform: string;
  piattaformaClassificata: string;
}

/** true se, sul ramo nativo, il dialogo con ogni probabilità non è apparso. */
export function dialogoProbabilmenteNonMostrato(
  ramo: RamoConferma,
  durataMs: number,
): boolean {
  return ramo === "nativo" && durataMs < SOGLIA_DIALOGO_NON_MOSTRATO_MS;
}

/** Riga di log per `conferma()`. Non include mai testo proveniente dall'utente. */
export function formattaRigaConferma(m: MisuraConferma): string {
  const sospetto = dialogoProbabilmenteNonMostrato(m.ramo, m.durataMs)
    ? " sospetto=dialogo_non_mostrato"
    : "";
  return (
    `[conferma] azione=${m.azione} ramo=${m.ramo} durataMs=${m.durataMs.toFixed(1)} ` +
    `lunghezzaMessaggio=${m.lunghezzaMessaggio} esitoTipo=${m.tipoEsito} ` +
    `esitoValore=${String(m.valoreEsito)} platform="${m.platform}" ` +
    `piattaformaClassificata=${m.piattaformaClassificata}${sospetto}`
  );
}

export interface MisuraAvviso {
  azione: string;
  ramo: RamoConferma;
  /** Lunghezza del messaggio, MAI il messaggio stesso. */
  lunghezzaMessaggio: number;
}

/** Riga di log per `avvisa()`. Non include mai testo proveniente dall'utente. */
export function formattaRigaAvviso(m: MisuraAvviso): string {
  return `[avvisa] azione=${m.azione} ramo=${m.ramo} lunghezzaMessaggio=${m.lunghezzaMessaggio}`;
}
