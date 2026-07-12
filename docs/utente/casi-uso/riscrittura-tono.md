# Riscrittura tono

> Template per riscrivere testo cambiando registro: da bozza grezza a formale, da formale a conciso, da informale a divulgativo.

## Quando usarlo

- Bozza scritta di getto da pulire prima di inviare.
- Adattare un testo per audience diversa (interno → cliente, tecnico → esecutivo).
- Localizzare un tono tipico dell'inglese in italiano (e viceversa).

## Prompt

**Titolo:** Riscrivi testo cambiando tono
**Tag:** `riscrittura`, `tono`, `editing`
**Modello target:** `claude-sonnet`

```
Riscrivi il seguente testo cambiando registro/tono come richiesto.

Tono di partenza: {{tono_attuale}}
Tono di destinazione: {{tono_target}}

Lingua di output: {{lingua}}

Vincoli:
- Mantieni invariati: contenuto, fatti, nomi propri, citazioni, numeri.
- Cambia: lessico, registro, sintassi, formalità.
- Lunghezza target: {{lunghezza}} (puoi accorciare/allungare entro ±20%).
- Niente parafrasi vuote: ogni frase deve aggiungere informazione.
- Niente filler ("come dicevamo", "in conclusione" se non c'è davvero una conclusione).

Testo originale:

{{testo}}
```

## Esempi

### Da bozza informale a email formale

- `tono_attuale`: `bozza informale, prima persona, frasi corte`
- `tono_target`: `email formale italiana, terza persona, sintassi piana`
- `lingua`: `italiano`
- `lunghezza`: `simile all'originale`
- `testo`:

  ```
  ciao Mario, scusami ma quel report che mi avevi promesso lunedì non è arrivato.
  mi servirebbe entro venerdì perché ho la riunione con il cliente.
  fammi sapere se hai problemi, magari ti aiuto io.
  ```

### Output atteso

```
Gentile Mario,

ti scrivo in merito al report concordato per lunedì, che non risulta ancora consegnato.
Mi servirebbe disporne entro venerdì in vista della riunione con il cliente.
Resto a disposizione per supportarti nel caso vi fossero blocchi.

Cordiali saluti,
```

### Da paper accademico a divulgativo

- `tono_attuale`: `accademico, gergo specialistico ML, frasi lunghe`
- `tono_target`: `divulgativo, target lettori di Wired, analogie concrete`
- `lingua`: `italiano`
- `testo`:

  ```
  We propose a novel attention mechanism that achieves O(n log n) complexity
  through hierarchical clustering of token embeddings, demonstrating
  competitive perplexity on WikiText-103 with 4x training throughput.
  ```

### Output atteso

```
Abbiamo trovato un modo per rendere i Transformer più veloci durante
l'addestramento — quattro volte più rapidi, per essere precisi. Invece
di confrontare ogni parola con ogni altra (operazione che cresce
quadraticamente con la lunghezza del testo), raggruppiamo le parole
simili in cluster e confrontiamo solo i cluster. La qualità delle
predizioni resta paragonabile ai modelli tradizionali.
```

## Varianti

### Tono basato su esempio di riferimento

```
Riscrivi il testo nel tono dell'esempio di riferimento (NON nel suo contenuto):

Esempio di riferimento (per lo stile):
{{esempio_stile}}

Testo da riscrivere (per il contenuto):
{{testo}}
```

### A/B due varianti

Genera due versioni alternative:

```
Genera DUE versioni del testo riscritto in tono {{tono_target}}.

Versione A: privilegia chiarezza, evita gergo.
Versione B: privilegia concisione, accetta gergo se efficiente.

Separa le due versioni con `--- VERSIONE B ---`.

Testo:
{{testo}}
```

Utile in PaP perché puoi salvare le due versioni come **varianti** dello stesso prompt (vedi [`varianti-prompt.md`](../varianti-prompt.md)) e confrontarle.

## Anti-pattern

- **Non cambiare tono E lingua nello stesso prompt** se possibile: i risultati migliorano se traduci prima e poi riscrivi (due step).
- **Non usare "rendi più professionale"**: troppo vago. Specifica il registro target (es. "formale italiano per email B2B").
- **Non lasciar decidere al modello la lunghezza**: imponi un range. Senza vincoli, il modello tende a espandere.
