# Linting — Catalogo regole

> Il catalogo delle regole di linting del pannello Diagnosi, con esempi pratici e istruzioni per silenziarle o personalizzarle. Disponibile da `v0.3.0`.

L'editor del prompt mostra un pannello **Diagnosi** che segnala
problemi (errori, warning, info) sul body del prompt, in tempo reale
mentre scrivi. Tutto avviene **localmente** — nessuna chiamata
esterna, nessun salvataggio dei risultati nel vault.

## Severità

| Livello | Effetto UI | Esempi |
|---|---|---|
| **Error** | Rosso, problema da correggere | Carta di credito, ciclo di import |
| **Warning** | Giallo, revisione consigliata | Body troppo lungo, email PII |
| **Info** | Grigio, suggerimento | Body troppo corto, ripetizione n-gram |

Nessuna regola blocca il salvataggio: la diagnosi è puramente informativa, decidi tu se e come intervenire.

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
| `IMP004` | Info | Il prompt corrente è **importato da N altri prompt** — utile saperlo prima di modifiche breaking, perché le modifiche al body si propagano a chi lo importa |

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

## Personalizzare il linter

Le regole che ti danno fastidio si possono **silenziare**, per
famiglia o per singola regola.

### Dove

**Impostazioni → Linter.** Trovi il catalogo completo raggruppato
per categoria (Lunghezza, Segnaposti, Privacy, Stile, Import). Ogni
voce mostra il codice (`PII001`), la severità e una breve
descrizione.

### Come

- **Interruttore di famiglia** (es. *Privacy / PII*): spento,
  nasconde tutte le regole di quella categoria. Quando una famiglia
  è spenta, gli interruttori delle sue singole regole sono
  disattivati (è già tutto silenziato).
- **Interruttore di regola** (es. *PII001*): silenzia solo quella
  regola, lasciando attive le altre della stessa famiglia. Utile
  per zittire un singolo falso positivo ricorrente.
- **Severità** (menu a tendina per ogni regola): cambia il livello
  con cui l'avviso viene mostrato — *Errore*, *Avviso* o *Info*.
  Es. declassare *PII001* da Avviso a Info se le email nei tuoi
  prompt sono legittime.
- **Soglie numeriche**: alcune regole hanno un valore regolabile —
  *Caratteri massimi* (LEN001), *Caratteri minimi* (LEN002) e
  *Ripetizioni minime* (STY001). Alzale o abbassale per adattare la
  sensibilità al tuo stile.
- **Ripristina** (per regola) riporta severità e soglia di quella
  regola ai valori predefiniti; **Ripristina tutto** azzera ogni
  personalizzazione (disattivazioni, severità, soglie).

Le modifiche hanno effetto **subito**: la tab Diagnosi si
ri-analizza appena cambi qualcosa, senza bisogno di rieditare il
prompt. Valori fuori scala vengono riportati entro limiti sensati
(es. la soglia ripetizioni non scende sotto 2).

### Note

- Le regole silenziate sono **escluse dal conteggio** e dalla lista
  della tab Diagnosi, ma il lint gira comunque lato backend: il
  costo è invariato (vedi [Performance](#performance)).
- Le preferenze sono **locali a questo dispositivo** (non
  sincronizzate). Reinstallando o cambiando macchina riparti dal
  catalogo completo.
- Il pannello Diagnosi è inoltre **collassabile** (chevron in alto a
  destra): quando è chiuso non ricarica le diagnosi al cambio body,
  ma riapre con i risultati dell'ultimo run.

## Performance

Il lint gira lato backend Rust con regex pre-compilati (cached
in `OnceLock`). Costo tipico:

- Body 1 000 caratteri: ~0.1 ms
- Body 10 000 caratteri: ~1 ms
- Body con 50 import (IMP* attivi): ~5 ms (DFS sul grafo)

Il debounce frontend è 600 ms, quindi non vedi il pannello
"flickerare" mentre digiti.

## Vedi anche

- [`fase-3-intelligence.md`](../roadmap/fase-3-intelligence.md) — spec completa delle regole (Step 5).
- [`glossario-sintassi.md`](./glossario-sintassi.md) — tabella riassuntiva dei codici linter nel reference di sintassi.
- Implementazione: `apps/client/src-tauri/src/linting.rs` — motore di lint (coperto da unit test, `cargo test --lib linting::`).
