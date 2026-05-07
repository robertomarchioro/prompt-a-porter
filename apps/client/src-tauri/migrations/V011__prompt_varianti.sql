-- V011: varianti / A-B testing dello stesso prompt (Fase 4 Step 1).
--
-- Una "variante" è un prompt che condivide l'INTENTO con un altro
-- prompt (il `Parent`) ma testa una formulazione alternativa. UseCount,
-- rating e versioning di ogni variante sono indipendenti, così emerge
-- naturalmente quale formulazione funziona meglio.
--
-- Perché stessa tabella e non `PromptVariants` separata? Perché:
-- - Il modello dati è identico al prompt principale (titolo, body,
--   tag, target_model). Replicare la struttura raddoppia il codice
--   CRUD senza beneficio
-- - FTS5/vec0/lint si applicano già automaticamente alle varianti
-- - Filtro `WHERE IsVariant = 0` per query "solo principali"
--
-- ParentPromptId == NULL per i prompt principali. IsVariant è un
-- flag ridondante con `ParentPromptId IS NOT NULL`, ma fornisce
-- un indice diretto e rende le query "solo principali" leggibili.

ALTER TABLE Prompts
    ADD COLUMN ParentPromptId TEXT REFERENCES Prompts(Id);

ALTER TABLE Prompts
    ADD COLUMN VariantLabel TEXT;

ALTER TABLE Prompts
    ADD COLUMN IsVariant INTEGER NOT NULL DEFAULT 0;

-- Indice per query "tutte le varianti di un prompt".
CREATE INDEX IF NOT EXISTS idx_prompts_parent
    ON Prompts(ParentPromptId)
    WHERE ParentPromptId IS NOT NULL AND DeletedAt IS NULL;

-- Indice per query "solo principali" (lo Step 1 lo userà sia in
-- libreria_lista che nelle ricerche per default).
CREATE INDEX IF NOT EXISTS idx_prompts_main
    ON Prompts(WorkspaceId, IsVariant)
    WHERE DeletedAt IS NULL;
