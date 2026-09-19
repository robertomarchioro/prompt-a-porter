-- V017: traccia quali collezioni curate sono state importate nel vault e
-- con quale impronta, così la modale «Collezioni» sa distinguere «da
-- importare» / «aggiornata» / «aggiornamento disponibile» confrontando lo
-- sha256 remoto con quello importato. Vedi collezioni.rs e
-- docs/roadmap/collezioni-curate.md.
--
-- Una riga per collezione (slug = nome file in docs/collezioni/). Non c'è
-- FK verso Prompts: i prompt importati sono dell'utente e possono sparire
-- senza che la riga perda senso (si può sempre re-importare).

CREATE TABLE IF NOT EXISTS CollezioniImportate (
    Slug         TEXT PRIMARY KEY,
    Sha256       TEXT NOT NULL,
    ImportataA   TEXT NOT NULL DEFAULT (datetime('now')),
    AggiornataA  TEXT NOT NULL DEFAULT (datetime('now'))
);
