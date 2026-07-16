# Blueprint — "Ritocco": suggerimenti AI per migliorare un prompt

> **Stato**: design / ideazione. Nessun codice ancora.
> **Revisione**: v1 (2026-07-16).
> **Obiettivo utente**: dal pannello di editing, chiedere a un LLM configurato
> dall'utente di **suggerire migliorie** al prompt corrente — tarate sul
> modello per cui il prompt è scritto — e poter **accettare** la versione
> riscritta salvandola come nuova versione (storico preservato).
> **Nome**: *Ritocco* — la modifica sartoriale, l'*alteration* che fa cadere
> bene il capo: piccoli interventi mirati che migliorano la vestibilità del
> prompt. In UI il bottone è **"Ritocco"**, la modale **"Ritocco — come
> migliorare il prompt"**.
> **Decisioni utente (2026-07-16)**: (1) nome *Ritocco*; (2) le "istruzioni
> ufficiali" sono **linee guida impacchettate** nell'app, non fetch live;
> (3) la modale mostra **consigli + riscrittura** con diff, e "Accetta"
> applica la riscrittura.
> **Gating**: abilitato solo se l'utente ha configurato le API key di almeno un
> provider LLM supportato (→ vault cifrato).

## Perché (contesto e stato dell'arte nel repo)

Il repo ha già quasi tutti i mattoni; *Ritocco* li orchestra, non li reinventa.

