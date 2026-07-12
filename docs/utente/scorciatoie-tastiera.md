# Scorciatoie tastiera

> Tutte le scorciatoie attive in Prompt a Porter: hotkey globale, palette, modali, editor e menu contestuale.

La notazione `Ctrl` si applica a Windows/Linux; su macOS sostituisci con `⌘` (Cmd). Quando entrambe le piattaforme usano `Ctrl` (es. global hotkey), è esplicitato.

## Globali (funzionano fuori dall'app)

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Shift+P` | Apri Command Palette (configurabile in **Impostazioni → Sistema → Hotkey**) |

La hotkey globale è registrata via `tauri-plugin-global-shortcut` e funziona anche quando l'app è minimizzata o in background. Su macOS la combinazione di default è `⌃⇧P`.

## Nella Shell (app in primo piano)

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

`Tab` / `Shift+Tab` per navigare fra i campi segnaposti (comportamento standard browser).

## Nell'editor del prompt (CodeMirror)

L'editor usa CodeMirror 6: tutte le scorciatoie standard di CodeMirror sono attive. Le più comuni:

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` o `Ctrl+Shift+Z` | Redo |
| `Ctrl+A` | Seleziona tutto |
| `Ctrl+F` | Cerca nel body |
| `Ctrl+D` | Seleziona la prossima occorrenza della selezione |
| `Ctrl+/` | Toggle commento (Markdown comment HTML) |

Quando l'autocomplete `{{import "..."}}` è attivo (M4):

| Scorciatoia | Azione |
|---|---|
| `Ctrl+Space` | Apri autocomplete manualmente |
| `↑` / `↓` | Naviga suggerimenti |
| `Enter` | Conferma suggerimento |
| `Esc` | Chiudi autocomplete |

## Editing del prompt

- **Salvataggio**: automatico (debounce ~2 secondi dopo l'ultima modifica). Nessuna scorciatoia esplicita: ti basta scrivere.
- **Nuovo prompt**: click sul bottone `+ Nuovo` nel ListPane (in alto a sinistra). In v1.0 non c'è una scorciatoia da tastiera dedicata.

## Modali in generale

| Scorciatoia | Azione |
|---|---|
| `Esc` | Chiudi modale (attivo in tutte le modali: Compila, Impostazioni, Palette, etc.) |
| `Tab` / `Shift+Tab` | Cycle fra elementi focus all'interno della modale (focus trap WCAG 2.1) |

## Tray icon (tutti gli OS)

| Voce di menu | Azione |
|---|---|
| Apri palette | Apre la Command Palette |
| Nuovo prompt | Crea un nuovo prompt vuoto e apre l'editor |
| Mostra libreria | Porta la finestra in primo piano |
| Impostazioni | Apre la modale Impostazioni |
| Esci | Chiudi l'app |

## Menu contestuale (tasto destro)

Non è una scorciatoia da tastiera (non esiste una scorciatoia `Shift+F10` dedicata), ma il tasto destro apre un menu contestuale ricco:

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

- **Vim/Emacs binding**: non supportati in v1.0 (l'editor CodeMirror è in modalità default). Roadmap post-v1.0.
- **Macros / hotkey custom in-app**: oltre alla global hotkey, le scorciatoie sopra elencate sono fisse in v1.0. Future versioni esporranno una configurazione utente.
- **macOS**: alcune hotkey con `Ctrl` (es. `Ctrl+K`) sono già usate dal sistema (Emacs-style cut). L'app rispetta la convenzione macOS usando `⌘K`.
