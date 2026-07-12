# Varianti / A-B testing dei prompt

> Come creare e navigare varianti B/C/…/Z di un prompt per testare formulazioni alternative, e in cosa differiscono dai fork. Disponibile da `v0.4.0`.

Una **variante** è un prompt che condivide l'intento col Parent ma
testa una formulazione alternativa. UseCount, rating, versioning di
ogni variante sono indipendenti — emerge naturalmente quale
formulazione funziona meglio.

## Quando usare le varianti

- Hai un prompt "ruolo esperto X" e vuoi provare un tono più
  formale/informale senza perdere l'originale.
- Sperimenti formulazioni equivalenti per scegliere la migliore con
  ratings + golden examples (vedi
  [`regression-testing.md`](./regression-testing.md)).
- Mantieni più stili dello stesso prompt allineati nello stesso
  workspace senza copia-incolla.

## Crea una variante

Dal **detail pane** della Libreria, click su **"+ Variante"** nel
pannello laterale destro: si apre la modale **"Crea variante"** con
un campo etichetta opzionale (vuoto = etichetta auto-assegnata).

1. Il prompt corrente diventa il "principale" e una nuova copia
   appare con etichetta `B` (la `A` è riservata al principale).
2. La copia eredita titolo (con suffisso `(B)`), descrizione, body,
   target_model, folder, e tutti i tag dell'originale.
3. La nuova variante viene selezionata automaticamente nel detail
   pane: puoi cambiare body / metadata indipendentemente.

Chiamate successive aggiungono `C`, `D`, …, `Z`. Oltre 25 varianti
il sistema usa fallback `V<N>` (es. `V26`, `V27`).

## Naviga tra varianti

Sotto i tag del detail pane appare la riga **"Varianti: B C D"** con
pillole cliccabili. Click su una pillola seleziona quella variante
nel pane. Click sull'etichetta del principale (titolo senza suffisso)
torna al principale.

Nella lista della Libreria le varianti sono evidenziate con rientro
e connettore "↳" verso il prompt padre **solo con ordinamento
"A-Z"** (dove le sorelle tendono a stare vicine); con gli altri
ordinamenti la lista resta piatta.

## Etichette custom

Per dare un nome semantico (es. `Formal`, `Concise`, `Per-junior`),
compila il campo etichetta nella modale **"Crea variante"** (bottone
"+ Variante" nel pannello laterale del dettaglio). Se lo lasci
vuoto, il sistema assegna la prossima lettera libera.

## Differenza con i Fork

| Aspetto | Variante | Fork |
|---|---|---|
| Intento | Stesso del Parent | Sperimentazione indipendente |
| Workspace | Stesso del Parent | Sempre `ws-personale` |
| Visibility | Eredita dal Parent | Forzata a `private` |
| Relazione | `ParentPromptId` | `ForkOfPromptId` |
| Esempi tipici | "Formale vs Informale" | "Editing privato di prompt team" |

Vedi anche [`fork-prompt.md`](./fork-prompt.md).

## Comandi Tauri esposti

| Comando | Cosa fa |
|---|---|
| `prompt_crea_variante(parent_id, etichetta?)` | Crea variante con etichetta auto-generata o custom. Riggancia automaticamente al grandparent se `parent_id` punta a una variante (gerarchie a 1 livello, no nipoti). |
| `varianti_lista(parent_id)` | Lista varianti attive ordinate per `VariantLabel ASC`. |

## Limiti noti

- ✅ **UI "Crea variante" nel dettaglio**: il bottone "+ Variante"
  nel pannello laterale del detail pane apre la modale con etichetta
  custom opzionale.
- **Confronto varianti dedicato** (vista N colonne con metadata
  affiancate) — riusabile via [`Confronto fianco-a-fianco`](./README.md):
  Cmd/Ctrl+click sulle pillole varianti per metterle nel set di
  confronto.
- ✅ **Promozione a principale** (swap main ↔ variant): voce
  "Promuovi a principale" nel menu contestuale della pillola
  variante. La "Rinomina etichetta" della pillola è invece ancora
  disabilitata.

## Vedi anche

- [`schema-dati.md`](../architettura/schema-dati.md) — schema dati delle varianti (§ V011).
- [`fase-4-workflow.md`](../roadmap/fase-4-workflow.md) — spec roadmap (Step 1).
- Implementazione: `apps/client/src-tauri/src/varianti.rs` — backend delle varianti.
