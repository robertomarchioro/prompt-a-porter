# Stile della documentazione utente

Regole per ogni pagina sotto `docs/utente/`. Obiettivo: le pagine devono sembrare
scritte dalla stessa mano, e la navigazione non va mai scritta a mano.

## Il template

Ogni pagina segue questa struttura, nell'ordine:

```markdown
# Titolo della pagina

> Una o due righe che dicono cosa impari in questa pagina.

## Prima sezione
…

## Limiti noti / roadmap      ← opzionale, penultima sezione
…

## Vedi anche                  ← opzionale, ultima sezione, max 4 link
- [`altra-pagina.md`](./altra-pagina.md) — perché è pertinente
```

## Le regole

1. **H1 unico + sintesi in blockquote.** Ogni pagina apre con `# Titolo` e, subito
   sotto, un blockquote `>` di 1-2 righe. Niente preamboli prima del primo `##`
   oltre alla sintesi.
2. **Nessuna navigazione manuale.** Vietate le sezioni "Prossimi passi",
   "Continua con…", "Pagina successiva": la sequenza di lettura la danno la
   sidebar e i link precedente/successiva del sito (VitePress), governati in
   `apps/site/.vitepress/config.ts`. Se cambi l'ordine di lettura, cambi il
   config, non le pagine.
3. **Cross-reference inline** dove servono nel testo. La sezione finale
   `## Vedi anche` è opzionale e serve solo per riferimenti non già linkati
   inline; massimo 4 voci, ognuna con una mezza riga di motivazione.
4. **Percorsi UI in grassetto** nel formato **Impostazioni → Gruppo → Sezione**,
   con i nomi esatti dell'app (verificarli nel codice, non a memoria).
5. **Sintassi PaP sempre in backtick**: `{{nome}}`, `{{global …}}`,
   `{{import "…"}}` fuori dai backtick rompono il build VitePress
   (interpolazione Vue) — vedi il commento in `apps/site/.vitepress/config.ts`.
6. **Niente numeri che invecchiano** (conteggi di test, righe di codice, date
   relative). Le versioni si citano solo quando raccontano una differenza di
   comportamento (`da v0.8.32 …`).
7. **Heading in sequenza**: da `##` in giù, senza saltare livelli.

## Perché markdown resta la strada

Le pagine sono lette in tre contesti con la stessa sorgente: il sito GitHub Pages
(VitePress, `srcDir=docs`), GitHub stesso, e i link d'aiuto in-app. Un formato
diverso (CMS, wiki, generatore dedicato) romperebbe almeno uno dei tre e
aggiungerebbe un posto in più da manutenere. Il "template uniforme" non si ottiene
cambiando formato: si ottiene con queste regole più il tema del sito, che
aggiunge da solo sidebar, ricerca, indice della pagina e navigazione
precedente/successiva a ogni file.
