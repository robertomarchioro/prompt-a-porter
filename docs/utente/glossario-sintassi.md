# Glossario sintassi

Reference rapido della sintassi del body di un prompt: segnaposti, globali, import, codici linter.

Per approfondimenti: [`prompt-componibili.md`](./prompt-componibili.md) (import) e [`linting-regole.md`](./linting-regole.md) (regole linter complete).

## Segnaposti — `{{nome}}`

Parametri da compilare al momento dell'uso.

```
Ciao {{nome}}, oggi parliamo di {{argomento}}.
```

**Regole:**
- Doppie graffe **obbligatorie** (singola graffa è errore `PH001`).
- Il nome deve essere un identificatore: lettere, cifre, `_`, niente spazi né trattini.
- Spazi interni alle graffe sono tollerati: `{{ nome }}` ≡ `{{nome}}`.
- Lo **stesso segnaposto** può comparire più volte: viene compilato una sola volta, il valore è riapplicato a ogni occorrenza.

In fase di compilazione, l'app mostra una form con un campo per ogni segnaposto unico. Se lasci un campo vuoto, il segnaposto resta intatto nel testo finale (così puoi riempirlo dopo manualmente).

### Esempio

Body:
```
Riscrivi il seguente testo in tono {{tono}}:

"{{testo}}"

Lingua di output: {{lingua}}.
```

Compilazione:
- `tono` → `formale`
- `testo` → `Ciao, ti scrivo per...`
- `lingua` → `italiano`

Output:
```
Riscrivi il seguente testo in tono formale:

"Ciao, ti scrivo per..."

Lingua di output: italiano.
```

## Segnaposti globali — `{{globale nome}}`

Valori riusati in molti prompt senza dover ricompilarli ogni volta. Tipici: nome utente, ruolo, azienda, tono editoriale di default.

```
Sono {{globale autore}}, responsabile {{globale ruolo}} a {{globale azienda}}.
```

I valori dei globali sono gestiti da **Impostazioni → Segnaposti globali**: chiave-valore semplice, condivisi fra tutti i prompt del vault.

**Differenza chiave con i segnaposti normali:**

| Caratteristica | `{{nome}}` (normale) | `{{globale nome}}` |
|---|---|---|
| Form di compilazione | Chiede ogni volta | Mai (valore preso dal DB) |
| Storage | Solo nel testo del prompt | Tabella dedicata `GlobalPlaceholders` |
| Override per uso | Sì (puoi modificare nel form) | No (per cambiarlo: Impostazioni) |
| Tipico per | Valori variabili (input, contesto) | Valori stabili (firma, ruolo, azienda) |

Se un globale non esiste nel DB, il testo `{{globale nome}}` resta nel risultato finale (così te ne accorgi e lo aggiungi).

## Import — `{{import "path"}}`

Importa il body di un altro prompt nel corrente. Permette di costruire prompt modulari: un "ruolo esperto" riusato in molti casi d'uso, un "tono editoriale" condiviso, etc.

```
{{import "ruolo-esperto-marketing"}}

Scrivi una email cold outreach per: {{contesto}}.
```

**Path:**
- Stringa fra **virgolette doppie** (case-insensitive per il titolo).
- Forme accettate:
  - `"titolo"` — match per titolo a root o in qualunque cartella (priorità a root).
  - `"cartella/sotto/titolo"` — path esplicito.
  - Lo slash iniziale è opzionale: `"/marketing/email"` ≡ `"marketing/email"`.

**Modifiers (sintassi estesa M4):**

| Sintassi | Effetto |
|---|---|
| `{{import "path" version=3}}` | Importa la versione N dal repository di versioni (non l'ultima). |
| `{{import "path" with tono=formale, lingua=en}}` | Sostituisce i segnaposti del prompt importato con i valori dati. |
| `{{import "path" version=3 with tono=formale}}` | Combina i due (version **prima** di with). |

**Limiti:**
- Profondità massima: **5 livelli** di import nidificati.
- Output massimo: **~1 MB** (anti-bomba).
- Cicli **vietati**: A → B → A non è permesso. Il linter li segnala con `IMP002`.

Vedi [`prompt-componibili.md`](./prompt-componibili.md) per esempi completi, anti-pattern e troubleshooting.

## Codici linter

L'editor segnala in tempo reale problemi nel body. Catalogo riassuntivo (dettaglio in [`linting-regole.md`](./linting-regole.md)):

### LEN — Lunghezza body

| Codice | Severità | Quando |
|---|---|---|
| `LEN001` | Warning | Body > 4 000 caratteri |
| `LEN002` | Info | Body < 30 caratteri |

### PH — Segnaposti

| Codice | Severità | Quando |
|---|---|---|
| `PH001` | Error | Singola graffa: `{nome}` invece di `{{nome}}` |
| `PH003` | Warning | Nome con caratteri non consentiti (spazi, trattini) |

### PII — Privacy

| Codice | Severità | Quando |
|---|---|---|
| `PII001` | Warning | Email nel body (`utente@example.com`) |
| `PII003` | Error | Carta di credito Luhn-valida (13–19 cifre) |
| `PII004` | Error | API key di provider noti (`sk-…`, `sk-ant-…`, `AKIA…`, `ghp_…`, `AIza…`) |

### STY — Stile

| Codice | Severità | Quando |
|---|---|---|
| `STY001` | Info | Ripetizione di n-gram (frase identica 3+ volte) |
| `STY002` | Info | Maiuscole eccessive (parola in CAPS > 10 char) |

### IMP — Import / dipendenze

| Codice | Severità | Quando |
|---|---|---|
| `IMP001` | Error | Import non risolvibile (path non esistente) |
| `IMP002` | Error | Ciclo di import (diretto o indiretto) |
| `IMP003` | Warning | Profondità import ≥ 6 (limite hard a 5) |

Disabilita singole categorie da **Impostazioni → Linter** (la scelta è persistita locale).

## Esempi compositi

### Email parametrica con globali

```
Da: {{globale autore}} <{{globale email}}>
A: {{destinatario}}
Oggetto: {{oggetto}}

Ciao {{destinatario}},

{{corpo}}

Cordiali saluti,
{{globale autore}}
```

### Code review con import del "ruolo"

```
{{import "ruoli/senior-engineer"}}

Esamina il seguente codice {{linguaggio}}. Concentrati su:
- Edge case non gestiti
- Performance
- Leggibilità

\`\`\`
{{codice}}
\`\`\`
```

### Prompt versionato con override

```
{{import "system-prompts/analista-dati" version=2 with tono=conciso}}

Analizza il seguente dataset: {{dataset}}.
```

## Quando NON usare segnaposti

I segnaposti sono pensati per **valori brevi** (parole, frasi, qualche riga). Per testi lunghi (interi articoli, codice multi-file):

- Considera di **dividere** il prompt in fasi (prompt 1 → output → prompt 2).
- Usa la modalità **doppia vista Sorgente/Compilato** dell'editor (M5) per validare visivamente il render.
- Evita segnaposti dentro segnaposti: PaP non supporta interpolazione ricorsiva nei valori (il risultato è trattato come stringa letterale).
