-- V012: fork / clone di un prompt (Fase 4 Step 5).
--
-- Un "fork" è una copia indipendente di un prompt che mantiene un
-- riferimento all'originale (`ForkOfPromptId`). Caso d'uso primario:
-- un utente vuole sperimentare modifiche su un prompt team senza
-- toccare l'originale. La copia entra nel workspace personale
-- dell'utente con visibilità privata; le modifiche restano locali.
--
-- Differenza con varianti (V011):
-- - Variante = stesso intento, formulazione alternativa, IsVariant=1,
--   ParentPromptId punta al "principale". Convive nello stesso
--   workspace del parent.
-- - Fork = clone indipendente per sperimentazione. Forked prompt vive
--   nel proprio workspace; muove la "responsabilità" delle modifiche.
--
-- ForkOfPromptId resta valorizzato anche se l'originale viene cancellato
-- (no FK ON DELETE SET NULL): la tracciabilità storica è più importante
-- dell'integrità referenziale stretta. La UI gestirà il caso "originale
-- non più visibile".

ALTER TABLE Prompts
    ADD COLUMN ForkOfPromptId TEXT REFERENCES Prompts(Id);

-- Indice per query "tutti i fork di un prompt X" (utile per
-- contatore "N fork attivi" lato originale, futuro UX team).
CREATE INDEX IF NOT EXISTS idx_prompts_fork_of
    ON Prompts(ForkOfPromptId)
    WHERE ForkOfPromptId IS NOT NULL AND DeletedAt IS NULL;
