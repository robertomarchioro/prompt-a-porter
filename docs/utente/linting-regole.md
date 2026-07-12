# Linting — Catalogo regole

> Il catalogo delle regole di linting del pannello Diagnosi, con esempi pratici e istruzioni per silenziarle o personalizzarle.

Un prompt può essere sbagliato in modi che non si vedono rileggendolo di fretta: una graffa singola dove ne servivano due, un'email vera dimenticata in un esempio, una chiave API incollata per sbaglio, un import che punta a un prompt rinominato. Il linter di Prompt a Porter esiste per accorgersene al posto tuo: mentre scrivi, la tab **Diagnosi** del pannello di dettaglio analizza il body e segnala errori, warning e suggerimenti in tempo reale.

Tutto avviene **localmente**: nessuna chiamata esterna, nessun salvataggio dei risultati nel vault. L'analisi è pressoché istantanea anche su body lunghi; il pannello attende una breve pausa nella digitazione prima di aggiornarsi, così non "sfarfalla" mentre scrivi.

Questa pagina è il catalogo completo: cosa controlla ogni regola, con quale severità, e come adattare il linter al tuo stile quando una regola ti dà fastidio. Consultala quando compare un codice che non conosci, o quando vuoi capire perché la Diagnosi insiste su qualcosa che per te è legittimo.

## Severità

| Livello | Effetto UI | Esempi |
|---|---|---|
| **Error** | Rosso, problema da correggere | Carta di credito, ciclo di import |
| **Warning** | Giallo, revisione consigliata | Body troppo lungo, email PII |
| **Info** | Grigio, suggerimento | Body troppo corto, ripetizione n-gram |

Nessuna regola blocca il salvataggio: la diagnosi è puramente informativa, decidi tu se e come intervenire.

## Catalogo

Nel catalogo alcuni numeri risultano saltati (`PH002`, `PII002`, `STY002`): corrispondono a controlli valutati e poi accantonati, e i loro codici restano riservati.

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

### Privacy / PII

| Codice | Severità | Quando scatta |
|---|---|---|
| `PII001` | Warning | Email rilevata (`mario.rossi@example.com`) |
| `PII003` | Error | Numero carta di credito Luhn-valido (13–19 cifre) |
| `PII004` | Error | API key di provider noti (OpenAI `sk-…`, Anthropic `sk-ant-…`, AWS `AKIA…`, GitHub `ghp_…` / `github_pat_…`, Google `AIza…`) |

### Stile

| Codice | Severità | Quando scatta |
|---|---|---|
| `STY001` | Info | Stesso n-gram (3 parole) ripetuto ≥ 4 volte nel body |

### Import (prompt componibili)

| Codice | Severità | Quando scatta |
|---|---|---|
| `IMP001` | Error | `{{import "path"}}` con path che non risolve a nessun prompt |
| `IMP002` | Error | Ciclo di import (self-loop o catena A→B→A multi-hop) |
| `IMP003` | Warning | Profondità di import > 5 livelli |
| `IMP004` | Info | Il prompt corrente è **importato da N altri prompt** — utile saperlo prima di modifiche breaking, perché le modifiche al body si propagano a chi lo importa |

Le regole IMP* funzionano al meglio sui prompt **già salvati** nel vault: su un prompt nuovo, mai salvato, il controllo completo dei cicli che partono dal prompt stesso non è possibile, ma i cicli interni alla catena degli import vengono comunque rilevati.

## Esempi pratici

Questo body contiene un numero di carta valido secondo l'algoritmo Luhn — il tipo di dato che non vuoi mai lasciare in un prompt:

```
La carta da rimuovere è 4111 1111 1111 1111 grazie.
                        ^^^^^^^^^^^^^^^^^^^
                        PII003 — Luhn valido
```

Il linter lo marca in rosso come `PII003`: un errore da correggere prima di condividere o compilare il prompt.

Qui invece l'import punta a un titolo che non esiste nel vault:

```
{{import "ricetta-segreta-ma-non-esiste"}}
                                          ^
                                          IMP001 — non risolto
```

La Diagnosi segnala `IMP001`: alla compilazione quell'import non porterebbe alcun contenuto — di solito è un titolo scritto male o un prompt rinominato.

Un warning tipico è l'email lasciata in un esempio:

```
Manda un messaggio a mario.rossi@example.com appena puoi.
                     ^^^^^^^^^^^^^^^^^^^^^^^
                     PII001 — possibile email
```

`PII001` non blocca nulla: ti invita solo a controllare se quell'indirizzo è un esempio innocuo o un dato reale finito nel posto sbagliato.

Infine, un suggerimento a bassa priorità:

```
ciao
^^^^
LEN002 — body corto (<30 caratteri)
```

Un body di quattro caratteri è quasi certamente un prompt lasciato a metà: `LEN002` te lo ricorda in grigio, senza insistere.

## Personalizzare il linter

Le regole sono tarate su un uso generico, e prima o poi qualcuna entrerà in rotta di collisione con il tuo modo di lavorare — magari le email nei tuoi prompt sono legittime, o i tuoi body superano fisiologicamente i 4 000 caratteri. In quel caso non devi conviverci: ogni regola si può silenziare, declassare o ritarare.

### Dove

**Impostazioni → Linter.** Trovi il catalogo completo raggruppato per categoria (Lunghezza, Segnaposti, Privacy, Stile, Import). Ogni voce mostra il codice (`PII001`), la severità e una breve descrizione.

### Come

- **Interruttore di famiglia** (es. *Privacy / PII*): spento, nasconde tutte le regole di quella categoria. Quando una famiglia è spenta, gli interruttori delle sue singole regole sono disattivati (è già tutto silenziato).
- **Interruttore di regola** (es. *PII001*): silenzia solo quella regola, lasciando attive le altre della stessa famiglia. Utile per zittire un singolo falso positivo ricorrente.
- **Severità** (menu a tendina per ogni regola): cambia il livello con cui l'avviso viene mostrato — *Errore*, *Avviso* o *Info*. Es. declassare *PII001* da Avviso a Info se le email nei tuoi prompt sono legittime.
- **Soglie numeriche**: alcune regole hanno un valore regolabile — *Caratteri massimi* (LEN001), *Caratteri minimi* (LEN002) e *Ripetizioni minime* (STY001). Alzale o abbassale per adattare la sensibilità al tuo stile.
- **Ripristina** (per regola) riporta severità e soglia di quella regola ai valori predefiniti; **Ripristina tutto** azzera ogni personalizzazione (disattivazioni, severità, soglie).

Le modifiche hanno effetto **subito**: la tab Diagnosi si ri-analizza appena cambi qualcosa, senza bisogno di rieditare il prompt. Valori fuori scala vengono riportati entro limiti sensati (es. la soglia ripetizioni non scende sotto 2).

### Note

Le regole silenziate sono **escluse dal conteggio** e dalla lista della tab Diagnosi: spariscono dalla vista, non ti distraggono più.

Le preferenze sono **locali a questo dispositivo** (non sincronizzate): reinstallando l'app o cambiando macchina riparti dal catalogo completo.

Il pannello Diagnosi è inoltre **collassabile** (chevron in alto a destra): quando è chiuso non aggiorna le diagnosi al cambio del body, ma riapre con i risultati dell'ultimo controllo.

## Vedi anche

- [`glossario-sintassi.md`](./glossario-sintassi.md) — tabella riassuntiva dei codici linter nel reference di sintassi.
- [`prompt-componibili.md`](./prompt-componibili.md) — la guida agli import, per capire a fondo le regole `IMP*`.
