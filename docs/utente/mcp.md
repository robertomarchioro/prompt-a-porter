# Integrazione MCP — Prompt a Porter

> **Stato**: Beta (Fase 2 Step 7). Read-only. Solo trasporto stdio.

Il server MCP (`@pap/mcp-server`) espone il vault Prompt a Porter al **Model Context Protocol**, permettendo a Claude Desktop, Cursor e qualunque agente MCP-compatibile di cercare, leggere e compilare i tuoi prompt direttamente.

## Tools esposti

| Tool | Descrizione | Input chiave |
|---|---|---|
| `pap_search` | Ricerca FTS5 nel vault con filtri opzionali per modello target / tag | `query`, `limit?`, `target_model?`, `tags?` |
| `pap_get` | Dettaglio completo di un prompt, inclusi tag e segnaposti | `prompt_id` |
| `pap_list_recent` | I prompt usati più di recente | `limit?` |
| `pap_render` | Compila un prompt con valori per i segnaposti `{{...}}` | `prompt_id`, `vars?` |

## Limitazioni MVP

- **Read-only**: niente create/update/delete da MCP. Per modificare prompt usa il client desktop.
- **Vault non cifrati**: il server MCP usa `better-sqlite3` standard. Per vault SQLCipher serve l'estensione `better-sqlite3-multiple-ciphers` + password — supporto in arrivo.
- **Single workspace**: assume vault locale personale. Multi-workspace via server di sync arriverà in step successivi.

## Setup

### 1. Build del server

```bash
cd apps/mcp-server
pnpm install
pnpm build
```

L'output finisce in `dist/index.js`. Si può anche eseguire in dev mode senza build: `pnpm dev`.

### 2. Configurazione del vault path

Il server cerca il vault in (per piattaforma):

- **Linux**: `~/.local/share/com.pap.app/pap-vault.db`
- **macOS**: `~/Library/Application Support/com.pap.app/pap-vault.db`
- **Windows**: `%APPDATA%\com.pap.app\pap-vault.db`

Override via env var:

```bash
PAP_VAULT_PATH=/path/custom/pap-vault.db pap-mcp-server
```

## Integrazione con Claude Desktop

Modifica il file di config di Claude Desktop:

- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Aggiungi nella sezione `mcpServers`:

```json
{
  "mcpServers": {
    "pap": {
      "command": "node",
      "args": ["/percorso/assoluto/a/apps/mcp-server/dist/index.js"],
      "env": {
        "PAP_VAULT_PATH": "/percorso/al/tuo/pap-vault.db"
      }
    }
  }
}
```

Riavvia Claude Desktop. I 4 tool `pap_*` saranno disponibili nella finestra di chat.

## Integrazione con Cursor

In Cursor, vai su **Settings → MCP → Add new server**:

- **Name**: `pap`
- **Command**: `node`
- **Args**: `["/percorso/assoluto/a/apps/mcp-server/dist/index.js"]`
- **Env**: `PAP_VAULT_PATH=/percorso/al/tuo/pap-vault.db`

## Integrazione con altri client MCP

Qualunque client MCP che supporti trasporto stdio funziona. Esempio launcher:

```bash
PAP_VAULT_PATH=/path/to/vault.db node /path/to/dist/index.js
```

Il server attende messaggi MCP via stdin e risponde via stdout. Logging diagnostico va su stderr (non interferisce col protocollo).

## Esempi d'uso

### Cercare prompt da Claude Desktop

> Tu: cerca prompt sul tema "email business" nel mio vault PaP
>
> Claude: [chiama `pap_search({query: "email business"})`] → ti mostra i top risultati con titolo, descrizione, ID.

### Compilare un prompt con valori

> Tu: usa il prompt "scrivi-email-formale" sostituendo destinatario="ufficio acquisti" e oggetto="ritardo consegna"
>
> Claude: [chiama `pap_render({prompt_id: "scrivi-email-formale", vars: {...}})`] → ti restituisce il prompt compilato con i tuoi valori.

## Sicurezza

- Il server MCP gira con i permessi dell'utente che lo lancia. Accede solo al file SQLite del vault, in modalità **read-only** strict (`{ readonly: true }` di better-sqlite3).
- Niente comunicazione di rete (trasporto stdio puro).
- Niente telemetria. Niente log esterni. Logging diagnostico solo su stderr locale.
- Il vault file è leggibile da chiunque abbia accesso al filesystem dell'utente; per workspace altamente sensibili attendere l'integrazione SQLCipher.

## Roadmap

- **Sub-step**: aggiunta tool `pap_create_draft` con coordinamento col client desktop (file watcher o IPC).
- **Sub-step**: trasporto HTTP/SSE con auth Bearer per uso remoto.
- **Sub-step**: supporto vault cifrati via `better-sqlite3-multiple-ciphers`.
- **Fase 5**: MCP server per workspace team via API server di sync (multi-tenant).

## Troubleshooting

### "Vault non trovato"

Il file `pap-vault.db` non esiste al path atteso. Soluzioni:

1. Apri il client desktop almeno una volta per inizializzare il vault.
2. Verifica il path stampato nell'errore.
3. Override con `PAP_VAULT_PATH`.

### "database is locked"

Il vault è aperto in scrittura dal client desktop. Per ora chiudi il client desktop prima di avviare il server MCP. Future versioni supporteranno coabitazione (read-only su WAL mode).

### Tool non visibili in Claude Desktop

1. Verifica path assoluto in `claude_desktop_config.json`.
2. Riavvia completamente l'app Claude (chiudi anche dalla tray).
3. Controlla i log di Claude Desktop per messaggi del server MCP (`stderr` viene catturato).
