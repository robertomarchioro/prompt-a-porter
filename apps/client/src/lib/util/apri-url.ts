/**
 * Helper condiviso per aprire link esterni nel browser di sistema
 * (Fix #554/#555/#557: nessun plugin opener era mai stato registrato,
 * quindi i vecchi `<a target="_blank">` non hanno mai funzionato dentro
 * la webview Tauri — non è una regressione).
 *
 * Usa `tauri-plugin-opener` (`openUrl`), iniettabile per i test.
 *
 * VINCOLO DI SICUREZZA (non negoziabile): questo helper apre SOLO URL con
 * schema `http:` o `https:`. Il motivo è concreto, non teorico: il box
 * delle note di rilascio (vedi UpdaterModal/AboutSezione) renderizza
 * markdown scaricato da una GitHub Release remota — contenuto non fidato
 * che può contenere link arbitrari. Senza questa whitelist, un link
 * `file:///...` o uno schema custom (es. `ms-settings:`) nel markdown
 * potrebbe far leggere/eseguire risorse locali tramite il plugin opener.
 * Rifiutiamo tutto ciò che non è http/https, incluso `javascript:` e i
 * protocol handler personalizzati.
 */
import { openUrl } from "@tauri-apps/plugin-opener";

const SCHEMI_AMMESSI = new Set(["http:", "https:"]);

export type RisultatoApriUrl =
  | { ok: true }
  | {
      ok: false;
      motivo: "url_non_valido" | "schema_non_ammesso" | "apertura_fallita";
      messaggio: string;
    };

function messaggioErrore(errore: unknown): string {
  if (errore instanceof Error) return errore.message;
  return String(errore);
}

/**
 * Apre `url` nel browser/applicazione di sistema predefinita, dopo aver
 * validato che lo schema sia `http` o `https`. Non lancia mai: ogni esito
 * (incluso il rifiuto per schema non ammesso o il fallimento del plugin)
 * torna come `RisultatoApriUrl`, così il chiamante decide come segnalarlo
 * esplicitamente all'utente/log invece di un errore silenzioso.
 *
 * @param url URL da aprire.
 * @param apri Funzione di apertura, iniettabile nei test (default: `openUrl` di `@tauri-apps/plugin-opener`).
 */
export async function apriUrlEsterno(
  url: string,
  apri: (url: string) => Promise<void> = openUrl,
): Promise<RisultatoApriUrl> {
  let analizzato: URL;
  try {
    analizzato = new URL(url);
  } catch {
    return {
      ok: false,
      motivo: "url_non_valido",
      messaggio: `URL non valido: "${url}"`,
    };
  }

  if (!SCHEMI_AMMESSI.has(analizzato.protocol)) {
    return {
      ok: false,
      motivo: "schema_non_ammesso",
      messaggio: `Schema non ammesso: "${analizzato.protocol}" (consentiti solo http/https)`,
    };
  }

  try {
    await apri(analizzato.href);
    return { ok: true };
  } catch (errore: unknown) {
    return {
      ok: false,
      motivo: "apertura_fallita",
      messaggio: messaggioErrore(errore),
    };
  }
}
