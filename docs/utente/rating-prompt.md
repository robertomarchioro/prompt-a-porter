# Rating dei prompt

> Disponibile da `v0.4.0`.

Dopo aver compilato e copiato un prompt dalla Command Palette o dal
Compilatore, puoi lasciare un **feedback discreto** a 3 valori
(👎 / 😐 / 👍). Il rating è **append-only** con timestamp: ogni
voto è una riga separata, così emerge la traiettoria nel tempo (un
prompt molto usato che inizia a prendere voti bassi è candidato a
refactor).

## Come dare un rating

1. Compila il prompt (Command Palette `Ctrl+Shift+P` o detail pane →
   "Compila").
2. Click su **Copia** — il testo finisce nella clipboard.
3. In basso a destra appare un toast:
   ```
   Com'è andata con questo prompt?
   👎  😐  👍
   ```
4. Click su un'emoji registra il voto + mostra "Grazie!" per 1 secondo
   prima di chiudersi.
5. Se non interagisci, il toast si chiude da solo dopo 5 secondi senza
   registrare nulla — il rating è sempre opzionale.

Errori di rete/DB sono silenziosi: il rating è non-bloccante per la
UX di "compila & usa".

## Aggregato visibile nel detail pane

Il detail pane della Libreria mostra un badge accanto agli altri
metadata:

```
Privato · claude-sonnet · Usato 24 volte · 👍 87% su 23 · 2g fa
```

| Colore | Soglia | Interpretazione |
|---|---|---|
| 🟢 verde | ≥ 80% positivi | "Funziona bene" |
| 🟡 giallo | 50–79% | "Risultati misti" |
| 🔴 rosso | < 50% | "Candidato a refactor" |

Tooltip sul badge mostra la distribuzione completa: `N positivi · N
neutri · N negativi`.

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

- ✅ **Modale "Aggiungi nota"** atterrata in `v0.5.0`: dopo voto
  👎 o 😐 si apre una modale con textarea opzionale (max 500
  caratteri). 👍 salva subito senza friction. "Salta" memorizza il
  voto senza nota.
- ✅ **Sort by quality "Migliori prompt"** atterrato in `v0.5.0`:
  nuova option "Migliori" nel dropdown ordine della Libreria.
  Ordina per `AVG(Rating)` ultimi 90 giorni (DESC), prompt senza
  rating in fondo. Tie-breaker su `UseCount` + `UpdatedAt`.
- **Privacy team**: oggi `usr-locale` (single-user). Nel workspace
  team gli admin vedono aggregati ma non singoli rating con note
  — scope Fase 5 con E2E.
- **Aggregato per modello**: il campo `UsedWithModel` è popolato ma
  il dashboard non lo filtra. Atterrabile post-v0.4 in `Insight.svelte`.

## Riferimenti

- Implementazione: `apps/client/src-tauri/src/rating.rs`
- Schema: `docs/architettura/schema-dati.md` § V013
- Spec roadmap: `docs/roadmap/fase-4-workflow.md` Step 2
