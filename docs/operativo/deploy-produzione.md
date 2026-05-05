# Deploy Produzione — Prompt a Porter

## Client desktop

Distribuito come installer nativo per OS tramite Tauri bundler:

| OS | Formato | Comando |
|----|---------|---------|
| Windows | `.msi` | `pnpm --filter @pap/client tauri build` |
| macOS | `.dmg` | `pnpm --filter @pap/client tauri build` |
| Linux | `.deb` / `.AppImage` | `pnpm --filter @pap/client tauri build` |

Il build produce installer nella directory `apps/client/src-tauri/target/release/bundle/`.

### Configurazione release (Cargo.toml)

```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"     # ottimizza per dimensione
strip = true
```

## Server sync

Single binary Go o container Docker.

### Docker

```bash
docker build -t pap/sync-server apps/server/

docker run -d \
  --name papsync \
  -p 8443:8443 \
  -e PAP_JWT_SECRET=<segreto-produzione> \
  -e PAP_ADMIN_EMAIL=admin@example.com \
  -e PAP_ADMIN_PASSWORD=<password-forte> \
  -v pap-data:/app/data \
  pap/sync-server
```

### Docker Compose

```yaml
services:
  papsync:
    image: pap/sync-server:latest
    ports:
      - "8443:8443"
    volumes:
      - pap-data:/app/data
    environment:
      PAP_JWT_SECRET: ${PAP_JWT_SECRET}
      PAP_ADMIN_EMAIL: ${PAP_ADMIN_EMAIL}
      PAP_ADMIN_PASSWORD: ${PAP_ADMIN_PASSWORD}
      PAP_DB_PATH: /app/data/papsync.db
    restart: unless-stopped

volumes:
  pap-data:
```

### Variabili d'ambiente

| Variabile | Obbligatoria | Default | Descrizione |
|-----------|:---:|---------|-------------|
| `PAP_JWT_SECRET` | **Sì** | (generato con warning) | Segreto HMAC per firma JWT |
| `PAP_ADMIN_EMAIL` | No | — | Email admin iniziale |
| `PAP_ADMIN_PASSWORD` | No | — | Password admin iniziale |
| `PAP_PORT` | No | `8443` | Porta HTTP |
| `PAP_DB_PATH` | No | `papsync.db` | Percorso file SQLite |
| `PAP_WORKSPACE_NAME` | No | `Team` | Nome workspace default |

### Sicurezza

- Impostare sempre `PAP_JWT_SECRET` in produzione (non usare quello auto-generato)
- Usare HTTPS (reverse proxy nginx/traefik davanti al server)
- Il DB SQLite del server **non** è cifrato (diverso dal vault client)
- Backup regolari del volume `/app/data`
