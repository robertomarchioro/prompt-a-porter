// Registro dei modelli AI: **unica fonte** per ogni elenco di modelli
// dell'app (tendine di Ritocco e Test Golden, segnaposto del pannello
// provider). I dati stanno in `modelli-registro.json` e sono mantenuti a
// livello di sorgente: li aggiorna il workflow settimanale
// `modelli-refresh.yml` via `scripts/aggiorna-modelli.ts`, e la modifica
// entra nella release successiva di PAP.
//
// `ollama` e `openai-compat` restano volutamente fuori: non hanno un listino
// pubblico da interrogare (dipendono dall'installazione o dall'endpoint
// dell'utente) e continuano a usare un campo di testo libero.
//
// NON confondere con `modelli-target.ts`, che è un vocabolario di etichette
// per dire "questo prompt è pensato per Claude Sonnet": è volutamente grosso,
// stabile e scollegato dagli id reali delle API.

import registroJson from "./modelli-registro.json";

/** Provider con listino noto (→ tendina invece che campo libero). */
export type ProviderConElenco = "anthropic" | "openai" | "gemini";

export interface ModelloAI {
  /** Id reale passato all'API del provider. */
  id: string;
  provider: ProviderConElenco;
  /** Nome leggibile dal listino (es. "Claude Opus 4.8"). */
  etichetta: string;
  /** Sparito dal listino: resta selezionabile, ma segnalato e in fondo. */
  obsoleto: boolean;
  anteprima: boolean;
  /** Ultima data in cui il listino lo riportava (ISO, solo giorno). */
  visto_il: string;
  contesto: number | null;
  prezzo: { input: number; output: number } | null;
}

export interface RegistroModelli {
  aggiornato_a: string;
  fonte: string;
  modelli: ModelloAI[];
}

export const REGISTRO: RegistroModelli = registroJson as RegistroModelli;

/** Opzione pronta per un `<option>`: valore reale + etichetta da mostrare. */
export interface OpzioneModello {
  value: string;
  etichetta: string;
}

/**
 * Modelli di un provider: attivi prima, obsoleti in fondo; a parità di
 * stato l'ordine è quello del registro (alfabetico per id).
 */
export function modelliDelProvider(provider: string): ModelloAI[] {
  return REGISTRO.modelli
    .filter((m) => m.provider === provider)
    .sort((a, b) => Number(a.obsoleto) - Number(b.obsoleto));
}

/** Solo gli id, nell'ordine di `modelliDelProvider`. */
export function modelliNoti(provider: string): string[] {
  return modelliDelProvider(provider).map((m) => m.id);
}

/** True per i provider con listino noto (→ tendina). */
export function providerHaModelliNoti(provider: string): boolean {
  return modelliDelProvider(provider).length > 0;
}

/**
 * Modello da proporre come segnaposto nel pannello provider: il primo
 * attivo e non in anteprima. Sostituisce i valori che erano ricopiati a mano
 * in `PannelloProviderConfig.svelte`.
 */
export function modelloPredefinito(provider: string): string {
  const m = modelliDelProvider(provider).find((x) => !x.obsoleto && !x.anteprima);
  return m?.id ?? "";
}

export function etichettaModello(m: ModelloAI): string {
  const note = [m.anteprima ? "anteprima" : null, m.obsoleto ? "obsoleto" : null].filter(
    Boolean,
  );
  return note.length > 0 ? `${m.etichetta} (${note.join(", ")})` : m.etichetta;
}

/**
 * Opzioni per il selettore. Se il valore corrente non è in registro (modello
 * scritto a mano, o rimosso da una versione futura) viene aggiunto in coda
 * così non sparisce dalla tendina sotto gli occhi dell'utente.
 */
export function opzioniModello(provider: string, corrente: string): OpzioneModello[] {
  const opzioni = modelliDelProvider(provider).map((m) => ({
    value: m.id,
    etichetta: etichettaModello(m),
  }));
  if (corrente && !opzioni.some((o) => o.value === corrente)) {
    return [...opzioni, { value: corrente, etichetta: `${corrente} (non in elenco)` }];
  }
  return opzioni;
}
