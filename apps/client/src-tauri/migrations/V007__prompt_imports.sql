-- V007: dependency graph dei prompt che importano altri prompt.
--
-- Scope MVP: schema minimo per tracciare quale prompt importa quale.
-- Vars (`with k=v`) e versioning pinning verranno aggiunti in PR
-- successive.
--
-- Sintassi supportata in body: `{{import "path/al/prompt"}}`. Il path
-- viene risolto via Folders.Path + Prompts.Title (slug).

CREATE TABLE IF NOT EXISTS PromptImports (
    ParentPromptId   TEXT NOT NULL,
    Position         INTEGER NOT NULL,    -- ordine 0-based nel body
    ImportedPath     TEXT NOT NULL,       -- path dichiarato nel body
    ImportedPromptId TEXT,                -- risolto, NULL se path non valido
    PRIMARY KEY (ParentPromptId, Position),
    FOREIGN KEY (ParentPromptId) REFERENCES Prompts(Id),
    FOREIGN KEY (ImportedPromptId) REFERENCES Prompts(Id)
);

-- Indice per query "chi importa X" (utile per UI dipendenze inverse).
CREATE INDEX IF NOT EXISTS idx_imports_imported
    ON PromptImports(ImportedPromptId)
    WHERE ImportedPromptId IS NOT NULL;
