# Contribuire a Prompt a Porter

Grazie per l'interesse. Questo progetto è attivamente sviluppato da Roberto Marchioro e accoglie contributi esterni.

Prima di iniziare, leggi anche:

- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) — comportamento atteso nella community
- [`SECURITY.md`](SECURITY.md) — come segnalare vulnerabilità (NON aprire issue pubblici per problemi di sicurezza)
- [`docs/architettura.md`](docs/architettura.md) — overview tecnico
- [`docs/todo-fase-*.md`](docs/) — roadmap per fase

## Prerequisiti

- Node.js 22.x LTS
- pnpm 9.x+
- Rust toolchain stable (per il client Tauri)
- Go 1.22+ (per il server sync)
- Vedi [`docs/setup-sviluppo.md`](docs/setup-sviluppo.md) per istruzioni dettagliate

## Workflow di sviluppo

1. **Forka** il repository
2. **Crea un branch** per la tua feature (`git checkout -b feature/nome-feature`) o fix (`fix/nome-bug`)
3. Sviluppa seguendo le convenzioni di codice del progetto
4. Assicurati che **lint e test passino**: `pnpm lint && pnpm test` (e `cargo test`, `go test ./...` se hai toccato quei moduli)
5. **Committa** con messaggi chiari (vedi convenzioni sotto) e **firma DCO** (`git commit -s`)
6. **Apri una Pull Request** verso `main` usando il template (`.github/PULL_REQUEST_TEMPLATE.md`)

## Convenzioni di commit

Usiamo Conventional Commits in versione minimale:

```
<tipo>(<scope opzionale>): <descrizione breve in italiano>

<corpo opzionale, perché del cambio>
```

Tipi accettati: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`, `build`, `style`.

Esempi:

```
feat(libreria): aggiungi filtro per modello target
fix(rust): borrow checker errore in prompt_cerca
docs(roadmap): aggiorna scope Fase 3
ci: vendoriza OpenSSL per build SQLCipher su Windows
```

Niente emoji nei messaggi. Lingua italiana per commenti e descrizioni di dominio. Inglese tollerato solo per termini tecnici già consolidati (es. "borrow checker", "FTS5").

## DCO (Developer Certificate of Origin)

Tutti i commit devono essere firmati con il DCO. Aggiungi il flag `-s` quando committi:

```bash
git commit -s -m "feat(...): tuo messaggio"
```

Questo aggiunge una riga `Signed-off-by: Tuo Nome <email>` in fondo al commit. Equivale a dichiarare:

> Confermo di avere il diritto di contribuire questo codice (vedi https://developercertificate.org/), che lo offro sotto la licenza del progetto, e che la firma è un'identità reale.

Le PR senza DCO sign-off **non vengono mergiate**. Niente CLA, niente burocrazia: solo un sign-off per commit.

## Convenzioni di codice

- **Lingua**: italiano per commenti, variabili di dominio, documentazione utente, stringhe UI. Inglese per identificatori tecnici universali.
- **Naming**: PascalCase per tipi/componenti/struct, camelCase per variabili/funzioni TypeScript, snake_case per Rust/Go.
- **CSS**: solo CSS custom properties, niente preprocessor, niente `!important`.
- **Commenti**: spiegare il *perché*, non il *cosa*. Docstring per funzioni non triviali.
- **Test**: coverage minima 70% sui moduli core. Test E2E per i flussi critici.
- **Immutabilità**: preferire immutable updates (spread, struct copy) a mutazione in-place.

## Reportare bug e proporre feature

- **Bug riproducibile**: usa il template `.github/ISSUE_TEMPLATE/bug_report.yml`
- **Feature request**: usa il template `.github/ISSUE_TEMPLATE/feature_request.yml`
- **Domande generiche / discussioni di design**: vai in [Discussions](https://github.com/robertomarchioro/prompt-a-porter/discussions), non aprire un issue
- **Vulnerabilità di sicurezza**: vedi [`SECURITY.md`](SECURITY.md) — canali privati, mai issue pubblici

## Licenza

Contribuendo accetti che il tuo codice sia distribuito sotto la licenza del progetto (vedi [`LICENSE`](LICENSE)). L'attribuzione all'autore originale e ai contributor deve essere mantenuta secondo i termini della licenza.
