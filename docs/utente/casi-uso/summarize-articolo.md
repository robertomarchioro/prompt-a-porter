# Summarize articolo

> Template per riassumere articoli tecnici, paper, post di blog in N punti chiave.

## Quando usarlo

- Triage di newsletter / RSS feed.
- Decidere se un paper merita lettura completa.
- Estrarre "executive summary" da documenti lunghi per condividere in team.

## Prompt

**Titolo:** Summarize articolo strutturato
**Tag:** `summary`, `reading`, `productivity`
**Modello target:** `claude-sonnet` o `gpt-4-mini`

```
Riassumi il seguente articolo / paper / post in italiano.

Vincoli:
- Esattamente {{numero_punti}} punti chiave (bullet list).
- Ogni punto: 1 frase, max 25 parole.
- Ordine per importanza decrescente.
- Niente fluff ("L'articolo discute di...", "L'autore sostiene che...").

Aggiungi alla fine:
- **TL;DR**: 1 frase di max 20 parole.
- **Per chi**: target audience (1 frase).
- **Tempo di lettura originale**: stima in minuti (lettura standard 200 wpm).
- **Vale la pena leggerlo completo?**: sì / no / dipende, con motivazione 1 frase.

Articolo:

{{articolo}}
```

## Esempio

- `numero_punti`: `5`
- `articolo`: *[testo di "Designing Data-Intensive Applications" cap. 1, ~3000 parole]*

### Output atteso (tipico)

```
- I sistemi data-intensive falliscono per limiti di dati/complessità, non CPU.
- Tre obiettivi chiave: reliability, scalability, maintainability — sempre in trade-off.
- Reliability = funzionamento corretto anche con fault hardware/software/umani.
- Scalability ≠ "scala tanto": è la capacità di gestire crescita di carico specifico.
- Maintainability = operability + simplicity + evolvability nel tempo.

**TL;DR**: I tre pilastri (reliability, scalability, maintainability) guidano le scelte di design dei sistemi data-intensive.

**Per chi**: backend engineer e architetti che progettano sistemi distribuiti production.

**Tempo di lettura originale**: ~15 minuti.

**Vale la pena leggerlo completo?**: sì — è il framework concettuale che lega tutto il libro.
```

## Varianti

### Summary "per il manager"

Aggiungi al prompt:

```
Stile: business-friendly. Niente gergo tecnico se non strettamente necessario.
Per ogni punto, prepend l'impatto di business (es. "Riduce costi infra del 30%:...").
```

### Confronto fra articoli

Riassumi N articoli e confrontali:

```
Riassumi i seguenti {{n}} articoli in modo COMPARATIVO.

Per ogni articolo:
- 2-3 bullet sintetici
- Tesi principale in 1 frase

Alla fine, una tabella di confronto:
| Aspetto | Articolo 1 | Articolo 2 | ... |

Articoli:
{{articoli}}
```

### Extract action items (per meeting notes / lunghe email)

```
Estrai SOLO le azioni concrete dal seguente testo. Ignora tutto il resto.

Per ogni azione:
- Cosa: descrizione 1 riga
- Chi: assegnatario (se citato, altrimenti "TBD")
- Quando: scadenza (se citata, altrimenti "TBD")

Testo:
{{testo}}
```

## Anti-pattern

- **Non riassumere riassunti**: la perdita di informazione è quadratica. Riassumi sempre dall'originale.
- **Non chiedere "riassumi" senza vincoli**: il modello produce 200 parole vaghe. Imponi numero punti + lunghezza per punto.
- **Non incollare PDF intere**: estrai testo prima (la maggior parte dei modelli ignora layout PDF). Se il PDF ha tabelle/figure critiche, includile come testo separato.
