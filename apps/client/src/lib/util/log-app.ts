/**
 * Wrapper condiviso per scrivere nel log applicativo via `tauri-plugin-log`
 * (permesso `log:default`, già presente in capabilities/default.json).
 *
 * Stesso identico meccanismo che prima era duplicato come funzioni locali
 * `logInfo`/`logErrore` in `ImpostazioniModal.svelte` (ora migrato a questo
 * wrapper) e passato via dependency injection a
 * `$lib/superfici/installa-aggiornamento-logic.ts`: fallback su `console.*`
 * se il comando IPC non è disponibile (es. preview senza backend Tauri, o
 * test) — il logging diagnostico non deve mai far fallire il flusso
 * applicativo che avvolge. Estratto qui per essere riusato dai punti di
 * controllo aggiunti a `conferma.ts` e ai chiamanti, invece di ripetere lo
 * stesso try/catch in ogni file (DRY).
 *
 * Livello scelto: `info`. **Non basta di per sé**: il target file di
 * `tauri-plugin-log` accetta `info`/`debug` solo quando l'utente ha
 * attivato "Debug log" in Impostazioni → Sviluppo — il default applicativo
 * è `warn` (vedi `lib.rs`, `carica_debug_log_abilitato` /
 * `debug_log_imposta_livello`, e il messaggio "Debug log attivato. I
 * prossimi eventi verranno scritti su file." nel toggle). Va attivato
 * prima di riprodurre il difetto da diagnosticare — esattamente come già
 * richiesto per i log dell'updater esistenti in
 * `installa-aggiornamento-logic.ts`, che usano lo stesso livello per lo
 * stesso motivo.
 */
import { info as pluginInfo, error as pluginErrore } from "@tauri-apps/plugin-log";

export function logInfoApp(messaggio: string): void {
  void pluginInfo(messaggio).catch(() => {
    console.info(messaggio);
  });
}

export function logErroreApp(messaggio: string): void {
  void pluginErrore(messaggio).catch(() => {
    console.error(messaggio);
  });
}
