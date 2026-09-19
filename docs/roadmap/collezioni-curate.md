# Collezioni curate vs import da siti di condivisione

> **Stato**: decisione di rotta (2026-09-19) → **prima PR in lavorazione lo
> stesso giorno** (backend `collezioni.rs`, card in Impostazioni → Dati, due
> collezioni). Decisioni aperte chiuse in fondo al documento.
> **Domanda**: per rendere PaP più accattivante, conviene offrire liste di
> prompt curate da scaricare, oppure l'import da siti di condivisione e
> archivio prompt?
> **Risposta breve**: **collezioni curate**, con un import generico ridotto
> all'osso. Nessun connettore per singolo sito.

## Punto di partenza: cosa c'è già nel codice

Le due strade non partono dalla stessa riga.

| Pezzo | Dove | Stato |
|---|---|---|
| Formato vault JSON v1 (tag, cartelle, prompt, globali, versioni) | `import_export.rs` (`ExportV1`) | ✅ |
| Import JSON con modalità `skip` / `overwrite` / `rename` | `vault_import_json` | ✅ |
| Un catalogo curato già esistente (17 prompt, 7 cartelle, 8 tag, varianti, fork, cronologia, globali) | `docs/demo/demo-vault.json` | ✅ importato all'onboarding (#284) |
| Import Markdown bulk con front-matter Obsidian/Foam | `vault_import_markdown_bulk` | ✅ |
| Regola «dati esterni = registro in sorgente aggiornato da workflow, mai chiamata a runtime dal client» | `modelli-registro.json` + `modelli-refresh.yml` | ✅ consolidata |

La strada A (collezioni) è infrastrutturalmente all'80%: manca il contenuto e
una UI di scelta. La strada B (siti) parte da zero.

## Strada B — import da siti di condivisione

«I siti» non sono una cosa sola. Censimento delle fonti plausibili:

| Fonte | Accesso | Licenza | Formato |
|---|---|---|---|
| awesome-chatgpt-prompts (GitHub) | CSV pubblico | CC0 | `act,prompt` |
| Fabric patterns (danielmiessler/fabric) | cartelle `.md` su git | MIT | `system.md` per pattern |
| Anthropic / OpenAI prompt library | HTML doc, nessuna API | proprietaria, uso documentale | — |
| PromptBase | marketplace a pagamento | scraping vietato | — |
| FlowGPT, PromptHub, LangChain Hub | API con chiave o ToS restrittivi | variabile | ognuno il suo |

Contro, in ordine di peso:

1. **Un adapter per fonte**, da mantenere: si rompe quando il sito cambia,
   e i siti di prompt cambiano o chiudono spesso.
2. **Le fonti chiuse non si possono toccare** (ToS, licenza). Le fonti
   aperte sono file su git → **già importabili oggi** clonando il repo e
   usando l'import Markdown bulk.
3. **Contenuto non verificato** che finisce nel LLM dell'utente con la
   *sua* chiave API: qualità random, prompt injection, quasi tutto in inglese.
4. **Arrivano come testo piatto**: senza `{{segnaposti}}`, `{{global}}`,
   `{{import}}`, varianti, commenti. Non vendono il prodotto, lo
   appiattiscono a un blocco note.
5. **Chiamate di rete dal client** contro la regola di progetto (registro in
   sorgente, non runtime).

## Strada A — collezioni curate

Costo quasi tutto nel **contenuto**, non nel codice.

Pro:

- qualità controllata, licenza chiara (contenuti originali del progetto,
  come il demo vault), italiano;
- offline, distribuibili come gli altri asset (bundlate o firmate in release);
- ogni prompt può **mostrare le feature che nessun sito ha**: composizione con
  `{{import}}`, segnaposti, globali, varianti, commenti `{{!-- --}}`;
- il brand calza: *Prompt à Porter* → **Collezioni** per stagione, in linea con
  [`stagioni-e-nomi-rilascio.md`](./stagioni-e-nomi-rilascio.md).

Contro, onesti:

- catalogo piccolo rispetto alle migliaia di prompt online;
- qualcuno deve curarlo e tenerlo fresco: **se non c'è chi scrive, A muore di
  inedia dopo la prima collezione**;
- rischio percezione «giardino recintato».

## Decisione di rotta

**A, con una B ridotta all'osso.**

1. **Collezioni curate** in `docs/collezioni/*.json`, stesso formato del demo
   vault (così i test `demo_vault_importa_pulito` si estendono a costo zero).
   Una per persona-tipo — sviluppatore, chi scrive, chi insegna, chi analizza
   dati — 10–20 prompt l'una, costruite apposta per far vedere composizione e
   segnaposti. Prima versione: tradurre in collezioni le ricette di
   [`../utente/casi-uso/`](../utente/casi-uso/README.md).
