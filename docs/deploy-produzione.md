# Deploy Produzione — Prompt a Porter

> Documento in costruzione. Sarà completato durante la Fase 1.

## Client desktop

Distribuito come installer nativo per OS:
- **Windows**: `.msi` via Tauri bundler
- **macOS**: `.dmg` via Tauri bundler
- **Linux**: `.deb` / `.AppImage` via Tauri bundler

## Server sync

Distribuito come container Docker (multistage build, binario Go minimal).

```yaml
# docker-compose.yml esempio
version: "3.8"
services:
  papsync:
    image: pap/sync-server:latest
    ports:
      - "8443:8443"
    volumes:
      - pap-data:/data
    environment:
      - PAP_ADMIN_EMAIL=admin@example.com
      - PAP_DB_PATH=/data/papsync.db
volumes:
  pap-data:
```
