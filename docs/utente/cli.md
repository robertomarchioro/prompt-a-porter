# La CLI `pap`

> Come cercare, leggere e compilare i prompt del vault direttamente dal terminale con la CLI `pap`, in sola lettura.

Il vault non vive solo dentro l'app. Se passi la giornata nel terminale — script, pipe, sessioni SSH — la CLI `pap` porta i prompt dove stai già lavorando: cerchi un prompt, ne guardi il contenuto, lo compili con i tuoi valori e lo passi in pipe al comando successivo, senza mai aprire il client desktop.

`pap` è un singolo eseguibile, multipiattaforma, senza dipendenze da installare a fianco. Legge lo stesso file di vault del client desktop, quindi quello che vedi nell'app è esattamente quello che trovi da terminale — nessuna sincronizzazione, nessuna copia.

Una promessa importante: la CLI è **in sola lettura**. Non crea, non modifica e non cancella nulla, perché apre il vault in modalità read-only a livello di database. Per scrivere prompt c'è il client desktop; la CLI serve a consumarli.

## I comandi

I comandi coprono il ciclo tipico «trovo → guardo → compilo»:

| Comando | Scopo |
|---|---|
| `pap version` | Mostra la versione della CLI e il percorso del vault che verrà usato |
| `pap search [query]` | Cerca prompt nel vault, con filtri per modello target o tag |
| `pap get <id>` | Mostra il dettaglio di un prompt (titolo, corpo, tag, segnaposti) |
| `pap recent` | Elenca i prompt usati più di recente |
| `pap render <id> --var k=v` | Compila un prompt sostituendo i `{{segnaposti}}` con i valori forniti |
| `pap completion <shell>` | Genera lo script di tab-completion per bash, zsh, fish o PowerShell |

`search` e `recent` accettano `--limit` (abbreviato `-n`, default 10, massimo 100). `search` accetta anche `--target` per filtrare per modello target e `--tag` per filtrare per tag (match esatto). Se lanci `pap search` senza query ottieni i prompt più recenti, come `pap recent`.

## Dove `pap` cerca il vault

La CLI risolve da sola il percorso del vault, usando la posizione standard del client desktop per ogni piattaforma:

- **Linux**: `~/.local/share/com.pap.client/pap-vault.db` (rispetta `XDG_DATA_HOME` se impostata)
- **macOS**: `~/Library/Application Support/com.pap.client/pap-vault.db`
- **Windows**: `%APPDATA%\com.pap.client\pap-vault.db`

Se il tuo vault è altrove, la variabile d'ambiente `PAP_VAULT_PATH` ha la precedenza su tutto:

```bash
PAP_VAULT_PATH=/path/custom/pap-vault.db pap search "email"
```

In caso di dubbio, `pap version` stampa il percorso che verrà effettivamente usato:

```bash
pap version
```

## I formati di output

I comandi che elencano o mostrano prompt (`search`, `get`, `recent`) accettano `--format`, così scegli tu se l'output è per i tuoi occhi o per uno script:

| Valore | Descrizione |
|---|---|
| `table` (default) | Tabella a colonne allineate, pensata per la lettura a schermo |
| `json` | JSON indentato standard |
| `yaml` | YAML standard |
| `plain` | Una riga per risultato, formato `id<TAB>titolo` — comodo con `cut`, `awk`, `head` |

Con `pap get --format plain` ottieni invece il solo corpo del prompt, senza metadati: utile quando vuoi il testo grezzo in pipe.

`pap render` fa eccezione: su stdout produce **solo il testo compilato**, così puoi metterlo in pipe senza sorprese; gli eventuali avvisi (segnaposti non compilati, ecc.) vanno su stderr.

## Esempi

### Cercare prompt

Questi esempi mostrano le combinazioni più comuni di query e filtri:

```bash
# Senza query: i prompt più recenti
pap search

# Ricerca full-text
pap search "email business"

# Filtra per modello target
pap search "code review" --target claude-sonnet

# Filtra per tag, limitando i risultati
pap search --tag bug --limit 5

# Output JSON per scripting
pap search "email" --format json | jq '.[].id'
```

