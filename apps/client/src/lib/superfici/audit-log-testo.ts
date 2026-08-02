/**
 * Testi della sezione "Registro attività" (audit log) delle Impostazioni
 * (issue #583). Estratti in un modulo a parte — invece che inline nel
 * markup di ImpostazioniModal.svelte — per poterli verificare con un test
 * senza montare l'intero componente, ed evitare che un futuro refactor
 * reintroduca il riferimento fuorviante all'AI o affermazioni di privacy
 * non verificate contro `audit.rs`.
 *
 * Il registro traccia azioni generiche su vault/prompt/cartelle/import
 * export/versioni/valutazioni (~30 tipi, vedi call site di
 * `audit::registra` in src-tauri/src). Le uniche 2 voci relative a un
 * provider AI (`provider.salvato`, `provider.eliminato`) riguardano il
 * salvataggio/l'eliminazione della sua configurazione — mai le richieste
 * inviate al modello, che non vengono registrate.
 *
 * Precisazione post-review (#587, rilievo HIGH #1 + #2): il campo
 * `Metadata` di `AuditLog` contiene testo libero scritto dall'utente in
 * più call site, non solo il titolo di un prompt:
 *   - `dati.titolo` in `editor::prompt_crea`/`prompt_aggiorna`;
 *   - `parsed.titolo` in `import_export.rs`;
 *   - `path`/`nuovo_nome` — il percorso gerarchico completo della
 *     cartella, costruito da `calcola_path` sui nomi scelti dall'utente
 *     (es. "Clienti/Rossi Spa/Preventivi 2026") — in `cartelle.rs`
 *     (`folder.creato`, `folder.rinominato`, `folder.eliminato`);
 *   - la `label` di variante in `varianti.rs` (`variante.creata`);
 *   - `dati.etichetta` nei golden test in `regression.rs`.
 * Di questi, il più rivelatore è il percorso di cartella: può portare
 * l'intera gerarchia nome-cliente/nome-progetto, non un singolo titolo.
 *
 * Cosa è stato effettivamente verificato, e con quale portata:
 * - che NESSUNO dei ~35 call site di `audit::registra` passi mai il
 *   `Body` o la `Description` del prompt — verificato leggendo ogni call
 *   site; ancorato lato Rust dai test `audit::test::
 *   titolo_nei_metadati_non_include_body_ne_descrizione` (prompt) e
 *   `audit::test::path_cartella_nei_metadati_di_folder_creato_e_rinominato`
 *   (cartella).
 * - l'elenco qui sopra dei call site che scrivono testo libero è quello
 *   emerso da quella stessa lettura manuale: non c'è un test automatico
 *   che lo tenga esaustivo per sempre — un futuro call site potrebbe
 *   aggiungerne altri senza che questo commento se ne accorga.
 *
 * `AUDIT_COSA` deve riflettere entrambe le cose: cosa resta fuori
 * (Body/Description/conversazioni) E cosa (titoli, etichette, percorsi
 * di cartella) finisce nei metadati e quindi nell'export CSV.
 */

export const AUDIT_LABEL_SIDEBAR = "Registro attività";

export const AUDIT_TITOLO = "Registro attività (audit log)";

export const AUDIT_COSA =
  "Registra le azioni compiute sul vault e sui prompt: creazione, " +
  "modifica, spostamento, eliminazione di prompt e cartelle, import/" +
  "export, versioni, valutazioni e configurazione dei provider AI. " +
  "Non registra il contenuto dei prompt (corpo e descrizione) né le " +
  "conversazioni, e non traccia le richieste inviate ai modelli AI. " +
  "Registra però testi liberi scelti da te, per identificare a quale " +
  "elemento si riferisce ogni voce: il titolo di un prompt, " +
  "l'etichetta di un golden test o di una variante e, soprattutto, il " +
  "nome e il percorso completo di una cartella quando la crei o la " +
  "rinomini (ad es. \"Clienti/Rossi Spa/Preventivi 2026\"). Se questi " +
  "testi contengono nomi di clienti o progetti, finiscono anche " +
  "nell'export CSV.";

export const AUDIT_DOVE =
  "Le voci restano nel database locale del vault, sul tuo computer: " +
  "non vengono inviate altrove.";

export const AUDIT_COME =
  "Oggi l'unico modo per consultarlo è esportarlo in CSV con il " +
  "pulsante qui sotto; non esiste ancora un visualizzatore in-app.";

export const AUDIT_A_COSA_SERVE =
  "Serve a ricostruire cosa è stato fatto e quando, ad esempio per " +
  "verificare una modifica sospetta o capire quando un prompt è stato " +
  "eliminato.";