2. **Distribuzione senza rete a runtime**: bundlate nell'app come il registro
   modelli (sono testo, pesano niente). Alternativa se crescono: asset della
   release GitHub con hash verificato — mai un endpoint interrogato dal client.
3. **UI**: card «Collezioni» in Impostazioni → Dati (accanto a Importa JSON),
   e la stessa scelta nell'onboarding al posto del solo demo vault. Import
   sempre in modalità `skip` (non sovrascrive mai roba dell'utente).
4. **Nessun connettore per sito.** Al massimo un importer **CSV generico**
   (`titolo,corpo`, separatore rilevato) che copre awesome-chatgpt-prompts e
   simili in poche righe: l'80% della strada B al 5% del costo, senza rete,
   senza ToS. Va in `import_export.rs` accanto al Markdown bulk.
5. **Ponte fra le due** (non ora): prompt piatto importato → **Ritocco** che
   propone dove mettere i segnaposti. È la risposta naturale al «testo piatto»
   del punto 4 della strada B.

## Riserve da pesare prima di partire

- **Il collo di bottiglia è scrivere prompt buoni**, non il codice. Decidere
  chi cura le collezioni e con che cadenza *prima* di aprire la prima PR.
- **«Accattivante» per chi?** Senza telemetria (giustamente) non sappiamo se il
  problema è il vault vuoto al primo avvio — che il demo vault già copre — o
  il «e adesso cosa ci faccio» dopo una settimana. Le collezioni per persona
  rispondono al secondo; per il primo potrebbe bastare migliorare il demo.
- **Fiducia**: un contenuto curato è di fatto firmato dal progetto. Se un
  prompt di collezione si rivela scadente o dannoso, il danno reputazionale
  è nostro, non del sito di provenienza. Vale una review per collezione.

## Cosa NON si fa

- Connettori per PromptBase, FlowGPT, PromptHub, LangChain Hub.
- Scraping di prompt library HTML (Anthropic, OpenAI).
- Un «marketplace» o server di collezioni: fuori scope Personale, e in
  Enterprise arriverebbe semmai come feature del server (v2.x).

## Decisioni chiuse (2026-09-19)

| # | Domanda | Decisione |
|---|---|---|
| D1 | Chi cura le collezioni e con che cadenza? | **Aperta.** Le prime due le ha scritte il maintainer con la PR; contributi via PR con review, procedura in `docs/collezioni/README.md`. La cadenza si decide quando c'è un secondo contributore. |
| D2 | Bundle nell'app o scaricate? | **Scaricate da GitHub, non bundlate**: aggiornabili senza rilasciare l'app. In pratica da `raw.githubusercontent.com/…/main/docs/collezioni/` (stesso pattern e stesse difese di `changelog.rs`: URL da costanti, slug validato, timeout, cap byte) e non da asset di release — ottiene lo stesso senza toccare `release.yml` né il box firma. L'indice porta lo sha256 di ogni file e il client lo verifica. Rete **solo al click** «Sfoglia». |
| D3 | Per persona o per feature? | **Per persona**: Sviluppatore, Scrittura; poi Insegnamento, Analisi dati. Ogni collezione mostra comunque tutte le feature. |
| D4 | Importer CSV generico? | **Rinviato** ([`rinvii.md`](./rinvii.md)); si fa se qualcuno lo chiede. |

Scelte di dettaglio prese in implementazione:

- cartella radice `Collezioni/` condivisa, una sottocartella per collezione;
  import sempre `skip`, idempotente per id (re-import = tutto «già presente»);
- `import_pure` ora **rimappa i tag per nome**: un tag omonimo dell'utente
  con id diverso viene riusato invece di far fallire l'INSERT
  (`UNIQUE (WorkspaceId, Name)`) e perdere l'associazione — senza questo le
  collezioni si rompevano in ogni vault già popolato;
- nessun `global_placeholders` nelle collezioni: non si seminano valori nel
  vault dell'utente;
- la scelta nell'onboarding è rinviata a una PR separata, dopo la prova dal
  vivo della card.

## Residuo di fiducia (dichiarato, non risolto)

Lo sha256 nell'indice arriva dalla stessa origine dei file: difende da
download troncati e dallo scarto di cache della CDN, **non** da un repo
compromesso. È lo stesso livello di fiducia dell'updater, che punta allo
stesso repo; una firma Ed25519 con la chiave dell'updater alzerebbe
l'asticella ma richiede il box firma a ogni modifica di collezione — non
vale il costo finché il contenuto è testo importato in `skip`.
