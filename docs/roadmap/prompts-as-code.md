# Idea strategica — "Prompts as Code" (storage + posizionamento)

> **Stato**: ideazione / da decidere. Nessun impegno preso, nessun codice.
> **Origine**: discussione 2026-06-03 sul dubbio "SQLite-only è troppo
> chiuso, ma il vault a file è tanto lavoro — se lo faccio, voglio qualcosa
> all'avanguardia che differenzi il prodotto".
> **Collegato a**: [`vault-a-cartella.md`](./vault-a-cartella.md) (il blueprint
> della modalità a file, che è un *prerequisito tecnico* di questa visione).

## La provocazione: "file vs SQLite" è l'asse sbagliato

Fare il vault a file *e basta* dà un "Obsidian per prompt": tanto lavoro per
una feature **me-too**. L'apertura da sola non differenzia (chiunque esporta in
markdown). Il differenziatore non è *dove* salvi i dati, è la **tesi di
prodotto** che lo storage abilita.

Realizzazione di fondo: **PaP è già al ~70% un "IDE per prompt engineering"**,
non un semplice gestore. Ha già:
- versioning, varianti, fork
- rating / qualità, golden test (regression)
- ricerca semantica (embeddings) + FTS
- composizione `{{import}}` con pinning `version=N`
- MCP server (prompt consumabili dagli agent)

Nessun gestore "normale" ha tutto questo. La vera domanda è: **vogliamo rendere
esplicita e ownable quell'identità?**

## La tesi: "Prompts as Code"

> PaP diventa il **primo ambiente di prompt engineering che tratta i prompt
> come vero codice sorgente**: versionati, diffabili, ramificabili, testabili,
> con dipendenze gestite — locale-first e in formato totalmente aperto
> (plain-text + git, zero lock-in).

Non "metti i file in git", ma usare **git come motore concettuale**, mappando
ciò che già esiste:

| Asset PaP esistente | Diventa in "Prompts as Code" |
|---|---|
| Cronologia versioni | **git history** → diff, blame, time-travel reali |
| Varianti / fork | **branch** → linee di esperimento, merge della vincente |
| Golden test / regression | **CI gate**: ogni modifica gira i golden, blocca il merge se la qualità regredisce |
| `{{import}}` componibili | **grafo di dipendenze**: impact analysis, rename propagato |
| Pinning `version=N` | **pin a git SHA**: riproducibilità esatta in produzione/agent |
| MCP server | agent che consumano prompt **versionati e testati**, pinnati a una revisione |

Risolve il lock-in **in modo definitivo**: git + markdown è il formato più
aperto e duraturo che esista. La decisione "sidecar vs git" del blueprint si
ribalta — **git diventa il titolo, non la nota a piè di pagina** (lo store
delle versioni È git).

## Perché è avanguardia (gap competitivo, 2026)

- Tool cloud (LangSmith, Langfuse, PromptLayer, Humanloop): versioning/eval ma
  **lock-in SaaS**.
- Tool a file (Obsidian & co.): aperti ma **senza rigore ingegneristico**.
- Eval tool (Promptfoo): testano ma **non gestiscono**.

**Nessuno fa l'unione: locale-first + formato aperto + rigore da software
engineering, usabile sia da umani che da agent.** È il vuoto che PaP può
occupare.

## I tre pilastri (in ordine di "wow")

1. **Diff / branch / blame sui prompt** — vedere *cosa* è cambiato tra due
   versioni con diff consapevole dei segnaposti e del grafo import (non diff
   testuale stupido). Da solo è una demo che fa "oh".
2. **Eval-gated changes ("CI per i prompt")** — salvi/mergi → girano i golden
   (+ opzionali chiamate reali al modello) → verde/rosso. Eval-driven
   development, locale.
3. **Riproducibilità per gli agent** — pin di un prompt (e del suo albero di
   import) a una revisione esatta. Risolve il *prompt drift* in produzione,
   dolore reale e attuale: "l'agente usa `code-review@a1b2c3`, riproducibile
   per sempre".

## Tensioni oneste

- **Editing esterno + merge automatico non si compongono bene.** I CRDT (vera
  frontiera local-first) vogliono operazioni strutturate; gli editor esterni
  danno snapshot di testo. **Git è il compromesso pragmatico**: merge manuale
  ma standard e potente. Il sogno "stesso prompt su 3 device che si fondono da
  soli senza server" è una **scommessa diversa** (CRDT), più costosa.
- **Rischio over-engineering per un single-user.** "CI per i prompt" è ottimo
  con decine di prompt in produzione; overkill con 20 prompt personali.
  Dipende da **chi è l'utente**.
- **È una scommessa di posizionamento**: "PaP è per chi fa prompt engineering
  sul serio", non "per chi tiene quattro prompt in ordine".

## MVP della visione (per non rischiare mesi al buio)

Primo slice che già differenzia, senza riscrivere lo storage:
- **Vault opzionale "a cartella git"** (la F2 del blueprint, ma con git come
  store delle versioni invece del sidecar) +
- **diff visuale prompt-aware tra due commit** (riusa il diff già esistente
  per le versioni) +
- **eval-gate al salvataggio** (riusa i golden test).

Tre cose già al ~70%, ricombinate sotto una bandiera nuova. Si differenzia
subito; SQLite resta come cache/indice (semantica + FTS intatte) → non si butta
via niente.

## Bet alternativi (se "Prompts as Code" non accende)

- **Local-first multi-device via CRDT** — "i tuoi prompt si sincronizzano tra i
  tuoi PC, offline, senza server e senza conflitti". Più sexy sul fronte
  local-first, ma confligge con l'editing esterno e costa di più.
- **Vault come data-source aperta via API/MCP** — no-lock-in *per interfaccia*
  invece che per formato: spec aperta + API locale/MCP con cui qualsiasi tool
  legge/scrive i prompt live (LangChain, script, altri editor). Meno "wow"
  visivo, ma fa di PaP un hub d'ecosistema.

## La domanda che decide la scommessa

Due variabili cambiano tutto:

1. **Per chi è PaP davvero?** Strumento personale da tenere aperto e bello,
   *oppure* prodotto da posizionare/vendere a chi fa prompt engineering serio?
2. **Quanti prompt** si gestiscono/immagina di gestire — decine, o
   centinaia/migliaia?

Da qui si capisce se la risposta è "Prompts as Code" in pieno, il suo MVP, o
uno dei due bet alternativi.

## Stato decisionale

- [ ] Definire utente target + scala (le due variabili sopra)
- [ ] Scegliere la scommessa: Prompts as Code (pieno) / MVP / CRDT / API-MCP
- [ ] Se Prompts as Code: ribaltare la decisione "versioni in sidecar" del
      blueprint → "versioni in git"
- [ ] Solo dopo: passare dal design all'implementazione (F1 resta il primo
      passo tecnico in ogni caso)
