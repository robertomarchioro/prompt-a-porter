# API Server Sync — Prompt a Porter

> ⚠️ **Documento stale** — fermo a Fase 1 Step 11. Riflette il server come era a fine Fase 1 e non è stato aggiornato per le evoluzioni di Fase 2 (CLI, MCP, schema versionato V004 cartelle, etc.). L'aggiornamento è programmato in Fase 5 quando il server cross-OS senza Docker (Step 0a) sarà implementato. Vedi [`../roadmap/fase-5-enterprise.md`](../roadmap/fase-5-enterprise.md).

## Panoramica

Server Go single binary (`papsync`) per sync opzionale dei workspace team.
Stack: Go 1.25, chi router, SQLite (WAL), Argon2id, JWT HS256, gorilla/websocket.

## Configurazione

| Variabile | Default | Descrizione |
|-----------|---------|-------------|
| `PAP_PORT` | `8443` | Porta HTTP |
| `PAP_DB_PATH` | `papsync.db` | Percorso file SQLite |
| `PAP_JWT_SECRET` | (generato) | Segreto per firma JWT — **impostare in produzione** |
| `PAP_ADMIN_EMAIL` | — | Email admin iniziale (seed) |
| `PAP_ADMIN_PASSWORD` | — | Password admin iniziale (seed) |
| `PAP_WORKSPACE_NAME` | `Team` | Nome workspace creato con l'admin |

## Endpoint

### `GET /health`
Health check pubblico.
```json
{"status":"ok","version":"0.1.0","clients":2}
```

### `POST /auth/login`
Login con email + password. Ritorna JWT.

**Request:**
```json
{"email":"admin@example.com","password":"..."}
```
**Response 200:**
```json
{
  "token": "eyJ...",
  "expiresAt": 1714780800,
  "user": {"id":"usr-...","email":"admin@example.com","role":"Admin",...}
}
```
**Response 401:** `{"error":"credenziali non valide"}`

### `POST /auth/refresh` (auth required)
Rinnova il token JWT.

**Response 200:**
```json
{"token":"eyJ...","expiresAt":1714780800}
```

### `GET /sync/pull?since=` (auth required)
Pull delta dal server. `since` è un timestamp UTC (`YYYY-MM-DD HH:MM:SS`).
Ritorna solo prompt con `Visibility = 'workspace'` nel workspace dell'utente.

**Response 200:**
```json
{
  "prompts": [...],
  "tags": [...],
  "promptTags": [...],
  "timestamp": "2026-05-03 12:00:00"
}
```

### `POST /sync/push` (auth required)
Push delta locali. Il server applica last-write-wins: se `UpdatedAt` del dato inviato è precedente a quello server, viene contato come conflitto (non applicato).

**Request:**
```json
{
  "prompts": [...],
  "tags": [...],
  "promptTags": [...]
}
```
**Response 200:**
```json
{"accepted":3,"conflicts":0,"timestamp":"2026-05-03 12:00:01"}
```

### `GET /ws?token=` (auth via query param)
WebSocket per notifiche push real-time. Il client riceve messaggi quando un altro client nello stesso workspace esegue un push.

**Messaggio WS:**
```json
{"type":"sync_update","workspaceId":"ws-...","timestamp":"..."}
```

## Autenticazione

- **Password hashing:** Argon2id (m=65536, t=3, p=4, 32 byte output, 16 byte salt)
- **JWT:** HS256, TTL 24h, claims custom `userId`, `workspaceId`, `role`
- **Admin iniziale:** creato via env var al primo avvio (seed idempotente)
- Nessuna registrazione pubblica

## Schema DB server

Compatibile con lo schema client V001 + tabella aggiuntiva:

- **SyncChangelog** — log append-only di tutte le modifiche ricevute via push (per auditing e replay)

## Docker

```bash
docker build -t pap/sync-server apps/server/
docker run -p 8443:8443 \
  -e PAP_JWT_SECRET=your-secret \
  -e PAP_ADMIN_EMAIL=admin@example.com \
  -e PAP_ADMIN_PASSWORD=secure-password \
  -v pap-data:/app/data \
  pap/sync-server
```

## Test

```bash
cd apps/server
go test ./internal/ -v
```

12 test di integrazione coprono: login valido/invalido, refresh token, sync push/pull, conflict detection, hash password, generazione ID, migrazione idempotente, seed admin idempotente, endpoint senza auth, token invalido.
