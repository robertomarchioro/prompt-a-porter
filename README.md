# Prompt a Porter

**Libreria locale e di team per prompt AI** — template parametrici, vault cifrato, sync opzionale.

> Il nome è un gioco su "prêt-à-porter": prompt pronti all'uso.

## Cos'è

Prompt a Porter (PaP) è un'app desktop local-first per knowledge worker che usano LLM quotidianamente. Permette di salvare prompt efficaci, renderli parametrici con segnaposti `{{nome}}`, compilarli in pochi secondi e copiarli nella chat AI di destinazione.

Non è un'app cloud. È un'app desktop invocabile via hotkey globale (tipo Raycast/Alfred), con storage primario su un **SQLite cifrato** locale. Il sync con il team è opzionale e passa attraverso un server self-hosted minimale.

## Funzionalità principali

- **Command palette globale** — `Ctrl+Shift+P` (o `⌘⇧P` su macOS) per cercare e compilare prompt in pochi secondi
- **Template parametrici** — segnaposti `{{nome}}` con tipi (testo, multilinea, enum), default e hint
- **Vault cifrato** — SQLite + SQLCipher, AES-256, password mai persistita
- **Libreria organizzata** — tag, preferiti, ricerca full-text, visibilità privata/team
- **Sync team opzionale** — server Go self-hosted, WebSocket real-time, last-write-wins
- **RBAC** — ruoli Admin / Editor / User per workspace team
- **Cross-platform** — Windows 10+, macOS 12+, Linux

## Stack tecnico

| Componente | Tecnologia |
|------------|------------|
| Client desktop | Tauri 2.x (Rust) + Svelte 5 + TypeScript |
| Editor prompt | CodeMirror 6 |
| Storage locale | SQLite + SQLCipher |
| Server sync | Go 1.22+, single binary |
| Icone | Lucide (subset tree-shakable) |
| Font | Sistema per UI, JetBrains Mono per codice |

## Struttura del repository

```
prompt-a-porter/
├── apps/
│   ├── client/          ← Tauri + Svelte (app desktop)
│   └── server/          ← Go sync server
├── packages/
│   └── shared-schema/   ← Tipi TypeScript condivisi
├── docs/                ← Documentazione tecnica
├── design_handoff/      ← Asset di design (prototipi HTML + token CSS)
└── .github/workflows/   ← CI/CD
```

## Setup sviluppo

> Documentazione dettagliata in [`docs/setup-sviluppo.md`](docs/setup-sviluppo.md).

### Prerequisiti

- Node.js 22.x LTS
- pnpm 9.x+
- Rust toolchain stable (per Tauri)
- Go 1.22+ (per il server sync)

### Quick start

```bash
git clone https://github.com/robertomarchioro/prompt-a-porter.git
cd prompt-a-porter
pnpm install
pnpm --filter @pap/client dev
```

## Stato del progetto

**Fase 1 (MVP) completata** — 15 step, 71 test (37 Rust + 22 TypeScript + 12 Go).
Vedi [`CHANGELOG.md`](CHANGELOG.md) per i dettagli e [`docs/todo-fase-1.md`](docs/todo-fase-1.md) per la checklist.

## Licenza

**[GNU AGPL 3.0](LICENSE)** (Affero General Public License). Chiude il loophole SaaS — chi ospita il codice come servizio ha l'obbligo di pubblicare modifiche. Tutto il codice del progetto è libero, ispezionabile e portabile.

Per dettagli sulle implicazioni pratiche del cambio licenza vedi `docs/licenza.md` (in arrivo).

## Autore

**Roberto Marchioro** — ICT Manager

---

*Prompt a Porter — perché i prompt migliori meritano di essere riutilizzati.*
