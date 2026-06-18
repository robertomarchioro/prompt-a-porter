# Casi d'uso

Ricette concrete che mostrano Prompt a Porter all'opera su task reali. Ogni ricetta include:

- Il prompt completo (template con segnaposti).
- Come l'abbiamo strutturato e perché.
- Esempi di input e output.
- Varianti / estensioni.

| Ricetta | A cosa serve |
|---|---|
| [`email-professionale.md`](./email-professionale.md) | Email professionali parametriche (richieste, follow-up, reclami) |
| [`code-review.md`](./code-review.md) | Code review automatica con focus su edge case, performance, leggibilità |
| [`summarize-articolo.md`](./summarize-articolo.md) | Riassumere articoli tecnici / paper in N punti chiave |
| [`riscrittura-tono.md`](./riscrittura-tono.md) | Riscrivere testo cambiando registro (formale, conciso, divulgativo) |
| [`brainstorm-idee.md`](./brainstorm-idee.md) | Brainstorm strutturato con vincoli (numero, formato, criteri) |
| [`traduzione-tecnica.md`](./traduzione-tecnica.md) | Traduzioni con glossario tecnico custom |
| [`commit-message.md`](./commit-message.md) | Generare commit message da diff seguendo Conventional Commits |

## Come usare queste ricette

1. Apri PaP, clicca **+ Nuovo** nella ListPane.
2. Copia titolo + descrizione + body dalla ricetta nel form dell'editor.
3. Aggiungi i tag suggeriti (`email`, `code-review`, etc.) per ritrovarlo facilmente.
4. Personalizza i segnaposti: tutti i `{{nome}}` sono variabili che compilerai al momento dell'uso.
5. Considera di salvare il **ruolo / contesto** comune in un prompt separato e importarlo con `{{import "..."}}` — vedi [`prompt-componibili.md`](../prompt-componibili.md).

## Convenzioni

- I segnaposti sono in `italiano_lowercase_con_underscore`. Sostituisci se preferisci inglese.
- Quando una ricetta cita `{{global autore}}` o simili, intende un segnaposto globale da impostare una sola volta in **Impostazioni → Segnaposti globali** (vedi [`glossario-sintassi.md`](../glossario-sintassi.md)).
- I modelli target citati (`claude-sonnet`, `gpt-4`, etc.) sono suggerimenti basati su test empirici. Adatta al modello che usi tu.
