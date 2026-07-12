# Email professionale

> Template per email professionali parametriche: richieste, follow-up, reclami, ringraziamenti.

## Quando usarlo

- Email simili che scrivi spesso (follow-up settimanali, richieste ricorrenti).
- Tono consistente attraverso tutta la corrispondenza.
- Situazioni delicate (reclami) dove l'AI ti aiuta a restare fermo ma cortese.

## Prompt

**Titolo:** Email professionale parametrica
**Tag:** `email`, `comunicazione`
**Modello target:** `claude-sonnet` (consigliato per tono) o `gpt-4`

```
Scrivi un'email professionale in italiano con questi parametri:

- Tipo: {{tipo}}
- Destinatario: {{destinatario}} (relazione: {{relazione}})
- Oggetto: {{oggetto}}
- Tono: {{tono}}
- Lunghezza approssimativa: {{lunghezza}}

Contenuto / contesto principale:
{{contenuto}}

Eventuale richiesta esplicita: {{richiesta}}

Vincoli:
- Saluto e firma appropriati al rapporto.
- Niente formule abusate ("Spero che questa email ti trovi bene").
- Se è una richiesta: chiara, con scadenza esplicita se ha senso.
- Se è un reclamo: fermo, basato sui fatti, costruttivo.

Firma con: {{global autore}}.
```

## Esempi di valori

### Richiesta (collega senior)

- `tipo`: `richiesta`
- `destinatario`: `Mario Rossi`
- `relazione`: `collega senior in altro team`
- `oggetto`: `Feedback su proposta architetturale Q3`
- `tono`: `cordiale ma diretto`
- `lunghezza`: `150 parole`
- `contenuto`: `Ho preparato una proposta di refactoring del modulo billing. Servirebbe la tua opinione visto il tuo lavoro su Stripe.`
- `richiesta`: `Una call di 30 minuti entro venerdì`

### Reclamo (fornitore)

- `tipo`: `reclamo`
- `destinatario`: `Customer Service Trenitalia`
- `relazione`: `cliente / fornitore`
- `oggetto`: `Treno IC 510 del 12 ottobre cancellato senza preavviso`
- `tono`: `fermo ma professionale`
- `lunghezza`: `200 parole`
- `contenuto`: `Treno cancellato 30 minuti prima della partenza. Costretto a prendere taxi per €85. Riunione cliente saltata.`
- `richiesta`: `Rimborso biglietto + spese taxi entro 14 giorni`

### Follow-up (potenziale cliente)

- `tipo`: `follow-up`
- `destinatario`: `Anna Bianchi`
- `relazione`: `potenziale cliente, primo contatto 2 settimane fa`
- `oggetto`: `Re: Demo PaP — c'è interesse?`
- `tono`: `cortese, no pressione`
- `lunghezza`: `100 parole`
- `contenuto`: `Demo fatta 2 settimane fa, nessuna risposta. Voglio capire se c'è ancora interesse o chiudere il thread.`
- `richiesta`: *(lasciato vuoto)*

## Varianti

### Spezza in due prompt

Se gestisci email in più lingue, separa "ruolo / vincoli" e "contenuto specifico":

- `role-email-pro`: solo vincoli stilistici e firma.
- `email-pro-italiano`: importa `role-email-pro` e aggiunge i parametri.

```
{{import "role-email-pro" with lingua=italiano}}

Tipo: {{tipo}}
...
```

### Aggiungi disclaimer legale

Per ambito B2B con responsabilità contrattuali, importa un disclaimer standard:

```
{{import "disclaimer-legale-azienda"}}
```

## Anti-pattern

- **Non usare per email molto brevi** (1-2 frasi): l'overhead del template supera il valore.
- **Non hardcoddare il destinatario**: tienilo come segnaposto, così il template è riusabile.
- **Non specificare troppi vincoli**: 3-5 vincoli generano output prevedibili; 10+ confondono il modello.
