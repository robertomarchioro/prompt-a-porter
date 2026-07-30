---
name: dipendenze-transitive
description: Svuota gli alert Dependabot che Dependabot NON sa chiudere da solo — le vulnerabilità in dipendenze npm/pnpm transitive. Censisce, rinfresca il lockfile e misura, classifica solo il residuo, apre PR, attende CI, mergia il rinfresco se la CI è verde, apre issue sui fallimenti e lascia gli override alla decisione dell'utente. Usala quando il contatore degli alert nella scheda Security non scende, o l'utente dice "svuota gli alert" / "gestisci le dipendenze transitive".
---

# /dipendenze-transitive — svuotare gli alert che Dependabot non chiude

Invocazione: `/dipendenze-transitive` (nessun argomento). Variante
`/dipendenze-transitive --solo-censimento`: esegui i passi 1–3 e presenta il referto in
chat, poi **scarta tutto** (branch e modifiche) lasciando il working tree pulito. Serve
per sapere cosa c'è senza impegnarsi a nulla.

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

## Il principio: prima misura, poi classifica

**Il rinfresco È il triage.** Non provare a prevedere cosa si sposterà: fai
`pnpm update`, misura, e classifica **solo ciò che sopravvive**.

Questo non è pigrizia, è accuratezza. Una prima versione di questa skill classificava a
tavolino confrontando la versione di fix col range del genitore immediato. Al primo giro
reale ha sbagliato **2 pacchetti su 9**, entrambi in direzione pessimistica:

- **`@hono/node-server`** dato per «bloccato» perché l'SDK 1.29.0 dichiarava `^1.19.9` e
  il fix era `2.0.5`. Ma `apps/mcp-server/package.json` dichiarava
  `@modelcontextprotocol/sdk: ^1.0.0`: l'SDK stesso si è spostato a 1.30.0, che richiede
  la linea 2, e il problema si è risolto da sé.
- **`fast-uri`** dato per «override da aggiornare». L'override `>=3.1.2` già permetteva
  la 4.1.1: bastava rinfrescare.

L'errore è lo stesso: guardare se il range del genitore **immediato** permette il fix,
senza chiedersi se **il genitore stesso possa muoversi** entro i range dichiarati nei
nostri manifest. La catena è ricorsiva, e `pnpm update` la risolve tutta in una volta —
gratis e senza sbagliare.

Quindi: classificare *a priori* è più lavoro **e** più errori. Misura.

## Le tre destinazioni

Dopo il rinfresco, ogni alert è in una di tre situazioni.

| | **Risolto dal rinfresco** | **Override applicabile** | **Bloccato a monte** |
|---|---|---|---|
| Come lo scopri | è sparito da `pnpm audit` | sopravvive, ma esiste un vincolo imponibile | sopravvive e nessuna mossa è sicura |
| Cosa hai fatto | applicato l'intenzione dei genitori | **scavalcato** l'intenzione del genitore | riconosciuto il limite |
| Dove può rompersi | a build time → la CI lo prende | **a runtime** → la CI può non prenderlo | — |
| Esito | ✅ **auto-merge su CI verde** | ⛔ **PR aperta, decide l'utente** | 📋 **referto, con la condizione per riprovare** |

Il motivo per cui l'override non si auto-mergia: forzare una transitiva di
`@modelcontextprotocol/sdk` a una versione che l'SDK non ha mai dichiarato di supportare
può rompere il server MCP **a runtime**. La CI verde non dimostra che Claude Desktop ci
parla ancora.

### «Bloccato a monte» non è «override»

Confonderle porta a proporre override che rompono. Caso reale del primo giro:
**`vite` + `esbuild`** — la leva sarebbe bumpare `vitepress`, ma **anche l'ultima
vitepress (1.6.4) dichiara `vite: ^5.4.14`**: non esiste una versione che risolva, e
forzare vite 6 dentro vitepress 1.x romperebbe la build del sito.

