# Traduzione tecnica

Template per traduzioni con glossario tecnico custom: mantiene termini del dominio nella lingua target corretta, evita "false friends".

## Quando usarlo

- Documentazione tecnica EN ↔ IT con terminologia consolidata in azienda.
- Articoli di settore (DevOps, ML, sicurezza) dove la traduzione automatica generica spesso sbaglia.
- Articoli marketing dove vuoi controllare il registro.

## Prompt

**Titolo:** Traduzione tecnica con glossario
**Tag:** `traduzione`, `tecnico`, `glossario`
**Modello target:** `claude-sonnet` o `gpt-4`

```
Traduci il seguente testo da {{lingua_sorgente}} a {{lingua_target}}.

Glossario obbligatorio (rispetta SEMPRE queste corrispondenze):
{{glossario}}

Termini da NON tradurre (lasciare invariati):
{{termini_invariati}}

Stile:
- Registro: {{registro}}
- Lunghezza: simile all'originale (±10%).
- Mantieni la struttura (heading, liste, code block).
- Code block e nomi di file/variabili: mai tradotti.

Note culturali:
- Per misure: converti unità imperiali a metriche, ma lascia entrambe ("5 miles (~8 km)").
- Per esempi con nomi: se generici (John/Jane), italianizza (Mario/Anna). Se citazioni di persone reali, mantieni.

Testo:

{{testo}}
```

## Esempio: doc tecnica EN → IT

- `lingua_sorgente`: `inglese tecnico americano`
- `lingua_target`: `italiano tecnico`
- `glossario`:

  ```
  - deployment → deploy (sostantivo) / "fare il deploy di" (verbo). NON "schieramento".
  - container → container (invariato).
  - workflow → flusso di lavoro (sostantivo neutro) / workflow (se citato come termine tecnico GitHub Actions).
  - artifact → artefatto.
  - rollback → rollback (invariato).
  - log → log (sostantivo) / "loggare" è un anglicismo accettabile in contesto dev.
  ```

- `termini_invariati`: `Tauri, Svelte, SQLite, SQLCipher, Cargo, npm, pnpm, vault`
- `registro`: `documentazione formale ma piana, no gergo "guru"`
- `testo`:

  ```
  After the workflow completes successfully, the artifact is uploaded
  to GitHub Releases. The deployment process triggers a rollback if
  any healthcheck fails within the first 5 minutes.
  ```

### Output atteso

```
Una volta che il workflow è completato con successo, l'artefatto viene
caricato su GitHub Releases. Il processo di deploy avvia un rollback se
uno dei healthcheck fallisce entro i primi 5 minuti.
```

## Varianti

### Traduzione + spiegazione (per onboarding)

```
Traduci il testo seguente e, per ogni termine tecnico, aggiungi
in parentesi una spiegazione di 5-10 parole (utile per chi sta
imparando il dominio).

Esempio: "deploy (rilascio del codice in produzione)"

Testo:
{{testo}}
```

### Localizzazione UI (con vincoli di lunghezza)

Per stringhe di UI dove la lunghezza in pixel conta:

```
Traduci le seguenti stringhe UI da inglese a {{lingua}}.

Vincoli:
- Ogni stringa target NON può superare 1.3x la lunghezza in caratteri dell'originale.
- Se necessario, abbrevia mantenendo chiarezza.
- Stile: imperativo per i pulsanti, neutro per le etichette.

Formato output: una stringa per riga, stesso ordine input.

Stringhe:
{{stringhe}}
```

### Reverse check (traduci e ritraduci)

Per controllo di qualità:

```
Esegui DUE traduzioni consecutive:

1. Traduci il testo da {{lingua_a}} a {{lingua_b}}.
2. Ritraduci il risultato da {{lingua_b}} a {{lingua_a}}.

Mostra entrambe. Alla fine, evidenzia eventuali divergenze
significative tra il testo originale e la ritraduzione: indicano
ambiguità o errori della prima traduzione.

Testo:
{{testo}}
```

## Anti-pattern

- **Non passare glossari di 100 voci**: il modello ne ignora la metà. Per vocabolari grandi, dividi in più prompt per dominio (es. `glossario-devops.md` come import).
- **Non lasciar tradurre nomi di prodotti**: aggiungi sempre `termini_invariati`. "Prompt a Porter" non è "Pronto da Indossare".
- **Non usare per traduzioni legali / mediche**: queste richiedono certificazione professionale. PaP/AI può essere un primo draft, non il risultato finale.
