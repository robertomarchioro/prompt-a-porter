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
  -e PAP_JWT_SECRET=<segreto-produzione-almeno-32-byte> \
  -e PAP_ADMIN_EMAIL=admin@example.com \
  -e PAP_ADMIN_PASSWORD=<password-forte> \
  -e PAP_ALLOWED_ORIGINS=https://app.example.com \
  -e PAP_BEHIND_PROXY=1 \
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
      PAP_ALLOWED_ORIGINS: ${PAP_ALLOWED_ORIGINS}
      PAP_BEHIND_PROXY: "1"
    restart: unless-stopped

volumes:
  pap-data:
```

### Variabili d'ambiente

| Variabile | Obbligatoria | Default | Descrizione |
|-----------|:---:|---------|-------------|
| `PAP_JWT_SECRET` | **Sì** | (generato con warning) | Segreto HMAC per firma JWT — **minimo 32 byte**, altrimenti il server rifiuta di avviarsi |
| `PAP_ADMIN_EMAIL` | No | — | Email admin iniziale |
| `PAP_ADMIN_PASSWORD` | No | — | Password admin iniziale |
| `PAP_PORT` | No | `8443` | Porta HTTP/HTTPS |
| `PAP_DB_PATH` | No | `papsync.db` | Percorso file SQLite |
| `PAP_WORKSPACE_NAME` | No | `Team` | Nome workspace default |
| `PAP_ALLOWED_ORIGINS` | No | (nessuna) | Allow-list CSV di origin per CORS e per il controllo Origin del WebSocket (es. `https://a.example.com,https://b.example.com`). Il client desktop non passa da CORS: serve solo per abilitare eventuali client browser |
| `PAP_TLS_CERT` / `PAP_TLS_KEY` | No* | — | Percorsi del certificato/chiave TLS. Se impostati il server serve HTTPS direttamente |
| `PAP_BEHIND_PROXY` | No* | — | Impostare a `1` per avviare in HTTP semplice quando un reverse proxy davanti termina il TLS |
| `PAP_TRUSTED_PROXY_CIDR` | No | (nessuno, single-hop) | Solo con `PAP_BEHIND_PROXY=1`: lista CSV di prefissi CIDR dei reverse proxy fidati (es. `10.0.0.0/8`), usata per leggere l'IP reale del client da `X-Forwarded-For` per il rate-limit su `/auth/login`. Senza impostarla si assume un singolo hop fidato immediatamente davanti al server (va bene per un solo nginx/traefik in locale, non per una catena di proxy/CDN) |

\* Il server **rifiuta di avviarsi** se non è impostata né una coppia `PAP_TLS_CERT`/`PAP_TLS_KEY` né `PAP_BEHIND_PROXY=1`: non è previsto un avvio in HTTP in chiaro esposto direttamente, perché su queste rotte transitano credenziali e JWT (`PAP_TLS_CERT`/`PAP_TLS_KEY` vanno impostate entrambe insieme, mai una sola).

### Sicurezza

- Impostare sempre `PAP_JWT_SECRET` in produzione, **almeno 32 byte** (non usare quello auto-generato, non persistente tra riavvii)
- TLS obbligatorio: o il server lo termina direttamente (`PAP_TLS_CERT`/`PAP_TLS_KEY`), o gira dietro un reverse proxy TLS-terminating con `PAP_BEHIND_PROXY=1` (nginx/traefik davanti al server) — il server non si avvia in nessun altro caso
- Configurare `PAP_ALLOWED_ORIGINS` solo se serve davvero abilitare client browser; di default nessuna origine è consentita (CORS e WebSocket).
  **Prima di popolarla in produzione**, verificare l'header `Origin` effettivo inviato dal webview del client desktop Tauri (via DevTools del webview o i log del server) e aggiungerlo esplicitamente alla allow-list: se il client desktop invia comunque un `Origin` (dipende dal webview di sistema — WebView2/WebKitGTK/WKWebView — e non è garantito che sia assente), una allow-list che non lo include rifiuterebbe le connessioni del client desktop stesso, non solo quelle di un browser. In alternativa, per le chiamate HTTP dirette dal codice Rust si può valutare in futuro `@tauri-apps/plugin-http` (fa la richiesta lato nativo, senza passare dal contesto CORS del webview) invece del `fetch`/`WebSocket` del webview
- Se `PAP_BEHIND_PROXY=1`, impostare `PAP_TRUSTED_PROXY_CIDR` con i CIDR reali della propria infrastruttura quando c'è più di un singolo hop fidato: senza CIDR fidati espliciti il rate-limit assume un solo proxy davanti al server, e un client potrebbe falsificare `X-Forwarded-For` per aggirare il rate-limit o far bloccare un altro utente
- `/auth/login` è protetto da rate-limit per-IP (5 richieste/minuto di default) contro attacchi a forza bruta
- Il DB SQLite del server **non** è cifrato (diverso dal vault client)
- Backup regolari del volume `/app/data`
