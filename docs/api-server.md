# API Server Sync — Prompt a Porter

> Documento in costruzione. Sarà completato durante lo Step 11 (server Go).

## Panoramica

Server Go single binary per sync opzionale dei workspace team.

### Endpoint previsti

| Metodo | Path | Scopo |
|--------|------|-------|
| POST | `/auth/login` | Login con email+password, ritorna JWT |
| POST | `/auth/refresh` | Refresh token JWT |
| GET | `/sync/pull?since=` | Pull delta dal server |
| POST | `/sync/push` | Push delta locali |
| WS | `/ws` | WebSocket per push real-time |

### Autenticazione

- Argon2id per hashing password
- JWT per sessioni
- Admin iniziale creato via env var (niente registrazione pubblica)

La specifica OpenAPI 3.1 completa sarà generata durante l'implementazione.
