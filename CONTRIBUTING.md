# Contribuire a Prompt a Porter

Grazie per l'interesse! Questo progetto è attivamente sviluppato da Roberto Marchioro.

## Prerequisiti

- Node.js 22.x LTS
- pnpm 9.x+
- Rust toolchain stable (per il client Tauri)
- Go 1.22+ (per il server sync)
- Vedi [`docs/setup-sviluppo.md`](docs/setup-sviluppo.md) per istruzioni dettagliate

## Workflow di sviluppo

1. Forka il repository
2. Crea un branch per la tua feature (`git checkout -b feature/nome-feature`)
3. Sviluppa seguendo le convenzioni di codice del progetto
4. Assicurati che lint e test passino: `pnpm lint && pnpm test`
5. Committa con messaggi chiari in italiano
6. Apri una Pull Request verso `main`

## Convenzioni

- **Lingua**: italiano per commenti, variabili di dominio, documentazione, stringhe UI
- **Naming**: PascalCase per tipi/componenti/struct, camelCase per variabili/funzioni TS, snake_case per Rust/Go
- **CSS**: solo CSS custom properties, niente preprocessor, niente `!important`
- **Commenti**: spiegare il *perché*, non il *cosa*. Docstring per funzioni non triviali.
- **Test**: coverage minima 70% sui moduli core

## Licenza

Contribuendo accetti che il tuo codice sia distribuito sotto licenza GPL 2.0.
L'attribuzione all'autore originale deve essere mantenuta.