Il risultato è una tabella con ID, titolo, visibilità, modello target, contatore d'uso e tag. L'ID nella prima colonna è quello che passerai a `get` e `render`.

### Mostrare il dettaglio

Dato un ID, `get` mostra tutto quello che il vault sa del prompt, inclusi i segnaposti trovati nel corpo:

```bash
pap get prm-abc123
pap get prm-abc123 --format yaml
```

### Compilare un prompt

`render` sostituisce i `{{segnaposti}}` del corpo con i valori che fornisci, inline o da file:

```bash
# Variabili inline (--var è ripetibile)
pap render prm-abc123 --var nome=Mario --var azienda=Bluenergy

# Variabili da file YAML
cat > vars.yaml <<EOF
nome: Mario
azienda: Bluenergy
data: 2026-05-04
EOF
pap render prm-abc123 --var-file vars.yaml

# Il testo compilato in pipe, dritto negli appunti
pap render prm-abc123 --var-file vars.yaml | xclip -selection clipboard

# Un --var passato a riga di comando vince sul valore nel var-file
pap render prm-abc123 --var-file vars.yaml --var nome=Carlo
```

Da `v0.8.32` `pap render` espande anche i **segnaposti globali** `{{global nome}}`, leggendo i valori dal vault, prima di applicare i `--var`.

Se qualcosa resta incompleto, `pap` te lo dice su stderr (l'output su stdout non è mai silenziosamente incompleto):

- segnaposti rimasti senza valore;
- globali non trovati nel vault;
- `{{import}}` presenti nel corpo: la CLI **non li espande** (usa il client desktop per compilarli).

### Prompt recenti

```bash
pap recent --limit 20
pap recent --format plain | head -5 | awk '{print $1}'
```

Il secondo esempio estrae gli ID dei 5 prompt usati più di recente: un buon punto di partenza per script che lavorano «sul prompt di ieri».

## Tab-completion

Il comando `completion` genera lo script di completamento per la tua shell; installalo una volta e da lì in poi `Tab` completa comandi e flag.

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
| `1` | Errore (vault non trovato, prompt non trovato, query fallita, ecc.) |

Il messaggio di errore va su stderr, in una riga, senza stack trace: pensato per essere letto da un umano o intercettato da uno script.

## Sicurezza

- La CLI gira con i permessi dell'utente e apre il vault in modalità **read-only** a livello di database: non può scrivere nemmeno per errore.
- Nessuna comunicazione di rete: tutto avviene sul file locale.
- Niente telemetria, niente log esterni.

## Troubleshooting

### "Vault non trovato"

Il file `pap-vault.db` non esiste al percorso atteso. Verifica con:

```bash
pap version  # mostra il percorso risolto
```

Soluzioni:

1. Apri il client desktop almeno una volta per inizializzare il vault.
2. Se il vault è in posizione custom, imposta `PAP_VAULT_PATH`.

### "database is locked"

Il vault è aperto in scrittura dal client desktop. Chiudi il client desktop e rilancia il comando.

### Tab-completion non funziona

1. Ricarica la shell (`exec $SHELL` o nuovo terminale).
2. Verifica che il file di completion sia in una directory presente nei path di completion della shell.
3. Bash: serve `bash-completion` installato e abilitato (`source /etc/bash_completion`).

## Limiti noti

- La CLI è in **sola lettura**: creare, modificare, importare ed esportare prompt si fa dal client desktop.
- I **vault cifrati** non sono supportati: la CLI apre solo vault non cifrati.
- Lavora sul **vault personale locale**: un solo workspace.
- `pap render` non espande gli `{{import}}`: i prompt componibili vanno compilati dal client desktop.

## Vedi anche

- [`mcp.md`](./mcp.md) — l'altra via per leggere il vault da fuori: gli assistenti AI via Model Context Protocol.
- [`glossario-sintassi.md`](./glossario-sintassi.md) — la sintassi dei segnaposti che `pap render` compila.
- [`formato-export-json.md`](./formato-export-json.md) — per portare i dati fuori dal vault in modo strutturato e documentato.
