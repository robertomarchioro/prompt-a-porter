# Stagioni e nomi di rilascio

> **Cosa fissa questo documento**: la convenzione con cui battezziamo e collochiamo nel tempo i rilasci di Prompt a Porter, presa in prestito dal mondo della moda.
> **Stato**: convenzione approvata dall'autore (2026-06-04). Da applicare ai rilasci futuri; quelli passati restano col loro tag tecnico, non si rinominano retroattivamente.
> **Relazione con la pianificazione**: questo doc definisce *come si nominano* i rilasci. La timeline autorevole di *quali* rilasci escono resta [`release-plan.md`](./release-plan.md).

## Perché le stagioni

Prompt a Porter è un omaggio al **prêt-à-porter**. Nella moda il tempo non si misura in numeri di versione: si misura in **stagioni**. Adottiamo la stessa logica come cornice editoriale del versionamento — senza rinunciare alla precisione del numero `vX.Y.Z`, che resta il riferimento tecnico.

Tre livelli, descritti sotto:

1. **Etichetta stagionale** — colloca il rilascio nel tempo (la "stagione in negozio").
2. **Doppia linea di versionamento** — `1.x` Personale / `2.x` Enterprise, scandite dalla stessa cadenza.
3. **Nomi cardine** — codename evocativi (stile distro Ubuntu) per le release di svolta.

I tre livelli sono **complementari e indipendenti**: la stagione dice *quando*, il numero dice *cosa* con precisione, il codename battezza *la svolta*.

---

## 1. Etichetta stagionale

### Formato

```
Autunno-Inverno 2026 · v1.2.0
Primavera-Estate 2027 · v1.4.0
```

La stagione + anno è l'etichetta **editoriale/di comunicazione**; il `vX.Y.Z` è il riferimento **tecnico** (tag git, changelog, updater). Convivono, non si sostituiscono.

### Le due stagioni madri

Due rilasci maggiori all'anno, uno per stagione:

- **Primavera-Estate (PE)**
- **Autunno-Inverno (AI)**

### Taglio dei mesi — basato sul momento del rilascio

> Decisione: la stagione nell'etichetta indica **quando il rilascio è "in negozio"** (tag pubblicato e scaricabile), non quando la roadmap è stata presentata.

Confine netto a metà anno, sulla **data del tag**:

| Mese del tag | Stagione |
|---|---|
| Gennaio → Giugno (H1) | **Primavera-Estate** dell'anno corrente |
| Luglio → Dicembre (H2) | **Autunno-Inverno** dell'anno corrente |

### Lo sfasamento "sfilata vs negozio"

Nella moda la collezione **sfila mesi prima** di arrivare nei negozi: il nome di stagione anticipa l'uso, non la presentazione. Da noi vale lo stesso, ma con una regola semplice che evita ambiguità:

- La **roadmap/annuncio** di una stagione può vivere prima — è la "sfilata": si comunica cosa arriverà.
- L'**etichetta stagionale** segue però sempre la **data del tag effettivo** — il capo "in negozio".

Quindi una roadmap presentata a maggio per un rilascio che esce a settembre sarà etichettata **Autunno-Inverno**, perché conta la messa in vendita.

---

## 2. Doppia linea di versionamento

PaP ha due SKU (vedi [`README.md`](./README.md) §"SKU"), e quindi due linee di versionamento parallele:

| Linea | SKU | Stato |
|---|---|---|
| `1.x.x` | **Personale** (single user, local-first) | attiva |
| `2.x.x` | **Enterprise** (multi-user, server) | si apre quando rilasciamo Enterprise |

Regole:

- Finché esiste solo la Personale, la linea `1.x` avanza da sola, stagione dopo stagione.
- Quando nasce l'Enterprise, **apre la linea `2.x`**; da quel momento le due linee **procedono di pari passo**, ciascuna con la propria progressione semantica.
- La **cadenza stagionale è comune**: una stessa stagione (es. *Autunno-Inverno 2027*) può ospitare un rilascio della linea `1.x` **e** uno della linea `2.x`. L'etichetta stagionale è condivisa; il numero distingue la linea.

Esempio di come si leggono insieme (illustrativo, non impegnativo):

```
Autunno-Inverno 2027 · v1.6.0   (Personale)
Autunno-Inverno 2027 · v2.0.0   (Enterprise, debutto)
```

---

## 3. Nomi cardine (stile Ubuntu)

Non tutti i rilasci hanno un codename. Lo ricevono le **versioni cardine**: le major **e** le minori che segnano una svolta. Un patch di routine non viene battezzato.

### Schema

Ricalca le release di **Ubuntu** (`aggettivo + animale`, allitterante, in marcia alfabetica). Adattato al nostro mondo:

