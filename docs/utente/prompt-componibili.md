# Prompt componibili

> Come importare il body di un altro prompt con `{{import "path"}}` e costruire un
> vault modulare: sintassi, nidificazione, blocco a una versione, variabili scopate.

Chi lavora molto con i prompt finisce quasi sempre per riscrivere le stesse
cose: la definizione del ruolo ("sei un esperto di…"), le regole di tono, il
formato di output richiesto. All'inizio si copia e incolla, e sembra
funzionare. Poi un giorno decidi di migliorare quella frase sul tono — e ti
accorgi che vive, in versioni leggermente diverse, dentro quindici prompt.
Correggerle tutte a mano è noioso; dimenticarne una è quasi garantito.

Gli import risolvono esattamente questo problema. Un prompt può **importare**
il body di un altro prompt con la sintassi `{{import "path"}}`: il blocco
comune lo scrivi una volta sola, in un prompt dedicato, e gli altri lo
richiamano. Quando lo aggiorni, tutti i prompt che lo importano vedono la
nuova versione alla compilazione successiva, senza che tu debba toccarli.

L'espansione avviene al volo quando compili il prompt finale — dalla modale
**Compila prompt** (bottone **Compila** nel pannello di dettaglio, o
selezionando il prompt dalla Command Palette) oppure via MCP. Fino a quel
momento il body resta com'è: un testo con dentro i suoi `{{import}}`,
leggibile e modificabile. Attenzione a un'eccezione: la CLI **non espande gli
import** — `pap render` li lascia intatti ed emette un warning su stderr
(vedi [`cli.md`](./cli.md)).

Il vault diventa così un sistema modulare: definisci una volta un "ruolo
esperto" o un "tono editoriale" e lo riusi in molti prompt senza copia-incolla.

## La prima volta

Il modo più rapido per capire gli import è costruirne uno.

1. Crea un prompt che faccia da modulo riusabile — ad esempio un prompt a
   root chiamato `ruolo-esperto-marketing`, con dentro solo la definizione
   del ruolo. Nessuna sintassi speciale: è un prompt normale.
2. Apri (o crea) il prompt che deve usarlo e, nel body, digita `{{import "`.
   L'editor ti viene incontro: parte l'autocomplete con i prompt del vault
   (lo puoi richiamare anche con `Ctrl+Space`). Scegli
   `ruolo-esperto-marketing` e chiudi le graffe.
3. Guarda la tab **Anteprima** del pannello di dettaglio: accanto al form dei
   segnaposti vedi il body già espanso, con il testo del prompt importato al
   posto della direttiva. È il modo più comodo per verificare che la catena
   di import produca quello che ti aspetti.
4. Compila con **Compila** (o dalla Command Palette): l'output copiato negli
   appunti contiene il testo espanso, non la direttiva.

Da qui in poi ogni ritocco a `ruolo-esperto-marketing` si propaga da solo a
tutti i prompt che lo importano.

## Sintassi

```
{{import "path/al/prompt"}}
```

Le regole sono poche e coerenti con i segnaposti `{{nome}}` che già conosci:

- Le doppie graffe sono **obbligatorie**.
- La keyword `import` è case-sensitive.
- Il path va **fra virgolette doppie**.
- Spazi extra dentro le graffe sono tollerati: `{{  import  "x"  }}`.

### Come viene risolto il path

Puoi indicare il prompt da importare in due forme, provate in quest'ordine:

1. **Cartella + titolo**: `marketing/email/cold-outreach` cerca il prompt
   `cold-outreach` dentro la cartella `/marketing/email`.
2. **Solo titolo**: `cold-outreach` cerca per titolo, senza distinguere
   maiuscole e minuscole. Se esistono più prompt con lo stesso titolo in
   cartelle diverse, vince quello a root oppure quello modificato più di
   recente.

Lo slash iniziale è opzionale: `"/marketing/email/cold"` e
`"marketing/email/cold"` sono equivalenti.

## Esempio

Vediamo la coppia modulo + prompt che lo usa. Questo è il modulo,
`ruolo-esperto-marketing`, salvato a root:

```
Sei un esperto di marketing B2B con 15 anni di esperienza. Le tue
risposte sono concise, dirette, basate su dati.
```

E questo è `email-cold-outreach`, in `/marketing/email/`, che lo importa
come prima riga:

```
{{import "ruolo-esperto-marketing"}}

Scrivi un'email cold outreach per il seguente contesto:
{{contesto}}

Tono: {{tono}}.
```

Quando compili `email-cold-outreach`, l'import viene sostituito dal body del
modulo e ottieni:

```
Sei un esperto di marketing B2B con 15 anni di esperienza. Le tue
risposte sono concise, dirette, basate su dati.

Scrivi un'email cold outreach per il seguente contesto:
{{contesto}}

Tono: {{tono}}.
```

Nota che `{{contesto}}` e `{{tono}}` sono sopravvissuti all'espansione:
sono segnaposti normali, e restano da compilare nel form come sempre.
L'import espande solo sé stesso.

## Nidificazione

Un prompt importato può a sua volta contenere `{{import "..."}}`: il client
risolve la catena in profondità. Per proteggerti da catene fuori controllo
esistono due limiti fissi: la profondità massima è di **5 livelli** (oltre,
la compilazione fallisce, e il linter ti avvisa con `IMP003` già quando la
catena può raggiungere il sesto livello) e l'output espanso non può superare
**~1 MB** (una salvaguardia contro le "bombe di compilazione", dove pochi
import nidificati moltiplicano il testo a dismisura).

