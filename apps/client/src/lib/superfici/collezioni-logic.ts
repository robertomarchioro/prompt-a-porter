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

/** Voce dell'indice arricchita con lo stato locale (`CollezioneVoce` in Rust). */
export interface CollezioneVoce extends CollezioneInfo {
  /** Sha256 importato in questo vault, null se mai importata. */
  importata_sha256: string | null;
  /** Data ISO dell'ultima importazione o aggiornamento, null se mai importata. */
  importata_a: string | null;
}

export type StatoCollezione = "nuova" | "aggiornata" | "aggiornabile";

/** Stato di una collezione rispetto a questo vault. */
export function statoCollezione(v: CollezioneVoce): StatoCollezione {
  if (v.importata_sha256 === null) return "nuova";
  return v.importata_sha256 === v.sha256 ? "aggiornata" : "aggiornabile";
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
  elenca: () => Promise<CollezioneVoce[]>;
  importa: (slug: string) => Promise<ImportReport>;
  aggiorna: (slug: string) => Promise<ImportReport>;
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

/**
 * Riepilogo di un aggiornamento. Qui `conflitti` sono i prompt che l'utente
 * ha modificato e che sono stati lasciati intatti: «conservati», non un
 * problema. Cartelle e tag già presenti non vengono contati.
 */
export function formattaRiepilogoAggiorna(report: ImportReport): string {
  const parti: string[] = [];
  if (report.aggiornati > 0) {
    parti.push(
      report.aggiornati === 1 ? "1 aggiornato" : `${report.aggiornati} aggiornati`,
    );
  }
  if (report.nuovi > 0) {
    parti.push(report.nuovi === 1 ? "1 nuovo" : `${report.nuovi} nuovi`);
  }
  if (report.conflitti > 0) {
    parti.push(
      report.conflitti === 1
        ? "1 conservato perché modificato da te"
        : `${report.conflitti} conservati perché modificati da te`,
    );
  }
  if (report.errori.length > 0) {
    parti.push(
      report.errori.length === 1
        ? "1 non aggiornato"
        : `${report.errori.length} non aggiornati`,
    );
  }
  return parti.length > 0 ? parti.join(" · ") : "Già allineata: nessuna modifica";
}

export type EsitoElenco =
  | { ok: true; collezioni: CollezioneVoce[] }
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

async function esegui(
  slug: string,
  comando: (slug: string) => Promise<ImportReport>,
  formatta: (report: ImportReport) => string,
  notificaListaMutata: () => void,
): Promise<EsitoImportazione> {
  try {
    const report = await comando(slug);
    // Notifica la libreria SOLO se qualcosa è stato scritto: un re-import
    // tutto in conflitto o un aggiornamento a vuoto non cambiano la lista.
    if (report.nuovi > 0 || report.aggiornati > 0) {
      notificaListaMutata();
    }
    return {
      ok: true,
      esito: { slug, riepilogo: formatta(report), errori: report.errori },
    };
  } catch (err) {
    return { ok: false, errore: messaggioErrore(err) };
  }
}

/** Importa una collezione (modalità `skip`). */
export function eseguiImporta(
  slug: string,
  deps: Pick<DipendenzeCollezioni, "importa" | "notificaListaMutata">,
): Promise<EsitoImportazione> {
  return esegui(slug, deps.importa, formattaRiepilogo, deps.notificaListaMutata);
}

/** Aggiorna una collezione già importata (modalità `aggiorna`). */
export function eseguiAggiorna(
  slug: string,
  deps: Pick<DipendenzeCollezioni, "aggiorna" | "notificaListaMutata">,
): Promise<EsitoImportazione> {
  return esegui(
    slug,
    deps.aggiorna,
    formattaRiepilogoAggiorna,
    deps.notificaListaMutata,
  );
}

/** «19 set 2026» dalla data ISO salvata dal backend; stringa vuota se assente. */
export function formattaDataImport(iso: string | null): string {
  if (!iso) return "";
  const d = new Date(iso.includes("T") ? iso : `${iso.replace(" ", "T")}Z`);
  if (Number.isNaN(d.getTime())) return "";
  return d.toLocaleDateString("it-IT", {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}
