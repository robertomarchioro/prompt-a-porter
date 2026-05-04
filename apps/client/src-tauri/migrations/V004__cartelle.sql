-- V004: Organizzazione gerarchica via cartelle.
--
-- I tag restano etichette trasversali (un prompt può averne molti); le
-- cartelle sono ubicazione canonica (un prompt sta in 1 sola cartella).
-- Modelli ortogonali e coesistenti.
--
-- In Fase 3 (e in v0.2.1 quick win, dove anticipiamo questo pezzo) le
-- cartelle sono solo organizzative — niente ACL. I permessi per cartella
-- arrivano in Fase 4 con il workflow di approvazione.

CREATE TABLE IF NOT EXISTS Folders (
    Id              TEXT PRIMARY KEY,
    WorkspaceId     TEXT NOT NULL,
    ParentFolderId  TEXT,                          -- NULL = root del workspace
    Name            TEXT NOT NULL,
    Path            TEXT NOT NULL,                 -- denormalizzato: "/marketing/email"
    CreatedAt       TEXT NOT NULL DEFAULT (datetime('now')),
    UpdatedAt       TEXT NOT NULL DEFAULT (datetime('now')),
    DeletedAt       TEXT,
    FOREIGN KEY (WorkspaceId) REFERENCES Workspaces(Id),
    FOREIGN KEY (ParentFolderId) REFERENCES Folders(Id)
);

CREATE INDEX IF NOT EXISTS idx_folders_workspace_path
    ON Folders(WorkspaceId, Path);

CREATE INDEX IF NOT EXISTS idx_folders_parent
    ON Folders(ParentFolderId)
    WHERE DeletedAt IS NULL;

-- Vincolo: nome unico fra fratelli (sotto stesso ParentFolderId, escludendo
-- soft-deleted). SQLite non supporta UNIQUE filtrato a livello di colonna,
-- quindi creiamo un indice unique su (WorkspaceId, ParentFolderId, Name)
-- where DeletedAt IS NULL.
CREATE UNIQUE INDEX IF NOT EXISTS idx_folders_unique_sibling_name
    ON Folders(WorkspaceId, COALESCE(ParentFolderId, ''), Name)
    WHERE DeletedAt IS NULL;

-- Aggiunge FolderId ai prompt. NULL = root del workspace (nessuna cartella).
ALTER TABLE Prompts ADD COLUMN FolderId TEXT REFERENCES Folders(Id);

CREATE INDEX IF NOT EXISTS idx_prompts_folder
    ON Prompts(FolderId)
    WHERE DeletedAt IS NULL;
