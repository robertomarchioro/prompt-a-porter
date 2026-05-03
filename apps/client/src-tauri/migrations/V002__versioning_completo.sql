-- V002: Versioning completo dei prompt (Fase 2 Step 2)
-- Estende PromptVersions con campi Visibility e TargetModel per snapshot completo,
-- aggiunge indice per recupero rapido della history, backfill v1 per i prompt
-- esistenti che non hanno ancora una versione registrata.

-- ─── Campi mancanti per snapshot completo ───
ALTER TABLE PromptVersions ADD COLUMN Visibility TEXT;
ALTER TABLE PromptVersions ADD COLUMN TargetModel TEXT;

-- ─── Indice per recupero history ordinata ───
CREATE INDEX IF NOT EXISTS idx_prompt_versions_recent
    ON PromptVersions(PromptId, Version DESC);

-- ─── Backfill: per ogni prompt esistente senza v1 in PromptVersions, ───
-- ─── inserisce uno snapshot v1 con i dati attuali del prompt.       ───
INSERT INTO PromptVersions
    (Id, PromptId, Version, Title, Description, Body, Visibility, TargetModel,
     CreatedAt, CreatedByUserId)
SELECT
    'pv-' || p.Id || '-001' AS Id,
    p.Id AS PromptId,
    1 AS Version,
    p.Title,
    p.Description,
    p.Body,
    p.Visibility,
    p.TargetModel,
    p.CreatedAt,
    p.AuthorUserId
FROM Prompts p
WHERE p.DeletedAt IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM PromptVersions pv
      WHERE pv.PromptId = p.Id AND pv.Version = 1
  );
