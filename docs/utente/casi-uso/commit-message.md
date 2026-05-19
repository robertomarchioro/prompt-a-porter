# Commit message

Template per generare commit message da un diff seguendo Conventional Commits.

## Quando usarlo

- Hai un diff staged ma scarsa voglia di pensare al wording.
- Vuoi enforce-are Conventional Commits in team senza ricordare le regole a memoria.
- Devi sintetizzare un PR con N commit in un titolo riassuntivo per merge.

## Prompt

**Titolo:** Commit message da diff (Conventional Commits)
**Tag:** `git`, `commit`, `dev`
**Modello target:** `claude-sonnet` o `gpt-4-mini`

````
Genera un commit message seguendo Conventional Commits per il diff seguente.

Regole:
1. Formato: `<tipo>(<scope opzionale>): <descrizione imperativa>`
2. Tipo: feat | fix | refactor | docs | test | chore | perf | ci
3. Scope: opzionale, modulo o area (es. `auth`, `api`, `ui`).
4. Descrizione: imperativo presente in italiano, minuscola, max 72 char totali (incluso prefisso).
5. NO punto finale.
6. Se il diff è "breaking change": prefisso `!` dopo il tipo (es. `feat!:`).

Se il diff merita un body (>1 paragrafo di motivazione):
- Riga vuota dopo la descrizione.
- Body in italiano, frasi piane, max 72 char per riga.
- Spiega il "perché", non il "cosa" (il diff mostra cosa).

NON includere "Co-Authored-By" o altre attribuzioni — quelle le aggiungo io.

Diff:

```diff
{{diff}}
```
````

## Esempio

- `diff`:

  ```diff
  --- a/src/auth/session.ts
  +++ b/src/auth/session.ts
  @@ -42,8 +42,12 @@ export class SessionManager {
     async refresh(token: string): Promise<Session> {
  -    const session = await this.api.refresh(token);
  -    return session;
  +    try {
  +      const session = await this.api.refresh(token);
  +      this.cache.set(session.id, session);
  +      return session;
  +    } catch (e) {
  +      if (e.code === "EXPIRED") this.cache.clear();
  +      throw e;
  +    }
     }
  ```

### Output atteso

```
feat(auth): cache della sessione dopo refresh + cleanup su expired

Aggiunge persistenza in cache della session post-refresh per ridurre
roundtrip al backend nelle chiamate successive (~30% del traffico
auth). Su errore EXPIRED svuota la cache per evitare di servire
session morte.
```

## Varianti

### Solo titolo (no body)

```
... Genera SOLO il titolo (prima riga), niente body.
Output: una sola riga.
```

### Multipli commit logici (split diff)

Quando il diff contiene cambiamenti scorrelati:

````
Il diff seguente potrebbe contenere modifiche scorrelate.

1. Identifica i "commit logici" (cambiamenti coesi).
2. Per ognuno, genera:
   - Lista dei file/righe coinvolti
   - Commit message Conventional Commits

Output formattato come:

### Commit 1
File: ...
Message: ...

### Commit 2
File: ...
Message: ...

Diff:

```diff
{{diff}}
```
````

Utile per fare `git add -p` mirato dopo aver visto il risultato.

### Stile alternativo (Angular-like, EN)

Per repository in inglese o standard non-Italian:

```
... Use English, present imperative ("add", "fix", "refactor"), no period.
Optional scope from the project's modules: {{moduli}}.
Subject line max 50 chars (Angular convention).
```

## Anti-pattern

- **Non includere il diff intero se è gigante**: per diff > 500 righe, il modello produce titoli vaghi. Spezza in più commit o riassumi prima.
- **Non lasciar inventare scope**: se non lo specifichi nel prompt, il modello inventa scope inconsistenti. Usa `with scope=auth` o passa la lista dei moduli del progetto come vincolo.
- **Non usare per commit "WIP"**: i WIP non vanno in main; semmai aspetta lo squash finale.
