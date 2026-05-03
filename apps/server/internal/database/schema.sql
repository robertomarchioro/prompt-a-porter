-- Schema server PaP Sync — embedded via go:embed
-- IF NOT EXISTS per idempotenza.

CREATE TABLE IF NOT EXISTS _Migrazioni (
    Versione    TEXT PRIMARY KEY,
    ApplicataIl TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS Workspaces (
    Id          TEXT PRIMARY KEY,
    Name        TEXT NOT NULL,
    Type        TEXT NOT NULL CHECK (Type IN ('personal','team')),
    ServerUrl   TEXT,
    AccentColor TEXT,
    CreatedAt   TEXT NOT NULL,
    UpdatedAt   TEXT NOT NULL,
    DeletedAt   TEXT
);

CREATE TABLE IF NOT EXISTS Users (
    Id           TEXT PRIMARY KEY,
    WorkspaceId  TEXT NOT NULL REFERENCES Workspaces(Id),
    Email        TEXT NOT NULL UNIQUE,
    DisplayName  TEXT NOT NULL,
    Role         TEXT NOT NULL CHECK (Role IN ('Admin','Editor','User')),
    PasswordHash TEXT NOT NULL,
    CreatedAt    TEXT NOT NULL,
    UpdatedAt    TEXT NOT NULL,
    DeletedAt    TEXT
);

CREATE TABLE IF NOT EXISTS Prompts (
    Id              TEXT PRIMARY KEY,
    WorkspaceId     TEXT NOT NULL REFERENCES Workspaces(Id),
    AuthorUserId    TEXT NOT NULL REFERENCES Users(Id),
    Title           TEXT NOT NULL,
    Description     TEXT,
    Body            TEXT NOT NULL,
    Visibility      TEXT NOT NULL CHECK (Visibility IN ('private','workspace')),
    TargetModel     TEXT,
    IsFavorite      INTEGER NOT NULL DEFAULT 0,
    UseCount        INTEGER NOT NULL DEFAULT 0,
    LastUsedAt      TEXT,
    Version         INTEGER NOT NULL DEFAULT 1,
    CreatedAt       TEXT NOT NULL,
    UpdatedAt       TEXT NOT NULL,
    UpdatedByUserId TEXT REFERENCES Users(Id),
    DeletedAt       TEXT
);

CREATE TABLE IF NOT EXISTS Tags (
    Id          TEXT PRIMARY KEY,
    WorkspaceId TEXT NOT NULL REFERENCES Workspaces(Id),
    Name        TEXT NOT NULL,
    Color       TEXT,
    CreatedAt   TEXT NOT NULL,
    UpdatedAt   TEXT NOT NULL,
    DeletedAt   TEXT,
    UNIQUE (WorkspaceId, Name)
);

CREATE TABLE IF NOT EXISTS PromptTags (
    PromptId TEXT NOT NULL REFERENCES Prompts(Id) ON DELETE CASCADE,
    TagId    TEXT NOT NULL REFERENCES Tags(Id) ON DELETE CASCADE,
    PRIMARY KEY (PromptId, TagId)
);

CREATE TABLE IF NOT EXISTS AuditLog (
    Id          TEXT PRIMARY KEY,
    WorkspaceId TEXT NOT NULL,
    UserId      TEXT,
    Action      TEXT NOT NULL,
    EntityType  TEXT NOT NULL,
    EntityId    TEXT,
    Metadata    TEXT,
    OccurredAt  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS SyncChangelog (
    Id          INTEGER PRIMARY KEY AUTOINCREMENT,
    WorkspaceId TEXT NOT NULL REFERENCES Workspaces(Id),
    EntityType  TEXT NOT NULL,
    EntityId    TEXT NOT NULL,
    Action      TEXT NOT NULL CHECK (Action IN ('upsert','delete')),
    Payload     TEXT NOT NULL,
    ChangedAt   TEXT NOT NULL,
    ChangedBy   TEXT REFERENCES Users(Id)
);

CREATE INDEX IF NOT EXISTS idx_users_workspace ON Users(WorkspaceId);
CREATE INDEX IF NOT EXISTS idx_users_email ON Users(Email);
CREATE INDEX IF NOT EXISTS idx_prompts_workspace ON Prompts(WorkspaceId);
CREATE INDEX IF NOT EXISTS idx_prompts_author ON Prompts(AuthorUserId);
CREATE INDEX IF NOT EXISTS idx_prompts_updated ON Prompts(UpdatedAt);
CREATE INDEX IF NOT EXISTS idx_tags_workspace ON Tags(WorkspaceId);
CREATE INDEX IF NOT EXISTS idx_audit_workspace ON AuditLog(WorkspaceId);
CREATE INDEX IF NOT EXISTS idx_changelog_workspace ON SyncChangelog(WorkspaceId);
CREATE INDEX IF NOT EXISTS idx_changelog_changed ON SyncChangelog(ChangedAt);
