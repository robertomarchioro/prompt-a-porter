# Analisi — Commenti nel prompt (issue #643)

> **Stato**: implementata — PR-1 #645 (motore + conformità), PR-2 #646 (editor), PR-3 (documentazione). Issue #643 chiusa il 2026-09-12; prova dal vivo ancora da fare.
> **Issue**: [#643](https://github.com/robertomarchioro/prompt-a-porter/issues/643) — «Possibilità di inserire commenti nel prompt».
> **Obiettivo utente**: annotare un prompt (perché una frase c'è, cosa provare, cosa non funziona) senza che l'annotazione finisca nel testo copiato o inviato al modello.
> **Data**: 2026-09-12

## 1. Sintesi

- **Strada 2 (commenti inline nel body), subito.** La strada 1 («revisioni di Word») ha bisogno di ancoraggi a intervalli di testo che sopravvivano alle modifiche, cioè di Ordito; è materia 2.0 e non è alternativa alla 2, è complementare.
- **Non `/// … ///`.** Delimitatore simmetrico (apertura = chiusura), ambiguo con più commenti, e collide con contenuto legittimo (`file:///`, doc-comment Rust in un blocco di codice incollato). Deciso: riusare la grammatica `{{ }}` che l'app già possiede, con la sola forma Mustache/Handlebars **`{{!-- commento --}}`** (può contenere `}}`).
- **Il lavoro vero non è l'editor, è la coerenza.** Il body oggi viene interpretato da **cinque parser in tre linguaggi** (TS client, Rust import/golden, Go CLI, TS MCP). Ognuno deve togliere i commenti *prima* di tutto il resto, con la stessa regola. Serve una **tabella di conformità condivisa** (fixture JSON letta dai test di tutti e tre i linguaggi), altrimenti divergono in silenzio.
- **Stima**: 3 PR, dell'ordine di 2 giornate; nessuna migrazione DB, nessun impatto su export/versioni/sync (il commento è testo del body).

## 2. Strada 1 vs strada 2

| | 1 — Commenti esterni (revisioni) | 2 — Commenti inline |
|---|---|---|
| Dove vive | Tabella nuova, ancorata a un intervallo del body | Nel body stesso |
| Sopravvive alle modifiche del testo | Solo con ancoraggi robusti (offset che si spostano, testo che sparisce) → serve l'oplog per-campo di Ordito o un diff-match-patch | Sempre (si muove col testo) |
| Versioni, export JSON, import, sync | Tutto da estendere (schema, formato export §`formato-export-json.md`, Ordito) | Zero: già coperti |
| CLI / MCP / Golden | Devono imparare una tabella in più | Devono imparare una regex in più |
| Team (2.0) | È la feature giusta: thread, autore, risolto/non risolto | Non copre il dialogo fra persone |
| Costo | Settimane | Giorni |

Conclusione: la 2 non preclude la 1. Quando arriverà il team, i commenti inline restano «note dell'autore nel sorgente» e le revisioni diventano «discussione sul sorgente» — due cose diverse anche in Word (commenti vs. note nel testo) e in ogni linguaggio di programmazione (commenti vs. code review).

## 3. Sintassi

### Perché non `/// … ///`

1. **Simmetria**: con lo stesso token in apertura e chiusura, `/// a /// b /// c ///` non dice se `b` è testo o commento; un `///` dimenticato inverte tutto il resto del prompt.
2. **Collisioni**: `file:///Users/…` è un URL valido; `///` è la doc-comment di Rust, e un prompt che contiene codice incollato è un caso d'uso primario di quest'app. Andrebbero escluse le fence ``` ``` ```, e a quel punto serve un parser Markdown in Go, Rust e TS.
3. **Terza grammatica**: oggi l'utente sa una regola sola — «tutto ciò che sta fra `{{ }}` non è testo, è una direttiva» (`{{nome}}`, `{{global nome}}`, `{{import "…"}}`). Un secondo meta-carattere raddoppia il glossario.

### Deciso: solo `{{!-- … --}}`

È la forma «blocco» dei commenti di Mustache e Handlebars, che è già il modello mentale della sintassi dei segnaposti (`{{nome}}`).

```
{{!-- Versione B del tono: da confrontare col golden "email-formale" --}}
Riscrivi il seguente testo in tono {{tono}}:

"{{testo}}"   {{!-- il testo arriva già senza firma --}}
```

- **Una sola forma**: chiude solo su `--}}`, quindi il commento può contenere `}}`, un segnaposto d'esempio (`{{nome}}`) o un import. È quella che inserisce il pulsante della toolbar.
- **La forma breve `{{! … }}` NON è riconosciuta** (decisione: una grammatica sola). Chi la digita ottiene un `PH003` (nome con caratteri non consentiti): va reso utile aggiungendo al messaggio un suggerimento «per un commento usa `{{!-- … --}}`» quando il nome inizia con `!`.
- **Multiriga**: sì (flag `s`/`(?s)`). Un commento non chiuso viene lasciato **inalterato** nel testo (comportamento identico ai segnaposti non compilati: si vede, non sparisce nulla di nascosto) e segnalato dal linter.
- **Regola della riga intera** (decisa): se un commento occupa una riga da solo (solo spazi attorno), sparisce **anche il newline**; altrimenti sparisce solo il commento. Senza questa regola, tre righe di commenti in testa al prompt lasciano tre righe vuote nel testo copiato.

Regex di riferimento (identica nei tre linguaggi):

```
\{\{!--[\s\S]*?--\}\}
```

### Alternative scartate

- **`<!-- … -->`**: Markdown-nativo, ma un prompt può legittimamente *contenere* un commento HTML da inviare al modello (es. «genera questo template HTML») e i tag XML-like sono idioma corrente nei prompt per Claude. Ambiguo.
- **`%% … %%`** (Obsidian): pulito, ma è la terza grammatica di cui sopra e non lo conosce nessuno fuori da Obsidian.
- **`# `/`//` a inizio riga**: `#` è un titolo Markdown, `//` compare nel codice. Solo-riga-intera limiterebbe comunque il caso «annotazione a fine riga».

