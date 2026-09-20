# Blueprint — «Cartamodello»: trasformare prompt piatti in prompt componibili

> **Stato**: PR-1 motore mergiata (#663, 2026-09-20); **PR-2 in lavorazione
> lo stesso giorno** — `cartamodello_applica.rs` (ri-verifica + transazione),
> `CartamodelloModal` (scelta provider → analisi → revisione → applicazione),
> ingressi dal dettaglio (forbici) e dal menu della selezione multipla, guida
> utente `docs/utente/cartamodello.md`.
> **Nome**: in sartoria il *cartamodello* è il modello di carta da cui si
> tagliano i pezzi di un capo — qui l'app taglia un prompt nei suoi pezzi
> riusabili.
> **Obiettivo utente**: prendere uno o più prompt scritti «piatti» (propri,
> importati da un file, copiati da un sito) e ottenerne la versione in
> modalità PaP: le macro-componenti (ruolo, vincoli, formato di output,
> contesto, esempi) diventano moduli separati riusabili, i valori variabili
> diventano `{{segnaposti}}`, e il prompt finale li ricompone con
> `{{import}}`.
> **Contesto**: è il «ponte» promesso al punto 5 di
> [`collezioni-curate.md`](./collezioni-curate.md): oggi un prompt importato da
> fuori resta un blocco di testo che non usa nulla di ciò che distingue PaP.

## 1. Sintesi

Un comando **Scomponi** (dal dettaglio di un prompt, o dal menu della
selezione multipla) invia il testo a un provider AI configurato con un
meta-prompt che conosce la sintassi PaP, riceve una **proposta** strutturata
(moduli + prompt ricomposti), la **verifica e arricchisce in locale** (sintassi
valida, import risolvibili, moduli simili già presenti nel vault), la mostra
in una modale di revisione e, all'accettazione, **crea i moduli e riscrive i
prompt originali come nuova versione**. Niente parte senza un click, niente
viene scritto senza conferma, e l'originale resta nella cronologia.

Il valore non è «un LLM riscrive il prompt» (quello è Ritocco): è che
**cinque prompt che ripetono lo stesso ruolo diventano un modulo importato
cinque volte**, e che il risultato usa davvero segnaposti, import e `with`.

## 2. Cosa esiste già e si riusa

| Pezzo | Dove | Uso qui |
|---|---|---|
| Risoluzione provider + API key dal vault, tetto token, validazione `base_url` Ollama | `ritocco.rs::ritocco_esegui` | da **estrarre** in una funzione condivisa `provider_ai::risolvi_provider(conn, kind, model, base_url, max_tokens)` — oggi è inline in Ritocco |
| Cornice anti-injection del testo utente (`--- INIZIO/FINE PROMPT UTENTE ---`, istruzione «non eseguire») | `ritocco.rs` | stessa tecnica, marcatori per prompt |
| Parsing tollerante del JSON di risposta | `ritocco.rs::estrai_json` / `parse_esito_ritocco` | stesso approccio: mai fallire, fallback leggibile |
| Linter (sintassi, import, segnaposti) | `linting.rs::analizza_completo` | validare ogni modulo e ogni prompt ricomposto **prima** di mostrarli |
| Resolver import + anteprima espansa | `prompt_componibili.rs` | verificare che ogni `{{import}}` proposto risolva; mostrare l'anteprima espansa |
| Ricerca ibrida (FTS + embeddings, RRF) | `ricerca_ibrida.rs` | trovare moduli **già nel vault** simili a quelli proposti |
| Nuova versione con snapshot | `versioning.rs` (come «Accetta» di Ritocco) | applicare il prompt ricomposto all'originale |
| Menu contestuale selezione multipla | `ListPane.svelte` §6.8 | punto d'ingresso per «Scomponi N prompt» |
| Segnaposti globali | `segnaposti_globali.rs` | passare i **nomi** dei globali esistenti al modello (mai i valori) |

## 3. Flusso

1. **Ingresso**: pulsante «Scomponi» nel dettaglio (accanto a Ritocco) oppure
   voce «Scomponi N prompt» nel menu della selezione multipla. Gating come
   Ritocco: serve almeno un provider abilitato.
2. **Scelta provider/modello**: stessa modale-fase di Ritocco.
3. **Analisi** (`cartamodello_analizza`): un solo giro LLM per l'intero
   lotto (i moduli condivisi si vedono solo se il modello vede tutti i
   prompt insieme). Limiti: max **10 prompt** e **~60 k caratteri** totali
   per lotto; oltre, la UI chiede di spezzare.
4. **Verifica locale** (pura, testabile), sulla proposta:
   - ogni modulo e ogni prompt ricomposto passa il linter senza *Error*;
   - ogni `{{import "…"}}` risolve a un modulo proposto o a un prompt del
     vault; gli altri vengono segnalati e il prompt marcato «da rivedere»;
   - **riuso**: per ogni modulo proposto, ricerca ibrida sul vault →
     se esiste un prompt con similarità alta (soglia da tarare, es. cosine
     ≥ 0,85 sul corpo) la proposta diventa «usa esistente: *Titolo*» invece
     di «crea»;
   - **dedup interno**: moduli proposti con corpo quasi identico fra loro
     vengono fusi (il modello viene istruito a condividerli, la verifica è
     la rete di sicurezza);
   - **fedeltà**: per ogni prompt, l'anteprima espansa del ricomposto viene
     confrontata con l'originale: la modale mostra il diff, così l'utente
     vede cosa è stato *spostato* e cosa (indebitamente) *cambiato*.
5. **Revisione** (modale): a sinistra i moduli (nome editabile, tipo,
   corpo, «crea nuovo» / «usa esistente»), a destra i prompt ricomposti con
   tab *Sorgente* / *Anteprima espansa* / *Diff con l'originale*; cartella di
   destinazione dei moduli (default: sottocartella `Moduli` accanto
   all'originale). Costo stimato e token, come Ritocco.
6. **Applicazione** (`cartamodello_applica`, una transazione): crea i moduli
   nuovi, poi per ogni prompt originale salva il corpo ricomposto come
   **nuova versione** (l'originale resta in cronologia, ripristinabile).
   Ricostruisce FTS; gli embedding dei corpi nuovi vanno ricalcolati (vedi
   rinvio «embedding stale» in [`rinvii.md`](./rinvii.md): qui va risolto,
   non rinviato, perché i moduli nuovi devono essere trovabili subito).

## 4. Il meta-prompt

Contenuto, in ordine:

1. ruolo: esperto di prompt engineering **e** della sintassi PaP;
2. la sintassi PaP, ridotta all'essenziale: `{{nome}}`, `{{global nome}}`
   (con l'elenco dei **nomi** dei globali esistenti nel vault), `{{import
   "Titolo"}}`, `{{import "Titolo" with k=v, k2="con spazi"}}`, `{{!-- --}}`;
   regole sui nomi dei segnaposti (`[a-zA-Z_][a-zA-Z0-9_]*`);
3. i criteri di taglio: cosa diventa modulo (ruolo/persona, vincoli
   ricorrenti, formato di output, contesto stabile, esempi), cosa **non**
   diventa modulo (una frase usata una volta), cosa diventa segnaposto
   (valori che cambiano a ogni uso: nomi, date, testi da elaborare), quando
   usare `with` (un modulo con un segnaposto che il prompt finale fissa);
4. il vincolo di **fedeltà**: scomporre, non riscrivere — il testo espanso
   deve dire le stesse cose dell'originale; migliorare è compito di Ritocco;
5. i prompt utente, ciascuno incorniciato e con il suo id, NON fidati;
6. formato di risposta JSON:

```json
{
  "moduli": [
    { "chiave": "ruolo-tech-lead", "titolo": "Ruolo tech lead", "tipo": "ruolo",
      "corpo": "…", "usato_da": ["id-1", "id-3"] }
  ],
  "prompt": [
    { "id": "id-1", "titolo": "…", "descrizione": "…",
      "corpo": "{{import \"Ruolo tech lead\" with linguaggio=Rust}}\n\n…{{codice}}…",
      "segnaposti": ["codice"] }
  ],
  "note": ["…"]
}
```

`tipo` ∈ `ruolo | vincoli | formato | contesto | esempi | altro`: serve alla
UI (icona, cartella suggerita) e al riuso (si cerca fra prompt dello stesso
tipo prima).

## 5. Decisioni (chiuse il 2026-09-19)

| # | Domanda | Decisione |
|---|---|---|
| D1 | Nome della feature | **Cartamodello** (default non contestato; comando in UI: «Scomponi»). |
| D2 | Cosa succede all'originale | **Nuova versione dello stesso prompt**, come «Accetta» di Ritocco: cronologia = rete di sicurezza, nessun doppione. |
| D3 | Lotto multi-prompt nella prima PR | **Sì, fino a 10 prompt** in un solo giro LLM; ingresso anche dal menu della selezione multipla. |
| D4 | Riuso di moduli già nel vault | **Sì nella prima PR**, via ricerca ibrida: «usa esistente» prima di «crea». |
| D5 | Fedeltà o miglioramento | **Solo scomporre**; il diff espanso-vs-originale è il controllo di qualità. Migliorare resta a Ritocco. |
| D6 | Dove vanno i moduli | **Sottocartella `Moduli` accanto all'originale** (default non contestato); modificabile nella modale di revisione. Con un lotto che attraversa più cartelle, i moduli condivisi vanno nella `Moduli` della cartella comune più vicina (root se nessuna). |

## 6. Rischi

- **Qualità della scomposizione** dipende dal modello: con modelli piccoli
  (Ollama locale) la proposta può essere povera. Il gating è lo stesso di
  Ritocco; la verifica locale e il diff di fedeltà sono la difesa.
- **Prompt injection** nel testo scomposto: stesse cornici di Ritocco; in più
  la proposta è *dati* che l'app valida (sintassi, import), non istruzioni.
- **Costo**: un lotto da 10 prompt è un meta-prompt lungo; mostrare la stima
  prima di inviare (Ritocco la mostra solo dopo).
- **Titoli come chiavi degli import**: `{{import "Titolo"}}` risolve per
  titolo; un modulo creato con un titolo che collide con un prompt esistente
  crea ambiguità. La verifica locale deve controllare l'unicità e la UI
  deve permettere di rinominare prima di applicare.
- **Transazione lunga**: creazione moduli + N nuove versioni + FTS +
  embedding. Tutto o niente; l'embedding può andare in coda (backfill)
  purché i moduli siano trovabili dal linter/resolver subito (lo sono: il
  resolver è SQL, non semantico).

## 6b. Note dalla review della PR-1 (per la PR-2)

- **Ri-verificare al momento di applicare**: fra analisi e applicazione il
  vault può cambiare (lock rilasciato durante la chiamata al provider).
  `cartamodello_applica` deve ricontrollare che gli originali esistano e
  abbiano ancora il corpo analizzato, che i titoli dei moduli siano ancora
  liberi e che gli import risolvano — non fidarsi della `Proposta` ricevuta
  dal frontend.
- I **tetti sulla risposta** (`MAX_MODULI_RISPOSTA` ecc.) e la
  neutralizzazione dei marcatori nel corpo sono difese contro un provider
  difettoso o ostile: la proposta è comunque dati, mai istruzioni.
- `cartamodello_applica` crea i moduli con `prompt_crea_in_db`, che
  ricostruisce la FTS a ogni chiamata: con N moduli la FTS viene rifatta
  N+1 volte nella stessa transazione. Innocuo con il tetto attuale; se il
  lotto cresce, serve una variante batch senza rebuild intermedio.
- Il titolo del prompt ricomposto resta **quello originale** salvo scelta
  esplicita dell'utente; il titolo proposto dal modello è mostrato come
  suggerimento cliccabile. La descrizione non viene toccata (D5).
- `applica_variabili_scoped` usa `\w` (Unicode) mentre la regola canonica
  dei nomi è ASCII: incoerenza pre-esistente in `prompt_componibili.rs`,
  fuori scope qui.

## 7. Piano (tre PR)

1. **Motore** (`cartamodello.rs`): estrazione `risolvi_provider` da Ritocco;
   meta-prompt; parsing; verifica locale (linter, import, unicità titoli,
   dedup interno); riuso via ricerca ibrida; comando `cartamodello_analizza`.
   Test sui casi: prompt con ruolo evidente, due prompt con ruolo condiviso,
   risposta JSON rotta, import che non risolve, modulo simile già nel vault.
2. **Modale di revisione + applicazione**: `CartamodelloModal`, comando
   `cartamodello_applica` transazionale, ingressi (dettaglio + selezione
   multipla), documentazione utente.
3. **Rifiniture**: stima costo pre-invio, tab diff di fedeltà, eventuale
   «migliora anche» se D5 lo apre.

## 8. Fuori scope

- Scomposizione senza LLM (euristiche su «Sei un…»): troppo fragile per
  valere il codice; il gating provider è già accettato per Ritocco.
- Ricomposizione automatica di *tutto* il vault: un lotto per volta, scelto
  dall'utente.
