# Documentazione — Prompt a Porter

Questa è la documentazione tecnica del progetto. Il `README.md` di repository copre l'overview e l'installazione; qui trovi i dettagli operativi, di sviluppo e di architettura.

La documentazione è organizzata in **5 cluster** per scopo:

| Cluster | Per chi | Contiene |
|---|---|---|
| [**Utente**](./utente/) | Chi usa Prompt a Porter | Guide d'uso del client desktop, della CLI `pap`, dell'integrazione MCP, del formato di export |
| [**Contribuire**](./contribuire/) | Chi vuole contribuire codice | Setup ambiente di sviluppo, branching strategy, convenzioni |
| [**Architettura**](./architettura/) | Chi vuole capire o estendere il sistema | Overview moduli, schema dati, API server, design system, decisioni tecniche (ADR) |
| [**Roadmap**](./roadmap/) | Maintainer e contributor che pianificano | Fasi del progetto, rinvii, quality gate, lessons learned |
| [**Operativo**](./operativo/) | Chi deploya e opera in produzione | Deploy del server, configurazione, runbook |

## Tipi di file

- **`README.md`** dentro ogni cluster = landing del cluster con TOC.
- **Documenti tecnici**: ogni file copre un singolo argomento, naming kebab-case in italiano.
- **Decisioni architetturali**: in `architettura/decisioni/`, ognuna documenta una scelta che condiziona il sistema (Architecture Decision Records).

## Convenzioni

- Lingua: italiano (la maggior parte dei contributor è italofona).
- Formato: Markdown CommonMark + tabelle GFM.
- Naming: kebab-case, no prefissi numerici nei nomi file.
- Cross-reference: link relativi (`./altro-doc.md` o `../altro-cluster/doc.md`).

## File a livello root del repo

Sono esterni a questo cluster ma fanno parte della documentazione:

- [`README.md`](../README.md) — overview progetto + install rapido
- [`CHANGELOG.md`](../CHANGELOG.md) — log per versione
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — DCO, conventional commits, processo PR
- [`CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md) — Contributor Covenant 2.1 IT
- [`SECURITY.md`](../SECURITY.md) — disclosure policy
