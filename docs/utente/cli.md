# CLI `pap` — Reference

> **Stato**: Beta (Fase 2 Step 8). Read-only. MVP con 5 comandi.

`pap` è il CLI di Prompt a Porter per cercare, leggere e compilare prompt dal vault locale direttamente da terminale. Single binary, multipiattaforma, zero dipendenze runtime.

## Comandi MVP

| Comando | Scopo |
|---|---|
| `pap version` | Mostra versione + path del vault risolto |
| `pap search [query]` | Cerca prompt via FTS5, filtra per modello target o tag |
| `pap get <id>` | Mostra il dettaglio di un prompt (titolo, body, tag, segnaposti) |
| `pap recent` | Lista i prompt usati più di recente |
| `pap render <id> --var k=v` | Compila il template sostituendo i `{{segnaposti}}` |
| `pap completion <shell>` | Genera script di tab-completion per la shell |

## Limitazioni MVP

- **Read-only**. Niente `new`, `import`, `export`, `login` in questo step. Arriveranno quando il server di sync esporrà API HTTP complete (sub-step successivo).
- **Solo vault non cifrati**. Per vault SQLCipher servirà l'estensione SQLite dedicata; rimandato.
- **Single workspace personale**. Multi-workspace via server di sync arriverà in Fase 5.

## Vault discovery

Il CLI cerca il file `pap-vault.db` in (per piattaforma):

- **Linux**: `~/.local/share/com.pap.client/pap-vault.db`
- **macOS**: `~/Library/Application Support/com.pap.client/pap-vault.db`
- **Windows**: `%APPDATA%\com.pap.client\pap-vault.db`

Override via env var:

```bash
PAP_VAULT_PATH=/path/custom/pap-vault.db pap search "email"
```

Verifica il path che `pap` userà:

```bash
pap version
```

## Output formats

Tutti i comandi che producono output strutturato (`search`, `get`, `recent`) accettano `--format`:

| Valore | Descrizione |
|---|---|
| `table` (default) | Tabella ASCII con `text/tabwriter` |
| `json` | JSON indented standard |
| `yaml` | YAML standard |
| `plain` | Una riga per result, formato `id<TAB>title` (shell-friendly) |

`pap render` produce solo il testo compilato puro su stdout (per `\| pipe`); avvisi di segnaposti non compilati vanno su stderr.

## Esempi

### Cercare prompt

```bash
# Recenti
pap search

# Per query
pap search "email business"

# Filtra per modello target
pap search "code review" --target claude-sonnet

# Filtra per tag
pap search --tag bug --limit 5

# Output JSON per scripting
pap search "email" --format json | jq '.[].id'
```

### Mostrare dettaglio

```bash
pap get prm-abc123
pap get prm-abc123 --format yaml
```

### Compilare un template

```bash
# Variabili inline
pap render prm-abc123 --var nome=Mario --var azienda=Bluenergy

# Variabili da file YAML
cat > vars.yaml <<EOF
nome: Mario
azienda: Bluenergy
data: 2026-05-04
EOF
pap render prm-abc123 --var-file vars.yaml

# Combinare con pipe
pap render prm-abc123 --var-file vars.yaml | xclip -selection clipboard

# Override da CLI sopra var-file
pap render prm-abc123 --var-file vars.yaml --var nome=Carlo
```

Da `v0.8.32` `pap render` espande anche i **segnaposti globali** `{{global nome}}`, leggendo i valori dal vault, prima di applicare i `--var`.

Avvisi su stderr (l'output su stdout non è mai silenziosamente incompleto):

- segnaposti rimasti senza valore;
- globali non trovati nel vault;
- `{{import}}` presenti nel body: la CLI **non li espande** (usa il client desktop per compilarli).

### Recenti

```bash
pap recent --limit 20
pap recent --format plain | head -5 | awk '{print $1}'
```

## Tab-completion

Il comando `completion` (auto-aggiunto da Cobra) genera lo script per la shell:

### Bash

```bash
pap completion bash > /etc/bash_completion.d/pap
# oppure (per utente, senza root):
pap completion bash > ~/.local/share/bash-completion/completions/pap
```

### Zsh

```bash
pap completion zsh > "${fpath[1]}/_pap"
```

Oppure (con Oh My Zsh):

```bash
mkdir -p ~/.oh-my-zsh/completions
pap completion zsh > ~/.oh-my-zsh/completions/_pap
```

### Fish

```bash
pap completion fish > ~/.config/fish/completions/pap.fish
```

### PowerShell

```powershell
pap completion powershell | Out-String | Invoke-Expression
# Persistente:
pap completion powershell >> $PROFILE
```

## Installazione

L'installazione avviene **da sorgente** (la pagina Releases pubblica solo gli installer del client desktop, non i binari CLI). Serve Go installato:

```bash
git clone https://github.com/robertomarchioro/prompt-a-porter
cd prompt-a-porter/apps/cli
go install .
```

`go install` mette il binario in `$GOPATH/bin` (default `~/go/bin`); assicurati che sia in `$PATH`. In alternativa `go build -o pap .` produce il binario nella directory corrente, da spostare dove preferisci (es. `/usr/local/bin/pap`).

## Codici di uscita

| Codice | Significato |
|---|---|
| `0` | Successo |
| `1` | Errore generico (vault non trovato, prompt non trovato, query SQL fallita, ecc.) |

Il messaggio di errore va su stderr; il CLI non stampa stack trace per default (uso da terminale pulito). Per debug più verbose, settare `--verbose` (rimandato).

## Sicurezza

- Il CLI gira con i permessi dell'utente. Apre il vault SQLite in modalità **read-only** (DSN `?mode=ro`).
- Niente comunicazione di rete in MVP.
- Niente telemetria. Niente log esterni.
- Per workspace altamente sensibili attendere il supporto SQLCipher.

## Roadmap

- **Sub-step**: comandi write (`new`, `import`, `export`) tramite API server o IPC con client desktop.
- **Sub-step**: `login` per workspace team con JWT + config persistente in `~/.config/pap/config.toml`.
- **Sub-step**: supporto vault cifrati.
- **Fase 4**: `pap test <id>` per regression testing dei prompt (golden examples).

## Troubleshooting

### "Vault non trovato"

Il file `pap-vault.db` non esiste al path atteso. Verifica con:

```bash
pap version  # mostra il path risolto
```

Soluzioni:

1. Apri il client desktop almeno una volta per inizializzare il vault.
2. Se hai installato il vault in posizione custom, override con `PAP_VAULT_PATH`.

### "database is locked"

Il vault è aperto in scrittura dal client desktop. Per ora chiudi il client desktop prima di lanciare il CLI. Future versioni supporteranno coabitazione (read-only su WAL mode).

### Tab-completion non funziona

1. Ricarica la shell (`exec $SHELL` o nuovo terminale).
2. Verifica che il file di completion sia in una directory presente nei path di completion della shell.
3. Bash: serve `bash-completion` installato e abilitato (`source /etc/bash_completion`).
