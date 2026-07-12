# Rating dei prompt

> Come valutare un prompt dopo l'uso (👎 / 😐 / 👍) e leggere l'aggregato nella tab Valutazioni per capire quali prompt funzionano nel tempo. Disponibile da `v0.4.0`.

Dopo aver compilato e copiato un prompt dalla Command Palette o dal
Compilatore, puoi lasciare un **feedback discreto** a 3 valori
(👎 / 😐 / 👍). Il rating è **append-only** con timestamp: ogni
voto è una riga separata, così emerge la traiettoria nel tempo (un
prompt molto usato che inizia a prendere voti bassi è candidato a
refactor).

## Come dare un rating

1. Compila il prompt (Command Palette `Ctrl+Shift+P` o detail pane →
   "Compila").
2. Sotto l'output compilato, la modale mostra il blocco espandibile
   **"Valuta il risultato"** con tre bottoni: **Negativo** / **Neutro**
   / **Positivo** (icone 👎 😐 👍).
3. Click su un bottone registra il voto e mostra la conferma
   "— grazie!" inline accanto al titolo del blocco.
4. Prima di votare puoi espandere il campo **"Aggiungi nota"**
   (collassato di default) per annotare cosa ha funzionato o meno.
5. Se chiudi la modale senza votare non viene registrato nulla — il
   rating è sempre opzionale.

Errori di rete/DB sono silenziosi: il rating è non-bloccante per la
UX di "compila & usa".

## Aggregato visibile nella tab "Valutazioni"

Il detail pane della Libreria ospita la tab **Valutazioni** con
l'aggregato dei voti:

- **Media firmata** a 2 decimali (es. `+0.75`) su N voti.
- **Barre di distribuzione** Positivi / Neutri / Negativi con
  conteggio per fascia.

Senza voti la tab mostra uno stato vuoto con l'invito a valutare
dalla modale Compila. Nella riga dei metadata il detail pane mostra
solo il chip di utilizzo `Usato N×`.

## Convenzioni dei 3 valori

- **👍 (+1)**: l'output del modello rispetta l'intento del prompt
  (con il modello scelto).
- **😐 (0)**: parziale — funziona ma serve editing manuale.
- **👎 (−1)**: il prompt non ha funzionato come atteso.

I valori sono volutamente discreti per ridurre il bias culturale (5
stelle ha il problema "italiani 3 = ok, americani 5 = ok").

## Schema dati

Vedi `docs/architettura/schema-dati.md` § V013. Riassunto:

| Campo | Note |
|---|---|
| `Rating` | INTEGER `CHECK IN (-1, 0, 1)` |
| `Note` | TEXT opzionale (whitespace-only → NULL) |
| `UsedWithModel` | TEXT opzionale, popolato dal Compilatore con `target_model` del prompt |
| `CreatedAt` | DEFAULT `datetime('now')` |

Append-only: nessun UPDATE. L'utente che cambia idea aggiunge un
nuovo voto, e l'aggregato media le entry più recenti.

## Comandi Tauri esposti

| Comando | Cosa fa |
|---|---|
| `rating_aggiungi(nuovo)` | Insert con `UserId='usr-locale'`. Ritorna l'id `rtg-<hex>`. Errore se rating fuori range o prompt inesistente. |
| `rating_aggregato(prompt_id)` | Media + conteggio + distribuzione `pos/neu/neg`. `media: null` se nessun rating. |

## Limiti noti / roadmap

- ✅ **Modale "Aggiungi nota"** atterrata in `v0.5.0`: solo il voto
  👎 apre la modale "Cosa non ha funzionato?" con textarea opzionale.
  😐 e 👍 salvano subito senza friction (resta la nota inline
  opzionale nel blocco di valutazione). "Salta e registra voto"
  memorizza il voto senza nota.
- ✅ **Sort by quality "Migliori prompt"** atterrato in `v0.5.0`:
  nuova option "Migliori" nel dropdown ordine della Libreria.
  Ordina per `AVG(Rating)` ultimi 90 giorni (DESC), prompt senza
  rating in fondo. Tie-breaker su `UseCount` + `UpdatedAt`. Con
  l'ordine "Migliori" attivo le card della lista mostrano il voto
  medio `±X.XX` (da `v0.8.29`).
- **Privacy team**: oggi `usr-locale` (single-user). Nel workspace
  team gli admin vedono aggregati ma non singoli rating con note
  — scope Fase 5 con E2E.
- **Aggregato per modello**: il campo `UsedWithModel` è popolato ma
  il dashboard non lo filtra. Atterrabile post-v0.4 in `Insight.svelte`.

## Vedi anche

- [`schema-dati.md`](../architettura/schema-dati.md) — schema dati dei rating (§ V013).
- [`fase-4-workflow.md`](../roadmap/fase-4-workflow.md) — spec roadmap (Step 2).
- Implementazione: `apps/client/src-tauri/src/rating.rs` — backend del rating.
