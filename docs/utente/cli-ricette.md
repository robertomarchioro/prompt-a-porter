# pap in pratica — ricette da riga di comando

> Idee concrete per comporre `pap` con il resto del terminale: dare un prompt
> in pasto a un modello AI, copiarlo negli appunti, sceglierlo al volo,
> generarne a raffica, o pescarlo dentro uno script.

La CLI `pap` non è un secondo modo, più povero, di usare Prompt a Porter: è
il modo di portare i tuoi prompt **fuori** dall'app, dentro il flusso di
lavoro che già hai nel terminale. È di sola lettura (non crea né modifica
nulla nel vault), gira offline e non chiama nessun servizio esterno — quindi
puoi metterla in una pipe, in uno script, in un git hook senza sorprese.

La sua superpotenza è una sola: **`pap render` scrive il prompt compilato su
`stdout`, e nient'altro.** Gli avvisi (un valore globale non trovato, un
`{{import}}` che la CLI non espande) vanno su `stderr`, separati. Vuol dire
che `pap render … | qualcosa` passa esattamente il prompt, pulito, a
qualunque cosa venga dopo. Tutte le ricette qui sotto nascono da lì.

## Dal vault al modello AI, in un colpo

L'esempio che riassume tutto: prendi un prompt salvato, riempi i segnaposti
e mandalo direttamente al tuo client AI da terminale.

```bash
pap render email-reclamo --var servizio="corriere" --var problema="pacco smarrito" | llm
```

Qui `llm` è un client LLM da riga di comando (funziona allo stesso modo con
`ollama run <modello>`, `sgpt`, o qualsiasi strumento che legga il prompt da
`stdin`). Se il tuo preferisce ricevere il prompt come argomento invece che
da pipe, usa la sostituzione:

```bash
claude "$(pap render email-reclamo --var servizio=corriere --var problema='pacco smarrito')"
```

In entrambi i casi hai sostituito il copia-incolla manuale — cercare il
prompt, incollarlo, riempire le parti variabili a mano — con una riga
riproducibile.

## Riassumere quello che hai negli appunti

Un prompt `riassumi` salvato una volta, applicato al contenuto della
clipboard senza aprire nulla:

```bash
pap render riassumi --var testo="$(xclip -o -selection clipboard)" | llm
```

Su macOS l'equivalente è `$(pbpaste)`, su Windows (PowerShell)
`$(Get-Clipboard)`. Lo stesso schema vale per un file: `--var testo="$(cat
articolo.txt)"`.

## Un messaggio di commit dal diff, tutto in una riga

Salva un prompt `commit-msg` che, dato un diff, produce un messaggio di
commit nel tuo stile. Poi:

```bash
pap render commit-msg --var diff="$(git diff --cached)" | llm | git commit -F -
```

Il diff in stage diventa il valore del segnaposto, il modello scrive il
messaggio, e `git commit -F -` lo legge dalla pipe. La convenzione di commit
la decidi una volta, nel prompt, e la applichi ovunque.

## Negli appunti invece che nel modello

A volte vuoi solo il prompt compilato pronto da incollare a mano da qualche
parte. Manda l'output alla clipboard:

```bash
pap render bug-report --var-file valori.yaml | xclip -selection clipboard   # Linux
pap render bug-report --var-file valori.yaml | pbcopy                        # macOS
pap render bug-report --var-file valori.yaml | clip                          # Windows
```

Con `--var-file` i valori arrivano da un file YAML (`chiave: valore`),
comodo quando sono tanti o li vuoi versionare accanto al progetto.

## Sceglierlo al volo con un menu interattivo

Se non ricordi l'id, lascia che sia `fzf` a farti scegliere: `pap recent`
in formato JSON, filtrato con `jq`, diventa un menu a filtro incrementale.

```bash
id=$(pap recent --format json | jq -r '.[] | "\(.id)\t\(.title)"' | fzf | cut -f1)
pap render "$id"
```

Digiti qualche lettera del titolo, premi Invio, e il prompt scelto viene
compilato. Lo stesso con `pap search "onboarding" --format json` se vuoi
partire da una ricerca invece che dai recenti.

## Un alias per il tuo "esperto" personale

Trasforma un prompt-ruolo in un comando di shell. In `~/.bashrc` o
`~/.zshrc`:

```bash
ask() { pap render esperto-rust --var domanda="$*" | llm; }
```

Da quel momento `ask "come funziona il borrow checker?"` compila il tuo
prompt "esperto-rust" con la domanda e te la gira al modello. Un prompt
curato, dietro tre lettere.

## Generarne a raffica

Un ciclo sui file di valori produce un output per ciascuno — utile per
promemoria, email personalizzate, risposte a tante varianti dello stesso
caso:

```bash
for f in clienti/*.yaml; do
  pap render promemoria --var-file "$f" > "out/$(basename "$f" .yaml).txt"
done
```

## Ispezionare il vault dentro uno script

I formati `json`/`yaml`/`plain` rendono `pap` un buon cittadino delle
pipeline. Qualche estrazione tipica:

```bash
pap search "" --tag email --format json | jq -r '.[].id'   # gli id di tutti i prompt taggati "email"
pap get prm-abc123 --format plain                          # solo il body, niente cornice
pap recent --format table                                  # sguardo rapido a colonne, per te
```

Da qui in poi è normale scripting: conta, filtra, esporta, alimenta un altro
comando.

## CLI o MCP?

Sono due porte sullo stesso vault, per due momenti diversi. La **CLI** è per
quando **sei tu** a guidare: script, pipe, automazioni, git hook. L'**MCP**
(vedi [`mcp.md`](./mcp.md)) è per quando è **l'assistente AI** a dover
leggere i tuoi prompt da solo, durante una conversazione in Claude Desktop o
Cursor. Se stai scrivendo un comando, vuoi la CLI; se stai chattando con un
agente e vuoi che attinga alla tua libreria, vuoi l'MCP.

## Limiti noti

- **Sola lettura.** `pap` non crea, modifica o cancella prompt: per quello
  c'è l'app. È una scelta di sicurezza, non una mancanza.
- **Gli `{{import}}` non vengono espansi.** Se un prompt ne include altri,
  la CLI lascia l'`{{import}}` com'è e te lo segnala su `stderr`; per la
  composizione completa usa l'app desktop.
- **La ricerca è full-text**, non semantica: `pap search` trova per parole
  presenti nel testo. La ricerca per significato (embedding) vive nell'app.

## Vedi anche

- [`cli.md`](./cli.md) — il riferimento completo di comandi, flag e formati.
- [`mcp.md`](./mcp.md) — l'altra porta sul vault, per gli assistenti AI.
- [`glossario-sintassi.md`](./glossario-sintassi.md) — segnaposti e valori globali che `pap render` risolve.
- [`prompt-componibili.md`](./prompt-componibili.md) — gli `{{import}}` che invece richiedono l'app.
