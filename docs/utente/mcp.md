# Integrazione MCP

> Come collegare il vault a Claude Desktop, Cursor e altri assistenti AI con il server MCP, in sola lettura.

Il tuo vault può diventare una fonte per gli assistenti AI che usi già. Con il server MCP di Prompt a Porter, Claude Desktop, Cursor e qualunque altro client compatibile con il **Model Context Protocol** possono cercare i tuoi prompt, leggerne il contenuto e compilarli con i valori che detti in chat — senza copia-incolla e senza che i tuoi dati lascino il computer.

In pratica: chiedi al tuo assistente «usa il mio prompt per le email formali, il destinatario è l'ufficio acquisti», e lui va a prenderlo nel vault, lo compila e lo usa. La libreria che curi nel client desktop diventa disponibile in ogni conversazione, sempre aggiornata perché letta direttamente dal file del vault.

Due garanzie di fondo. Il server è **in sola lettura**: gli assistenti possono cercare, leggere e compilare, mai creare, modificare o cancellare. E comunica con il client AI solo via **stdio** (input/output standard del processo): nessuna porta aperta, nessuna comunicazione di rete.

## I tool esposti

Una volta collegato, l'assistente vede quattro tool con prefisso `pap_`, uno per ogni operazione sul vault:

| Tool | Descrizione | Input chiave |
|---|---|---|
| `pap_search` | Ricerca full-text nel vault con filtri opzionali per modello target / tag | `query`, `limit?`, `target_model?`, `tags?` |
| `pap_get` | Dettaglio completo di un prompt, inclusi tag e segnaposti | `prompt_id` |
| `pap_list_recent` | I prompt usati più di recente | `limit?` |
| `pap_render` | Compila un prompt con valori per i segnaposti `{{...}}` | `prompt_id`, `vars?` |

Qualche dettaglio di comportamento utile da sapere: `limit` vale 10 per default e viene comunque limitato a 50; una `query` vuota in `pap_search` restituisce i prompt recenti; il filtro `tags` richiede che il prompt abbia **tutti** i tag indicati. Se `pap_render` trova segnaposti senza valore, li elenca in una nota in coda al testo compilato, così l'assistente (e tu) ve ne accorgete subito.

## Preparare il server

Il server si compila una volta sola dal repository:

```bash
cd apps/mcp-server
pnpm install
pnpm build
```

L'output finisce in `dist/index.js`: è il file che indicherai nella configurazione del tuo client AI. In alternativa `pnpm dev` esegue il server senza build (utile per provarlo al volo).

## Dove il server cerca il vault

Il server usa la posizione standard del vault del client desktop, per piattaforma:

- **Linux**: `~/.local/share/com.pap.client/pap-vault.db`
- **macOS**: `~/Library/Application Support/com.pap.client/pap-vault.db`
- **Windows**: `%APPDATA%\com.pap.client\pap-vault.db`

Se il vault è altrove, la variabile d'ambiente `PAP_VAULT_PATH` ha la precedenza:

```bash
PAP_VAULT_PATH=/path/custom/pap-vault.db pap-mcp-server
```

Se il file non esiste, il server si ferma subito con un messaggio che indica il percorso cercato.

## Collegare Claude Desktop

Modifica il file di config di Claude Desktop:

- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Aggiungi nella sezione `mcpServers` (il percorso a `dist/index.js` deve essere assoluto):

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

Riavvia Claude Desktop: i quattro tool `pap_*` compaiono tra quelli disponibili nella finestra di chat.

## Collegare Cursor

In Cursor, vai su **Settings → MCP → Add new server** e compila:

- **Name**: `pap`
- **Command**: `node`
- **Args**: `["/percorso/assoluto/a/apps/mcp-server/dist/index.js"]`
- **Env**: `PAP_VAULT_PATH=/percorso/al/tuo/pap-vault.db`

## Collegare altri client MCP

Qualunque client MCP che supporti il trasporto stdio funziona. Il launcher minimo è:

```bash
PAP_VAULT_PATH=/path/to/vault.db node /path/to/dist/index.js
```

Il server attende messaggi MCP via stdin e risponde via stdout. Il logging diagnostico va su stderr, quindi non interferisce col protocollo.

## Esempi d'uso

### Cercare prompt da Claude Desktop

Un esempio di conversazione: chiedi una ricerca in linguaggio naturale, l'assistente traduce in una chiamata a `pap_search`.

> Tu: cerca prompt sul tema "email business" nel mio vault PaP
>
> Claude: [chiama `pap_search({query: "email business"})`] → ti mostra i migliori risultati con titolo, descrizione e ID.

### Compilare un prompt con valori

Qui l'assistente compila un prompt del vault con i valori che gli detti, e può usare il risultato direttamente nella conversazione:

> Tu: usa il prompt "scrivi-email-formale" sostituendo destinatario="ufficio acquisti" e oggetto="ritardo consegna"
>
> Claude: [chiama `pap_render({prompt_id: "scrivi-email-formale", vars: {...}})`] → ti restituisce il prompt compilato con i tuoi valori.

## Sicurezza

- Il server gira con i permessi dell'utente che lo lancia e apre il file del vault in **sola lettura** a livello di database: non può scrivere nemmeno per errore.
- Nessuna comunicazione di rete: solo stdio con il client AI.
- Niente telemetria, niente log esterni; il logging diagnostico resta su stderr locale.
- I contenuti letti dal vault (titoli, descrizioni, corpi dei prompt) vengono restituiti all'assistente racchiusi in delimitatori espliciti che li marcano come contenuto non fidato: una difesa in più contro prompt malevoli nascosti nei dati.
- Il file del vault resta leggibile da chiunque abbia accesso al filesystem del tuo utente: tienilo in conto per contenuti molto sensibili.

## Troubleshooting

### "Vault non trovato"

Il file `pap-vault.db` non esiste al percorso atteso. Soluzioni:

1. Apri il client desktop almeno una volta per inizializzare il vault.
2. Verifica il percorso stampato nell'errore.
3. Imposta `PAP_VAULT_PATH` con il percorso giusto.

### "database is locked"

Il vault è aperto in scrittura dal client desktop. Chiudi il client desktop prima di avviare il server MCP.

### Tool non visibili in Claude Desktop

1. Verifica che il percorso in `claude_desktop_config.json` sia assoluto.
2. Riavvia completamente l'app Claude (chiudila anche dalla tray).
3. Controlla i log di Claude Desktop: i messaggi del server MCP su stderr vengono catturati lì.

## Limiti noti

- Il server è in **sola lettura**: per creare o modificare prompt serve il client desktop.
- I **vault cifrati** non sono supportati: il server apre solo vault non cifrati.
- Lavora sul **vault personale locale**: un solo workspace.
- L'unico trasporto è **stdio**: niente accesso remoto via rete.

## Vedi anche

- [`cli.md`](./cli.md) — l'altra via per leggere il vault da fuori: il terminale.
- [`glossario-sintassi.md`](./glossario-sintassi.md) — la sintassi dei segnaposti che `pap_render` compila.
- [`getting-started.md`](./getting-started.md) — se non hai ancora un vault da esporre, parti da qui.
