# Scorciatoie tastiera

> Tutte le scorciatoie attive in Prompt a Porter: hotkey globale, palette, modali, editor e menu contestuale.

Prompt a Porter è pensato per essere guidato dalla tastiera: il flusso tipico — hotkey, due lettere nella palette, `Enter`, `Ctrl+Enter` — non tocca mai il mouse. Questa pagina raccoglie tutte le scorciatoie in un posto solo, organizzate per contesto: quelle globali che funzionano ovunque, quelle attive nella finestra dell'app, e quelle specifiche di palette, modali ed editor.

Tienila a portata di mano le prime settimane: le scorciatoie della palette e della modale Compila sono quelle che ripagano prima, perché stanno sul percorso che ripeti ogni giorno.

Una nota sulla notazione: `Ctrl` si applica a Windows/Linux; su macOS sostituisci con `⌘` (Cmd). Quando entrambe le piattaforme usano `Ctrl` (es. la hotkey globale), è esplicitato.

## Globali (funzionano fuori dall'app)

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Shift+P` | Apri Command Palette (configurabile in **Impostazioni → Sistema → Hotkey**) |

La hotkey globale funziona anche quando l'app è minimizzata o in background: è il gesto con cui richiami Prompt a Porter mentre lavori altrove. Su macOS la combinazione di default è `⌃⇧P`.

## Nella finestra principale (app in primo piano)

| Scorciatoia | Azione |
|---|---|
| `Ctrl+K` o `Ctrl+Shift+P` | Apri Command Palette |
| `Ctrl+,` | Apri Impostazioni |

## Nella Command Palette

| Scorciatoia | Azione |
|---|---|
| `↑` / `↓` | Naviga fra i risultati |
| `Enter` | Compila il prompt selezionato (apre la modale Compila) |
| `Ctrl+.` | Apri pannello filtri avanzati (vista, tag, modello) |
| `Esc` | Chiudi palette |

## Nella modale Compila

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Enter` | Compila e copia negli appunti |
| `Esc` | Chiudi senza compilare |

`Tab` / `Shift+Tab` per navigare fra i campi segnaposti (comportamento standard).

## Nell'editor del prompt

L'editor del body supporta le scorciatoie di editing più diffuse. Le più comuni:

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` o `Ctrl+Shift+Z` | Redo |
| `Ctrl+A` | Seleziona tutto |
| `Ctrl+F` | Cerca nel body |
| `Ctrl+D` | Seleziona la prossima occorrenza della selezione |
| `Ctrl+/` | Toggle commento (commento HTML in Markdown) |

Quando l'autocomplete degli import (`{{import "..."}}`) è attivo:

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Space` | Apri autocomplete manualmente |
| `↑` / `↓` | Naviga suggerimenti |
| `Enter` | Conferma suggerimento |
| `Esc` | Chiudi autocomplete |

## Editing del prompt

Il salvataggio è automatico: scatta circa 2 secondi dopo l'ultima modifica, senza bisogno di una scorciatoia esplicita — ti basta scrivere. Per creare un nuovo prompt si clicca il bottone **Nuovo** in alto nella colonna della lista: non esiste una scorciatoia da tastiera dedicata.

## Modali in generale

| Scorciatoia | Azione |
|---|---|
| `Esc` | Chiudi modale (attivo in tutte le modali: Compila, Impostazioni, Palette, etc.) |
| `Tab` / `Shift+Tab` | Sposta il focus fra gli elementi della modale (il focus resta sempre all'interno della modale aperta) |

## Icona nel tray (tutti gli OS)

| Voce di menu | Azione |
|---|---|
| Apri palette | Apre la Command Palette |
| Nuovo prompt | Crea un nuovo prompt vuoto e apre l'editor |
| Mostra libreria | Porta la finestra in primo piano |
| Impostazioni | Apre la modale Impostazioni |
| Esci | Chiudi l'app |

## Menu contestuale (tasto destro)

Non è una scorciatoia da tastiera (non esiste una combinazione `Shift+F10` dedicata), ma il tasto destro apre un menu contestuale ricco:

**Su un prompt:**

| Voce | Azione |
|---|---|
| Apri | Apre il prompt nell'editor |
| Apri in Compila | Apre direttamente la modale Compila |
| Duplica (fork) | Crea un fork indipendente del prompt |
| Preferiti | Aggiunge/rimuove dai preferiti |
| Sposta in cartella | Sposta il prompt in un'altra cartella |
| Gestisci tag | Aggiunge/rimuove tag |
| Esporta come Markdown | Esporta il prompt in `.md` |
| Elimina | Sposta il prompt nel Cestino |

**Su una selezione multipla:** Confronta (N), Sposta N, Aggiungi tag a N, Esporta N, Elimina N.

**Su una cartella:** Nuovo prompt qui, Nuova sottocartella, Rinomina, Elimina cartella.

## Personalizzare la hotkey globale

1. Apri **Impostazioni → Sistema → Hotkey**.
2. Clicca sul campo e premi la nuova combinazione, poi salva. La modifica ha effetto al prossimo riavvio dell'app.
3. Combinazioni valide: almeno un modificatore (`Ctrl`/`Alt`/`Shift`/`Cmd`) + un tasto. Es. `Ctrl+Alt+P`, `Cmd+Shift+L`.
4. Il sistema verifica che non sia già in uso da un'altra app. In caso di conflitto, scegli un'altra combinazione.

> **Reset**: se la hotkey impostata smette di funzionare (es. conflitto con un nuovo programma installato), apri **Impostazioni → Sistema → Hotkey** e reinserisci il valore di default `Ctrl+Shift+P` (effettivo al riavvio).

## Limitazioni note

I binding Vim/Emacs non sono supportati: l'editor usa le scorciatoie standard elencate sopra. Allo stesso modo, a parte la hotkey globale, le scorciatoie non sono personalizzabili: le combinazioni di questa pagina sono fisse.

Su macOS, alcune combinazioni con `Ctrl` (es. `Ctrl+K`) sono già usate dal sistema per l'editing in stile Emacs: l'app rispetta la convenzione macOS e usa `⌘K` al loro posto.

## Vedi anche

- [`getting-started.md`](./getting-started.md) — il flusso hotkey → palette → Compila raccontato passo-passo.
- [`troubleshooting.md`](./troubleshooting.md) — cosa fare quando la hotkey globale non risponde.
