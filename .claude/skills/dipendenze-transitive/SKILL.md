---
name: dipendenze-transitive
description: Svuota gli alert Dependabot che Dependabot NON sa chiudere da solo — le vulnerabilità in dipendenze npm/pnpm transitive. Censisce, classifica, rinfresca il lockfile, propone override, apre PR, attende CI, mergia la corsia sicura e apre issue sui fallimenti. Usala quando il contatore degli alert nella scheda Security non scende, o l'utente dice "svuota gli alert" / "gestisci le dipendenze transitive".
---

# /dipendenze-transitive — svuotare gli alert che Dependabot non chiude

Invocazione: `/dipendenze-transitive` (nessun argomento). Variante
`/dipendenze-transitive --solo-censimento`: esegui i passi 1 e 2 e presenta il
referto in chat, senza creare branch né PR.

---

## Perché esiste

Il repo ha gli alert Dependabot e i *security updates* attivi (2026-07-30). Ma i
security updates aprono una PR **solo per ciò che è dichiarato in un manifest**.

Alla prima ricognizione, **31 alert su 32 erano `relationship: transitive`**:
`undici` arriva dentro `jsdom` e `vitest`, `hono` dentro
`@modelcontextprotocol/sdk` → `@hono/node-server`. Nessun `package.json` li nomina,
quindi Dependabot non ha una riga su cui agire: zero PR, contatore fermo a 32 per
giorni.

**Questa skill copre quel buco.** Non sostituisce Dependabot: fa il lavoro che
Dependabot strutturalmente non può fare.

### Perimetro: solo npm/pnpm

Cargo e Go **non** rientrano. `cargo audit` e i due `govulncheck` in
`security-audit.yml` sono bloccanti e funzionano, e `govulncheck` fa anche analisi di
raggiungibilità (segnala solo ciò che è invocabile dal codice) — copertura migliore di
quella che darebbe questa skill. Se un alert riguarda `Cargo.lock` o `go.sum`,
segnalalo nel referto e **fermati**: si tratta a mano.

---

## Le due corsie, e perché hanno politiche diverse

Ogni alert transitivo finisce in una delle due. La distinzione è il cuore della skill:
non appiattirla.

| | **Corsia A — rinfresco** | **Corsia B — override** |
|---|---|---|
| Quando | il range del genitore **già permette** la versione corretta | il range del genitore **non** la permette |
| Come | `pnpm update` | voce in `pnpm.overrides` |
| Cosa stai facendo | applichi l'intenzione del genitore | **scavalchi** l'intenzione del genitore |
| Dove può rompersi | a build time → la CI lo prende | **a runtime** → la CI può non prenderlo |
| Merge | ✅ **automatico su CI verde** | ⛔ **mai automatico: PR aperta, decide l'utente** |

Esempio reale di corsia A: `@hono/node-server` dichiara `hono: ^4`, installato
`4.12.16`, fix `4.12.25`. `^4` lo permette già → un `pnpm update` basta, senza rischio.

Il motivo per cui la corsia B non si auto-mergia: forzare una transitiva di
`@modelcontextprotocol/sdk` a una versione che l'SDK non ha mai dichiarato di
supportare può rompere il server MCP **a runtime**. La CI verde non dimostra che
Claude Desktop ci parla ancora.

---

## Passo 0 — Preflight

```bash
cd /home/roberto/prompt-a-porter
git checkout main && git pull --ff-only
git status --short          # deve essere pulito
```

Se il working tree è sporco, **fermati** e dillo all'utente. Non stashare da sola.

---

## Passo 1 — Censimento

```bash
R=robertomarchioro/prompt-a-porter
gh api "repos/$R/dependabot/alerts?state=open&per_page=100" \
  -q '.[] | "\(.number)\t\(.security_advisory.severity)\t\(.dependency.package.ecosystem)\t\(.dependency.package.name)\t\(.dependency.relationship // "n/d")\t\(.dependency.scope)\tfix:\(.security_vulnerability.first_patched_version.identifier // "NESSUNO")"'
```

Raccogli per ogni alert: numero, severità, ecosistema, pacchetto, relationship
(`direct`/`transitive`), scope (`runtime`/`development`), versione che corregge.

Poi separa subito:

