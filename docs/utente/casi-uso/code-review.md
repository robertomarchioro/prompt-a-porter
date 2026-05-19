# Code review

Template per code review automatica con focus su edge case, performance, leggibilità e sicurezza.

## Quando usarlo

- Prima di aprire una pull request (auto-review).
- Per ricevere un secondo paio d'occhi quando lavori in solo.
- Per onboardare nuovi linguaggi: chiedi al modello idiomi tipici.

## Prompt

**Titolo:** Code review strutturata
**Tag:** `code-review`, `dev`, `qualità`
**Modello target:** `claude-opus` (raccomandato per code), `gpt-4`

````
Sei un senior engineer specializzato in {{linguaggio}} con 10+ anni di esperienza.

Fai una code review del seguente codice. Concentrati nell'ordine su:
1. **Bug e edge case non gestiti** (priorità più alta)
2. **Sicurezza** (input validation, injection, secret hardcoded)
3. **Performance** (N+1, allocazioni inutili, complessità)
4. **Leggibilità** (naming, struttura, commenti)
5. **Idiomatica** ({{linguaggio}}-specific best practice)

Per ogni problema:
- Severità: 🔴 critical / 🟡 warning / 🔵 nit
- File:riga (se rilevante)
- Spiegazione breve
- Suggerimento di fix (codice o pseudocodice)

Non commentare cose già corrette. Non essere prolisso: una review da 15 punti reali è meglio di una da 50 punti vaghi.

Contesto progetto: {{contesto}}

Codice da revisionare:

```{{linguaggio}}
{{codice}}
```
````

## Esempio: review TypeScript

- `linguaggio`: `typescript`
- `contesto`: `Componente Svelte 5 che gestisce upload file con drag&drop. Production app per knowledge worker.`
- `codice`:

  ```typescript
  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    const files = e.dataTransfer.files;
    for (let i = 0; i < files.length; i++) {
      const text = await files[i].text();
      await fetch("/upload", { method: "POST", body: text });
    }
  }
  ```

### Output atteso (tipico)

```
🔴 critical, riga 3: e.dataTransfer può essere null.
  Fix: const files = e.dataTransfer?.files; if (!files) return;

🔴 critical, riga 4-7: files[i].text() carica tutto in memoria.
  Per file > 100 MB rompe la tab. Usa stream/chunk.

🟡 warning, riga 6: nessun error handling sul fetch.
  Aggiungi try/catch o controlla response.ok.

🟡 warning, riga 6: niente CSRF token. Verifica che /upload sia same-origin.

🔵 nit, riga 4: preferire for...of files invece di indici.
```

## Varianti

### Review focalizzata

Se hai un focus specifico (es. solo performance), modifica il prompt:

```
... Concentrati ESCLUSIVAMENTE su performance:
1. Algoritmi (complessità)
2. I/O (DB, network, disk)
3. Allocazioni (GC pressure, lifetime)
4. Concurrency (deadlock, race)
Ignora stile, leggibilità, naming.
```

### Review con diff Git

Per un diff (non file completo):

````
Codice rivisto (diff unified):

```diff
{{diff}}
```

Vincolo aggiuntivo: commenta SOLO le righe modificate (+/-), non il contesto.
````

### Specializzazione per linguaggio

Crea prompt separati per linguaggio:

- `review-rust`: importa il "ruolo senior" + regole Rust (unsafe, lifetime, ownership).
- `review-python`: PEP 8, typing, async.

```
{{import "review-base" with linguaggio=rust}}

Inoltre per Rust verifica:
- Usi di unwrap() / expect() in production code
- unsafe block con safety comment
- Lifetime espliciti dove necessari
- Errori propagati con ? invece di match
```

## Anti-pattern

- **Non incollare repository interi**: il context window è finito. Limita a 1-2 file (~500 righe max).
- **Non chiedere "Cosa fa questo codice?"**: per quello usa un prompt summarize, non review.
- **Non passare segreti**: anche se il modello "non ricorda", evita di incollare API key, token, credenziali. Il linter PaP segnala `PII004`.
