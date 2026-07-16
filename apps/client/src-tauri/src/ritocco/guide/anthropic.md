<!-- Guida di prompting — famiglia Anthropic (Claude). Ultima revisione: 2026-07-16.
     Fonte di riferimento (umana): https://docs.anthropic.com/claude/docs/prompt-engineering
     Testo curato e sintetico: NON è un dump della documentazione. -->

# Linee guida di prompting per Claude (Anthropic)

Principi per scrivere prompt efficaci per i modelli Claude.

## Struttura
- Usa **tag XML** per delimitare le sezioni: `<istruzioni>`, `<contesto>`,
  `<documento>`, `<esempio>`. Claude è addestrato a rispettarli e riduce le
  ambiguità.
- Metti **documenti e dati lunghi PRIMA** delle istruzioni; le domande e i
  compiti vanno in fondo.
- Assegna un **ruolo** esplicito ("Sei un revisore legale esperto…") per
  orientare tono e competenza.

## Chiarezza
- Sii **esplicito e diretto**: descrivi il compito, il contesto e il perché.
  Claude segue meglio istruzioni motivate.
- Preferisci dire **cosa fare** piuttosto che cosa non fare.
- Specifica **formato, lunghezza e destinatario** dell'output.

## Ragionamento ed esempi
- Per compiti complessi, concedi a Claude di **ragionare passo-passo** (chain
  of thought), eventualmente in un blocco `<ragionamento>` separato dalla
  risposta finale.
- Fornisci **1-3 esempi** (multishot) del comportamento voluto: sono il modo
  più affidabile per fissare formato e stile.

## Controllo dell'output
- Descrivi con precisione la struttura attesa (JSON, elenco, sezioni).
- Se serve un formato rigido, mostra un esempio dell'output esatto.

## Anti-pattern
- Istruzioni vaghe o contraddittorie.
- Contesto insufficiente ("indovina cosa intendo").
- Mescolare dati non fidati e istruzioni senza delimitatori.
