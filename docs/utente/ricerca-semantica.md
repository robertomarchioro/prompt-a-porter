# Ricerca semantica

> Come cercare i prompt per significato e non solo per parola esatta: cosa
> cambia, come si attiva, come si bilancia, e perché tutto resta sulla tua
> macchina.

Quando il vault cresce, ritrovare un prompt diventa un problema. Ricordi
di averne scritto uno per le email formali, ma lo cerchi con "email
formale" e non salta fuori — perché quel prompt parla di "redigere una
comunicazione professionale", e non contiene nessuna delle parole che stai
digitando. La ricerca per parola esatta trova solo ciò che chiami con il
nome giusto, e a distanza di settimane il nome giusto lo hai dimenticato.

La ricerca semantica cerca **per significato**. Chiedi "scrivere email
formale" e trova anche i prompt che parlano di "redigere comunicazione
professionale", perché ne riconosce il senso pur senza una parola in
comune. È la differenza fra cercare una stringa e cercare un'idea.

Nella pratica non devi scegliere fra i due modi: la ricerca di PaP li
combina. Continua a premiare i match letterali quando cerchi un termine
tecnico preciso, e aggiunge la comprensione del significato quando cerchi
un concetto. Il risultato è una singola lista, ordinata meglio.

Tutto avviene in locale: nessun testo lascia mai la tua macchina, come
vedremo nella sezione sulla privacy.

## La prima volta

La ricerca semantica non è attiva di default, perché richiede di scaricare
un modello. Si abilita una volta sola:

1. Apri **Impostazioni → Ricerca & Embeddings** (nel gruppo delle
   impostazioni avanzate).
2. Nella sezione dello stato del modello, premi **Scarica modello**: il
   client scarica il modello di embedding (~118 MB) e il runtime necessario
   a eseguirlo, mostrando l'avanzamento in byte.
3. A download completato premi **Inizializza**: il modello viene caricato
   in memoria e diventa pronto all'uso.
4. Abilita la spunta **Ricerca semantica abilitata**. Da questo momento la
   Command Palette calcola gli embedding dei prompt esistenti in background
   (una tantum) e comincia a usare la ricerca combinata.

Dopo il primo caricamento, cerca qualcosa nella Command Palette con parole
diverse da quelle del titolo: vedrai comparire prompt che prima non
trovavi. Sui risultati che devono la loro posizione al significato appare
un piccolo badge **sem**.

## Come funziona

Sotto il cofano la ricerca combina due segnali indipendenti e ne fonde le
classifiche in un'unica lista, premiando i prompt che compaiono in entrambe:

| Segnale | Cosa fa | Quando vince |
|---|---|---|
| **Lessicale** | Match esatto di parole e prefissi | Termini tecnici, nomi propri, keyword specifiche |
| **Semantico** | Vicinanza di significato fra i testi | Sinonimi, parafrasi, descrizioni concettuali |

### Il modello di embedding

Il modello usato è `paraphrase-multilingual-MiniLM-L12-v2` (~118 MB),
scelto perché lavora bene su **italiano e inglese mescolati** — e su altre
decine di lingue — il che è utile su un vault reale, dove i prompt spesso
alternano le due lingue. Si scarica al primo uso e poi resta sul disco:
da lì in avanti gira interamente sulla tua macchina.

## Bilanciare lessicale e semantico

Quanto peso dare al significato rispetto alla parola esatta lo decidi tu,
con lo slider **Hybrid alpha (lessicale ↔ semantico)** in
**Impostazioni → Ricerca & Embeddings**:

| Valore | Effetto |
|---|---|
| `0` | Solo lessicale — come la ricerca prima che la semantica esistesse |
| `0.5` (default) | Bilanciato: coglie sia le keyword sia il significato |
| `1` | Solo semantico — utile per esplorare "prompt simili" ignorando le parole |

Il valore di default (0.5) va bene per l'uso quotidiano. Alzalo verso 1 se
stai esplorando il vault a caccia di prompt affini; abbassalo verso 0 se
lavori con termini molto specifici in cui conta la parola precisa.

## Prestazioni

La ricerca resta rapida anche su vault grandi: l'embedding della query si
calcola in una trentina di millisecondi e la fusione delle classifiche è
quasi istantanea, per un totale ben sotto i 100 ms anche con decine di
migliaia di prompt. In pratica, non la sentirai.

## Memoria e scarico automatico

Il modello e il suo runtime occupano circa 150 MB di RAM. Se non usi la
ricerca semantica per un po', il client li **scarica automaticamente** per
liberare memoria, e la ricerca torna temporaneamente al solo lessicale.
Alla ricerca successiva il modello si ricarica da solo, senza che tu debba
fare nulla.

La soglia di inattività si regola in
**Impostazioni → Ricerca & Embeddings** (default 5 minuti; imposta 0 per
non scaricare mai).

## Tag suggeriti

Lo stesso modello alimenta un secondo aiuto: mentre scrivi un prompt,
l'editor ti propone **tag pertinenti** in base al testo, sotto l'etichetta
**Suggeriti**. Funziona quando il vault ha già almeno una decina di tag con
embedding calcolato; sotto quella soglia, i suggerimenti ripiegano sui tag
che usi più spesso.

## Privacy

Nessun testo lascia mai la tua macchina. Il modello gira in locale sulla CPU
e l'embedding di ogni prompt è semplicemente un vettore di 384 numeri,
salvato accanto al prompt dentro lo stesso vault cifrato. Non ci sono
chiamate a servizi esterni durante le ricerche: attivare la ricerca
semantica non cambia in nulla le garanzie di riservatezza del vault.

## Limiti noti

- L'embedding della query viene ricalcolato a ogni ricerca (~30 ms): non
  c'è cache per query identiche ripetute di seguito.
- La ricerca per vicinanza esamina tutti i prompt in modo lineare: su vault
  molto grandi (oltre le centinaia di migliaia di prompt) i tempi possono
  crescere.

## Disabilitare

Per tornare alla sola ricerca lessicale, togli la spunta **Ricerca
semantica abilitata** in **Impostazioni → Ricerca & Embeddings**: la
Command Palette smette di usare gli embedding. I file del modello restano
sul disco; se vuoi recuperare quello spazio devi eliminarli a mano dalle
cartelle dei modelli nella directory dati del client.

## Vedi anche

- [`regression-testing.md`](./regression-testing.md) — la similarità `cosine` dei test golden riusa questo stesso modello di embedding.
- [`scorciatoie-tastiera.md`](./scorciatoie-tastiera.md) — come aprire velocemente la Command Palette dove la ricerca vive.
- [`troubleshooting.md`](./troubleshooting.md) — cosa fare se il download del modello o la ricerca danno problemi.
