// Modelli concreti suggeriti per i provider AI "pubblici", usati come opzioni
// a discesa nei selettori dei test golden (#423). I provider non elencati
// (ollama, openai-compat) usano un campo di testo libero, perché il nome del
// modello dipende dall'installazione/endpoint dell'utente.
//
// Sono ID di modello reali (quelli passati all'API del provider). Vanno
// aggiornati quando escono nuovi modelli; allineati ai placeholder di
// PannelloProviderConfig.

export const MODELLI_PROVIDER: Record<string, string[]> = {
  anthropic: [
    "claude-opus-4-8",
    "claude-sonnet-4-6",
    "claude-haiku-4-5",
    "claude-fable-5",
  ],
  openai: ["gpt-4o", "gpt-4o-mini", "gpt-5"],
  gemini: ["gemini-2.5-pro", "gemini-2.5-flash"],
};

export function modelliNoti(provider: string): string[] {
  return MODELLI_PROVIDER[provider] ?? [];
}

/** True per i provider pubblici con lista modelli nota (→ selettore a discesa). */
export function providerHaModelliNoti(provider: string): boolean {
  return modelliNoti(provider).length > 0;
}

/**
 * Opzioni da mostrare nel selettore: i modelli noti più il valore corrente se
 * non è già in lista (così un default configurato o un valore pre-esistente
 * non sparisce dalla tendina).
 */
export function opzioniModello(provider: string, corrente: string): string[] {
  const noti = modelliNoti(provider);
  if (corrente && !noti.includes(corrente)) {
    return [...noti, corrente];
  }
  return noti;
}
