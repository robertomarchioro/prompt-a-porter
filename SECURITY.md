# Security Policy

## Versioni supportate

| Versione | Supportata |
|----------|------------|
| 0.1.x    | sì         |
| < 0.1    | no         |

Durante la fase beta, solo l'ultima minor release riceve fix di sicurezza. Dopo la 1.0, manterremo backport sulle ultime due major in linea con politiche standard FOSS.

## Segnalazione di vulnerabilità

Per segnalare una vulnerabilità di sicurezza **non aprire un issue pubblico su GitHub**. Usa il canale privato dedicato:

- **GitHub Security Advisory privato**: apri una segnalazione riservata dalla scheda **Security → Report a vulnerability** del repository.

Includi nella segnalazione:

- Descrizione della vulnerabilità
- Passi di riproduzione minimi
- Impatto stimato (chi è esposto, a cosa, come)
- Eventuali Proof-of-Concept o exploit (anche solo bozza)
- Suggerimento di fix se ne hai uno

Nella segnalazione **non includere dati sensibili reali** (chiavi, password, dump di vault). Se servono per illustrare la vulnerabilità, sostituiscili con sintetici.

PGP/GPG: rimandato. Sarà aggiunto se la severità delle segnalazioni cresce.

## Tempi di risposta attesi

| Fase | Tempo |
|------|-------|
| Acknowledge della segnalazione | entro 7 giorni lavorativi |
| Triage iniziale + valutazione severità | entro 14 giorni |
| Fix per vulnerabilità **Critical** (RCE, auth bypass, leak chiavi/dati) | entro 30 giorni |
| Fix per vulnerabilità **High** (escalation privilegi, data corruption) | entro 60 giorni |
| Fix per vulnerabilità **Medium** | nel prossimo ciclo di release minor |
| Fix per vulnerabilità **Low** | backlog, gestita come issue normale |

Se il fix richiede più tempo del previsto, il segnalatore viene aggiornato proattivamente.

## Disclosure responsabile

Concordiamo una **finestra di disclosure di 90 giorni** dalla segnalazione iniziale, estendibile previo accordo se la fix è complessa. La disclosure pubblica della vulnerabilità avverrà attraverso:

- GitHub Security Advisory (CVE assegnato se applicabile)
- `CHANGELOG.md`
- Release notes della versione che contiene il fix

## Riconoscimento

Le segnalazioni serie sono accreditate (con consenso del segnalatore) nel `CHANGELOG.md` e nelle release notes. Niente bug bounty monetari per ora — il progetto è gestito da volontari.

## Modello di trust

Per chi vuole capire il perimetro di sicurezza di Prompt a Porter, vedere `docs/sicurezza.md` (in arrivo nelle fasi successive) per il threat model dettagliato. Sintesi:

- **Vault locale**: cifrato con SQLCipher (AES-256), password derivata via Argon2id, mai persistita.
- **Sync server** (opzionale): autenticazione JWT + Argon2id. Le password utente non lasciano mai il client.
- **Aggiornamenti** (in arrivo): firma asimmetrica Ed25519 sulle release. Niente update senza signature valida.
- **End-to-end encryption** (Fase 5): roadmap per workspace ad alta sensibilità.
