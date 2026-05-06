# Prompt componibili

> Disponibile da `v0.3.0`.

Un prompt può **importare** il body di un altro prompt usando la
sintassi `{{import "path"}}`. Il client espande la catena al volo
quando compili il prompt finale (Command Palette > Compila & copia,
oppure via MCP/CLI).

Il vault diventa così un sistema modulare: definisci una volta un
"ruolo esperto" o un "tono editoriale" e lo riusi in molti prompt
senza copia-incolla.

## Sintassi

```
{{import "path/al/prompt"}}
```

- Le doppie graffe sono **obbligatorie** (coerenti con i segnaposti
  `{{nome}}`).
- La keyword `import` è case-sensitive.
- Il path va **fra virgolette doppie**.
- Spazi extra dentro le graffe sono tollerati: `{{  import  "x"  }}`.

### Path

Due forme accettate, in ordine di priorità:

1. **Cartella + titolo**: `marketing/email/cold-outreach` →
   risolve al prompt `cold-outreach` dentro `Folders.Path = '/marketing/email'`.
2. **Solo titolo**: `cold-outreach` → match case-insensitive su
   `Prompts.Title`. Se ci sono più prompt con lo stesso titolo
   (in cartelle diverse), vince quello a root o quello con
   `UpdatedAt` più recente.

Lo slash iniziale è opzionale (`"/marketing/email/cold"` ==
`"marketing/email/cold"`).

## Esempio

**Prompt `ruolo-esperto-marketing`** (a root):

```
Sei un esperto di marketing B2B con 15 anni di esperienza. Le tue
risposte sono concise, dirette, basate su dati.
```

**Prompt `email-cold-outreach`** (in `/marketing/email/`):

```
{{import "ruolo-esperto-marketing"}}

Scrivi un'email cold outreach per il seguente contesto:
{{contesto}}

Tono: {{tono}}.
```

Quando compili `email-cold-outreach`, il client espande in:

```
Sei un esperto di marketing B2B con 15 anni di esperienza. Le tue
risposte sono concise, dirette, basate su dati.

Scrivi un'email cold outreach per il seguente contesto:
{{contesto}}

Tono: {{tono}}.
```

I segnaposti `{{contesto}}` e `{{tono}}` restano da compilare nel
form della Command Palette (sono espansioni di parametri, non di
import).

## Nidificazione

Gli import si possono **nidificare**: il body importato può a sua
volta contenere `{{import "..."}}`. Il client risolve in profondità
fino a:

- **Profondità massima 5 livelli**. Oltre, errore di compilazione
  (e il linter avvisa con `IMP003` come warning già a 6 livelli
  raggiunti).
- **Output massimo ~1 MB**. Anti-bomba di compilazione (analogo
  billion-laughs in XML).

## Cicli

Un prompt **non può importarsi a vicenda** con un altro:

- A → A (self-loop): vietato
- A → B → A (indiretto): vietato
- A → B → C → A (multi-hop): vietato

Il linter rileva i cicli con `IMP002` (errore) e mostra il punto
esatto nell'editor. La compilazione fallisce con messaggio chiaro.

## Linting

Tre regole IMP* sui prompt componibili (vedi
[`linting-regole.md`](./linting-regole.md)):

| Codice | Severità | Cosa segnala |
|---|---|---|
| `IMP001` | Error | Path che non risolve a nessun prompt |
| `IMP002` | Error | Ciclo (self o multi-hop) |
| `IMP003` | Warning | Profondità raggiungibile > 5 livelli |

Le regole IMP* sono attive **solo sui prompt salvati** (non sul
draft non ancora committato). Compaiono nel pannello Diagnosi
dell'editor.

## Anti-pattern

### Catene troppo profonde

```
A imports B imports C imports D imports E imports F
```

Sei già al limite. Se devi andare oltre, probabilmente i moduli
sono troppo granulari: appiattisci o consolida.

### Import "fan-out estremo"

Un singolo prompt che importa 20 altri prompt diventa difficile
da capire. Considera di consolidare i frammenti in 2-3 import
strutturali.

### Import condizionali

Non sono supportati. Se hai bisogno di "include questo se quello",
oggi devi mantenere due varianti del prompt e scegliere a mano.
Future versioni potrebbero aggiungere `{{import_if cond "..."}}`.

### Pinning a versioni

Non supportato. L'import risolve sempre alla versione corrente del
prompt target. Se aggiorni `ruolo-esperto-marketing`, tutti i prompt
che lo importano vedono il nuovo body alla prossima compilazione.

Roadmap Fase 4: sintassi `{{import "x" version=N}}` per pinning a
una versione storica (richiede già `PromptVersions` esistente).

### Variabili a import

Non supportate in v0.3. Roadmap futura: `{{import "x" with k=v}}`
per passare parametri all'import (oggi i segnaposti sono globali al
prompt root, non scopati per import).

## Comandi Tauri esposti

| Comando | Cosa fa |
|---|---|
| `prompt_compila` | Espansione ricorsiva con cycle detection + depth check |

L'editor del prompt mostra un **CompilatorePrompt** dedicato
(superficie UI separata) che esegue la compilazione e mostra il
risultato finale espanso, utile per debug della catena di import.

## Riferimenti

- Implementazione: `apps/client/src-tauri/src/prompt_componibili.rs`
- Test: 16 unit test su parsing, resolve, cycle, depth
- Linting: [`linting-regole.md`](./linting-regole.md) regole IMP*
- Schema dipendenze: [`schema-dati.md`](../architettura/schema-dati.md) tabella `PromptImports`
