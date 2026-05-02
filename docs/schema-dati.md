# Schema Dati — Prompt a Porter

> Documento in costruzione. Sarà completato durante lo Step 2 (vault SQLite).

## Panoramica

Database SQLite cifrato con SQLCipher (AES-256).

### Tabelle principali

| Tabella | Scopo |
|---------|-------|
| `Workspaces` | Unità di scoping (personale o team) |
| `Users` | Utenti del workspace |
| `Prompts` | Oggetto principale — template parametrici |
| `PromptVersions` | Storico versioni (schema pronto, uso completo in Fase 2) |
| `Tags` | Tag per organizzazione, scopati per workspace |
| `PromptTags` | Relazione N:M prompt-tag |
| `AuditLog` | Log append-only di operazioni |
| `SyncMeta` | Metadata di sincronizzazione per workspace |
| `PromptsFts` | Full-text search (FTS5) |

### Convenzioni

- **PascalCase** per tabelle e colonne
- **`Id`** come PK (TEXT, ULID o UUIDv7)
- **Tombstone pattern**: `DeletedAt` per soft delete (necessario per sync)
- **Segnaposti**: parsati on-the-fly dal body, non persistiti in tabella separata

Lo schema SQL completo sarà documentato qui con diagramma ER (Mermaid).
