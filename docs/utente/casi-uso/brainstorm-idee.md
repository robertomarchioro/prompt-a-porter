# Brainstorm idee

> Template per brainstorm strutturato con vincoli precisi su numero, formato e criteri di valutazione.

## Quando usarlo

- Naming (prodotti, feature, componenti).
- Esplorare opzioni di design / architettura prima di un ADR.
- Generare titoli alternativi (post, video, talk).
- Stress-test di un'idea (devil's advocate).

## Prompt

**Titolo:** Brainstorm strutturato con criteri
**Tag:** `brainstorm`, `ideazione`, `decisione`
**Modello target:** `claude-opus` (creatività) o `gpt-4`

```
Genera idee per il seguente obiettivo: {{obiettivo}}.

Contesto: {{contesto}}

Vincoli:
- Esattamente {{numero}} idee distinte.
- Per ogni idea: nome breve (max 5 parole) + 1 frase di descrizione (max 25 parole).
- Niente idee filler ("Idea generica X" o quasi-duplicati).
- Coprire spettro intenzionalmente eterogeneo: includi almeno 2 idee "rischiose" o controverse.

Criteri di valutazione: {{criteri}}

Output structurato:

| # | Idea | Descrizione | Pro principale | Contro principale | Punteggio |
|---|------|-------------|----------------|-------------------|-----------|

Punteggio: 1-10 in base ai criteri sopra. Sii rigoroso, non concedere mediocrità (un 7 deve "guadagnarselo").

Alla fine, scegli **una sola** idea raccomandata e motiva in max 50 parole.
```

## Esempio: naming di feature

- `obiettivo`: `Nome italiano breve per la feature "diff fra due varianti di un prompt"`
- `contesto`: `App desktop per gestione prompt AI. Lessico user-friendly, no gergo dev. Italian-first.`
- `numero`: `8`
- `criteri`: `1) Memorabilità  2) Chiarezza per utente non-tech  3) Lunghezza (max 2 parole preferito)  4) Non ambiguo con altri termini PaP`

### Output atteso (parziale)

```
| # | Idea | Descrizione | Pro | Contro | Punteggio |
|---|------|-------------|-----|--------|-----------|
| 1 | Confronta | Mostra le differenze fra due varianti come diff. | Chiaro, una parola. | Generico, già usato altrove. | 7 |
| 2 | Affianca | Vista side-by-side delle due varianti. | Verbo evocativo. | Non comunica "diff". | 6 |
| 3 | Diff libero | Apri un diff fra due varianti senza selezione preventiva. | Compatibile con UX esistente. | "Diff" è gergo dev. | 5 |
| ... | ... | ... | ... | ... | ... |

**Raccomandata: Confronta** — copre il 90% dei casi d'uso, una parola sola, italiano nativo. Il "diff" è implicito nella UI a colonne con highlighting.
```

## Varianti

### Devil's advocate

Quando hai già un'idea e vuoi stressarla:

```
La mia idea è: {{idea}}.

Fai il devil's advocate. Genera {{numero}} controargomentazioni rigorose:
- Non strawman, attacca i punti più forti.
- Per ogni obiezione: la mia ipotetica risposta + perché la tua obiezione può comunque valere.

Conclusioni: l'idea va portata avanti? Sì / no / con modifica X.
```

### Combinatorio (matrice)

Per esplorare combinazioni di 2 dimensioni:

```
Genera idee combinando le seguenti dimensioni:

Dimensione A: {{dim_a}} (valori: {{valori_a}})
Dimensione B: {{dim_b}} (valori: {{valori_b}})

Per ogni combinazione (A_i × B_j), genera UNA idea concreta.
Formato: tabella con righe A e colonne B.
```

Esempio: dim_a = "tipo prodotto" (SaaS, marketplace, app), dim_b = "audience" (B2B, consumer, dev) → 9 combinazioni × 1 idea = 9 idee.

### Forced analogy

Per uscire da pattern di pensiero:

```
Brainstorm di idee per {{obiettivo}} usando come analogia forzata: {{analogia}}.

Per ogni elemento dell'analogia, trasponi in un'idea nel dominio target.

Esempio: se analogia = "biblioteca", elementi = catalogazione, prestito, scadenza, sezioni → idee corrispondenti nel tuo dominio.

Numero idee: {{numero}}.
```

## Anti-pattern

- **Non chiedere "qualche idea"**: il modello produce 3 idee ovvie. Imponi un numero (8-15 sweet spot).
- **Non saltare i criteri**: senza criteri di valutazione, il modello restituisce idee equiparabili. La differenziazione qualitativa richiede metriche esplicite.
- **Non accettare la prima lista**: rilancia con "Ora genera 5 idee MOLTO diverse da queste, in direzione X". I migliori brainstorm sono iterativi.
