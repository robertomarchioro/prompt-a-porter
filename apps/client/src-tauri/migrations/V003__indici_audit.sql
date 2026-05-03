-- V003: Indici di performance su AuditLog (Fase 2 Step 3)
-- Necessari per il comando audit_query con filtri lato server e paginazione,
-- soprattutto su workspace team con 10k+ righe storiche.

CREATE INDEX IF NOT EXISTS idx_audit_workspace_time
    ON AuditLog(WorkspaceId, OccurredAt DESC);

CREATE INDEX IF NOT EXISTS idx_audit_user_time
    ON AuditLog(UserId, OccurredAt DESC);

CREATE INDEX IF NOT EXISTS idx_audit_entity
    ON AuditLog(EntityType, EntityId);
