-- 001: Schema iniziale — PaP Sync Server
-- Compatibile con lo schema client V001.

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ───────────────────────── Migrazioni ─────────────────────────
CREATE TABLE IF NOT EXISTS _Migrazioni (
    Versione  TEXT PRIMARY KEY,
    ApplicataIl TEXT NOT NULL
);

-- ───────────────────────── Workspace ─────────────────────────
CREATE TABLE Workspaces (
    Id          TEXT PRIMARY KEY,
    Name        TEXT NOT NULL,
    Type        TEXT NOT NULL CHECK (Type IN ('personal','team')),
    ServerUrl   TEXT,
    AccentColor TEXT,
    CreatedAt   TEXT NOT NULL,
    UpdatedAt   TEXT NOT NULL,
    DeletedAt   TEXT
);

-- ──────────────────────────── Users ──────────────────────────
CREATE TABLE Users (
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

-- ─────────────────────────── Prompts ─────────────────────────
CREATE TABLE Prompts (
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

-- ──────────────────────────── Tags ───────────────────────────
CREATE TABLE Tags (
    Id          TEXT PRIMARY KEY,
    WorkspaceId TEXT NOT NULL REFERENCES Workspaces(Id),
    Name        TEXT NOT NULL,
    Color       TEXT,
    CreatedAt   TEXT NOT NULL,
    UpdatedAt   TEXT NOT NULL,
    DeletedAt   TEXT,
    UNIQUE (WorkspaceId, Name)
);

-- ────────────────────── PromptTags (N:M) ────────────────────
CREATE TABLE PromptTags (
    PromptId TEXT NOT NULL REFERENCES Prompts(Id) ON DELETE CASCADE,
    TagId    TEXT NOT NULL REFERENCES Tags(Id) ON DELETE CASCADE,
    PRIMARY KEY (PromptId, TagId)
);

-- ────────────────────── AuditLog (append-only) ──────────────
CREATE TABLE AuditLog (
    Id          TEXT PRIMARY KEY,
    WorkspaceId TEXT NOT NULL,
    UserId      TEXT,
    Action      TEXT NOT NULL,
    EntityType  TEXT NOT NULL,
    EntityId    TEXT,
    Metadata    TEXT,
    OccurredAt  TEXT NOT NULL
);

-- ────────────────── SyncChangelog (delta tracking) ──────────
CREATE TABLE SyncChangelog (
    Id          INTEGER PRIMARY KEY AUTOINCREMENT,
    WorkspaceId TEXT NOT NULL REFERENCES Workspaces(Id),
    EntityType  TEXT NOT NULL,
    EntityId    TEXT NOT NULL,
    Action      TEXT NOT NULL CHECK (Action IN ('upsert','delete')),
    Payload     TEXT NOT NULL,
    ChangedAt   TEXT NOT NULL,
    ChangedBy   TEXT REFERENCES Users(Id)
);

-- ──────────────────────── Indici ─────────────────────────────
CREATE INDEX idx_users_workspace ON Users(WorkspaceId);
CREATE INDEX idx_users_email ON Users(Email);
CREATE INDEX idx_prompts_workspace ON Prompts(WorkspaceId);
CREATE INDEX idx_prompts_author ON Prompts(AuthorUserId);
CREATE INDEX idx_prompts_updated ON Prompts(UpdatedAt);
CREATE INDEX idx_tags_workspace ON Tags(WorkspaceId);
CREATE INDEX idx_audit_workspace ON AuditLog(WorkspaceId);
CREATE INDEX idx_changelog_workspace ON SyncChangelog(WorkspaceId);
CREATE INDEX idx_changelog_changed ON SyncChangelog(ChangedAt);
