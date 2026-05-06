-- V009: osservazioni dei run di golden examples (Fase 4 Step 8).
--
-- Una row per esecuzione: chiama il provider AI col body del prompt
-- compilato (con `InputVars` del golden), ottiene `ActualOutput`,
-- calcola `Similarita` rispetto all'`ExpectedOutput` del golden, e
-- salva tutto qui.
--
-- `GoldenId` può essere `NULL` per run "liberi" (l'utente esegue il
-- prompt senza un golden di riferimento, solo per logging/audit).
-- `PromptVersionId` punta al `PromptVersions.Id` corrente al run, così
-- il trend "questo prompt era al 92% in v12, è sceso al 71% in v18"
-- si calcola con un GROUP BY.

CREATE TABLE IF NOT EXISTS PromptRunObservations (
    Id               TEXT PRIMARY KEY,
    PromptVersionId  TEXT NOT NULL,
    GoldenId         TEXT,                    -- NULL = run libero
    Provider         TEXT NOT NULL,           -- 'anthropic' | 'openai' | 'google' | 'ollama' | 'openai-compat'
    Model            TEXT NOT NULL,           -- 'claude-sonnet-4.6' | 'llama3.1' | ...
    ActualOutput     TEXT NOT NULL,
    Similarita       REAL,                    -- [0,1] — NULL solo se errore di calcolo
    Passed           INTEGER NOT NULL DEFAULT 0,   -- 1 se Similarita >= soglia, 0 altrimenti
    LatenzaMs        INTEGER,
    TokensUsed       INTEGER,
    CostoStimato     REAL,                    -- USD, opzionale (provider espone pricing)
    Errore           TEXT,                    -- messaggio errore se run fallito
    RanAt            TEXT NOT NULL,
    RanBy            TEXT NOT NULL,           -- UserId
    FOREIGN KEY (PromptVersionId) REFERENCES PromptVersions(Id),
    FOREIGN KEY (GoldenId) REFERENCES PromptGoldens(Id)
);

-- Trend per prompt nel tempo.
CREATE INDEX IF NOT EXISTS idx_observations_prompt_ranat
    ON PromptRunObservations(PromptVersionId, RanAt DESC);

-- "quante regressioni ho su questo provider/model?"
CREATE INDEX IF NOT EXISTS idx_observations_model_passed
    ON PromptRunObservations(Provider, Model, Passed);