- **Provider AI configurabili**: tabella `ProviderConfig`
  (`apps/client/src-tauri/migrations/V010__provider_config.sql`), comandi
  `provider_config_lista/salva/elimina` in
  `apps/client/src-tauri/src/provider_ai.rs`. Provider supportati:
  `anthropic`, `openai`, `gemini`, `ollama`, `openai-compat`. Le API key sono
  custodite nel **vault cifrato** (SQLCipher); salvarle richiede vault cifrato
  (#456). UI in `PannelloProviderConfig.svelte`.
- **Astrazione di chiamata**: `trait AIProvider { fn generate(&self, prompt:
  &str, model: &str) -> Result<GenerateOutput, PapErrore> }`
  (`provider_ai.rs:55`), con impl per ogni provider e parser puri unit-testati
  (`parse_anthropic_response`, `parse_openai_response`, …). `GenerateOutput =
  { content, latency_ms, tokens_used, provider, model }`.
- **Template di orchestrazione**: il comando `golden_esegui`
  (`apps/client/src-tauri/src/regression.rs:734`) fa già esattamente il giro
  che serve — risolve provider+key dal vault via `config_carica_completa` +
  `istanzia_provider` (che ri-valida `base_url`, #457), chiama
  `provider.generate()`, gestisce l'errore del provider senza propagarlo.
  *Ritocco* ricalca questo comando.
- **Modello target del prompt**: il campo **esiste già**. `target_model`
  (colonna `TargetModel`) su `PromptDettaglio` (`libreria.rs:49`) e su ogni
  snapshot `VersioneStorica` (`versioning.rs:22`). I valori preset sono in
  `apps/client/src/lib/modelli-target.ts`, che porta **già** un campo
  `famiglia: "anthropic" | "openai" | "google" | "meta" | "altro"`. È
  **facoltativo** (stringa vuota / `None` se non impostato).
- **Versioning**: `prompt_aggiorna` (`editor.rs:264`) con `crea_snapshot: true`
  bumpa `Prompts.Version` e scrive uno snapshot in `PromptVersions`.
  `prompt_rollback` (`versioning.rs:185`) è il precedente per "applica un
  contenuto come nuova versione più recente preservando lo storico".
- **Modale**: primitiva `Modale.svelte` (focus-trap, ESC, ARIA, scroll-lock già
  a posto). Le modali "azione da header" (es. `CompilaModal`) sono cablate
  via store globale `stores/modale.svelte.ts` + `Shell.svelte`.
- **Costo/diff**: `pricing.rs` (`stima_costo`), `DiffViewer.svelte`.

**Il buco che Ritocco riempie**: oggi nessun comando prende il *body* di un
prompt, lo dà in pasto a un LLM per migliorarlo e riscrive il risultato come
nuova versione. `golden_esegui` valuta un output contro un atteso; non riscrive
il prompt.

## Vincolo architetturale chiave

Il **webview non può fare fetch esterni**: CSP `default-src 'self'`
(`tauri.conf.json:37`), capabilities senza plugin `http`/`shell`. Solo il
backend Rust fa rete (via `ureq`: provider AI + download modello embeddings).
→ Per questo le linee guida sono **impacchettate** (decisione utente),
eliminando del tutto il problema fetch/parsing/SSRF.

## Architettura — flusso dati

```
[DetailPane] header, bottone "✦ Ritocco" accanto a "Compila"
  │  (disabilitato + tooltip se: 0 provider abilitati OR vault non cifrato)
  ▼
[RitoccoModal]  (store globale: { tipo:"ritocco"; promptId })
  ├─ se >1 provider abilitato: picker provider + modello (come GoldenTab)
  ├─ se target_model vuoto: picker famiglia (anthropic/openai/google/generico)
  └─ "Avvia Ritocco"
        ▼
   invoke("ritocco_esegui", { promptId, providerKind, model, baseUrl? })
        ▼  (Rust, riusa il pattern golden_esegui)
   1. carica Body + target_model del prompt (carica_prompt_body / SELECT)
   2. target_model → famiglia → seleziona guida impacchettata
   3. compone il meta-prompt: [guida ufficiale] + [prompt utente] +
      istruzione "restituisci SOLO JSON {suggerimenti[], prompt_migliorato}"
   4. risolve provider (config_carica_completa + istanzia_provider) e chiama
      provider.generate(meta_prompt, model)
   5. parse difensivo del JSON → RitoccoEsito
        ▼
   RitoccoEsito { suggerimenti[], prompt_migliorato, tokens, costo, provider, model }
        ▼
[RitoccoModal] mostra:
   - elenco suggerimenti (titolo + dettaglio: cosa/perché)
   - diff (Body attuale vs prompt_migliorato) via DiffViewer
   - [Accetta suggerimenti]  [Annulla]
        ▼ Accetta
   invoke("prompt_aggiorna", { dati: { id, titolo, descrizione, body:
     prompt_migliorato, target_model, folder_id, crea_snapshot: true, … } })
   → nuova versione (Version+1) col testo riscritto; target_model invariato;
     storico preservato. La modale si chiude, il DetailPane ricarica.
```

Nota: l'astrazione `AIProvider::generate` prende **un solo messaggio utente**
(`messages:[{role:"user",…}]` per Anthropic/OpenAI); non c'è ruolo *system*.
Quindi guida + prompt + istruzione di formato vanno assemblati in **un'unica
stringa** `meta_prompt`. (Evoluzione futura opzionale: aggiungere un ruolo
system all'astrazione; fuori scope qui.)

## §1 — Guide impacchettate

Testi curati da noi, **embeddati in Rust** via `include_str!` così restano
server-side e offline. Una guida per famiglia:

```
apps/client/src-tauri/src/ritocco/guide/
  anthropic.md    # best practice prompting Claude (XML tags, ruoli, esempi…)
  openai.md       # best practice GPT (delimitatori, struttura, few-shot…)
  google.md       # best practice Gemini
  generico.md     # principi trasversali, per famiglie senza guida dedicata
```

**Mapping `target_model → famiglia → guida`** (in Rust, specchia
`modelli-target.ts`), con fallback robusto:

| `famiglia` (da modelli-target) | guida        |
|--------------------------------|--------------|
| `anthropic`                    | anthropic.md |
| `openai`                       | openai.md    |
| `google`                       | google.md    |
| `meta` (llama)                 | generico.md  |
| `altro` / vuoto / sconosciuto  | generico.md  |

Poiché `target_model` in DB è una stringa libera normalizzata, la mappa Rust
lavora sul **prefisso/valore** (es. `claude-*`→anthropic, `gpt-*`→openai,
`gemini-*`→google, `llama-*`/`generic`/vuoto→generico). Se la modale ha fatto
scegliere una famiglia (target vuoto), quella prevale.

**Contenuto**: sintetico e azionabile (checklist + do/don't + 1-2 esempi
brevi), non un dump della documentazione. Mantenuto da noi; nota in testa a
ogni file con data di ultima revisione e link alla fonte ufficiale (come
riferimento umano, non come endpoint).

## §2 — Comando Rust `ritocco_esegui`

Nuovo modulo `apps/client/src-tauri/src/ritocco.rs`, registrato nel
`generate_handler!` di `lib.rs`.

```rust
#[tauri::command]
fn ritocco_esegui(
    prompt_id: String,
    provider_kind: String,     // provider AI che ESEGUE il ritocco (con key)
    model: String,
    base_url: Option<String>,  // override solo per ollama, come golden_esegui
    state: State<'_, VaultState>,
) -> Result<RitoccoEsito, PapErrore>
```

Passi (dentro `state.with_conn`):
1. `carica_prompt_body(conn, &prompt_id)` (già esiste, `regression.rs:342`) +
   lettura `TargetModel`.
2. `famiglia_da_target(&target_model)` → `guida_per_famiglia(fam)` (`&'static
   str` da `include_str!`).
3. `componi_meta_prompt(&guida, &body)` → stringa (funzione **pura**, testabile).
4. Risoluzione provider identica a `golden_esegui:745-767`: se
   `provider_kind=="ollama"` e `base_url` non vuoto → `OllamaProvider::new`;
   altrimenti `config_carica_completa` + `istanzia_provider` (key dal vault +
   `valida_base_url`).
5. `provider.generate(&meta_prompt, &model)` → `GenerateOutput`.
6. `parse_esito_ritocco(&out.content)` (funzione **pura**, testabile) →
   `RitoccoEsito`. Arricchisce con `tokens_used`, `stima_costo(...)`,
   `provider`, `model`.

**Schema I/O JSON** richiesto al modello (nel meta-prompt):
```json
{
  "suggerimenti": [
    { "titolo": "…", "dettaglio": "… cosa e perché …" }
  ],
  "prompt_migliorato": "…testo completo del prompt riscritto…"
}
```

**`RitoccoEsito`** (serde snake_case, restituito alla UI):
```rust
struct RitoccoEsito {
    suggerimenti: Vec<Suggerimento>,   // { titolo, dettaglio }
    prompt_migliorato: String,
    tokens_used: Option<u32>,
    costo_stimato: Option<f64>,
    provider: String,
    model: String,
    troncato: bool,   // euristica: output vicino al max_tokens
}
```

**Parsing difensivo** (`parse_esito_ritocco`): rimuove eventuali fence
```` ```json … ``` ````, isola il primo `{` … ultimo `}`, `serde_json::from_str`.
Se il parse fallisce → `PapErrore` con messaggio utente ("Il modello non ha
restituito un risultato interpretabile, riprova") **oppure** fallback che
mette il testo grezzo come singolo suggerimento e lascia `prompt_migliorato`
vuoto → la modale disabilita "Accetta" (niente testo da applicare).

⚠️ **`max_tokens`**: `AnthropicProvider` cabla `max_tokens: 4096`
(`provider_ai.rs`). Un prompt lungo riscritto **può troncarsi**. Interventi
previsti: (a) alzare il tetto per questa chiamata (costante dedicata o
parametro), (b) impostare `troncato=true` quando l'output è a ridosso del tetto
o il JSON risulta incompleto, e mostrarlo in modale. Da decidere in fase 2 se
rendere `max_tokens` parametro di `generate` (cambio di firma dell'astrazione)
o costante più alta.

## §3 — Modale `RitoccoModal.svelte`

In `apps/client/src/lib/superfici/`, cablata come `CompilaModal`:
- Aggiungere `{ tipo: "ritocco"; promptId: string }` a `ModaleAttiva` in
  `stores/modale.svelte.ts` e il ramo in `Shell.svelte`.
- Trigger: bottone `.primary`/`.ico` con icona lucide (`Wand2`/`Sparkles`,
  `title="Ritocco"`) nel cluster `.actions` di `DetailPane.svelte` (~riga 660),
  `onclick={() => apriModale({ tipo:"ritocco", promptId })}`.

Stato (`$state`, idioma GoldenTab) — attenzione al caveat `untrack`/no
self-write in `$effect` (#170):
```
{ fase: "scelta" | "esecuzione" | "risultato" | "errore",
  providerScelto, modelloScelto,        // picker se >1 provider
  famigliaScelta,                        // picker se target_model vuoto
  caricamento: boolean, errore: string,
  esito: RitoccoEsito | null }
```
- **fase "scelta"**: se un solo provider abilitato e target_model presente,
  saltabile (avvio diretto). Picker provider/modello come
  `caricaProviderAbilitati()` di GoldenTab.
- **fase "esecuzione"**: spinner + testo "Il sarto è al lavoro…"; disabilita
  tutto. Chiamata `invoke("ritocco_esegui", …)`.
- **fase "risultato"**: elenco `suggerimenti` (titolo in grassetto + dettaglio),
  `DiffViewer` Body-attuale vs `prompt_migliorato`, riga costo/token, badge
  "output troncato" se `troncato`. Footer: **[Accetta suggerimenti]**
  (disabilitato se `!prompt_migliorato`) + **[Annulla]**.
- **fase "errore"**: messaggio (`String(e).replace(/^Error: /,"")`) + [Riprova].

**Accetta** → `invoke("prompt_aggiorna", { dati: { …campi correnti…, body:
esito.prompt_migliorato, crea_snapshot: true } })`; poi chiude e fa ricaricare
il DetailPane (nuova versione visibile in CronologiaTab).

**a11y**: ereditata da `Modale.svelte`. Poiché i risultati arrivano async,
spostare il focus su "Accetta" al passaggio in fase "risultato" (l'autofocus
iniziale scatta prima che i dati arrivino). Righe/elementi interattivi = veri
`<button>`.

**i18n**: stringhe **inline in italiano** (nel repo l'i18n non è cablato).
Bozza: bottone `Ritocco`; titolo `Ritocco — come migliorare il prompt`;
`Il sarto è al lavoro…`; `Accetta suggerimenti`; `Annulla`; `Riprova`;
`Output forse troncato: prompt molto lungo`.

## §4 — Applicazione come nuova versione

Riuso puro di `prompt_aggiorna` con `crea_snapshot: true` — **nessun comando di
versioning nuovo**. Il body diventa `prompt_migliorato`, `target_model`
invariato, `Version+1`, snapshot in `PromptVersions`. Coerente con la
descrizione utente ("lascia come ultima versione quello suggerito dal modello").

## §5 — Gating

Requisito: **≥1 provider `Abilitato`** (`provider_config_lista`) **E vault
cifrato** (`vault_cifrato_impl`). `ollama` conta come provider abilitato anche
senza key (gira in locale) — quindi il gate è "almeno un provider abilitato",
ma la UI di config impedisce già di salvare key su vault non cifrato (#456).
- Bottone **visibile ma disabilitato** con `title` esplicativo quando il
  requisito manca (es. "Configura un provider AI nelle Impostazioni per usare
  Ritocco"). Evita un bottone che appare/sparisce.
- La modale rilegge lo stato provider all'apertura (nessuna cache stantia).

## §6 — Edge case ed errori

- **`target_model` vuoto** → picker famiglia in modale (default `generico`).
- **Nessun provider abilitato / vault non cifrato** → bottone disabilitato
  (§5); la modale non si apre.
- **Errore provider** (rete, key errata, 4xx/5xx) → fase "errore" con messaggio
  scrubato (`PapErrore` già ripulisce i segreti).
- **JSON non parsabile / `prompt_migliorato` vuoto** → §2 fallback; "Accetta"
  disabilitato.
- **Output troncato** (prompt lungo) → badge + possibilità di riprovare;
  eventuale nota di alzare il modello o accorciare il prompt.
- **Prompt cancellato/spostato durante la chiamata** → `carica_prompt_body`
  filtra `DeletedAt IS NULL`; l'aggiornamento fallisce con errore chiaro.
- **Costo**: mostrato stimato prima? No — la stima precisa dipende dai token di
  output. Mostriamo **costo/token effettivi a posteriori** (post-chiamata),
  come fa la regression. (Se in futuro si vuole un preventivo, `pricing.rs` può
  stimare sui soli token di input.)

## §7 — Sicurezza

- La **guida** è contenuto fidato (nostro, embeddato). Il **body** del prompt è
  contenuto utente e viene inviato al provider — stessa esposizione già
  accettata per i golden test; nessun dato nuovo esce.
- La **risposta** del modello (prompt riscritto) è **solo mostrata in diff e
  applicata su conferma esplicita** dell'utente: non viene eseguita né passata
  a tool. Nessun rischio di prompt-injection attivo (diverso dal caso MCP #462,
  dove l'output tornava al modello).
- Nessuna **nuova superficie di rete**: nessun fetch, tutto passa dai provider
  già configurati e validati (`valida_base_url`, #457).
- Le **API key** restano nel vault; risolte solo dentro il comando, mai verso
  il frontend (come `config_lista_pure`).

## §8 — Fix drift migration `gemini`

Pre-esistente, indipendente ma da chiudere in coda: il `CHECK` di
`V010__provider_config.sql` elenca `('anthropic','openai','ollama',
'openai-compat')` ma **non `gemini`**, pur essendo supportato in codice/UI.
→ micro-migration `V0xx` che allinea il vincolo (o lo rilassa). Necessario se
si vuole usare Gemini come provider *esecutore* di Ritocco.

## §9 — Test (TDD dove ha senso)

Backend (Rust, come i `parse_*` esistenti — funzioni pure senza server):
- `famiglia_da_target`: mapping di tutti i valori di `MODELLI_TARGET` + vuoto +
  sconosciuto → guida attesa.
- `componi_meta_prompt`: contiene guida + body + istruzione JSON.
- `parse_esito_ritocco`: JSON pulito; JSON con fence markdown; JSON con testo
  intorno; JSON malformato → errore/fallback; `prompt_migliorato` mancante.
- euristica `troncato`.

Frontend: smoke della modale (fasi, disabilitazioni, accept che invoca
`prompt_aggiorna` con `crea_snapshot:true`) — nei limiti del setup di test
attuale del client.

## §10 — Fasi di implementazione

1. **Guide + mapping**: file `guide/*.md`, `famiglia_da_target`,
   `guida_per_famiglia`, `componi_meta_prompt` (+ unit test). Nessuna UI.
2. **Comando** `ritocco_esegui` + `RitoccoEsito` + `parse_esito_ritocco`
   (+ unit test parser); gestione `max_tokens`/`troncato`; registrazione in
   `lib.rs`.
3. **Modale + trigger + gating**: `RitoccoModal`, voce store, ramo `Shell`,
   bottone in `DetailPane`, picker, loading/errore, diff.
4. **Applica → nuova versione** (riuso `prompt_aggiorna`), fix migration
   `gemini` (§8), voce CHANGELOG (v0.8.38).

## §11 — Punti aperti

- **`max_tokens`**: costante più alta dedicata a Ritocco, o parametro di
  `AIProvider::generate` (cambio firma su tutte le impl)? → deciso in fase 2.
- **Guida "meta"/Llama**: per ora → `generico.md`. Serve una `meta.md`
  dedicata? (rimandabile.)
- **Ruolo system**: l'astrazione manda un solo messaggio utente; separare
  system/user migliorerebbe la resa ma è un refactor dell'astrazione
  (fuori scope v1).
- **Preventivo costo** pre-chiamata: per ora solo consuntivo (§6).
