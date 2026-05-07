# Varianti / A-B testing dei prompt

> Disponibile da `v0.4.0`.

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

Dal **detail pane** della Libreria, click su **"+ Variante"**:

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

## Etichette custom

Per dare un nome semantico (es. `Formal`, `Concise`, `Per-junior`),
oggi serve modificarla via SQL nel vault — il backend supporta già
qualunque stringa, manca solo lo step UI per l'input. Quick win
candidato per `v0.5.0`.

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

- **UI Editor "Crea variante"**: oggi solo dalla Libreria. Quick win
  per `v0.5.0`.
- **Confronto varianti dedicato** (vista N colonne con metadata
  affiancate) — riusabile via [`Confronto fianco-a-fianco`](./README.md):
  Cmd/Ctrl+click sulle pillole varianti per metterle nel set di
  confronto.
- **Promozione a principale** (swap main ↔ variant) non implementata.
  Oggi il "principale" resta sempre il primo prompt creato; per
  cambiare paradigma duplica manualmente.

## Riferimenti

- Implementazione: `apps/client/src-tauri/src/varianti.rs`
- Schema: `docs/architettura/schema-dati.md` § V011
- Spec roadmap: `docs/roadmap/fase-4-workflow.md` Step 1
