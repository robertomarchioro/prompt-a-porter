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
 */

export const AUDIT_LABEL_SIDEBAR = "Registro attività";

export const AUDIT_TITOLO = "Registro attività (audit log)";

export const AUDIT_COSA =
  "Registra le azioni compiute sul vault e sui prompt: creazione, " +
  "modifica, spostamento, eliminazione di prompt e cartelle, import/" +
  "export, versioni, valutazioni e configurazione dei provider AI. " +
  "Non registra il contenuto dei prompt né le conversazioni, e non " +
  "traccia le richieste inviate ai modelli AI.";

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
