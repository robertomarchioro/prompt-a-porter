# Test plan utente — Prompt a Porter

Catalogo dei test cases manuali da eseguire prima di una release o quando si verifica un bug. Ogni test ha un ID univoco citabile nelle issue.

## Convenzioni

- **ID test**: `TC-<AREA>-<NNN>` (es. `TC-AUTH-001`, `TC-COMPILE-003`).
- **Severità**: 🔴 critical (blocca release) / 🟡 high / 🔵 medium / ⚪ low.
- **Tipo**: 🎯 golden path / 🧪 edge case / ⚠️ error path / ♿ a11y / ⚡ performance.

## Come segnalare un fallimento

Se un test case non passa:

1. Apri una issue su GitHub: [Bug report](https://github.com/robertomarchioro/prompt-a-porter/issues/new?template=bug_report.yml).
2. Titolo: `[TC-XXX-NNN] descrizione breve`. Esempio: `[TC-AUTH-003] Reset password non funziona se vault è bloccato`.
3. Nel corpo cita il test case completo, gli step eseguiti, l'output atteso e quello osservato.
4. Allega screenshot/log se disponibili (vedi [`docs/utente/troubleshooting.md`](../utente/troubleshooting.md#come-abilito-i-log-dettagliati) per abilitare debug log).

## Cataloghi

| Area | Doc | TC count | Cosa copre |
|---|---|---|---|
| Tutti i test cases | [`test-cases.md`](./test-cases.md) | ~120 | Catalogo completo numerato per area |

## Aree coperte (in `test-cases.md`)

- **AUTH** — vault create/unlock/lock/change-password/delete, password recovery
- **ONBOARD** — wizard 3-step primo avvio
- **PROMPT** — CRUD prompt (create, edit, autosave, delete, restore)
- **SEARCH** — ricerca FTS + filtri vista/tag/modello/cartella
- **SEMANTIC** — ricerca semantica + idle unload
- **COMPILE** — compilazione (palette + modale, copia clipboard)
- **PLACEHOLDER** — segnaposti normali + globali
- **IMPORT** — sintassi `{{import}}` + modifiers `version=`/`with`
- **INTELLISENSE** — autocomplete CodeMirror su titoli
- **LINT** — regole linter (LEN/PH/PII/STY/IMP)
- **FOLDER** — cartelle CRUD + drag&drop
- **TAG** — gestione tag
- **VARIANT** — varianti A/B + promozione
- **FORK** — fork prompt con tracciabilità
- **RATING** — rating discreto + badge
- **CRONOLOGIA** — versioning + rollback
- **EDITOR** — editor doppia vista Sorgente/Compilato
- **HOTKEY** — global hotkey + configurazione
- **PALETTE** — command palette
- **TRAY** — tray icon menu
- **EXP** — export/import JSON + Markdown bulk
- **PREF** — preferenze (tema, segnaposti globali, linter on/off, debug log)
- **MCP** — integrazione MCP server
- **CLI** — CLI `pap` read-only
- **UPDATER** — auto-update flow (downgrade refuse, signature mismatch refuse)
- **A11Y** — accessibilità (focus trap, contrasto, screen reader)
- **PERF** — performance (palette ≤1000 prompt, compile <1s)

## Manutenzione

- Quando aggiungi una feature: aggiungi i relativi TC al catalogo.
- Quando un bug viene fixato: aggiungi un TC di regressione.
- Quando un TC fallisce in modo permanente (feature rimossa): marca come **DEPRECATED** ma non rimuovere — serve la storia.
