# Fork / Clone dei prompt

> Disponibile da `v0.4.0`.

Un **fork** è una copia indipendente di un prompt che mantiene un
riferimento all'originale via `ForkOfPromptId`. Caso d'uso primario:
sperimentare modifiche su un prompt **team** senza toccare l'originale.

## Quando usare un fork

- Prompt team che vuoi adattare al tuo flusso senza chiedere il
  consenso dell'autore.
- Refactor sperimentale su un prompt critico — fai fork, modifichi,
  testi con golden examples ([`regression-testing.md`](./regression-testing.md)),
  poi decidi se sostituire l'originale.
- Conservazione storica: tieni una "copia" privata di un prompt che
  potrebbe essere modificato da altri nel team.

## Cosa fa "Fork"

Click su **"Fork"** nel detail pane della Libreria:

| Campo | Valore del fork |
|---|---|
| `Title` | `<originale> (fork)` |
| `Visibility` | `private` (forzato — anche se l'originale è team) |
| `WorkspaceId` | `ws-personale` (workspace personale dell'utente) |
| `AuthorUserId` | `usr-locale` |
| `ForkOfPromptId` | id dell'originale |
| Tag | tutti copiati dall'originale |
| Body / TargetModel / FolderId | ereditati |

In più: snapshot v1 in `PromptVersions`, hook FTS/embedding/imports
applicati, audit log `fork.creato`.

## Tracciabilità "Fork di X"

Quando apri il dettaglio di un fork, sotto i metadata appare un
banner:

```
Fork di "Nome originale"
```

Click sul banner naviga al prompt originale. Se l'originale è stato
soft-deletato, il banner mostra `(eliminato)` e diventa non-cliccabile.

Se l'originale è stato rimosso del tutto (caso edge: dati corrotti),
il banner mostra `Fork di un prompt non più disponibile` ma il fork
resta funzionante.

## Differenza con le Varianti

Vedi tabella in [`varianti-prompt.md`](./varianti-prompt.md) §
"Differenza con i Fork". Riassunto:

- **Variante** = formulazione alternativa stesso intento, stesso
  workspace, stessa visibility.
- **Fork** = clone indipendente con responsabilità separata, sempre
  privato.

## Fork di fork (chain)

Puoi forkare un fork. La policy V012 mantiene la `ForkOfPromptId`
sempre al **diretto** padre, non flatten transitivo:

```
prm-orig  ←  prm-fork-A  ←  prm-fork-B
              (ForkOfPromptId          (ForkOfPromptId
               = prm-orig)              = prm-fork-A)
```

Per tornare all'originale di una catena, naviga ricorsivamente via
banner.

## Comandi Tauri esposti

| Comando | Cosa fa |
|---|---|
| `prompt_fork(prompt_id)` | Duplica con `(fork)` suffix e visibility `private`. Ritorna l'id del fork. |
| `fork_info(prompt_id)` | Se è un fork ritorna `{ original_id, original_titolo?, original_eliminato }`. Resiliente a soft-delete e record assenti. |

## Limiti noti / roadmap

- **Contatore "N fork attivi"** lato originale (utile per workspace
  team) — schema già pronto via indice `idx_prompts_fork_of`,
  manca solo UI. Quick win `v0.5.0`.
- **Pull request leggera** dal fork verso l'originale ("propongo
  modifica") — dipende da Step 6 approval workflow, atterra in Fase 5.
- **Differenze con originale**: oggi serve usare il
  [Confronto fianco-a-fianco](./README.md) selezionando fork +
  originale. Quick win: pulsante "Vedi diff con originale" diretto
  nel banner.

## Riferimenti

- Implementazione: `apps/client/src-tauri/src/fork.rs`
- Schema: `docs/architettura/schema-dati.md` § V012
- Spec roadmap: `docs/roadmap/fase-4-workflow.md` Step 5
