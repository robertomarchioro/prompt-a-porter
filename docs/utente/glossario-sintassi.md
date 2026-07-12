# Glossario sintassi

> Reference rapido della sintassi del body di un prompt: segnaposti, globali, import, codici linter.

Un prompt salvato così com'è è poco più di un appunto: la prossima volta che ti serve, devi rileggerlo e adattarlo a mano. La sintassi di Prompt a Porter esiste per trasformare quell'appunto in uno strumento: separi le parti che hai rifinito una volta per tutte (le istruzioni, il tono, la struttura) dalle parti che cambiano a ogni uso (il destinatario, il testo da elaborare, la lingua). Le prime le scrivi nel body; le seconde le dichiari come **segnaposti**, e l'app te le chiederà al momento della compilazione.

Su questa idea si innestano altri due mattoni. I **segnaposti globali** coprono i valori che cambiano raramente ma compaiono ovunque — il tuo nome, il ruolo, l'azienda — e vengono riempiti automaticamente, senza domande. Gli **import** permettono a un prompt di includerne un altro, così un blocco che funziona (un "ruolo esperto", un tono editoriale) vive in un posto solo e viene riusato da molti prompt.

Questa pagina è il reference della grammatica: consultala quando hai un dubbio di sintassi. Per gli approfondimenti sui prompt modulari c'è [`prompt-componibili.md`](./prompt-componibili.md), per il catalogo completo delle regole del linter [`linting-regole.md`](./linting-regole.md).

## Segnaposti

Un segnaposto è un parametro `{{nome}}` da compilare al momento dell'uso.

```
Ciao {{nome}}, oggi parliamo di {{argomento}}.
```

La grammatica è volutamente minima. Le doppie graffe sono **obbligatorie**: una graffa singola non è un segnaposto, e il linter la segnala come errore `PH001`. Il nome deve essere un identificatore — lettere, cifre e `_`, senza spazi né trattini. Gli spazi interni alle graffe invece sono tollerati: `{{ nome }}` equivale a `{{nome}}`. Lo stesso segnaposto può comparire più volte nel body: lo compili una sola volta e il valore viene riapplicato a ogni occorrenza.

In fase di compilazione, l'app mostra una form con un campo per ogni segnaposto unico. Se lasci un campo vuoto, il segnaposto resta intatto nel testo finale (così puoi riempirlo dopo manualmente).

### Esempio

Questo body usa tre segnaposti per parametrizzare un'operazione di riscrittura: tono, testo da riscrivere e lingua di output.

```
Riscrivi il seguente testo in tono {{tono}}:

"{{testo}}"

Lingua di output: {{lingua}}.
```

Compilando con `tono` → `formale`, `testo` → `Ciao, ti scrivo per...` e `lingua` → `italiano`, ottieni:

```
Riscrivi il seguente testo in tono formale:

"Ciao, ti scrivo per..."

Lingua di output: italiano.
```

Ogni segnaposto è stato sostituito dal suo valore; il resto del testo è arrivato intatto negli appunti.

## Segnaposti globali

Alcuni valori compaiono in decine di prompt ma cambiano quasi mai: il tuo nome, il ruolo, l'azienda, il tono editoriale di default. Ridigitarli a ogni compilazione sarebbe assurdo — per questo esistono i segnaposti globali: `{{global nome}}` viene riempito automaticamente con il valore che hai salvato una volta sola nelle impostazioni.

```
Sono {{global autore}}, responsabile {{global ruolo}} a {{global azienda}}.
```

I valori dei globali si gestiscono da **Impostazioni → Segnaposti globali**: coppie chiave-valore semplici, condivise fra tutti i prompt del vault.

**Differenza chiave con i segnaposti normali:**

| Caratteristica | `{{nome}}` (normale) | `{{global nome}}` |
|---|---|---|
| Form di compilazione | Chiede ogni volta | Mai (valore riempito automaticamente) |
| Dove vive il valore | Solo nel testo che digiti | In **Impostazioni → Segnaposti globali**, condiviso da tutto il vault |
| Override per uso | Sì (puoi modificare nel form) | No (per cambiarlo: Impostazioni) |
| Tipico per | Valori variabili (input, contesto) | Valori stabili (firma, ruolo, azienda) |

Se un globale non è stato definito nelle impostazioni, il testo `{{global nome}}` resta nel risultato finale (così te ne accorgi e lo aggiungi).

## Import

`{{import "path"}}` importa il body di un altro prompt nel corrente. È il mattone dei prompt modulari: un "ruolo esperto" riusato in molti casi d'uso, un "tono editoriale" condiviso — se lo migliori nel prompt sorgente, tutti i prompt che lo importano ne beneficiano.

Questo esempio importa un ruolo predefinito e ci aggiunge sotto il compito specifico:

```
{{import "ruolo-esperto-marketing"}}

Scrivi una email cold outreach per: {{contesto}}.
```

Alla compilazione, al posto della riga di import compare l'intero body del prompt "ruolo-esperto-marketing", seguito dal resto del testo.

Il path va scritto fra **virgolette doppie** ed è case-insensitive per il titolo. Puoi indicare solo il titolo (`"titolo"`: match a root o in qualunque cartella, con priorità a root) oppure il percorso esplicito (`"cartella/sotto/titolo"`). Lo slash iniziale è opzionale: `"/marketing/email"` equivale a `"marketing/email"`.

**Modificatori (sintassi estesa):**