- **ecosistema ≠ npm** → fuori perimetro, elenca e non toccare
- **`relationship: direct`** → è competenza di Dependabot, non tua. Se ce ne sono e
  Dependabot non ha aperto la PR, segnalalo come **anomalia** nel referto: significa
  che qualcosa nella sua configurazione non funziona, e va indagato a mano.
- **`first_patched_version: NESSUNO`** → nessun rimedio esiste a monte. Non c'è niente
  da fare: va nel referto come voce che richiede una **decisione consapevole**
  dell'utente (accettare o rimuovere la dipendenza).
- **il resto** → transitive npm con un fix disponibile: è il materiale di lavoro.

---

## Passo 2 — Triage: corsia A o corsia B

Per ogni transitiva con fix, stabilisci se il range del genitore permette già la
versione corretta.

```bash
# chi lo tira dentro, e con quale range
pnpm why <pacchetto> --recursive
grep -B3 -A3 "<pacchetto>@" pnpm-lock.yaml | head -20
```

Nel lockfile cerca la riga del genitore, es. `'@hono/node-server@1.19.14':` seguita da
`hono: ^4`. Confronta quel range con la versione di fix:

- il fix **rientra** nel range → **corsia A**
- il fix **non rientra** (serve un major, o il genitore pinna esatto) → **corsia B**

### ⚠️ Controllo obbligatorio: gli override esistenti sono ancora validi?

Un override scritto per una vecchia vulnerabilità **non protegge da una nuova** su
versioni più alte. È già capitato in questo repo:

```json
"pnpm": { "overrides": { "fast-uri": ">=3.1.2" } }
```

`>=3.1.2` risolveva a `4.1.0` — **esattamente la versione vulnerabile** dell'advisory
successivo (`>=4.0.0 <=4.1.0`, fix `4.1.1`). L'override sembrava una protezione
attiva e non lo era più.

Per ogni voce già presente in `pnpm.overrides`, verifica che il range **escluda** la
finestra vulnerabile corrente. Se non la esclude, è una voce di corsia B da aggiornare,
e segnalalo esplicitamente: è la classe di errore più insidiosa qui.

### Referto di triage

Presenta in chat, prima di toccare qualsiasi file:

```
CORSIA A (rinfresco, auto-merge su verde)     — N voci: hono, ...
CORSIA B (override, richiede tua decisione)   — N voci: fast-uri (override stantio), ...
FUORI PERIMETRO (cargo/go)                    — N voci
SENZA FIX A MONTE                             — N voci
ANOMALIE (direct non gestite da Dependabot)   — N voci
```

Con `--solo-censimento`: **fermati qui.**

---

## Passo 3 — Corsia A: rinfresco del lockfile

```bash
git checkout -b deps/rinfresco-lockfile-<AAAAMMGG>
pnpm update --recursive
pnpm install            # riallinea node_modules al lockfile aggiornato
```

> **Gotcha noto**: dopo qualsiasi modifica al lockfile serve `pnpm install`, altrimenti
> i comandi successivi girano su `node_modules` stantio. È già costato un inciampo
> durante un `/bump`.

Misura l'effetto **prima** di aprire la PR:

```bash
pnpm audit --audit-level=low 2>&1 | grep -E "^Severity:|vulnerabilities found"
git diff --stat
```

Se `git diff` è vuoto, non c'è nulla da fare in questa corsia: cancella il branch e
passa al 4.

Commit e PR (convenzione del repo: italiano, conventional commit, **squash merge**,
nessuna riga di attribuzione):

```bash
git add pnpm-lock.yaml
git commit -m "fix(deps): rinfresca il lockfile per chiudere N alert transitivi

Aggiorna le transitive vulnerabili entro i range già dichiarati dai
genitori — nessun override, nessuna versione forzata.

Chiude gli alert Dependabot: <elenco numeri>.
Residuo dopo il rinfresco: <conteggio> (vedi PR di corsia B)."
git push -u origin HEAD
gh pr create --title "..." --body "..."
```

Nel corpo della PR metti sempre: la tabella prima/dopo di `pnpm audit`, l'elenco degli
alert che si chiudono, e per ognuno **il range del genitore che già permetteva il fix**
(è la prova che non stai forzando nulla).

### CI attesa

`pnpm-lock.yaml` attiva `client-build` e `mcp-server-build`. Se toccasse
`apps/site/**` attiva anche `site-deploy` (al merge). Attendi in poll con
`gh pr checks <n>` finché tutti i job sono pass o fail — non chiudere il task prima.

