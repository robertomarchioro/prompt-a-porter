# API Server Sync — Prompt à Porter

> **Stato**: allineato al codice al 2026-07-24. Il server è **opzionale**: il client
> desktop è local-first e non lo richiede. È la base del sync di team per lo SKU
> Enterprise (`v2.x`); l'evoluzione è pianificata in
> [`../roadmap/fase-5-enterprise.md`](../roadmap/fase-5-enterprise.md).

## Panoramica

Server Go single binary (`papsync`) per sync opzionale dei workspace team.
Stack: Go 1.25, chi v5 (+ `go-chi/cors`, `go-chi/httprate`), SQLite (WAL),
Argon2id, JWT HS256, gorilla/websocket.

Layout: `cmd/papsync` (main, bootstrap TLS/timeouts) + `internal/`
(`auth`, `config`, `database`, `httpx`, `middleware`, `models`, `server`, `sync`, `ws`).

## Configurazione

| Variabile | Default | Descrizione |
|-----------|---------|-------------|
| `PAP_PORT` | `8443` | Porta di ascolto |
| `PAP_DB_PATH` | `papsync.db` | Percorso file SQLite |
| `PAP_JWT_SECRET` | (generato) | Segreto firma JWT, **minimo 32 byte** (avvio rifiutato sotto soglia). Se assente ne viene generato uno casuale non persistente tra riavvii (warning nei log) — **impostarlo in produzione** |
| `PAP_ADMIN_EMAIL` | — | Email admin iniziale (seed idempotente) |
| `PAP_ADMIN_PASSWORD` | — | Password admin iniziale (seed idempotente) |
| `PAP_WORKSPACE_NAME` | `Team` | Nome workspace creato con l'admin |
| `PAP_TLS_CERT` / `PAP_TLS_KEY` | — | Terminazione TLS diretta (vanno impostate entrambe o nessuna) |
| `PAP_BEHIND_PROXY` | — | `1` se un reverse proxy termina TLS davanti al server |
| `PAP_TRUSTED_PROXY_CIDR` | — | CSV di prefissi CIDR dei proxy fidati: solo da questi hop viene letto `X-Forwarded-For` per l'IP reale del client (CWE-290) |
| `PAP_ALLOWED_ORIGINS` | (vuoto) | CSV di origin browser ammesse per CORS e per l'Origin check del WebSocket. Default: nessuna origine browser consentita |

**Niente HTTP in chiaro**: senza TLS diretto e senza `PAP_BEHIND_PROXY=1` il
server rifiuta di avviarsi.

## Hardening

- **Timeout `http.Server`** (anti-Slowloris, CWE-400 / gosec G112):
  `ReadHeaderTimeout` 5s, `ReadTimeout` 30s, `IdleTimeout` 60s. `WriteTimeout`
  volutamente assente: le connessioni WebSocket vengono hijacked e gestiscono i
  propri deadline in `internal/ws/hub.go`.
- **Rate-limit su `/auth/login`**: 5 tentativi/minuto per IP. Dietro proxy
  (`PAP_BEHIND_PROXY=1`) l'IP viene risolto da `X-Forwarded-For` fidandosi solo
  dei CIDR in `PAP_TRUSTED_PROXY_CIDR`.
- **Anti user-enumeration**: per email inesistente viene comunque eseguita una
  verifica Argon2id su hash dummy (tempo di risposta costante).
- **Refresh con revoca effettiva** (CWE-613): `/auth/refresh` rilegge l'utente
  dal DB — utente disattivato = refresh negato, ruolo aggiornato = claim nuovo,
  invece di copiare i claim dal vecchio token a oltranza.
- **Origin check WebSocket** (CWE-346): connessioni WS con header `Origin` fuori
  dalla allow-list rifiutate; richieste senza `Origin` (client non-browser, es.
  il client Tauri) consentite.
- Security headers e CORS con allow-list esplicita su tutte le rotte.

## Endpoint

### `GET /health`
Health check pubblico.
```json
{"status":"ok","version":"0.1.0"}
```

### `POST /auth/login` (rate-limited)
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
  "user": {"id":"usr-...","email":"admin@example.com","role":"Admin"}
}
```
**Response 401:** errore generico (nessun dettaglio che distingua email
inesistente da password errata).

### `POST /auth/refresh` (auth required)
Rinnova il token JWT rileggendo stato e ruolo dell'utente dal DB.

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
Push delta locali. Il server applica last-write-wins: se `UpdatedAt` del dato
inviato è precedente a quello server, viene contato come conflitto (non
applicato). Ogni modifica accettata è registrata in `SyncChangelog`.

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

### `GET /ws` (auth via subprotocol)
WebSocket per notifiche push real-time nello stesso workspace.

L'autenticazione **non** passa più da query param: il client offre il JWT come
sub-protocol nell'handshake, con prefisso obbligatorio:

```
Sec-WebSocket-Protocol: pap.sync.token.<jwt>
```

**Messaggio WS:**
```json
{"type":"sync_update","workspaceId":"ws-...","entityType":"...","entityId":"...","timestamp":"..."}
```
(`entityType`/`entityId` opzionali.)

## Autenticazione

- **Password hashing:** Argon2id (m=65536, t=3, p=4, 32 byte output, 16 byte salt)
- **JWT:** HS256, TTL 24h, claims custom `userId`, `workspaceId`, `role`
- **Admin iniziale:** creato via env var al primo avvio (seed idempotente)
- Nessuna registrazione pubblica

## Schema DB server

Migrazione `001_schema_iniziale.sql` (tracking in `_Migrazioni`), compatibile
con lo schema client V001: `Workspaces`, `Users`, `Prompts`, `Tags`,
`PromptTags`, `AuditLog`, più la tabella server-only:

- **SyncChangelog** — log append-only di tutte le modifiche ricevute via push
  (auditing e replay)

## Docker

```bash
docker build -t pap/sync-server apps/server/
docker run -p 8443:8443 \
  -e PAP_JWT_SECRET=your-32-byte-minimum-secret \
  -e PAP_ADMIN_EMAIL=admin@example.com \
  -e PAP_ADMIN_PASSWORD=secure-password \
  -e PAP_BEHIND_PROXY=1 \
  -v pap-data:/app/data \
  pap/sync-server
```

`PAP_BEHIND_PROXY=1` presuppone un reverse proxy che termina TLS davanti al
container; in alternativa montare cert e chiave e impostare
`PAP_TLS_CERT`/`PAP_TLS_KEY`. Per il deploy completo vedi
[`docs/operativo/deploy-produzione.md`](../operativo/deploy-produzione.md).

## Test

```bash
cd apps/server
go test ./...
```

34 test di integrazione in `internal/integration_test.go` (auth, sync
push/pull, conflitti, rate-limit, WS, hardening) più unit test nei package
(`config`, `cmd/papsync`). In CI: `server-build.yml` con `go vet`, test `-race`
e **gate coverage 50%** (vedi
[`../contribuire/ci-workflows.md`](../contribuire/ci-workflows.md)).