| Sintassi | Effetto |
|---|---|
| `{{import "path" version=3}}` | Importa la versione N dal repository di versioni (non l'ultima). |
| `{{import "path" with tono=formale, lingua=en}}` | Sostituisce i segnaposti del prompt importato con i valori dati. |
| `{{import "path" version=3 with tono=formale}}` | Combina i due (version **prima** di with). |

Gli import hanno tre limiti pensati per proteggerti: la profondità massima di nidificazione è **5 livelli**, l'output espanso non può superare **~1 MB**, e i cicli sono **vietati** — A che importa B che importa A non è permesso, e il linter lo segnala con `IMP002`.

Vedi [`prompt-componibili.md`](./prompt-componibili.md) per esempi completi, anti-pattern e troubleshooting.

## Codici linter

Mentre scrivi, l'editor analizza il body e segnala in tempo reale i problemi nella tab **Diagnosi**: sintassi rotta, dati sensibili dimenticati nel testo, import che non risolvono. Questo è il catalogo riassuntivo dei codici; il dettaglio di ogni regola, con esempi, è in [`linting-regole.md`](./linting-regole.md).

### LEN — Lunghezza body

| Codice | Severità | Quando |
|---|---|---|
| `LEN001` | Warning | Body > 4 000 caratteri |
| `LEN002` | Info | Body < 30 caratteri |

### PH — Segnaposti

| Codice | Severità | Quando |
|---|---|---|
| `PH001` | Error | Singola graffa: `{nome}` invece di `{{nome}}` |
| `PH003` | Warning | Nome con caratteri non consentiti (spazi, trattini) |

### PII — Privacy

| Codice | Severità | Quando |
|---|---|---|
| `PII001` | Warning | Email nel body (`utente@example.com`) |
| `PII003` | Error | Carta di credito Luhn-valida (13–19 cifre) |
| `PII004` | Error | API key di provider noti (`sk-…`, `sk-ant-…`, `AKIA…`, `ghp_…`, `AIza…`) |

### STY — Stile

| Codice | Severità | Quando |
|---|---|---|
| `STY001` | Info | Stesso n-gram (3 parole) ripetuto ≥ 4 volte |

### IMP — Import / dipendenze

| Codice | Severità | Quando |
|---|---|---|
| `IMP001` | Error | Import non risolvibile (path non esistente) |
| `IMP002` | Error | Ciclo di import (diretto o indiretto) |
| `IMP003` | Warning | Profondità import ≥ 6 (limite hard a 5) |
| `IMP004` | Info | Il prompt corrente è importato da N altri prompt |

Da **Impostazioni → Linter** puoi disabilitare intere categorie o singole regole, cambiare la severità di ogni regola e regolare le soglie numeriche (caratteri massimi/minimi, ripetizioni minime). Le scelte sono persistite in locale.

## Esempi compositi

### Email parametrica con globali

Qui i due mondi convivono: i globali (`autore`, `email`) si riempiono da soli, mentre destinatario, oggetto e corpo ti vengono chiesti a ogni uso.

```
Da: {{global autore}} <{{global email}}>
A: {{destinatario}}
Oggetto: {{oggetto}}

Ciao {{destinatario}},

{{corpo}}

Cordiali saluti,
{{global autore}}
```

Alla compilazione, la form mostra solo tre campi — `destinatario`, `oggetto`, `corpo` — e la firma esce già completa.

### Code review con import del "ruolo"

Il ruolo del reviewer vive in un prompt separato e viene importato in testa; il resto del body definisce il compito specifico.

```
{{import "ruoli/senior-engineer"}}

Esamina il seguente codice {{linguaggio}}. Concentrati su:
- Edge case non gestiti
- Performance
- Leggibilità

\`\`\`
{{codice}}
\`\`\`
```

Se un giorno migliori il prompt "senior-engineer", tutte le code review che lo importano si aggiornano di conseguenza.

### Prompt versionato con override

Questo import blocca una versione precisa del prompt sorgente e ne sovrascrive un segnaposto al volo:

```
{{import "system-prompts/analista-dati" version=2 with tono=conciso}}

Analizza il seguente dataset: {{dataset}}.
```

La versione 2 di "analista-dati" viene espansa con `tono` già fissato a `conciso`; alla compilazione ti resta da fornire solo `dataset`.

## Quando NON usare segnaposti

I segnaposti sono pensati per **valori brevi**: parole, frasi, al massimo qualche riga. Se ti accorgi di incollare interi articoli o codice multi-file in un campo della form, probabilmente il prompt sta chiedendo troppo in un colpo solo: considera di dividerlo in fasi (prompt 1 → output → prompt 2), dove ogni passaggio riceve un input gestibile.

Per verificare a colpo d'occhio come verrà reso il body, usa la tab **Anteprima** del pannello di dettaglio: mostra il risultato con gli import espansi, prima ancora di compilare.

Infine, evita segnaposti dentro segnaposti: PaP non supporta l'interpolazione ricorsiva nei valori. Se scrivi `{{altro}}` dentro il valore di un campo, il risultato lo tratta come stringa letterale, non come un secondo segnaposto da compilare.

## Vedi anche

- [`prompt-componibili.md`](./prompt-componibili.md) — la guida completa agli import: esempi, anti-pattern, troubleshooting.
- [`linting-regole.md`](./linting-regole.md) — ogni regola del linter spiegata, con istruzioni per personalizzarla o silenziarla.
- [`cartelle.md`](./cartelle.md) — i path degli import seguono l'albero delle cartelle: qui capisci come è fatto.
