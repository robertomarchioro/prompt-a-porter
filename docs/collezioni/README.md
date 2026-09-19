# Collezioni curate

Raccolte di prompt che l'app scarica a richiesta (Impostazioni → Dati →
Collezioni di prompt) e importa nel vault in modalità `skip`. Decisione di
rotta in [`../roadmap/collezioni-curate.md`](../roadmap/collezioni-curate.md);
guida utente in [`../utente/collezioni.md`](../utente/collezioni.md).

## File

| File | Cosa |
|---|---|
| `indice.json` | Elenco delle collezioni: `slug`, `titolo`, `descrizione` (a mano), `prompt` e `sha256` (generati). È ciò che l'app scarica per primo. |
| `<slug>.json` | Una collezione, nel formato di export v1 del vault ([`../utente/formato-export-json.md`](../utente/formato-export-json.md)). Lo `slug` è il nome file: `[a-z0-9-]`, max 40 caratteri. |

L'app li legge da `https://raw.githubusercontent.com/robertomarchioro/prompt-a-porter/main/docs/collezioni/`:
**basta il merge su `main`** perché una collezione nuova o corretta sia
disponibile, senza rilasciare l'app. Il client rifiuta un file il cui sha256
non corrisponde all'indice, quindi l'indice va rigenerato a ogni modifica.

## Aggiungere o modificare una collezione

1. Scrivi o modifica `docs/collezioni/<slug>.json`. Convenzioni:
   - ids **stabili e con prefisso** (`prm-col-<slug>-…`, `fld-col-<slug>-…`,
     `tag-col-…`): l'import è idempotente per id, cambiarli crea doppioni
     a chi ha già importato;
   - cartella radice `fld-collezioni` («Collezioni») condivisa fra tutte,
     poi una cartella per collezione con eventuali sottocartelle;
   - tag con lo stesso nome del demo vault → **stesso id** del demo
     (`tag-codice`, `tag-email`, …); per gli altri `tag-col-<nome>`. Un
     tag omonimo dell'utente con id diverso viene comunque riusato
     dall'import;
   - `global_placeholders` vuoto: non si seminano valori nel vault
     dell'utente. `{{global autore}}` può comparire nel body — se non è
     definito resta visibile, e la descrizione del prompt lo dice;
   - `use_count` 0, `last_used_at` null, `is_favorite` false, `versions`
     vuoto; date fisse alla data di creazione della collezione;
   - gli `{{import "…"}}` puntano per **titolo** a prompt della stessa
     collezione, con titoli che non collidono col demo vault.
2. Se è nuova, aggiungi la voce in `indice.json` con `slug`, `titolo` e
   `descrizione`.
3. Rigenera conteggi e impronte:
   ```sh
   node scripts/collezioni-indice.mjs
   ```
   (`--check` non scrive ed esce 1 se l'indice è stale.)
4. Se è nuova, aggiungila a `COLLEZIONI_COMMITTATE` in
   `apps/client/src-tauri/src/collezioni.rs` e alla tabella in
   [`../utente/collezioni.md`](../utente/collezioni.md).
5. `cargo test --locked collezioni` (in `apps/client/src-tauri`). I test
   verificano che ogni collezione: deserializzi come `ExportV1`, si importi
   pulita e in modo idempotente, conviva col demo vault, abbia tutti gli
   import interni risolvibili, e che l'indice non sia stale.

## Licenza dei contenuti

I prompt sono originali del progetto, sotto la stessa licenza del repo.
Niente materiale copiato da siti terzi: è una scelta di rotta, non solo di
licenza (vedi la roadmap).
