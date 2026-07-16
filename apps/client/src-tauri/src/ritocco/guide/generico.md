<!-- Guida di prompting — generica (trasversale alle famiglie). Ultima revisione: 2026-07-16.
     Usata quando il modello target non ha una guida dedicata (es. Llama/meta,
     "generico", valore assente o sconosciuto). Testo curato e sintetico. -->

# Linee guida di prompting (principi trasversali)

Principi validi per la maggior parte dei modelli LLM.

## Chiarezza dell'obiettivo
- Dichiara **scopo, destinatario, formato e lunghezza** dell'output.
- Sii **specifico e non ambiguo**: sostituisci gli aggettivi vaghi con criteri
  misurabili ("massimo 5 righe", "in elenco puntato").

## Contesto e vincoli
- Fornisci il **contesto** necessario e i **vincoli** (cosa includere/escludere,
  tono, pubblico).
- Separa **dati/contesto** dalle **istruzioni** con delimitatori chiari.

## Esempi e passi
- Mostra **1-2 esempi** del risultato voluto quando il formato conta.
- **Scomponi** i compiti complessi in passi espliciti.

## Ruolo e output
- Assegna un **ruolo/persona** adatto al compito.
- Chiedi un **formato strutturato** (es. JSON) quando l'output va elaborato.
- Preferisci istruzioni **positive** (cosa fare) ai divieti.

## Anti-pattern
- Richieste generiche senza contesto.
- Formato/lunghezza impliciti.
- Istruzioni e dati mescolati senza separazione.