Nell'override **esiste** una mossa e la domanda è se accettarne il rischio. Nel bloccato
**non esiste**, e l'unica azione corretta è registrare la condizione per riprovare
(es. *«quando vitepress passa a vite ≥6»*).

Per ogni voce bloccata **valuta l'esposizione reale** e scrivila: spesso è nulla, e
cambia l'urgenza. Esempi: vite/esbuild costruiscono il sito e non vengono spediti;
`@hono/node-server` è un adattatore HTTP mai istanziato perché il server MCP usa
`StdioServerTransport`.

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

## Passo 2 — Rinfresco e misura

Questo passo **è** il triage: fa il lavoro e ti dice cosa resta.

```bash
git checkout -b deps/rinfresco-lockfile-<AAAAMMGG>

# audit PRIMA — annota il numero
pnpm audit --audit-level=low 2>&1 | grep -E "^Severity:|vulnerabilities found"

pnpm update --recursive
```

### ⚠️ Ripristina i `package.json`

`pnpm update --recursive` **non tocca solo il lockfile**: alza anche i *floor* dichiarati
nei `package.json` (es. `"@codemirror/lang-markdown": "^6"` → `"^6.5.1"`). Al primo giro
ne ha modificati tre, incluso `@tauri-apps/plugin-updater` da `^2.0.0` a `^2.10.1` —
una dichiarazione sul percorso di rilascio.

Quei floor **non servono** al fix di sicurezza, e i range dei manifest sono competenza di
Dependabot. Ripristinali e verifica che il risultato non cambi:

```bash
git checkout -- apps/*/package.json packages/*/package.json package.json
pnpm install     # pnpm riallinea il lockfile ai specifier originali

# audit DOPO — deve dare lo stesso numero che dava coi package.json modificati
pnpm audit --audit-level=low 2>&1 | grep -E "^Severity:|vulnerabilities found"
git diff --stat  # atteso: SOLO pnpm-lock.yaml
```

Al primo giro: 31 → 4 in entrambi i casi, quindi il diff è passato da 4 file a 1. Se
invece il ripristino **peggiora** il risultato, allora quei floor servivano davvero:
tienili, e spiega nella PR quali e perché.

> **Gotcha noto**: dopo qualsiasi modifica al lockfile serve `pnpm install`, altrimenti i
> comandi successivi girano su `node_modules` stantio. È già costato un inciampo durante
> un `/bump`.

### Cosa hai ottenuto

```bash
# quali versioni sono ora installate, per i pacchetti degli alert
for p in <elenco pacchetti>; do
  printf "%-22s " "$p"
  grep -oE "^  '?${p}@[0-9][^':]*" pnpm-lock.yaml | sed "s|^  '\?${p}@||" | sort -uV | tr '\n' ' '
  echo
done
```

Confronta ognuna con la versione di fix dell'advisory: **≥ fix = risolto**. Il resto
sopravvive e va al passo 3.

Se `git diff` è vuoto il rinfresco non ha spostato nulla: cancella il branch e vai
direttamente al passo 3.

---

## Passo 3 — Triage del residuo

**Solo per gli alert sopravvissuti al rinfresco.** Per ognuno servono tre informazioni:
chi lo tira dentro, quale range il genitore dichiara, quale versione è installata.

### ⚠️ Il range NON si legge dal lockfile

`pnpm-lock.yaml` memorizza le versioni **risolte**, non i range dichiarati dai
genitori (i range compaiono solo per le `peerDependencies`). Cercare `hono: ^4` nel
lockfile funziona per caso e non in generale — al primo giro reale questo passo ha
prodotto dati inutilizzabili.

**La fonte autorevole è il registry:**