### Esito

- **verde** → `gh pr merge <n> --squash`. Il branch si cancella da solo
  (`delete_branch_on_merge` è attivo).
- **rosso** → vai al passo 5.

---

## Passo 4 — Corsia B: override

Un branch e una PR **separati** da quelli della corsia A: hanno rischio diverso e
destini diversi, non vanno mescolati.

```bash
git checkout main && git pull --ff-only
git checkout -b deps/override-<pacchetto>-<AAAAMMGG>
```

Modifica `pnpm.overrides` nel `package.json` di root. Usa il vincolo **più strettamente
sufficiente**: `">=<versione-fix>"`, non un pin esatto — così i futuri patch entrano da
soli senza dover ritoccare l'override.

```bash
pnpm install
pnpm audit --audit-level=low 2>&1 | grep -E "^Severity:"
```

Apri la PR e **fermati lì**. Nel corpo devi mettere, oltre al solito:

- **cosa stai scavalcando**: il genitore, il range che dichiara, la versione che stai
  forzando
- **perché la CI verde non è sufficiente**: quali percorsi runtime non sono coperti dai
  test (per il server MCP: il round-trip reale con Claude Desktop; per il client: lo
  smoke test manuale)
- **cosa dovrebbe provare l'utente a mano** prima di mergiare

Poi dillo esplicitamente in chat: *«PR di override aperta, non la mergio — serve la tua
decisione, ed ecco cosa conviene provare prima»*. **Non mergiare mai** questa corsia,
nemmeno su CI verde, nemmeno se l'utente ha approvato la corsia A nello stesso giro.

---

## Passo 5 — Se la CI è rossa

Non insistere e non tentare fix creativi: apri una issue e lascia la PR aperta come
contesto.

```bash
gh issue create \
  --title "deps: il rinfresco del lockfile rompe la CI (<job fallito>)" \
  --label dependencies --label ci \
  --body "..."
```

Se la creazione con label fallisce (label assente), riprova senza `--label`: è il
fallback che usa già `notifica-fallimento` in `security-audit.yml`.

Nel corpo della issue: link alla PR, link al run fallito, il job e lo step precisi,
l'errore citato **verbatim**, e l'ipotesi di causa. Se il fallimento è chiaramente
attribuibile a **un** pacchetto, proponi come prossimo passo di escluderlo dal rinfresco
e rifare il giro senza di lui — così il resto passa.

Poi riferisci all'utente, senza addolcire: quanti alert restano aperti e perché.

---

## Passo 6 — Referto finale

Sempre, anche quando tutto va bene:

```
Alert aperti prima  : N
Chiusi da corsia A  : N   (PR #nnn, mergiata)
In attesa corsia B  : N   (PR #nnn, aperta — serve tua decisione)
Fuori perimetro     : N   (cargo/go — a mano)
Senza fix a monte   : N   (decisione consapevole)
Alert aperti dopo   : N
```

Il conteggio finale va **riletto dall'API**, non calcolato per sottrazione — gli alert
si chiudono con un ritardo di qualche minuto dopo il merge:

```bash
gh api "repos/$R/dependabot/alerts?state=open&per_page=100" -q 'length'
```

Se il numero non è sceso come previsto, dillo: significa che il rinfresco non ha
raggiunto ciò che pensavi, non che il conteggio è sbagliato.

---

## Cosa questa skill NON fa

- **Non gira da sola.** Va invocata. Se serve l'esecuzione periodica non presidiata, è
  un lavoro schedulato da progettare a parte — e non prima di aver visto questa skill
  comportarsi bene qualche volta.
- **Non mergia gli override.** Mai. Vedi passo 4.
- **Non tocca `Cargo.lock` né `go.sum`.** Fuori perimetro.
- **Non tocca `/bump`, `release.yml` o i tag.** Se un alert richiedesse una modifica al
  percorso di release, fermati e segnalalo.
- **Non decide di accettare un rischio.** Le voci senza fix a monte vanno all'utente.

## Riferimenti

- Analisi completa delle impostazioni di sicurezza del repo, e la diagnosi da cui nasce
  questa skill: pagina Notion *"Impostazioni GitHub del repo — analisi e mappa delle
  azioni"*, nell'hub privato del progetto.
- Mappatura path → workflow CI: `docs/contribuire/ci-workflows.md`.
