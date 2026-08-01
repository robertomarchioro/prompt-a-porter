/**
 * #552 — Impostazioni → Aggiornamenti: recupero del testo da mostrare nel
 * box "Note di rilascio".
 *
 * Le note "vere" sono la sezione di `CHANGELOG.md` della versione offerta
 * dall'updater (cmd Tauri `changelog_sezione_remota`, che costruisce da sé
 * l'URL raw.githubusercontent.com — mai da campi della release — e valida
 * la versione prima di interpolarla). Quel CHANGELOG non è nel binario
 * installato (è quello della versione *nuova*): va recuperato da remoto.
 *
 * Se il recupero fallisce (rete assente, tag non ancora pubblicato,
 * sezione non trovata) si ricade sul corpo della release GitHub già
 * ottenuto da `check()` — mai un box vuoto — con un `fonte` esplicito che
 * il chiamante può mostrare all'utente invece di un fallback silenzioso.
 */

export type FonteNoteRilascio = "changelog" | "release" | "nessuna";

export interface NoteRilascio {
  testo: string;
  fonte: FonteNoteRilascio;
}

export interface RecuperaNoteRilascioParams {
  /** Versione offerta dall'updater (es. "0.8.44"), senza prefisso "v". */
  versione: string;
  /** Corpo della release GitHub, già ottenuto da `check()` (fallback). */
  corpoRelease: string;
  /** Invoca il cmd Tauri `changelog_sezione_remota`, iniettabile nei test. */
  invocaChangelog: (versione: string) => Promise<string>;
}

/**
 * Sceglie il testo da mostrare: preferisce la sezione CHANGELOG remota,
 * ricade sul corpo della release se il recupero fallisce o torna vuoto.
 */
export async function recuperaNoteRilascio({
  versione,
  corpoRelease,
  invocaChangelog,
}: RecuperaNoteRilascioParams): Promise<NoteRilascio> {
  try {
    const sezione = await invocaChangelog(versione);
    if (sezione.trim()) {
      return { testo: sezione, fonte: "changelog" };
    }
  } catch {
    // Rete assente, tag non ancora pubblicato, sezione non trovata:
    // ricadiamo sul corpo della release, gestito sotto. L'errore non è
    // loggato qui: è responsabilità del chiamante (che conosce il
    // contesto UI) decidere se e come segnalarlo.
  }

  if (corpoRelease.trim()) {
    return { testo: corpoRelease, fonte: "release" };
  }

  return { testo: "", fonte: "nessuna" };
}
