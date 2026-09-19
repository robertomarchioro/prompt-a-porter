/**
 * Logica pura della card «Collezioni» (Impostazioni → Dati).
 *
 * Estratta da CollezioniCard.svelte per essere testabile senza runtime
 * Svelte (pattern `-logic.ts` di questa cartella). I comandi Tauri
 * (`collezioni_elenca`, `collezioni_importa`) arrivano per iniezione,
 * così i test verificano le transizioni di stato e la formattazione del
 * report senza rete né webview.
 *
 * Decisione di rotta: `docs/roadmap/collezioni-curate.md`.
 */

export interface CollezioneInfo {
  slug: string;
  titolo: string;
  descrizione: string;
  /** Numero di prompt dichiarato dall'indice. */
  prompt: number;
  sha256: string;
}

export interface ImportReport {
  nuovi: number;
  aggiornati: number;
  conflitti: number;
  errori: string[];
}

export interface EsitoImport {
  slug: string;
  /** Riga di riepilogo pronta per la UI, es. «12 prompt aggiunti · 3 già presenti». */
  riepilogo: string;
  errori: string[];
}

export interface DipendenzeCollezioni {
  elenca: () => Promise<CollezioneInfo[]>;
  importa: (slug: string) => Promise<ImportReport>;
  /** Notifica la libreria che la lista è cambiata (`pap:lista-mutata`). */
  notificaListaMutata: () => void;
}

/** Ripulisce il messaggio di un errore Tauri per mostrarlo all'utente. */
export function messaggioErrore(err: unknown): string {
  return String(err).replace(/^Error: /, "");
}

/**
 * Riepilogo di un import in una riga. `aggiornati` non compare: in modalità
 * `skip` è sempre zero. `conflitti` sono i prompt/tag/cartelle già presenti,
 * che l'utente legge come «già presenti», non come un problema.
 */
export function formattaRiepilogo(report: ImportReport): string {
  const parti: string[] = [];
  parti.push(
    report.nuovi === 1 ? "1 elemento aggiunto" : `${report.nuovi} elementi aggiunti`,
  );
  if (report.conflitti > 0) {
    parti.push(`${report.conflitti} già presenti`);
  }
  if (report.errori.length > 0) {
    parti.push(
      report.errori.length === 1
        ? "1 non importato"
        : `${report.errori.length} non importati`,
    );
  }
  return parti.join(" · ");
}

export type EsitoElenco =
  | { ok: true; collezioni: CollezioneInfo[] }
  | { ok: false; errore: string };

export async function eseguiElenca(
  deps: Pick<DipendenzeCollezioni, "elenca">,
): Promise<EsitoElenco> {
  try {
    const collezioni = await deps.elenca();
    return { ok: true, collezioni };
  } catch (err) {
    return { ok: false, errore: messaggioErrore(err) };
  }
}

export type EsitoImportazione =
  | { ok: true; esito: EsitoImport }
  | { ok: false; errore: string };

/**
 * Importa una collezione e notifica la libreria SOLO se qualcosa è stato
 * scritto: un re-import tutto in conflitto non cambia nulla in lista.
 */
export async function eseguiImporta(
  slug: string,
  deps: Pick<DipendenzeCollezioni, "importa" | "notificaListaMutata">,
): Promise<EsitoImportazione> {
  try {
    const report = await deps.importa(slug);
    if (report.nuovi > 0 || report.aggiornati > 0) {
      deps.notificaListaMutata();
    }
    return {
      ok: true,
      esito: { slug, riepilogo: formattaRiepilogo(report), errori: report.errori },
    };
  } catch (err) {
    return { ok: false, errore: messaggioErrore(err) };
  }
}
