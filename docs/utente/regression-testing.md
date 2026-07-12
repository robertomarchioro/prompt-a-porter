# Regression testing dei prompt

> Come definire test golden sui prompt ed eseguirli contro un provider AI per
> accorgerti quando un prompt smette di funzionare — al cambio di modello o
> dopo un ritocco al testo.

I prompt si degradano in silenzio. Aggiorni il modello AI, ritocchi una
frase, sposti un'istruzione più in basso — e un prompt che per mesi ha
prodotto output impeccabili comincia a sbagliare, senza che nessuno se ne
accorga finché il danno non è fatto. A differenza del codice, un prompt non
"si rompe" in modo visibile: continua a produrre testo, solo un testo
peggiore.

I **test golden** portano sui prompt la stessa rete di sicurezza che i test
automatici danno al software. Un golden congela una coppia "input →
output atteso": dati certi valori per i segnaposti, ti aspetti una certa
risposta (con un margine di tolleranza che decidi tu). Ogni volta che lo
esegui, PaP compila il prompt, interroga il provider AI e misura quanto la
risposta reale somiglia a quella attesa. Il risultato viene salvato, e nel
tempo emerge la storia: il prompt regge, oppure sta derivando.

È lo strumento giusto in due momenti precisi: quando cambi modello
(da una versione di Claude o GPT alla successiva) e vuoi sapere quali
prompt del vault ne risentono, e quando modifichi un prompt importante e
vuoi la conferma che i casi che funzionavano funzionino ancora.

Quattro parole che incontrerai in questa pagina: il **golden** è il caso di
test salvato sul prompt (valori dei segnaposti, output atteso, funzione di
similarità, soglia); la **run** è una singola esecuzione, che chiama il
provider e registra l'esito; il **provider AI** è il servizio che genera la
risposta (locale come Ollama, o remoto come Anthropic e OpenAI); il
**drift** è la differenza fra la similarità media del periodo e quella
dell'ultima run — il segnale che qualcosa è cambiato.

## Prima di iniziare: un provider AI

Per eseguire i golden serve almeno un provider configurato in
**Impostazioni → Provider AI**:

| Provider | Cosa serve | Costo |
|---|---|---|
| **Ollama** | URL `http://localhost:11434` (default) — installa Ollama e scarica un modello (`ollama pull llama3.2`) | 0 € locale |
| **Anthropic** | API key `sk-ant-…` da console.anthropic.com | per token |
| **OpenAI** | API key `sk-…` da platform.openai.com | per token |
| **OpenAI-compat** | URL custom (LM Studio, vLLM) + key generica | varia |
| **Google (Gemini)** | API key da `aistudio.google.com/apikey`; modelli `gemini-2.5-flash` / `gemini-2.5-pro` | per token |

Se vuoi provare la funzione senza spendere nulla e senza mandare testo
fuori dalla tua macchina, Ollama è la scelta naturale.

> **Sicurezza**: le API key sono salvate dentro il vault, protette dalla
> stessa cifratura di tutto il resto (SQLCipher AES-256). Una volta
> salvate, l'app non le mostra mai più in chiaro.

## La prima volta

Il percorso completo, dal primo golden al primo risultato:

1. Apri un prompt e vai alla tab **Test golden** del pannello di dettaglio
   (accanto a Cronologia e Diagnosi).
2. Premi il bottone **Golden** (quello con il segno più): si apre la modale
   **"Nuovo golden test"**.
3. Compila i campi:
   - **Etichetta** — un nome leggibile per il caso: `caso comune`,
     `edge case lungo`.
   - **Variabili di input** — l'inserimento è guidato: un campo per ogni
     segnaposto letto dal body del prompt. Se il body non è caricabile
     resta il fallback JSON grezzo:
     ```json
     {"contesto":"reclamo cliente", "tono":"formale"}
     ```
     Questi valori sostituiranno le `{{var}}` nel body prima della
     chiamata al provider.
   - **Output atteso** — il testo che ti aspetti dal modello.
   - **Funzione di similarita** — come confrontare atteso e reale
     (vedi la sezione dedicata più sotto; `cosine` è il default).
   - **Soglia tolleranza** — un valore fra 0 e 1 (default 0.85): sopra la
     soglia il test passa, sotto fallisce.
4. Salva. Il golden compare nella lista della tab, con l'icona di stato a
   riposo (○).
