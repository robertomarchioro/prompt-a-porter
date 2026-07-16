<!-- Guida di prompting — famiglia OpenAI (GPT). Ultima revisione: 2026-07-16.
     Fonte di riferimento (umana): https://platform.openai.com/docs/guides/prompt-engineering
     Testo curato e sintetico: NON è un dump della documentazione. -->

# Linee guida di prompting per GPT (OpenAI)

Principi per scrivere prompt efficaci per i modelli GPT.

## Struttura
- Metti le **istruzioni all'inizio** e separa istruzioni e contesto con
  **delimitatori** chiari (`"""`, `###`, `---`).
- Indica in modo esplicito **contesto, esito atteso, lunghezza e formato**.

## Precisione
- Sii **specifico e concreto**: evita descrizioni "vaghe". Meglio "riassumi in
  3 punti da massimo 15 parole" che "riassumi brevemente".
- Descrivi la sequenza di **passi** per completare un compito articolato.

## Esempi e output
- Fornisci **esempi** (few-shot) del formato voluto.
- Quando l'output va elaborato da codice, chiedi un **formato strutturato**
  (JSON) e mostra lo schema.

## Robustezza
- Dai al modello una **via d'uscita**: "se l'informazione non è presente,
  rispondi «non disponibile»" per ridurre le invenzioni.
- Preferisci istruzioni **positive** (cosa fare) a lunghe liste di divieti.

## Anti-pattern
- Istruzioni sepolte in fondo a un testo lungo.
- Ambiguità su formato/lunghezza.
- Nessun esempio per compiti dal formato rigido.
