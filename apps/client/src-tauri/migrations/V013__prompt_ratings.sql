-- V013: rating granulare dei prompt dopo l'uso (Fase 4 Step 2).
--
-- Dopo aver compilato e copiato un prompt, l'utente può lasciare
-- un feedback con 3 valori discreti (-1, 0, +1). Schema append-only:
-- ogni feedback è una riga separata con timestamp, così emerge la
-- traiettoria nel tempo (es. un prompt molto usato che inizia a
-- prendere rating bassi è candidato per refactor).
--
-- Decisioni di design:
-- - 3 valori invece di 5 stelle: meno bias culturale (italiani danno 3
--   stelle = ok, americani 5 = ok), più discreto, più veloce da scegliere
-- - Note opzionale: i rating negativi spesso meritano una spiegazione
--   ma non costringerla — frizione UX inutile per il caso comune
-- - UsedWithModel opzionale: se il modello target del prompt è cambiato
--   nel tempo, i rating più vecchi non sono direttamente comparabili
-- - Append-only: l'utente può cambiare idea ma il vecchio rating resta
--   nello storico (semplifica, niente UPDATE complicato + audit chiaro)
--
-- L'aggregato (media, distribuzione) è ricalcolato al volo via SQL
-- quando il dettaglio prompt è caricato — niente colonna denormalizzata
-- in Prompts perché update concorrenti complicherebbero il sync futuro.

CREATE TABLE IF NOT EXISTS PromptRatings (
    Id              TEXT PRIMARY KEY,
    PromptId        TEXT NOT NULL REFERENCES Prompts(Id),
    UserId          TEXT NOT NULL REFERENCES Users(Id),
    Rating          INTEGER NOT NULL CHECK (Rating IN (-1, 0, 1)),
    Note            TEXT,
    UsedWithModel   TEXT,
    CreatedAt       TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indice per aggregazione "media degli ultimi N giorni di un prompt".
CREATE INDEX IF NOT EXISTS idx_ratings_prompt_created
    ON PromptRatings(PromptId, CreatedAt DESC);
