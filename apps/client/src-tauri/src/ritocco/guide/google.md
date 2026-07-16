<!-- Guida di prompting — famiglia Google (Gemini). Ultima revisione: 2026-07-16.
     Fonte di riferimento (umana): https://ai.google.dev/gemini-api/docs/prompting-strategies
     Testo curato e sintetico: NON è un dump della documentazione. -->

# Linee guida di prompting per Gemini (Google)

Principi per scrivere prompt efficaci per i modelli Gemini.

## Struttura
- Dai **istruzioni chiare e specifiche**: definisci con precisione il compito.
- Usa **prefissi** per etichettare le parti dell'input ("Testo:", "Domanda:",
  "Esempio:") e separare contesto, dati e richiesta.
- Aggiungi il **contesto e i vincoli** rilevanti (dominio, pubblico, limiti).

## Esempi e persona
- Fornisci **esempi** (few-shot): mostrano il pattern meglio di una descrizione.
  Mantieni gli esempi coerenti nel formato.
- Assegna una **persona/ruolo** per orientare stile e livello.

## Compiti complessi
- **Scomponi** i compiti articolati in passi o sottodomande.
- Specifica esplicitamente il **formato di output** desiderato.

## Iterazione
- Se l'output non convince, riformula: cambia parole, aggiungi un esempio,
  rendi più specifici i vincoli.

## Anti-pattern
- Richieste generiche senza contesto né vincoli.
- Esempi incoerenti tra loro nel formato.
- Formato di output lasciato implicito.
