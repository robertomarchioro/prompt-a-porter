# Linting — Catalogo regole

> Disponibile da `v0.3.0`. 11 regole su 14 dello spec implementate.

L'editor del prompt mostra un pannello **Diagnosi** che segnala
problemi (errori, warning, info) sul body del prompt, in tempo reale
mentre scrivi. Tutto avviene **localmente** — nessuna chiamata
esterna, nessun salvataggio dei risultati nel vault.

## Severità

| Livello | Effetto UI | Esempi |
|---|---|---|
| **Error** | Rosso, blocca salvataggio in casi gravi | Carta di credito, ciclo di import |
| **Warning** | Giallo, non blocca | Body troppo lungo, email PII |
| **Info** | Grigio, suggerimento | Body troppo corto, ripetizione n-gram |

## Catalogo

### Lunghezza body

| Codice | Severità | Quando scatta |
|---|---|---|
| `LEN001` | Warning | Body > 4 000 caratteri (spreco token) |
| `LEN002` | Info | Body < 30 caratteri (probabilmente incompleto) |

### Segnaposti

| Codice | Severità | Quando scatta |
|---|---|---|
| `PH001` | Error | Singola graffa: `{nome}` invece di `{{nome}}` |
| `PH003` | Warning | Caratteri non consentiti nel nome: `{{nome con spazi}}`, `{{nome-con-trattini}}` |

> `PH002` (segnaposto dichiarato non usato) non è implementato:
> il modello dati non distingue dichiarazione da uso.

### Privacy / PII

| Codice | Severità | Quando scatta |
|---|---|---|
| `PII001` | Warning | Email rilevata (`mario.rossi@example.com`) |
| `PII003` | Error | Numero carta di credito Luhn-valido (13–19 cifre) |
| `PII004` | Error | API key di provider noti (OpenAI `sk-…`, Anthropic `sk-ant-…`, AWS `AKIA…`, GitHub `ghp_…` / `github_pat_…`, Google `AIza…`) |

> `PII002` (codice fiscale italiano) non è implementato — la regex è complessa e il rapporto costo/beneficio è basso.

### Stile

| Codice | Severità | Quando scatta |
|---|---|---|
| `STY001` | Info | Stesso n-gram (3 parole) ripetuto ≥ 4 volte nel body |

> `STY002` (mancanza di istruzioni chiare) non è implementato — richiederebbe NLP IT/EN, troppo fragile a regex.

### Import (prompt componibili)

| Codice | Severità | Quando scatta |
|---|---|---|
| `IMP001` | Error | `{{import "path"}}` con path che non risolve a nessun prompt |
| `IMP002` | Error | Ciclo di import (self-loop o catena A→B→A multi-hop) |
| `IMP003` | Warning | Profondità di import > 5 livelli |

Le regole IMP* funzionano solo sui prompt **già salvati** nel vault.
Su un prompt nuovo (mai salvato) il controllo del ciclo che parte
dal root non è possibile, ma cicli interni alla catena vengono
comunque rilevati.

## Esempi pratici

### Errori bloccanti

```
La carta da rimuovere è 4111 1111 1111 1111 grazie.
                        ^^^^^^^^^^^^^^^^^^^
                        PII003 — Luhn valido
```

```
{{import "ricetta-segreta-ma-non-esiste"}}
                                          ^
                                          IMP001 — non risolto
```

### Warning (revisione consigliata)

```
Manda un messaggio a mario.rossi@example.com appena puoi.
                     ^^^^^^^^^^^^^^^^^^^^^^^
                     PII001 — possibile email
```

### Info (suggerimenti)

```
ciao
^^^^
LEN002 — body corto (<30 caratteri)
```

## Disabilitare il linting

Il pannello Diagnosi è collassabile (chevron in alto a destra).
Quando chiuso non ricarica le diagnosi al cambio body, ma riapre
con i risultati dell'ultimo run.

Non c'è (ancora) un toggle globale per disattivare il linter — è
sempre attivo lato backend. Le regole che ti danno fastidio si
possono ignorare guardando la severità.

## Performance

Il lint gira lato backend Rust con regex pre-compilati (cached
in `OnceLock`). Costo tipico:

- Body 1 000 caratteri: ~0.1 ms
- Body 10 000 caratteri: ~1 ms
- Body con 50 import (IMP* attivi): ~5 ms (DFS sul grafo)

Il debounce frontend è 600 ms, quindi non vedi il pannello
"flickerare" mentre digiti.

## Riferimenti

- Spec completa nello spec roadmap: [`fase-3-intelligence.md`](../roadmap/fase-3-intelligence.md) Step 5
- Implementazione: `apps/client/src-tauri/src/linting.rs`
- Test: 28 unit test sulla logica di lint (vedi `cargo test --lib linting::`)
