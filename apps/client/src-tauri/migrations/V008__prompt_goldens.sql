-- V008: golden examples per regression testing (Fase 4 Step 8a).
--
-- Un golden è un caso di test salvato sul prompt: dato un certo input
-- (variabili compilate), ci si aspetta un certo output. PaP misura nel
-- tempo se il prompt produce ancora output coerente con il golden,
-- soprattutto al cambio del modello AI sottostante.
--
-- `SimilarityFn` enumera la funzione usata per confrontare expected vs
-- actual: 'cosine' (embedding similarity), 'llm-judge' (rubric prompt),
-- 'exact-match' (== identico), 'regex' (expected è una regex).
--
-- `SoglieTolleranza` ∈ [0,1]. Per cosine/llm-judge è la similarità
-- minima per considerare il run "passed". Per exact-match/regex è
-- ignorato (1.0 di default).

CREATE TABLE IF NOT EXISTS PromptGoldens (
    Id                TEXT PRIMARY KEY,
    PromptId          TEXT NOT NULL,
    Etichetta         TEXT NOT NULL,
    InputVars         TEXT NOT NULL,            -- JSON delle variabili compilate
    ExpectedOutput    TEXT NOT NULL,
    SimilarityFn      TEXT NOT NULL DEFAULT 'cosine'
        CHECK (SimilarityFn IN ('cosine','llm-judge','exact-match','regex')),
    SoglieTolleranza  REAL NOT NULL DEFAULT 0.85
        CHECK (SoglieTolleranza >= 0 AND SoglieTolleranza <= 1),
    CreatedAt         TEXT NOT NULL,
    UpdatedAt         TEXT NOT NULL,
    DeletedAt         TEXT,
    FOREIGN KEY (PromptId) REFERENCES Prompts(Id)
);

CREATE INDEX IF NOT EXISTS idx_goldens_prompt
    ON PromptGoldens(PromptId)
    WHERE DeletedAt IS NULL;
