/**
 * Delega dei click sui link dentro markdown renderizzato con `{@html}`
 * (vedi `updater-notes.ts`: `marked` + `DOMPurify`, che etichetta ogni
 * link `target="_blank"`).
 *
 * Rilievo ereditato dal gruppo A: `target="_blank"` non naviga in una
 * webview Tauri — senza intercettazione un click su questi link non fa
 * nulla. Il contenuto renderizzato arriva da remoto (CHANGELOG del repo o,
 * in fallback, il corpo di una release GitHub), quindi va trattato come
 * non fidato: ogni link passa da `apriUrlEsterno`, che accetta solo
 * schemi `http`/`https` e apre nel browser di sistema.
 */
import { apriUrlEsterno, type RisultatoApriUrl } from "./apri-url";

/**
 * Handler da agganciare all'`onclick` del contenitore che ospita
 * `{@html ...}`: intercetta i click sui link (event delegation, non serve
 * un listener per ciascun `<a>` renderizzato) e li instrada su
 * `apriUrlEsterno`. Ignora i click che non ricadono su un `<a href>`.
 *
 * @param evento click ricevuto dal contenitore.
 * @param apri funzione di apertura, iniettabile nei test (default: `apriUrlEsterno`).
 * @returns l'esito dell'apertura, o `null` se il click non riguardava un link.
 */
export async function gestisciClickLinkMarkdown(
  evento: MouseEvent,
  apri: (url: string) => Promise<RisultatoApriUrl> = apriUrlEsterno,
): Promise<RisultatoApriUrl | null> {
  const target = evento.target;
  if (!(target instanceof Element)) return null;

  const link = target.closest("a");
  if (!link) return null;

  const href = link.getAttribute("href");
  if (!href) return null;

  // Il preventDefault evita che la webview tenti comunque una navigazione
  // interna (link relativi malformati, ancore, ecc.) prima che
  // `apriUrlEsterno` decida se e come aprirlo.
  evento.preventDefault();
  return apri(href);
}
