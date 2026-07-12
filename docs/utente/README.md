# Documentazione utente

> La guida per **chi usa** Prompt a Porter: il client desktop e gli strumenti che gli girano intorno.

Queste pagine raccontano Prompt a Porter dal punto di vista di chi lo usa ogni giorno: come si installa, come si scrivono e organizzano i prompt, come si portano i dati dentro e fuori, come si collega il vault al terminale e agli assistenti AI. Non serve alcuna conoscenza del codice: dove c'è un percorso da seguire nell'app, lo trovi scritto passo per passo.

Se è la prima volta che apri l'app, parti dalla guida al primo utilizzo e tieni a portata di mano il glossario della sintassi: sono le due pagine che ripagano subito. Tutto il resto puoi leggerlo quando la funzione ti serve davvero — ogni pagina è autonoma e spiega da sola il contesto.

## Inizio rapido

Se sei un nuovo utente, parti da qui:

- [`getting-started.md`](./getting-started.md) — installazione, onboarding, primo prompt, prima compilazione (5 minuti).
- [`glossario-sintassi.md`](./glossario-sintassi.md) — la sintassi del corpo dei prompt: segnaposti, globali, import, codici del linter.
- [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) — tutte le scorciatoie da tastiera, area per area.
- [`troubleshooting.md`](./troubleshooting.md) — risposte ai problemi più comuni.

## Documenti

| Doc | Descrizione |
|---|---|
| [`getting-started.md`](./getting-started.md) | Il percorso dei primi cinque minuti: installazione, onboarding, primo prompt e prima compilazione |
| [`glossario-sintassi.md`](./glossario-sintassi.md) | La sintassi del corpo dei prompt: segnaposti `{{nome}}`, globali `{{global ...}}`, import `{{import ...}}` e i codici del linter |
| [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) | Le scorciatoie da tastiera, area per area: finestra principale, palette, modali, editor |
| [`troubleshooting.md`](./troubleshooting.md) | Risposte ai problemi più comuni: installazione, hotkey, editor, backup |
| [`cli.md`](./cli.md) | Il vault dal terminale: cercare, leggere e compilare prompt con la CLI `pap`, con output pensato anche per gli script |
| [`mcp.md`](./mcp.md) | Collegare il vault a Claude Desktop, Cursor e altri assistenti AI via Model Context Protocol |
| [`formato-export-json.md`](./formato-export-json.md) | Il formato di export JSON documentato campo per campo: schema, gestione dei conflitti in import, garanzia anti lock-in |
| [`markdown-import-export.md`](./markdown-import-export.md) | Import ed export dei prompt come file Markdown, compatibili con Obsidian e Foam |
| [`ricerca-semantica.md`](./ricerca-semantica.md) | Trovare i prompt per significato, non solo per parola: come attivare la ricerca ibrida, bilanciarla e cosa comporta per la privacy |
| [`linting-regole.md`](./linting-regole.md) | Il catalogo delle regole del linter, con esempi pratici per ogni regola |
| [`cartelle.md`](./cartelle.md) | Organizzare il vault in cartelle: come funzionano e come usarle bene |
| [`prompt-componibili.md`](./prompt-componibili.md) | Riusare prompt dentro altri prompt con `{{import "..."}}`: sintassi, esempi, limiti e anti-pattern |
| [`varianti-prompt.md`](./varianti-prompt.md) | Confrontare versioni alternative dello stesso prompt (A/B) senza perdere l'originale, e la differenza con i fork |
| [`fork-prompt.md`](./fork-prompt.md) | Duplicare un prompt mantenendo traccia dell'originale: quando conviene un fork e come si riconosce |
| [`rating-prompt.md`](./rating-prompt.md) | Valutare i prompt dopo l'uso e seguire come cambiano le valutazioni nel tempo |
| [`regression-testing.md`](./regression-testing.md) | Test golden: verificare che un prompt continui a dare buoni risultati anche dopo le modifiche |
| [`auto-update.md`](./auto-update.md) | Come si aggiorna l'app: controllo solo su tua richiesta, firme verificate, privacy e troubleshooting |

## Integrazioni avanzate

- **Riga di comando**: vedi [`cli.md`](./cli.md). La CLI `pap` legge lo stesso vault del client.
- **Assistenti AI** (Claude Desktop, Cursor, ...): vedi [`mcp.md`](./mcp.md).
- **Backup / migrazione**: vedi [`formato-export-json.md`](./formato-export-json.md) (JSON) e [`markdown-import-export.md`](./markdown-import-export.md) (Markdown / Obsidian / Foam).