5. Premi **Esegui** sul golden: si apre una modale dove scegli provider e
   modello (l'ultima scelta viene ricordata). Durante l'esecuzione l'icona
   passa a "in corso" (…), poi a ✓ oppure ✗.
6. Al termine la modale mostra l'esito completo: similarità calcolata,
   esito, latenza, token usati. Cliccando **Output ricevuto** espandi il
   testo che il modello ha prodotto davvero — il modo più rapido per capire
   *perché* un test è fallito.

Da qui in poi, riesegui i golden quando cambi modello o ritocchi il prompt,
e lascia che la vista **Regressioni** (più sotto) accumuli la storia.

## Le funzioni di similarità

Confrontare due testi non ha una risposta unica: dipende da cosa consideri
"uguale". Per questo ogni golden sceglie la sua funzione:

| Funzione | Costo | Quando usarla |
|---|---|---|
| **`cosine`** | ~50 ms, locale | Default. Confronto semantico, tollerante alle parafrasi: due testi che dicono la stessa cosa con parole diverse ottengono similarità alta |
| **`exact-match`** | gratis | Output strutturati (es. JSON rigido): 1.0 se identico, 0.0 altrimenti |
| **`regex`** | gratis | L'output atteso è un'espressione regolare: il test passa se la risposta la matcha |
| **`llm-judge`** | ~1-3 s + costo provider | Casi sfumati: un modello "giudice" riceve atteso e reale e assegna un punteggio di aderenza da 0 a 1 |

Due funzioni hanno un prerequisito. `cosine` riusa il modello di embedding
locale della ricerca semantica: se non è inizializzato, la run si ferma
prima ancora di chiamare il provider, indicandoti di attivarlo in
**Impostazioni → Ricerca & Embeddings** (vedi
[`ricerca-semantica.md`](./ricerca-semantica.md)). `llm-judge` richiede un
secondo provider come giudice: nella modale di esecuzione compaiono
selettori separati per provider e modello del giudice — di default gli
stessi della run principale (economico), ma puoi indicare un modello più
capace per ridurre il bias del giudizio.

## Eseguire i golden

Ogni esecuzione segue sempre gli stessi passi: il body viene compilato con
i valori delle variabili di input, il provider viene chiamato con il
modello scelto, la similarità fra atteso e reale viene calcolata con la
funzione del golden, e l'esito viene salvato con similarità, esito
(passato/fallito rispetto alla soglia), latenza, token usati, provider,
modello e data.

Oltre alla run singola c'è il batch: il bottone **Esegui tutti** in cima
alla tab lancia tutti i golden del prompt in sequenza (uno alla volta, per
non incorrere nei rate limit dei provider), con l'avanzamento visibile
golden per golden e un riepilogo finale di quanti sono passati e quanti
falliti.

Gli errori del provider — problemi di rete, rate limit, modello
inesistente — non bloccano nulla: la run viene salvata comunque, marcata
come fallita e con l'errore registrato, così anche i guasti lasciano
traccia nella storia.

Per modificare un golden esistente usa la sua modale di modifica (stessa
struttura della creazione).

## La vista "Regressioni"

I risultati delle run acquistano valore quando li guardi nel tempo. Il link
**Regressioni** nel footer della barra laterale apre la vista d'insieme:
una tabella con una riga per ogni combinazione prompt × provider × modello,
sul periodo che scegli con lo slider **Periodo** (da 1 a 90 giorni,
default 30).

Ecco una riga d'esempio, per capire come si legge:

| Prompt | Provider · Model | Run | Pass | Fail | Sim. media | Sim. ultima | Drift | Ultima |
|---|---|---|---|---|---|---|---|---|
| email-cold-outreach | anthropic · claude-sonnet-4.6 | 12 | 11 | 1 | 0.892 | 0.853 | +4.4% | … |

Su 12 run nel periodo, 11 sono passate; la similarità media era 0.892 ma
l'ultima run si è fermata a 0.853. Il **drift positivo** (+4.4%) segnala un
peggioramento: l'ultima esecuzione è sotto la media storica. Un drift
negativo indica invece un miglioramento. Il bordo colorato a sinistra di
ogni riga riassume lo stato a colpo d'occhio: tutte passate, esiti misti, o
tutte fallite.

Il bottone **Esporta CSV** scarica il report (`regressioni-<N>g-<data>.csv`)
in formato CSV standard, pronto per un foglio di calcolo.

## Limiti noti

- I golden si eseguono solo dall'app: la CLI non ha un comando per
  lanciarli (ad esempio in una pipeline CI), e i golden non sono esposti
  come tool MCP per gli agenti.
- L'esecuzione batch è sequenziale: su molti golden con provider remoti i
  tempi si sommano.
- Un golden appartiene al singolo prompt: non esiste una "suite" che
  esegua i golden di più prompt in un colpo solo — il punto d'insieme è la
  vista Regressioni, che però aggrega i risultati, non le esecuzioni.

## Vedi anche

- [`ricerca-semantica.md`](./ricerca-semantica.md) — il modello di embedding locale che alimenta la similarità `cosine`.
- [`varianti-prompt.md`](./varianti-prompt.md) — con i golden puoi confrontare oggettivamente due formulazioni alternative.
- [`rating-prompt.md`](./rating-prompt.md) — il feedback umano dopo l'uso, complementare alla misura automatica.
- [`cli.md`](./cli.md) — cosa offre oggi la CLI (e cosa no) per l'automazione.