```bash
# 1. chi tira dentro il pacchetto, e in che catena
pnpm why <pacchetto> --recursive

# 2. che range dichiara il genitore (AUTOREVOLE)
npm view <genitore>@<versione-installata> dependencies.<pacchetto>

# 3. che versione è installata (attenzione: possono essercene più di una)
grep -oE "^  '?<pacchetto>@[0-9][^':]*" pnpm-lock.yaml | sort -uV
```

Sul punto 3: se ci sono **più versioni installate**, verifica quale è quella
vulnerabile. Capita che la copia recente sia già sana e solo una vecchia catena sia
esposta (es. `vite` presente in 5.4.21 *e* 8.1.5: solo la 5.x era vulnerabile).

### Come instradare

Confronta il range dichiarato con la versione di fix:

- il fix **rientra** nel range → è un caso che il rinfresco avrebbe dovuto risolvere:
  se è sopravvissuto, indaga (lockfile non riallineato? `pnpm install` mancante?)
- il fix **non rientra** (serve un major, o il genitore pinna esatto) → verifica se
  esiste una versione **del genitore** che risolve:
  - **sì** → è un bump del genitore: se è una dipendenza diretta lo fa Dependabot,
    altrimenti è un caso da **override**
  - **no** (nemmeno l'ultima versione del genitore) → **bloccato a monte**

```bash
# esiste una versione del genitore che risolve?
npm view <genitore> version                      # ultima pubblicata
npm view <genitore>@latest dependencies.<pacchetto>
```

### ⚠️ Controllo obbligatorio: gli override esistenti sono ancora validi?

Un override scritto per una vecchia vulnerabilità **non protegge da una nuova** su
versioni più alte. È già capitato in questo repo:

```json
"pnpm": { "overrides": { "fast-uri": ">=3.1.2" } }
```

`>=3.1.2` risolveva a `4.1.0` — **esattamente la versione vulnerabile** dell'advisory
successivo (`>=4.0.0 <=4.1.0`, fix `4.1.1`). L'override sembrava una protezione
attiva e non lo era più.

C'è di peggio, e va sempre verificato: **`ajv` dichiara `fast-uri: ^3.0.1`**. Da sola
la risoluzione sarebbe rimasta sulla linea 3.x, che non è vulnerabile. **È stato
l'override a spingere su 4.x, creando l'esposizione che poi doveva prevenire.**

Quindi per ogni voce in `pnpm.overrides` fai **due** controlli:

1. il range **esclude** la finestra vulnerabile corrente?
2. che versione si otterrebbe **senza** l'override (`npm view <genitore>@<ver>
   dependencies.<pacchetto>`)? Se senza override si starebbe su una linea sana, i
   rimedi sono **due e opposti** — alzare il vincolo, oppure **rimuovere l'override**.
   Presentali entrambi all'utente, con il motivo per cui l'override fu aggiunto (cercalo
   in `git log -p -- package.json`): rimuoverlo è sicuro solo se l'advisory originario
   non riguarda più la linea a cui si tornerebbe.

È la classe di errore più insidiosa qui: non decidere da sola.

### Referto di triage

Presenta in chat, prima di toccare qualsiasi file:

Conta gli **alert**, non i pacchetti — ma indica anche quanti pacchetti distinti sono:
è il numero che dice quanto lavoro c'è davvero (al primo giro, 32 alert erano 10
pacchetti).

```
RISOLTO DAL RINFRESCO (già fatto, va in PR)  — N alert / M pacchetti: hono ×15, undici ×7, ...
OVERRIDE (sopravvissuto, decidi tu)          — N alert: ...
BLOCCATO A MONTE                             — N alert: vite ×3 (vitepress ferma a ^5.4.14) ...
FUORI PERIMETRO (cargo/go)                   — N alert
SENZA FIX A MONTE                            — N alert
ANOMALIE (direct non gestite da Dependabot)  — N alert
```

Per ogni voce **bloccata** aggiungi la **condizione per riprovare** e l'**esposizione
reale**. Senza quelle due informazioni il referto dice solo "non si può fare", che è
inutile al prossimo giro.

### Con `--solo-censimento`: scarta e fermati

Il referto è pronto, e il rinfresco del passo 2 era solo strumentale. Butta via tutto:

```bash
git checkout -- .
git checkout main
git branch -D deps/rinfresco-lockfile-<AAAAMMGG>
pnpm install            # riporta node_modules al lockfile di main
git status --short      # deve essere pulito
```

Dillo esplicitamente: *«ho misurato rinfrescando su un branch usa-e-getta, ora scartato —
nulla è stato modificato»*.

---

## Passo 4 — PR del rinfresco

Il lavoro è già fatto nel passo 2: qui lo si consegna. Convenzione del repo: italiano,
conventional commit, **squash merge**, nessuna riga di attribuzione.

```bash
git add pnpm-lock.yaml
git commit -F -   # messaggio multilinea, vedi sotto
git push -u origin HEAD
gh pr create --title "..." --body "..."
```

Il messaggio di commit deve contenere la tabella `pacchetto: da → a (N alert, fix ≥X)` e
il conteggio `pnpm audit` prima/dopo. Nel corpo della PR, in più:

- **la prova che non stai forzando nulla**: per ogni pacchetto, chi dichiarava il range
  che permetteva il fix;
- **se un major è entrato**, spiega da dove viene. Al primo giro `@hono/node-server` è
  passato 1.x → 2.x, e sembrava una forzatura: era arrivato perché
  `@modelcontextprotocol/sdk` si era mosso 1.29 → 1.30 entro il `^1.0.0` dichiarato in
  `apps/mcp-server/package.json`. Senza quella riga la PR sembra pericolosa e non lo è;
- **le residue**, con condizione per riprovare ed esposizione reale;
- **se hai tenuto dei floor nei `package.json`**, quali e perché.

### CI attesa

`pnpm-lock.yaml` attiva `client-build` (job `lint-and-test` e `rust-test`) e
`mcp-server-build` (job `lint-and-build`). Se toccasse `apps/site/**` attiva anche
`site-deploy` (al merge). Attendi in poll con `gh pr checks <n>` finché tutti i job sono
pass o fail — non chiudere il task prima. `rust-test` è il più lento (~5 min).

### Esito

- **verde** → `gh pr merge <n> --squash`. Il branch si cancella da solo
  (`delete_branch_on_merge` è attivo).
- **rosso** → vai al passo 6.

---

## Passo 5 — Override, per ciò che il rinfresco non ha risolto

Un branch e una PR **separati** da quelli del rinfresco: hanno rischio diverso e
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
decisione, ed ecco cosa conviene provare prima»*. **Non mergiare mai** una PR di override,
nemmeno su CI verde, nemmeno se l'utente ha approvato quella del rinfresco nello stesso giro.

---

## Passo 6 — Se la CI è rossa

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

## Passo 7 — Referto finale

Sempre, anche quando tutto va bene:

```
Alert aperti prima  : N
Chiusi dal rinfresco: N   (PR #nnn, mergiata)
In attesa override  : N   (PR #nnn, aperta — serve tua decisione)
Bloccato a monte    : N   (condizione per riprovare: ...)
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
- **Non mergia gli override.** Mai. Vedi passo 5.
- **Non tocca `Cargo.lock` né `go.sum`.** Fuori perimetro.
- **Non tocca `/bump`, `release.yml` o i tag.** Se un alert richiedesse una modifica al
  percorso di release, fermati e segnalalo.
- **Non decide di accettare un rischio.** Le voci senza fix a monte vanno all'utente.

## Riferimenti

- Analisi completa delle impostazioni di sicurezza del repo, e la diagnosi da cui nasce
  questa skill: pagina Notion *"Impostazioni GitHub del repo — analisi e mappa delle
  azioni"*, nell'hub privato del progetto.
- Mappatura path → workflow CI: `docs/contribuire/ci-workflows.md`.
