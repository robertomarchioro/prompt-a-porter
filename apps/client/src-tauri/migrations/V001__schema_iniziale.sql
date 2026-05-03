-- V001: Schema iniziale — Prompt a Porter
-- Tutte le tabelle necessarie per Fase 1.
-- Convenzione: PascalCase per tabelle e colonne, Id TEXT (ULID/UUIDv7).

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ───────────────────────── Workspace ─────────────────────────
-- Unità di scoping. Il workspace "personal" è locale-only.
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
    Id          TEXT PRIMARY KEY,
    WorkspaceId TEXT NOT NULL REFERENCES Workspaces(Id),
    Email       TEXT,
    DisplayName TEXT NOT NULL,
    Role        TEXT NOT NULL CHECK (Role IN ('Admin','Editor','User')),
    CreatedAt   TEXT NOT NULL,
    UpdatedAt   TEXT NOT NULL,
    DeletedAt   TEXT
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

-- ────────────────────── PromptVersions ──────────────────────
-- Schema pronto per Fase 2 (rollback). In Fase 1 si inserisce solo v1.
CREATE TABLE PromptVersions (
    Id              TEXT PRIMARY KEY,
    PromptId        TEXT NOT NULL REFERENCES Prompts(Id),
    Version         INTEGER NOT NULL,
    Title           TEXT NOT NULL,
    Description     TEXT,
    Body            TEXT NOT NULL,
    CreatedAt       TEXT NOT NULL,
    CreatedByUserId TEXT NOT NULL REFERENCES Users(Id),
    UNIQUE (PromptId, Version)
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

-- ────────────────────────── SyncMeta ────────────────────────
CREATE TABLE SyncMeta (
    WorkspaceId    TEXT PRIMARY KEY REFERENCES Workspaces(Id),
    LastSyncAt     TEXT,
    LastSyncToken  TEXT,
    LastError      TEXT
);

-- ──────────────── Full-Text Search (FTS5) ───────────────────
CREATE VIRTUAL TABLE PromptsFts USING fts5(
    PromptId UNINDEXED,
    Title,
    Description,
    Body,
    Tags
);

-- ──────────────────────── Indici ─────────────────────────────
CREATE INDEX idx_prompts_workspace ON Prompts(WorkspaceId);
CREATE INDEX idx_prompts_author ON Prompts(AuthorUserId);
CREATE INDEX idx_prompts_updated ON Prompts(UpdatedAt);
CREATE INDEX idx_prompts_deleted ON Prompts(DeletedAt);
CREATE INDEX idx_tags_workspace ON Tags(WorkspaceId);
CREATE INDEX idx_users_workspace ON Users(WorkspaceId);
CREATE INDEX idx_audit_workspace ON AuditLog(WorkspaceId);
CREATE INDEX idx_audit_occurred ON AuditLog(OccurredAt);