## 4. Semantica: chi toglie i commenti, e chi no

Principio: **il commento non raggiunge mai un modello né gli appunti né un client esterno; resta in tutto ciò che è "sorgente"** (vault, versioni, export, ricerca). Unica eccezione voluta: il Ritocco. Applicato punto per punto:

| Consumatore | File | Decisione | Note |
|---|---|---|---|
| Compilazione client | `apps/client/src/lib/template.ts` | **Toglie** in `compila`, `estraiSegnaposti`, `contaCompilati` | Un `{{nome}}` citato dentro un commento non deve comparire nella form |
| Risoluzione import | `src-tauri/src/prompt_componibili.rs` | **Toglie** in testa a `compila_ricorsivo` (quindi anche nei figli) e in `parse_imports` | Un import commentato non va risolto, non crea dipendenza in `aggiorna_imports`, non blocca la cancellazione via `prompt_dipendenti` |
| Golden test | `src-tauri/src/regression.rs::compila_per_golden` | **Toglie** | Oggi non passa dal resolver import: va aggiunta la stessa chiamata |
| CLI `compila` | `apps/cli/main.go` | **Toglie** prima di `espandiGlobali`/import | Anche in `--list-vars`/nomi unici |
| MCP `pap_render` | `apps/mcp-server/src/lib/template.ts` | **Toglie** in `compila` ed `estraiSegnaposti` | |
| MCP `pap_get`, `pap_search`, `pap_list_recent` | `apps/mcp-server/src/index.ts` | **Toglie** (deciso: il livello MCP non espone mai i commenti) | Un solo helper `rimuoviCommenti` applicato al confine, su ogni body in uscita; per `pap_get` i segnaposti elencati sono quelli del body pulito |
| Anteprima | `components/AnteprimaTab.svelte` | **Toglie** (è l'anteprima di ciò che parte) | L'editor resta il posto dove si vedono |
| Ritocco AI | `src-tauri/src/ritocco.rs` | **Non toglie** (deciso) | Il diff che torna dal modello si applica al body: se il modello non vede i commenti, il diff li cancella. Effetto collaterale utile: i commenti diventano istruzioni per il revisore («qui voglio più conciso»). Da dire nella guida al Ritocco. |
| Embeddings / ricerca semantica | `src-tauri/src/editor.rs` → `compute_embedding_opt` | **Non toglie** | Il commento descrive l'intento: aiuta a ritrovare il prompt. Nessun backfill necessario. |
| Linter PII (`PII00x`) | `src-tauri/src/linting.rs` | **Non toglie** | Una carta di credito in un commento è comunque nel vault e nell'export |
| Linter lunghezza (`LEN00x`) e stile (`STY001`) | idem | **Toglie** | Misurano ciò che arriva al modello. |
| Stima costo (`pricing.rs::stima_costo`) | chiamata da `regression.rs` e `ritocco.rs` | Nulla da fare | Riceve il testo già compilato dal golden (pulito) o il body inviato al Ritocco (con commenti, per scelta): in entrambi i casi conta ciò che parte davvero. |
| CLI `get` | `apps/cli/main.go` | **Non toglie** | È il sorgente per un umano (come l'export); solo `render` compila. |
| Linter `PH003` | `regola_ph003_caratteri_speciali` | **Skip esplicito** di `{{!-- … --}}` come già per `import` | Oggi verrebbe segnalato come «caratteri non consentiti»: senza questo punto la feature nasce con un falso positivo. `{{! x }}` resta segnalato, con il suggerimento della forma giusta |
| Versioni, diff, export/import JSON, Cestino, Ordito | — | Nulla | Il commento è body |

Nuova regola linter proposta: **`CMT001`** (Warning) «Commento non chiuso: `{{!--` senza `--}}`» — è l'unico errore che l'utente può fare e che altrimenti si manifesta solo copiando il prompt.

## 5. Editor

- **Highlight**: un terzo `ViewPlugin` accanto a `placeholder-highlight.ts` e `import-tokens.ts` (`comment-highlight.ts`), mark `cm-commento`: colore `--text-muted`, corsivo, nessuno sfondo. Le decorazioni CodeMirror sono a intervalli, quindi il multiriga è gratis. Precedenza: il plugin commenti deve vincere su segnaposti/import quando si sovrappongono (un `{{nome}}` dentro un commento va grigio, non viola) — si ottiene registrando l'estensione prima e usando `Decoration.mark` con `inclusive` coerente, oppure facendo saltare ai due plugin esistenti gli intervalli commentati (più semplice e più robusto: esportare `_matchCommenti(testo)` e passarlo agli altri due).
- **Toolbar** (`MarkdownToolbar.svelte`): pulsante `MessageSquareDashed` (lucide) dopo «Inserisci import». Con selezione → la avvolge in `{{!-- … --}}`; senza → inserisce `{{!--  --}}` e mette il cursore in mezzo. Stesso pattern di `wrap()`.
- **Scorciatoia**: `Ctrl/Cmd+/` (convenzione universale «toggle comment»). Se la selezione è già interamente un commento, lo scommenta. Da registrare nel catalogo scorciatoie e nel tour.
- **Comando palette**: «Commenta/Scommenta selezione».

## 6. Il rischio principale: cinque parser

Oggi `{{global nome}}` è stato implementato quattro volte e `{{import}}` tre; le regex sono copiate a mano e **nessun test verifica che diano lo stesso risultato**. I commenti aggiungono la quinta copia. Proposta strutturale, da fare nella PR-1 perché è ciò che rende sicure le altre due:

- `packages/shared-schema/fixtures/template-conformance.json`: lista di `{ nome, body, valori, atteso }` che copre segnaposti, globali, import (dove il consumatore li supporta) e commenti — casi limite inclusi: commento non chiuso, `}}` dentro `{{!-- --}}`, commento su riga intera vs. inline, commento che contiene un segnaposto, commento che contiene un import, `{{!` senza spazio, commento a fine file senza newline.
- Test che la caricano in TS (`template.test.ts` client e MCP), Rust (`prompt_componibili`/`regression`) e Go (`main_test.go`). Un caso nuovo si aggiunge in un posto solo.
- La fixture include anche `{{! breve }}` con atteso = **inalterato** (non è sintassi valida): è l'assert che impedisce a un parser di «gentilmente» accettarla e divergere.

Senza questo passaggio la feature funziona nell'app e diverge in CLI/MCP alla prima modifica, e nessuno se ne accorge finché un utente non copia da CLI.

## 7. Piano

| PR | Contenuto | CI toccata |
|---|---|---|
| **PR-1 — motore + conformità** | Fixture condivisa; regex e stripping in `template.ts` (client), `prompt_componibili.rs`, `regression.rs`, `main.go`, MCP `template.ts` + confine `index.ts`; PH003 skip + suggerimento per `{{!`; `CMT001`; LEN/pricing sul body senza commenti. Test in tutti e tre i linguaggi. | client-test, rust, go, mcp |
| **PR-2 — editor** | `comment-highlight.ts` + esclusione intervalli dagli altri due plugin; pulsante toolbar; `Ctrl+/`; comando palette; Anteprima. | client-test |
| **PR-3 — documentazione** | `glossario-sintassi.md` (sezione «Commenti»), `linting-regole.md` (`CMT001`, nota PH003), `cli-ricette.md`, guida in-app (`aiuto/docs-links.ts`), description del tool MCP `pap_get`, nota nella guida Ritocco. | site |

Ordine obbligato: senza la PR-1 il pulsante della PR-2 produce un prompt che il CLI copia con i commenti dentro.

## 8. Decisioni chiuse (2026-09-12)

1. **Sintassi**: solo `{{!-- … --}}`. La forma breve `{{! }}` non è riconosciuta (una grammatica sola); il linter la indirizza alla forma giusta.
2. **Ritocco vede i commenti**: sì; il diff li preserva e diventano istruzioni per il revisore.
3. **MCP sempre pulito**: nessun tool MCP (`pap_get`, `pap_search`, `pap_list_recent`, `pap_render`) espone mai i commenti. Scostamento rispetto alla proposta iniziale (che teneva `pap_get` sorgente): il client MCP è un modello, vale il principio «mai a un modello».
4. **Fixture di conformità cross-linguaggio**: sì, nella PR-1.
5. **Riga intera → sparisce anche il newline**: sì.