- **Aggettivo + sostantivo**, entrambi con la **stessa lettera iniziale** (allitterazione).
- Il **sostantivo** pesca dal lessico **tessile / sartoriale / da passerella**; l'**aggettivo** ne colora il carattere.
- Ogni nuovo codename prende la **lettera successiva** dell'alfabeto.
- **Solo nomi comuni** del dizionario: niente marchi, niente nomi di maison, collezioni o creazioni registrate.

> **Regola anti-copyright**: prima di "blindare" un codename per un rilascio reale, fare una **verifica veloce sui marchi registrati nella classe software** (i nomi qui sotto sono sostantivi comuni a basso rischio, ma alcuni termini molto usati meritano un controllo in più). Evitare inoltre aggettivi con connotazioni storico-politiche.

### Stagione ≠ codename

Restano due cose distinte e complementari:

- **Etichetta stagionale** → colloca nel tempo (`Autunno-Inverno 2026`).
- **Codename cardine** → battezza la svolta (`Arioso Atelier`).

Esempio di intestazione completa di una release cardine:

```
Arioso Atelier — Autunno-Inverno 2026 · v1.2.0
```

### Il codename pre-1.0: «Ago e Filo»

Prima della 1.0 il prodotto è **ancora in cucitura**: l'intero ciclo di sviluppo verso la prima release stabile (da `v0.8.x` in poi) porta un unico **codename di laboratorio**, **«Ago e Filo»**. È volutamente fuori schema rispetto al pool qui sotto — *sostantivo + sostantivo*, non *aggettivo + tessuto* allitterante: non battezza una svolta, battezza **l'officina che la precede**. Vive nell'header in-app accanto al numero di versione (`Ago e Filo · v0.8.25`), implementato come costante in `apps/client/src/lib/codename.ts`.

Alla **1.0** si abbandona «Ago e Filo» e si pesca il **primo nome del pool in ordine alfabetico** — oggi **«Arioso Atelier»** (lettera A), coerente col suo registro di *"release fondante / di debutto"*.

### Pool di codename proposti (~20, in marcia alfabetica)

Lista di partenza. Si pesca dall'alto verso il basso man mano che arrivano le release cardine; ogni nome è un **suggerimento di registro**, l'abbinamento finale al rilascio si decide quando la release prende forma.

| Lettera | Codename | Tono / a quale tipo di svolta si addice |
|---|---|---|
| A | **Arioso Atelier** | Release fondante / di debutto di una linea (richiama anche il mood "ariosa" del brand) — **riservato alla 1.0** (subentra a «Ago e Filo») |
| B | **Brioso Broccato** | Release ricca, ornata di molte feature |
| C | **Cangiante Cashmere** | Release che cambia la percezione del prodotto (UX, redesign morbido) |
| D | **Dorato Damasco** | Release "premium", rifinitura di pregio |
| E | **Elegante Écru** | Release di essenzialità e pulizia (il neutro, il grezzo) |
| F | **Fluido Filato** | Release su continuità e flusso (import, pipeline, automazioni) |
| G | **Garbato Gabardine** | Release solida e sobria (struttura, fondamenta) |
| I | **Iridato Intreccio** | Release su interconnessione e integrazioni |
| L | **Leggero Lino** | Release su leggerezza e performance |
| M | **Morbida Mussola** | Release su accessibilità e comfort d'uso |
| N | **Nobile Nappa** | Release su robustezza e qualità (sicurezza, hardening) |
| O | **Onirico Ordito** | Release sperimentale / visionaria (direzione "Deluxe") |
| P | **Prezioso Pizzo** | Release di dettaglio fine (polish minuzioso) |
| Q | **Quieto Quadretto** | Release di stabilizzazione, ritorno all'ordine |
| R | **Raffinato Ricamo** | Release di rifinitura dettagliata |
| S | **Sontuosa Seta** | Release "flagship", lussuosa, di punta |
| T | **Tenue Taffetà** | Release leggera ma strutturata |
| U | **Uniforme Uncinetto** | Release di coerenza e consolidamento |
| V | **Vivido Velluto** | Release ad alto impatto visivo (grande redesign) |
| Z | **Zigrinato Zibellino** | Release "texture e finitura" di pregio |

> Nota: la **H** è saltata di proposito (manca un sostantivo tessile italiano naturale con quella iniziale). Come in Ubuntu, alcune lettere si possono saltare senza forzare il gioco. L'alfabeto italiano omette già J, K, W, X, Y.

---

## Manutenzione di questo documento

- Quando una release cardine viene battezzata: **spuntare il nome usato** dal pool sopra (annotando il `vX.Y.Z` a cui è stato assegnato) e, se il pool si assottiglia, aggiungere nuove lettere/nomi.
- Quando si apre la linea `2.x` Enterprise: aggiornare §2 con la prima etichetta reale.
- Se cambia il taglio dei mesi o la cadenza: aggiornare §1 e allineare [`release-plan.md`](./release-plan.md).
- L'abbinamento codename → rilascio va riportato anche in [`release-plan.md`](./release-plan.md) (fonte autorevole della timeline) e nel `CHANGELOG.md`.