## Cicli

Un ciclo di import — un prompt che, direttamente o passando per altri,
finisce per importare sé stesso — non può funzionare: l'espansione non
terminerebbe mai. Per questo è vietato in ogni forma:

- A → A (auto-import)
- A → B → A (indiretto)
- A → B → C → A (multi-passaggio)

Il linter rileva i cicli con la regola `IMP002` (errore) e mostra il punto
esatto nell'editor. Se provi comunque a compilare, la compilazione fallisce
con un messaggio chiaro.

## Bloccare l'import a una versione

Di default l'import risolve sempre alla versione corrente del prompt
importato: se aggiorni il modulo, tutti i prompt che lo importano vedono il
nuovo body alla prossima compilazione. È il comportamento che vuoi quasi
sempre — ma non sempre. Se un prompt delicato dipende da una formulazione
precisa del modulo, puoi **bloccarlo a una versione storica** con il
modifier `version=N`:

```
{{import "ruolo-esperto-marketing" version=3}}
```

Con `version=3` l'import legge lo snapshot 3 dalla cronologia delle versioni
del modulo: anche se il modulo evolve, questo prompt continua a usare il
testo che avevi verificato.

## Variabili a import

Il modifier `with` passa **valori scopati** ai segnaposti del prompt
importato. Serve quando il modulo contiene segnaposti che vuoi fissare al
momento dell'import, invece di compilarli ogni volta:

```
{{import "ruolo-esperto-marketing" with tono=formale, lingua="italiano formale"}}
```

I valori si applicano solo ai segnaposti del body importato, non a quelli
del prompt che importa. I valori con spazi vanno fra virgolette doppie; se
ripeti una chiave, vince l'ultima occorrenza.

I due modifiers si combinano — `version` va **prima** di `with`:

```
{{import "ruolo-esperto-marketing" version=3 with tono=formale}}
```

## Supporto nell'editor

L'editor conosce gli import e ti aiuta in più punti:

- **Autocomplete**: digitando `{{import "` (o con `Ctrl+Space`) l'editor
  suggerisce i prompt del vault da importare.
- **Token cliccabili**: gli `{{import}}` nel body sono evidenziati e
  cliccabili per saltare al prompt importato.
- **Tab "Import & Var."** nel pannello di dettaglio: mostra l'albero degli
  import (con eventuali varianti) e le variabili in gioco.
- **Tab "Anteprima"**: mostra il body con gli import già espansi, in tempo
  reale mentre editi — utile per verificare la catena senza compilare.
- La regola linter `IMP004` ti avvisa quando il prompt che stai modificando
  è importato da altri prompt: un promemoria che le tue modifiche si
  propagheranno.

## Linting

Quattro regole del linter sorvegliano i prompt componibili (per
personalizzarle vedi [`linting-regole.md`](./linting-regole.md)):

| Codice | Severità | Cosa segnala |
|---|---|---|
| `IMP001` | Error | Path che non risolve a nessun prompt |
| `IMP002` | Error | Ciclo (auto-import o multi-passaggio) |
| `IMP003` | Warning | Profondità raggiungibile > 5 livelli |
| `IMP004` | Info | Il prompt corrente è importato da N altri prompt |

Le regole `IMP*` sono attive **solo sui prompt salvati**, non sulla bozza
non ancora salvata. Le segnalazioni compaiono nella tab **Diagnosi** del
pannello di dettaglio.

## Anti-pattern

L'esperienza suggerisce qualche trappola da evitare.

**Catene troppo profonde.** Una catena come
`A importa B importa C importa D importa E` è già al limite dei 5 livelli.
Se senti il bisogno di andare oltre, probabilmente i moduli sono troppo
granulari: appiattisci la catena o consolida i frammenti.

**Fan-out estremo.** Un singolo prompt che importa venti moduli diversi è
difficile da leggere e da prevedere. Meglio consolidare i frammenti in due o
tre import strutturali (ad esempio: ruolo, tono, formato di output).

**Import condizionali.** "Includi questo modulo solo se…" non è supportato:
un import viene sempre espanso. Se ti serve una logica del genere, oggi la
strada è mantenere due varianti del prompt e scegliere a mano quale usare.

## Limiti noti

- Gli import condizionali non esistono: ogni `{{import}}` nel body viene
  sempre espanso.
- La CLI non espande gli import: `pap render` restituisce il body con le
  direttive intatte (con un warning). L'espansione è disponibile solo
  nell'app e via MCP.
- La profondità massima di nidificazione è 5 livelli e l'output espanso è
  limitato a ~1 MB: catene o moduli oltre queste soglie fanno fallire la
  compilazione.

## Vedi anche

- [`linting-regole.md`](./linting-regole.md) — dettaglio delle regole `IMP*` e come attivarle/disattivarle.
- [`glossario-sintassi.md`](./glossario-sintassi.md) — tutta la sintassi `{{…}}` di PaP in una pagina.
- [`cli.md`](./cli.md) — cosa fa (e non fa) `pap render` con gli import.
- [`varianti-prompt.md`](./varianti-prompt.md) — l'alternativa quando servono formulazioni parallele, non moduli condivisi.
