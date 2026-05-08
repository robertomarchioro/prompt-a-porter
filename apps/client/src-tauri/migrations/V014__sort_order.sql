-- V014: SortOrder per drag-reorder cartelle e prompt nel redesign v0.8.
-- Additiva: DEFAULT 0 + backfill ROW_NUMBER per workspace+parent.
-- Indici composti per scan ordinato dei siblings.

ALTER TABLE Folders ADD COLUMN SortOrder INTEGER NOT NULL DEFAULT 0;
ALTER TABLE Prompts ADD COLUMN SortOrder INTEGER NOT NULL DEFAULT 0;

-- Backfill: ordine basato sul timestamp di creazione (stabilità sui vault esistenti).
-- SortOrder = numero di siblings creati prima dell'item corrente (0-based).

UPDATE Folders SET SortOrder = (
    SELECT COUNT(*) FROM Folders f2
    WHERE f2.WorkspaceId = Folders.WorkspaceId
      AND COALESCE(f2.ParentFolderId, '') = COALESCE(Folders.ParentFolderId, '')
      AND f2.CreatedAt < Folders.CreatedAt
);

UPDATE Prompts SET SortOrder = (
    SELECT COUNT(*) FROM Prompts p2
    WHERE p2.WorkspaceId = Prompts.WorkspaceId
      AND COALESCE(p2.FolderId, '') = COALESCE(Prompts.FolderId, '')
      AND p2.CreatedAt < Prompts.CreatedAt
);

CREATE INDEX IF NOT EXISTS idx_folders_sort
    ON Folders(WorkspaceId, ParentFolderId, SortOrder);
CREATE INDEX IF NOT EXISTS idx_prompts_sort
    ON Prompts(WorkspaceId, FolderId, SortOrder);
