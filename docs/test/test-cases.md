# Catalogo test cases

Versione: per release **v0.8.11** e successive. Aggiornare a ogni release.

Convenzioni: vedi [`README.md`](./README.md) di questa cartella.

---

## AUTH — Vault e credenziali

### `TC-AUTH-001` 🔴 🎯 Creazione vault al primo avvio

**Setup**: app installata, nessun vault esistente nel path di default.

**Steps**:
1. Avvia l'app.
2. Si apre il wizard di onboarding.
3. Step "Crea vault": inserisci password ≥ 12 caratteri con mix lettere/numeri/simboli.
4. Conferma password (campo di re-digitazione).
5. Click "Crea".

**Atteso**: vault creato nel path di default (vedi `getting-started.md`). App entra nella Shell con vault aperto. File `pap-vault.db` esiste su disco.

---

### `TC-AUTH-002` 🔴 🎯 Sblocco vault con password corretta

**Setup**: vault esistente, app chiusa.

**Steps**:
1. Avvia l'app.
2. Schermata "Inserisci password" appare.
3. Digita la password corretta.
4. Premi Invio o click "Sblocca".

**Atteso**: vault sblocca, entri nella Shell con i prompt precedenti visibili.

---

### `TC-AUTH-003` 🔴 ⚠️ Sblocco vault con password errata

**Setup**: vault esistente.

**Steps**:
1. Avvia l'app.
2. Digita una password sbagliata.
3. Premi Invio.

**Atteso**: messaggio "Password errata" (o equivalente). Il vault resta chiuso. Nessun rate limit visibile (locale, non serve), ma il messaggio è chiaro.

---

### `TC-AUTH-004` 🟡 🎯 Cambio password del vault

**Setup**: vault sbloccato.

**Steps**:
1. Apri **Impostazioni → Sicurezza** (o equivalente).
2. Click "Cambia password".
3. Inserisci password attuale + nuova password (2x).
4. Conferma.

**Atteso**: password aggiornata. Riavvia l'app: la vecchia password NON funziona, la nuova sì.

---

### `TC-AUTH-005` 🟡 ⚠️ Cambio password con password attuale errata

**Setup**: vault sbloccato.

**Steps**:
1. Impostazioni → Sicurezza → Cambia password.
2. Password attuale: inserisci stringa sbagliata.
3. Nuova password: qualcosa di valido.
4. Conferma.

**Atteso**: errore "Password attuale errata". Nessuna modifica applicata.

---

### `TC-AUTH-006` 🟡 🎯 Lock manuale del vault

**Setup**: vault sbloccato.

**Steps**:
1. Cerca il pulsante "Blocca vault" (tipicamente in menu o Impostazioni → Sicurezza).
2. Click.

**Atteso**: ritorni alla schermata di unlock. I prompt non sono più visibili senza password.

---

### `TC-AUTH-007` 🔵 🎯 Eliminazione vault

**Setup**: vault esistente.

**Steps**:
1. Backup esterno del file `.db` (per ripristino se serve).
2. Impostazioni → Sicurezza → Elimina vault.
3. Conferma con password.

**Atteso**: vault eliminato, app ritorna a stato "nessun vault" (onboarding al prossimo avvio).

---

### `TC-AUTH-008` ⚪ 🧪 Vault path custom via env variable

**Setup**: vault in posizione non standard, es. `~/test-vault.db`.

**Steps**:
1. Avvia l'app da terminale: `PAP_VAULT_PATH=~/test-vault.db prompt-a-porter`.
2. App apre il vault specificato.

**Atteso**: il vault custom viene aperto invece di quello di default.

---

## ONBOARD — Wizard primo avvio

### `TC-ONBOARD-001` 🔴 🎯 Wizard completo a 3 step

**Setup**: prima installazione, nessun vault.

**Steps**:
1. Avvia app.
2. Step 1 "Crea vault": come `TC-AUTH-001`.
3. Step 2 "Hotkey": accetta default `Ctrl+Shift+P` o personalizza.
4. Step 3 "Prompt esempio": flag attivo (default).
5. Conferma.

**Atteso**: vault creato, hotkey registrata, prompt esempio popolati nella libreria. Shell aperta.

---

### `TC-ONBOARD-002` 🟡 🎯 Skip prompt esempio

**Setup**: come sopra.

**Steps**:
1. Step 3 wizard: disattiva flag "Crea prompt esempio".
2. Conferma.

**Atteso**: vault creato vuoto. Lista prompt vuota nella Shell. ListPane mostra empty state.

---

### `TC-ONBOARD-003` 🔵 🧪 Wizard con hotkey già in uso

**Setup**: altra app registra `Ctrl+Shift+P` (es. clipboard manager).

**Steps**:
1. Wizard step 2: lascia hotkey default `Ctrl+Shift+P`.
2. Conferma.

**Atteso**: o messaggio di conflitto, o registrazione silenziosa (l'app PaP riceve l'evento, l'altra no). Verifica che la palette si apra premendo la hotkey.

---

## PROMPT — CRUD prompt

### `TC-PROMPT-001` 🔴 🎯 Crea nuovo prompt

**Setup**: vault aperto.

**Steps**:
1. Click "+ Nuovo" nel ListPane.
2. Editor si apre con titolo placeholder "Nuovo prompt".
3. Modifica titolo, descrizione, body.
4. Attendi ~2 secondi (autosave debounce).

**Atteso**: il prompt appare in libreria. Riavvio dell'app: il prompt è persistente.

---

### `TC-PROMPT-002` 🔴 🎯 Modifica prompt esistente

**Setup**: prompt esistente selezionato.

**Steps**:
1. Modifica body nel editor.
2. Attendi ~2 secondi.
3. Verifica indicatore "Salvato" o similar.

**Atteso**: modifiche persistenti. Riapri l'app: il body modificato è quello mostrato.

---

### `TC-PROMPT-003` 🔴 🎯 Elimina prompt

**Setup**: prompt esistente.

**Steps**:
1. Tasto destro su prompt nel ListPane (o icona cestino).
2. "Elimina".
3. Conferma.

**Atteso**: prompt sparisce dalla lista. Va nel cestino (soft delete: `DeletedAt IS NOT NULL`).

---

### `TC-PROMPT-004` 🟡 🎯 Ripristina prompt dal cestino

**Setup**: prompt eliminato.

**Steps**:
1. Apri vista "Cestino" (se esiste in UI) o usa import per ripristinare.
2. Ripristina il prompt.

**Atteso**: il prompt torna in libreria. *Se la UI cestino non è implementata in v0.8.11, marca questo TC come TODO post-v1.0.*

---

### `TC-PROMPT-005` 🟡 ⚡ Autosave debounce 2s

**Setup**: editor aperto su un prompt.

**Steps**:
1. Modifica body.
2. **Non** aspettare.
3. Chiudi immediatamente l'app (kill).

**Atteso**: alla riapertura, le modifiche degli ultimi <2 secondi PRIMA della chiusura potrebbero essere perse. Modifiche più vecchie di 2s devono essere persistite.

---

### `TC-PROMPT-006` 🔵 🧪 Prompt con body molto lungo

**Setup**: editor aperto.

**Steps**:
1. Incolla un body di ~50.000 caratteri.
2. Attendi autosave.

**Atteso**: salvataggio ok. Linter mostra warning `LEN001` (body > 4000). Editor resta responsive.

---

### `TC-PROMPT-007` 🔵 🧪 Body vuoto

**Setup**: editor.

**Steps**:
1. Crea prompt, lascia body vuoto.
2. Salva.

**Atteso**: prompt creato ma linter info `LEN002` (body < 30 char). Nessun crash.

---

## SEARCH — Ricerca FTS

### `TC-SEARCH-001` 🔴 🎯 Ricerca testo nel titolo

**Setup**: prompt con titolo "Email marketing cold outreach" esistente.

**Steps**:
1. Digita "marketing" nella barra di ricerca.

**Atteso**: il prompt appare nei risultati.

---

### `TC-SEARCH-002` 🔴 🎯 Ricerca testo nel body

**Setup**: prompt con body contenente "B2B SaaS".

**Steps**:
1. Digita "B2B" nella barra di ricerca.

**Atteso**: il prompt appare nei risultati (FTS indica il match su Body).

---

### `TC-SEARCH-003` 🟡 🎯 Filtro vista "Preferiti"

**Setup**: alcuni prompt preferiti, altri no.

**Steps**:
1. Click sulla vista "Preferiti" in sidebar.

**Atteso**: solo i prompt con `IsFavorite=1` appaiono.

---

### `TC-SEARCH-004` 🟡 🎯 Filtro per tag

**Setup**: prompt con tag "email" e altri senza.

**Steps**:
1. Click sul tag "email" in sidebar.

**Atteso**: solo i prompt con quel tag.

---

### `TC-SEARCH-005` 🟡 🎯 Filtro per modello target

**Setup**: prompt con `TargetModel="claude-sonnet"` e altri senza.

**Steps**:
1. Apri filtri avanzati (Ctrl+. in palette o pannello dedicato).
2. Filtra per modello "claude-sonnet".

**Atteso**: solo i prompt con quel target.

---

### `TC-SEARCH-006` 🟡 🎯 Filtro per cartella (sottoalbero)

**Setup**: cartella `marketing/email` con prompt dentro.

**Steps**:
1. Click sulla cartella `marketing` nella sidebar.

**Atteso**: tutti i prompt nel sottoalbero (inclusi quelli in `marketing/email`).

---

### `TC-SEARCH-007` 🔵 🧪 Ricerca con caratteri speciali

**Setup**: prompt esistenti.

**Steps**:
1. Digita "!!! ???" nella ricerca.

**Atteso**: 0 risultati (caratteri non-alfanumerici stripped da `sanitizzaFts`). Nessun errore SQL.

---

### `TC-SEARCH-008` 🔵 ⚡ Ricerca su 1000+ prompt

**Setup**: vault con ≥1000 prompt (importati via JSON o creati manualmente).

**Steps**:
1. Digita "test" nella ricerca.

**Atteso**: risultati appaiono entro 200ms. UI resta responsive.

---

## SEMANTIC — Ricerca semantica

### `TC-SEMANTIC-001` 🟡 🎯 Abilita ricerca semantica

**Setup**: vault con prompt esistenti.

**Steps**:
1. Impostazioni → Ricerca → "Ricerca semantica abilitata" → ON.
2. Conferma download modello (~150 MB).

**Atteso**: download completa. Toggle resta ON. Al riavvio resta attivo.

---

### `TC-SEMANTIC-002` 🟡 🎯 Ricerca semantica trova prompt simili senza match testuale

**Setup**: prompt con titolo "Email cold outreach", ricerca semantica abilitata, backfill completato.

**Steps**:
1. Digita "scrivere a un potenziale cliente" nella ricerca.

**Atteso**: il prompt "Email cold outreach" appare nei risultati anche senza match testuale diretto.

---

### `TC-SEMANTIC-003` 🔵 🧪 Idle unload del modello

**Setup**: ricerca semantica attiva, `idle_unload_secondi` ≥ 60.

**Steps**:
1. Esegui una ricerca semantica.
2. Non interagire con l'app per `idle_unload_secondi` + 30s.
3. Esegui altra ricerca.

**Atteso**: la seconda ricerca richiede ~1-2s extra (ricarica modello). Memoria scende dopo l'idle.

---

### `TC-SEMANTIC-004` 🔵 🎯 Alpha bilanciamento FTS/semantic

**Setup**: ricerca ibrida attiva.

**Steps**:
1. Impostazioni → Ricerca → Alpha = 0 (solo FTS).
2. Cerca un termine: solo match testuali.
3. Alpha = 1 (solo semantic).
4. Stessa ricerca: solo match semantici.

**Atteso**: i risultati cambiano in base ad alpha.

---

## COMPILE — Compilazione prompt

### `TC-COMPILE-001` 🔴 🎯 Compilazione da Command Palette

**Setup**: prompt con segnaposti esistente.

**Steps**:
1. Premi hotkey globale (`Ctrl+Shift+P`).
2. Palette si apre.
3. Digita titolo del prompt.
4. Enter sul risultato.
5. Modale Compila si apre.
6. Inserisci valori segnaposti.
7. `Ctrl+Enter`.

**Atteso**: testo compilato copiato negli appunti. Toast "Copiato" visibile. Modale si chiude.

---

### `TC-COMPILE-002` 🔴 🎯 Compilazione con tutti i segnaposti vuoti

**Setup**: prompt `"Ciao {{nome}}"`.

**Steps**:
1. Apri Compila per quel prompt.
2. Lascia `nome` vuoto.
3. `Ctrl+Enter`.

**Atteso**: output `"Ciao {{nome}}"` (segnaposto resta intatto by design). Avviso o nota sul fatto che segnaposti non compilati.

---

### `TC-COMPILE-003` 🔴 🎯 Compilazione con segnaposto multiplo

**Setup**: prompt `"{{nome}} è {{nome}}"`.

**Steps**:
1. Compila con `nome=Mario`.

**Atteso**: output `"Mario è Mario"`. Il form chiede `nome` una sola volta.

---

### `TC-COMPILE-004` 🟡 🎯 Copia clipboard funziona su tutte le piattaforme

**Setup**: prompt compilato.

**Steps**:
1. `Ctrl+Enter` o "Compila & copia".
2. Apri editor esterno (Notepad, TextEdit, terminale).
3. Incolla.

**Atteso**: testo identico al compilato. Niente escape strani.

---

### `TC-COMPILE-005` 🔵 🧪 Compilazione di prompt con caratteri speciali

**Setup**: body con emoji, accenti, newline, tab.

**Steps**:
1. Compila.
2. Verifica testo copiato.

**Atteso**: tutti i caratteri preservati. Encoding UTF-8 corretto.

---

### `TC-COMPILE-006` 🔵 ⚡ Compile inline live nell'editor (M5)

**Setup**: editor doppia vista aperto su prompt con segnaposti.

**Steps**:
1. Modifica body con un segnaposto `{{x}}`.
2. Nella vista "Valori": inserisci `x=test`.
3. Osserva vista "Compilato".

**Atteso**: vista Compilato si aggiorna live (debounced ~300ms) mostrando `test`.

---

## PLACEHOLDER — Segnaposti

### `TC-PLACEHOLDER-001` 🔴 🎯 Segnaposto semplice `{{nome}}`

Vedi `TC-COMPILE-001`.

---

### `TC-PLACEHOLDER-002` 🔴 🎯 Segnaposto globale `{{globale autore}}`

**Setup**: in Impostazioni → Segnaposti globali aggiungi `autore=Mario`.

**Steps**:
1. Crea prompt con body `"Firmato: {{globale autore}}"`.
2. Apri Compila.

**Atteso**: il form NON chiede `autore` (è globale). Output: `"Firmato: Mario"`.

---

### `TC-PLACEHOLDER-003` 🟡 🎯 Globale modificato in Impostazioni

**Setup**: globale `autore=Mario`, prompt che lo usa.

**Steps**:
1. Modifica globale a `autore=Anna`.
2. Compila il prompt.

**Atteso**: output usa "Anna" (no cache stale).

---

### `TC-PLACEHOLDER-004` 🟡 🧪 Globale assente

**Setup**: prompt usa `{{globale inesistente}}` non definito.

**Steps**:
1. Compila.

**Atteso**: il testo `{{globale inesistente}}` resta intatto nel risultato (così l'utente se ne accorge).

---

### `TC-PLACEHOLDER-005` 🔵 🧪 Segnaposto con caratteri non validi

**Setup**: body con `{{nome con spazi}}`.

**Steps**:
1. Salva il prompt.
2. Apri linter.

**Atteso**: linter mostra warning `PH003` (nome invalido). Il segnaposto NON viene matchato in compilazione.

---

### `TC-PLACEHOLDER-006` 🔵 ⚠️ Singola graffa `{nome}`

**Setup**: body con `{nome}` (singola graffa).

**Steps**:
1. Salva.
2. Apri linter.

**Atteso**: linter `PH001` errore. Compilazione lascia `{nome}` intatto.

---

## IMPORT — Prompt componibili

### `TC-IMPORT-001` 🔴 🎯 Import semplice da titolo

**Setup**: prompt "ruolo-marketing" esistente con body `"Sei un esperto marketing."`.

**Steps**:
1. Crea prompt B con body `{{import "ruolo-marketing"}}\n\nScrivi una email.`.
2. Compila B.

**Atteso**: output = body di "ruolo-marketing" + "Scrivi una email.".

---

### `TC-IMPORT-002` 🔴 🎯 Import con path cartella

**Setup**: prompt in `marketing/email/cold-outreach`.

**Steps**:
1. Body B: `{{import "marketing/email/cold-outreach"}}`.
2. Compila.

**Atteso**: risolve correttamente al prompt nella cartella.

---

### `TC-IMPORT-003` 🟡 🎯 Import nidificato (depth ≤ 5)

**Setup**: A importa B che importa C.

**Steps**:
1. Compila A.

**Atteso**: chain risolta. Output finale contiene il body di C.

---

### `TC-IMPORT-004` 🔴 ⚠️ Import ciclo diretto A → A

**Setup**: prompt A con body `{{import "A"}}`.

**Steps**:
1. Apri linter.
2. Prova a compilare.

**Atteso**: linter `IMP002` (ciclo). Compilazione fallisce con messaggio chiaro.

---

### `TC-IMPORT-005` 🔴 ⚠️ Import ciclo indiretto A → B → A

**Setup**: A importa B, B importa A.

**Steps**: come sopra.

**Atteso**: linter `IMP002` su entrambi i prompt.

---

### `TC-IMPORT-006` 🟡 🧪 Depth eccessiva (>5 livelli)

**Setup**: A → B → C → D → E → F.

**Steps**:
1. Compila A.

**Atteso**: errore di compilazione + linter `IMP003` warning.

---

### `TC-IMPORT-007` 🟡 ⚠️ Import path inesistente

**Setup**: body `{{import "non-esiste"}}`.

**Steps**:
1. Apri linter.

**Atteso**: linter `IMP001` errore "Import non risolto".

---

### `TC-IMPORT-008` 🟡 🎯 Import con `version=N`

**Setup**: prompt P versionato (≥ 2 versioni nello storico).

**Steps**:
1. Body B: `{{import "P" version=1}}`.
2. Compila.

**Atteso**: usa la versione 1 (non l'ultima).

---

### `TC-IMPORT-009` 🟡 🎯 Import con `with k=v`

**Setup**: P contiene `{{tono}}`.

**Steps**:
1. Body B: `{{import "P" with tono=formale}}`.
2. Compila.

**Atteso**: il segnaposto `tono` di P viene sostituito con `formale` automaticamente.

---

### `TC-IMPORT-010` 🔵 🎯 Combinazione `version=N with k=v`

**Setup**: come `TC-IMPORT-008` + `TC-IMPORT-009`.

**Steps**:
1. Body B: `{{import "P" version=2 with tono=conciso}}`.

**Atteso**: usa versione 2 di P, override `tono`.

---

## INTELLISENSE — Autocomplete editor

### `TC-INTELLISENSE-001` 🟡 🎯 Suggerimenti su `{{import "<prefix>`

**Setup**: vault con prompt "marketing-email" e "marketing-social".

**Steps**:
1. Editor aperto.
2. Digita `{{import "mark`.
3. Attendi popup autocomplete.

**Atteso**: appaiono "marketing-email" e "marketing-social".

---

### `TC-INTELLISENSE-002` 🔵 🎯 Selezione con Enter

**Setup**: come sopra.

**Steps**:
1. Frecce ↑/↓ per navigare.
2. Enter sul suggerimento.

**Atteso**: il prefisso viene esteso al titolo completo.

---

### `TC-INTELLISENSE-003` 🔵 🧪 Self-import escluso

**Setup**: editor su prompt "P".

**Steps**:
1. Digita `{{import "P`.

**Atteso**: il prompt "P" stesso NON appare nei suggerimenti (no self-import).

---

### `TC-INTELLISENSE-004` ⚪ 🧪 Nessun popup se quote già chiusa

**Setup**: body `{{import "foo"}}` cursore dopo la quote di chiusura.

**Atteso**: nessun popup autocomplete.

---

## LINT — Regole linter

### `TC-LINT-LEN001` 🟡 🎯 Body > 4000 caratteri → warning

**Setup**: body lungo > 4000 char.

**Atteso**: linter mostra `LEN001` warning.

---

### `TC-LINT-LEN002` 🔵 🎯 Body < 30 caratteri → info

**Setup**: body "Ciao".

**Atteso**: linter mostra `LEN002` info.

---

### `TC-LINT-PH001` 🔴 🎯 Singola graffa → error

**Setup**: body con `{nome}`.

**Atteso**: linter `PH001` error.

---

### `TC-LINT-PH003` 🟡 🎯 Nome con spazi/trattini → warning

**Setup**: body con `{{nome con spazi}}` o `{{nome-trattino}}`.

**Atteso**: `PH003` warning.

---

### `TC-LINT-PII001` 🟡 🎯 Email nel body → warning

**Setup**: body contiene `utente@example.com`.

**Atteso**: `PII001` warning.

---

### `TC-LINT-PII003` 🔴 🎯 Numero carta credito Luhn-valido → error

**Setup**: body `"Carta: 4532015112830366"` (numero test Luhn-valido).

**Atteso**: `PII003` error. Salvataggio dovrebbe essere scoraggiato/bloccato.

---

### `TC-LINT-PII004` 🔴 🎯 API key di provider noti → error

**Setup**: body con `"sk-test-1234567890abcdef"` (formato OpenAI).

**Atteso**: `PII004` error.

---

### `TC-LINT-STY001` 🔵 🎯 Ripetizione n-gram → info

**Setup**: body con la stessa frase ripetuta 3+ volte.

**Atteso**: `STY001` info.

---

### `TC-LINT-STY002` 🔵 🎯 Parola in CAPS > 10 char → info

**Setup**: body con "IMPORTANTISSIMO".

**Atteso**: `STY002` info.

---

### `TC-LINT-IMP001` 🔴 🎯 Import non risolto → error

Vedi `TC-IMPORT-007`.

---

### `TC-LINT-IMP002` 🔴 🎯 Ciclo import → error

Vedi `TC-IMPORT-004`/`005`.

---

### `TC-LINT-IMP003` 🟡 🎯 Depth ≥ 6 → warning

Vedi `TC-IMPORT-006`.

---

### `TC-LINT-DISABLE` 🔵 🎯 Disabilita categoria linter

**Setup**: linter mostra warning attivi.

**Steps**:
1. Impostazioni → Linter.
2. Disattiva categoria (es. PII).
3. Riapri il prompt.

**Atteso**: i warning della categoria disabilitata non appaiono. Riavvio app: persistito (localStorage).

---

## FOLDER — Cartelle

### `TC-FOLDER-001` 🟡 🎯 Crea cartella

**Steps**:
1. Tasto destro su sidebar cartelle → "Nuova cartella".
2. Nome: "marketing".

**Atteso**: cartella creata, visibile in sidebar.

---

### `TC-FOLDER-002` 🟡 🎯 Crea sotto-cartella

**Setup**: cartella "marketing" esistente.

**Steps**:
1. Tasto destro su "marketing" → "Nuova sotto-cartella".
2. Nome: "email".

**Atteso**: `marketing/email` creata. Path corretto.

---

### `TC-FOLDER-003` 🟡 🎯 Rinomina cartella

**Steps**:
1. Tasto destro su cartella → "Rinomina".
2. Nuovo nome.

**Atteso**: cartella rinominata. Path aggiornato anche per i prompt dentro.

---

### `TC-FOLDER-004` 🟡 🎯 Sposta cartella

**Steps**:
1. Drag&drop o menu "Sposta in".
2. Destinazione: altra cartella.

**Atteso**: la cartella e tutto il sottoalbero spostati. Path di tutti i prompt discendenti aggiornato.

---

### `TC-FOLDER-005` 🔴 ⚠️ Sposta cartella dentro se stessa

**Steps**:
1. Tenta di spostare `marketing` dentro `marketing/email`.

**Atteso**: errore. L'app rifiuta l'operazione (no cicli).

---

### `TC-FOLDER-006` 🟡 🎯 Elimina cartella

**Setup**: cartella con prompt dentro.

**Steps**:
1. Elimina cartella.
2. Conferma.

**Atteso**: cartella eliminata (soft delete). I prompt dentro: tornano a root o vengono soft-deleted con essa (verificare comportamento attuale e documentarlo).

---

### `TC-FOLDER-007` 🟡 🎯 Sposta prompt fra cartelle

**Steps**:
1. Drag&drop di un prompt in un'altra cartella.

**Atteso**: `FolderId` aggiornato. Il prompt appare nella nuova cartella.

---

### `TC-FOLDER-008` 🔵 🎯 Riordina cartelle

**Steps**:
1. Drag&drop per riordinare cartelle siblings.

**Atteso**: ordine persistito. Riavvio: ordine rispettato.

---

## TAG — Gestione tag

### `TC-TAG-001` 🟡 🎯 Aggiungi tag a prompt

**Steps**:
1. Right Rail → campo tag → digita "email" → Enter.

**Atteso**: tag aggiunto al prompt. Visibile nelle card.

---

### `TC-TAG-002` 🟡 🎯 Rimuovi tag

**Steps**:
1. Right Rail → click su X accanto al tag.

**Atteso**: tag rimosso dal prompt (ma non eliminato dal vault se altri prompt lo usano).

---

### `TC-TAG-003` 🔵 🎯 Lista tag in sidebar

**Setup**: vault con tag.

**Atteso**: tutti i tag esistenti elencati in sidebar, click filtra.

---

### `TC-TAG-004` 🔵 🧪 Suggerimenti tag in autocomplete

**Setup**: vault con tag esistenti.

**Steps**:
1. Right Rail → campo tag → digita prima lettera.

**Atteso**: dropdown di suggerimenti dai tag esistenti.

---

## VARIANT — Varianti A/B

### `TC-VARIANT-001` 🟡 🎯 Crea variante

**Setup**: prompt P.

**Steps**:
1. Right Rail → "Crea variante".

**Atteso**: nuovo prompt creato come variante (linkato a P via `ParentPromptId`).

---

### `TC-VARIANT-002` 🟡 🎯 Lista varianti di un prompt

**Setup**: P con 2 varianti.

**Steps**:
1. Right Rail aperto su P → tab "Varianti".

**Atteso**: le 2 varianti elencate. Click per navigarci.

---

### `TC-VARIANT-003` 🟡 🎯 Promuovi variante a principale (M3)

**Setup**: P (principale), V (variante).

**Steps**:
1. Apri V.
2. Right Rail → "Promuovi a principale".
3. Conferma.

**Atteso**: V diventa principale, P diventa variante.

---

### `TC-VARIANT-004` 🔵 🎯 Confronta due varianti (diff)

**Setup**: 2 varianti dello stesso prompt.

**Steps**:
1. Seleziona 2 varianti (Ctrl+click su card).
2. Tasto destro → "Confronta".

**Atteso**: diff side-by-side visibile.

---

## FORK — Fork prompt

### `TC-FORK-001` 🔵 🎯 Crea fork

**Setup**: prompt P.

**Steps**:
1. Right Rail → "Fork".

**Atteso**: nuovo prompt indipendente con banner "Fork di P". `ParentPromptId` punta a P.

---

### `TC-FORK-002` 🔵 🎯 Chain di fork

**Setup**: fork A → fork B → fork C.

**Atteso**: C mostra catena di fork ancestrale.

---

## RATING — Rating discreto

### `TC-RATING-001` 🔵 🎯 Aggiungi rating dopo compilazione

**Setup**: prompt compilato.

**Steps**:
1. Modale Compila → click su 👎 / 😐 / 👍 (al momento della copia o successivamente).

**Atteso**: rating salvato. Badge percentuale aggiornato.

---

### `TC-RATING-002` 🔵 🎯 Badge percentuale visibile su card

**Setup**: prompt con ≥ 3 rating.

**Atteso**: card mostra badge colorato (verde/giallo/rosso) con percentuale media.

---

## CRONOLOGIA — Versioning

### `TC-CRONOLOGIA-001` 🟡 🎯 Visualizza storia versioni

**Setup**: prompt modificato N volte.

**Steps**:
1. Right Rail → tab "Cronologia".

**Atteso**: lista N entry con timestamp, autore (sempre "Utente locale" in personale), riassunto modifiche.

---

### `TC-CRONOLOGIA-002` 🟡 🎯 Rollback a versione precedente

**Steps**:
1. Cronologia → seleziona versione vecchia.
2. "Ripristina".

**Atteso**: body torna alla versione selezionata. Nuova entry aggiunta in cronologia (immutabile).

---

### `TC-CRONOLOGIA-003` 🔵 🎯 Diff fra due versioni

**Steps**:
1. Cronologia → seleziona 2 versioni → "Confronta".

**Atteso**: diff visibile.

---

## EDITOR — Editor doppia vista (M5)

### `TC-EDITOR-001` 🟡 🎯 Switch Sorgente / Compilato

**Setup**: prompt con segnaposti.

**Steps**:
1. Editor → toggle "Doppia vista" o tab "Compilato".

**Atteso**: vista split o tab dedicata. La vista Compilato mostra il body con segnaposti sostituiti.

---

### `TC-EDITOR-002` 🟡 🎯 Form valori live aggiorna compilato

**Setup**: doppia vista attiva.

**Steps**:
1. Modifica un valore nel form.
2. Osserva pannello "Compilato".

**Atteso**: aggiornamento entro ~300ms.

---

### `TC-EDITOR-003` 🔵 🎯 Import risolti live

**Setup**: prompt con `{{import "..."}}`.

**Atteso**: pannello Compilato mostra anche il body importato espanso.

---

## EDITORPREF — Preferenze editor (M10)

Le 6 opzioni configurabili in **Impostazioni → Editor**. Vedi anche `docs/utente/scorciatoie-tastiera.md` ed eventuali default in `apps/client/src-tauri/src/preferenze.rs`.

### `TC-EDITORPREF-001` 🟡 🎯 Autosave delay — slider modifica il valore

**Setup**: vault aperto, Impostazioni → Editor.

**Steps**:
1. Trascina lo slider "Autosave delay" (range 500-5000 ms, step 250).
2. Osserva il valore numerico accanto allo slider.

**Atteso**: il valore mostrato segue lo slider (es. "1500 ms"). Default iniziale 2000 ms.

---

### `TC-EDITORPREF-002` 🟡 🎯 Autosave delay — effetto sul salvataggio

**Setup**: autosave delay impostato a 500 ms.

**Steps**:
1. Apri un prompt, modifica il body.
2. Smetti di scrivere e conta ~0.5s.

**Atteso**: il salvataggio scatta dopo ~500 ms (indicatore "Salvato"). Imposta 5000 ms e verifica che ora attende ~5s.

---

### `TC-EDITORPREF-003` 🟡 🎯 Line wrapping — toggle ON/OFF

**Setup**: prompt con una riga molto lunga (oltre la larghezza dell'editor).

**Steps**:
1. Impostazioni → Editor → "Line wrapping" OFF.
2. Torna all'editor.
3. Riattiva ON.

**Atteso**: OFF → la riga lunga richiede scroll orizzontale. ON → la riga va a capo (soft wrap). Default ON.

---

### `TC-EDITORPREF-004` 🔵 🎯 Indent size — 2 vs 4 spazi

**Setup**: editor aperto.

**Steps**:
1. Impostazioni → Editor → Indent size = 4.
2. Nell'editor, posizionati a inizio riga e premi Tab.

**Atteso**: con 4 selezionato, Tab inserisce 4 spazi; con 2, ne inserisce 2. Default 2.

---

### `TC-EDITORPREF-005` 🔵 🎯 Font size — slider 12-20px

**Setup**: editor aperto.

**Steps**:
1. Impostazioni → Editor → trascina "Dimensione font" a 18.
2. Torna all'editor.

**Atteso**: il testo dell'editor diventa più grande (18px). Valore mostrato "18 px". Default 13 px. Range clampato 12-20.

---

### `TC-EDITORPREF-006` 🔵 🎯 Mostra numeri di riga — toggle

**Setup**: editor aperto.

**Steps**:
1. Impostazioni → Editor → "Numeri di riga" OFF.
2. Torna all'editor.

**Atteso**: OFF → gutter sinistro senza numeri. ON → numeri visibili. Default ON.

---

### `TC-EDITORPREF-007` 🔵 🎯 Evidenzia riga attiva — toggle

**Setup**: editor aperto.

**Steps**:
1. Impostazioni → Editor → "Evidenzia riga attiva" ON.
2. Torna all'editor, clicca su una riga.

**Atteso**: la riga sotto il cursore ha uno sfondo leggermente diverso. OFF → nessun highlight. Default OFF.

---

### `TC-EDITORPREF-008` 🟡 🎯 Persistenza preferenze dopo riavvio

**Setup**: modifica tutte e 6 le preferenze a valori non-default.

**Steps**:
1. Imposta: autosave 1000ms, wrapping OFF, indent 4, font 16, numeri OFF, highlight ON.
2. Chiudi e riapri l'app.
3. Apri Impostazioni → Editor.

**Atteso**: tutti i 6 valori riflettono le scelte (persistiti in `preferenze.json`). L'editor li applica al primo prompt aperto.

---

### `TC-EDITORPREF-009` 🔵 🧪 Forward-compat con preferenze.json vecchio

**Setup**: un `preferenze.json` salvato da una versione < M10 (senza i 6 campi editor).

**Steps**:
1. Avvia l'app con quel file.
2. Apri Impostazioni → Editor.

**Atteso**: i controlli mostrano i default (autosave 2000, wrapping ON, indent 2, font 13, numeri ON, highlight OFF) senza crash. Al primo salvataggio, i campi vengono aggiunti al file.

---

### `TC-EDITORPREF-010` 🔵 ⚡ Cambio prefs senza perdere il contenuto editor

**Setup**: editor aperto con body non vuoto, modifiche non ancora salvate.

**Steps**:
1. Cambia una preferenza che rimonta l'editor (es. font size).
2. Torna all'editor.

**Atteso**: il contenuto del body è preservato (il re-mount usa il testo corrente). Nessuna perdita di testo, nessun dirty fantasma.

---

## HOTKEY — Global hotkey

### `TC-HOTKEY-001` 🔴 🎯 Hotkey default `Ctrl+Shift+P`

**Setup**: app aperta o in background.

**Steps**:
1. Premi `Ctrl+Shift+P` da fuori l'app (es. dal browser).

**Atteso**: Command Palette appare in primo piano.

---

### `TC-HOTKEY-002` 🟡 🎯 Cambia hotkey

**Steps**:
1. Impostazioni → Hotkey → click sul campo.
2. Premi nuova combinazione (es. `Ctrl+Alt+L`).
3. Salva.

**Atteso**: nuova hotkey attiva. Quella vecchia non funziona più.

---

### `TC-HOTKEY-003` 🟡 🧪 Hotkey in conflitto con altra app

**Setup**: altra app registra prima la stessa combo.

**Steps**: 1. Avvia PaP, prova la hotkey.

**Atteso**: o messaggio di conflitto, o PaP non riceve l'evento. Comportamento documentato in `troubleshooting.md`.

---

### `TC-HOTKEY-004` 🔵 🎯 Reset hotkey a default

**Steps**:
1. Impostazioni → Hotkey → "Reset".

**Atteso**: torna a `Ctrl+Shift+P`.

---

## PALETTE — Command Palette

### `TC-PALETTE-001` 🔴 🎯 Apri palette in app

**Setup**: app in foreground.

**Steps**:
1. `Ctrl+K`.

**Atteso**: palette si apre centrata.

---

### `TC-PALETTE-002` 🟡 🎯 Navigazione frecce + Enter

**Steps**:
1. Palette aperta.
2. ↑ / ↓ per navigare.
3. Enter.

**Atteso**: seleziona risultato. Apre modale Compila.

---

### `TC-PALETTE-003` 🟡 🎯 Filtri avanzati `Ctrl+.`

**Steps**:
1. Palette aperta.
2. `Ctrl+.`.

**Atteso**: pannello filtri (vista/tag/modello) si apre.

---

### `TC-PALETTE-004` 🔵 🎯 Esc chiude palette

**Steps**: Esc nella palette.

**Atteso**: palette chiusa.

---

## TRAY — Tray icon

### `TC-TRAY-001` 🔵 🎯 Tray icon presente (Windows/Linux)

**Setup**: app avviata.

**Atteso**: icona nella tray bar di sistema.

---

### `TC-TRAY-002` 🔵 🎯 Voce "Apri" porta in primo piano

**Steps**: tray → "Apri Prompt a Porter".

**Atteso**: finestra principale in foreground.

---

### `TC-TRAY-003` 🔵 🎯 Voce "Nuovo prompt"

**Steps**: tray → "Nuovo prompt".

**Atteso**: app aperta + nuovo prompt creato + editor aperto.

---

### `TC-TRAY-004` 🔵 🎯 Voce "Impostazioni"

**Steps**: tray → "Impostazioni".

**Atteso**: modale Impostazioni aperta.

---

### `TC-TRAY-005` 🔵 🎯 Voce "Esci"

**Steps**: tray → "Esci".

**Atteso**: app chiusa completamente (nessun processo residuo).

---

## EXP — Export / Import

### `TC-EXP-JSON-001` 🟡 🎯 Export vault JSON

**Steps**:
1. Impostazioni → Dati → "Esporta vault".

**Atteso**: file `pap-export-<data>.json` salvato. Apri con editor: struttura coerente con `formato-export-json.md`.

---

### `TC-EXP-JSON-002` 🟡 🎯 Import vault JSON (modalità skip)

**Setup**: vault con prompt esistenti. JSON da importare con duplicati.

**Steps**:
1. Impostazioni → Dati → "Importa" → seleziona JSON → modalità "Skip".

**Atteso**: prompt nuovi importati, duplicati ignorati.

---

### `TC-EXP-JSON-003` 🟡 🎯 Import JSON modalità overwrite

**Atteso**: duplicati sovrascritti. Cronologia preserva la versione precedente.

---

### `TC-EXP-JSON-004` 🟡 🎯 Import JSON modalità rename

**Atteso**: duplicati importati con suffisso (es. "Email (2)").

---

### `TC-EXP-JSON-005` 🔵 🎯 Export di una sola cartella

**Setup**: vault con cartelle.

**Steps**: Impostazioni → Dati → "Esporta cartella" → seleziona cartella.

**Atteso**: JSON con solo i prompt del sottoalbero.

---

### `TC-EXP-MD-001` 🟡 🎯 Export Markdown bulk (zip)

**Steps**:
1. Impostazioni → Dati → "Esporta come Markdown".

**Atteso**: file `.zip` con un `.md` per ogni prompt. Struttura cartelle preservata.

---

### `TC-EXP-MD-002` 🟡 🎯 Import Markdown singolo file

**Setup**: file `.md` con front-matter YAML.

**Steps**:
1. Drag&drop o "Importa Markdown".

**Atteso**: prompt creato con titolo da front-matter, body, tag.

---

### `TC-EXP-MD-003` 🟡 🎯 Import bulk cartella `.md`

**Setup**: cartella locale con N file `.md` (Obsidian / Foam compatibile).

**Steps**:
1. Impostazioni → Dati → "Importa cartella Markdown".

**Atteso**: tutti i prompt importati. Sotto-cartelle preservate.

---

## PREF — Preferenze

### `TC-PREF-001` 🟡 🎯 Toggle tema dark/light

**Steps**: title bar → click icona sole/luna.

**Atteso**: tema cambia istantaneamente. Persistito.

---

### `TC-PREF-002` 🔵 🎯 Cambia tono (zinc/slate/...)

**Steps**: Impostazioni → Aspetto → tono.

**Atteso**: colori accent cambiano.

---

### `TC-PREF-003` 🔵 🎯 Cambia densità lista (compatto/anteprima)

**Steps**: ListPane → chip densità.

**Atteso**: card più alte o più basse.

---

### `TC-PREF-004` 🟡 🎯 Aggiungi segnaposto globale

**Steps**:
1. Impostazioni → Segnaposti globali → "+ Aggiungi".
2. Nome: `autore`, valore: `Mario`.

**Atteso**: salvato. Vedi `TC-PLACEHOLDER-002`.

---

### `TC-PREF-005` 🔵 🎯 Modifica segnaposto globale

**Steps**: edit valore esistente.

**Atteso**: persistito.

---

### `TC-PREF-006` 🔵 🎯 Elimina segnaposto globale

**Steps**: rimuovi entry.

**Atteso**: rimossa. `{{globale ...}}` che la referenziava non viene più risolto.

---

### `TC-PREF-007` 🔵 🎯 Debug log ON/OFF

**Steps**:
1. Impostazioni → Sviluppo → Debug log → ON.
2. Riavvia app.

**Atteso**: file log scritti in path documentato. Viewer in-app accessibile.

---

### `TC-PREF-008` 🔵 🎯 Updater abilitato ON/OFF

**Steps**: Impostazioni → Sviluppo → Aggiornamenti → toggle.

**Atteso**: ON mostra pulsante "Verifica aggiornamenti", OFF lo nasconde.

---

## MCP — MCP server

### `TC-MCP-001` 🟡 🎯 Avvio MCP server con vault unencrypted

**Setup**: vault non cifrato (development) o vault produzione (in v1.0 MCP supporta solo unencrypted — vedi `docs/utente/mcp.md`).

**Steps**:
1. Configura Claude Desktop con `pap-mcp-server` come MCP server.
2. Riavvia Claude Desktop.

**Atteso**: server si avvia. Tool `pap_search`, `pap_get`, `pap_list_recent`, `pap_render` disponibili.

---

### `TC-MCP-002` 🟡 🎯 `pap_search` ritorna risultati

**Setup**: MCP attivo, prompt esistenti.

**Steps**: in Claude, chiedi "Cerca prompt 'email' nel mio vault PaP".

**Atteso**: lista di prompt con campi serializzati.

---

### `TC-MCP-003` 🟡 🎯 `pap_render` compila prompt

**Steps**: chiedi a Claude di compilare un prompt PaP con valori specifici.

**Atteso**: testo compilato restituito.

---

### `TC-MCP-004` 🔵 ⚠️ Vault non trovato

**Setup**: `PAP_VAULT_PATH` punta a file inesistente.

**Steps**: avvia MCP.

**Atteso**: stderr "Vault non trovato: <path>". Exit code != 0.

---

## CLI — CLI `pap`

### `TC-CLI-001` 🔵 🎯 `pap search "termine"`

**Setup**: CLI installata (`pap` in PATH), vault esistente.

**Steps**: `pap search "email"`.

**Atteso**: tabella di risultati.

---

### `TC-CLI-002` 🔵 🎯 `pap get <id>`

**Steps**: `pap get prm-xxx`.

**Atteso**: dettaglio del prompt.

---

### `TC-CLI-003` 🔵 🎯 Output format `--json`

**Steps**: `pap search "email" --output json`.

**Atteso**: JSON valido.

---

### `TC-CLI-004` 🔵 🎯 Completion bash/zsh

**Steps**: `pap completion bash > /tmp/pap.bash && source /tmp/pap.bash && pap <TAB>`.

**Atteso**: completion attivo.

---

## UPDATER — Auto-update

### `TC-UPDATER-001` 🔴 🎯 Check on-demand mostra update disponibile

**Setup**: app v0.8.10 installata. Online.

**Steps**:
1. Impostazioni → Sviluppo → Aggiornamenti → "Verifica aggiornamenti".

**Atteso**: messaggio "v0.8.11 disponibile". Pulsante "Installa".

---

### `TC-UPDATER-002` 🔴 🎯 Download + install + restart

**Setup**: come sopra, update disponibile.

**Steps**:
1. Click "Installa".

**Atteso**: download del binario. Verifica firma Ed25519. Restart. Nuova versione attiva.

---

### `TC-UPDATER-003` 🔴 ⚠️ Signature mismatch refuse

**Setup**: modifica `latest.json` su mirror locale con signature manomessa (test environment).

**Steps**: configura app per puntare al mirror, prova update.

**Atteso**: app rifiuta l'update con errore "Firma non valida". Vedi `auto-update.md`.

---

### `TC-UPDATER-004` 🔴 ⚠️ Downgrade refuse

**Setup**: `latest.json` con `version: "0.7.0"` (più vecchio della corrente).

**Steps**: prova update.

**Atteso**: app rifiuta (no downgrade).

---

### `TC-UPDATER-005` 🟡 🎯 Disabilita updater

**Steps**: Impostazioni → Sviluppo → Aggiornamenti → OFF.

**Atteso**: pulsante "Verifica" sparisce. Nessun check al boot (mai eseguito by default).

---

### `TC-UPDATER-006` 🔵 ⚠️ Network error durante check

**Setup**: rete offline.

**Steps**: "Verifica aggiornamenti".

**Atteso**: messaggio "Errore di rete" amichevole. No crash.

---

## A11Y — Accessibilità

### `TC-A11Y-001` 🟡 ♿ Focus trap in modali

**Setup**: modale aperta.

**Steps**: Tab ripetuti per ciclare focus.

**Atteso**: focus resta dentro la modale (non esce mai). Shift+Tab cicla all'indietro.

---

### `TC-A11Y-002` 🟡 ♿ Esc chiude modali

**Atteso**: ogni modale risponde a Esc.

---

### `TC-A11Y-003` 🟡 ♿ Contrasto tema dark/light

**Steps**: usa strumento contrasto (DevTools Chromium o NVDA Inspect).

**Atteso**: testo body / sfondo ≥ 4.5:1 (WCAG AA). Bottoni ≥ 3:1.

---

### `TC-A11Y-004` 🔵 ♿ Navigazione tastiera Shell

**Steps**: solo tastiera, naviga sidebar → list pane → right rail con Tab.

**Atteso**: focus visibile, ordine logico, nessun trap.

---

### `TC-A11Y-005` 🔵 ♿ Screen reader (opzionale)

**Steps**: NVDA / VoiceOver / Orca attivo, naviga UI.

**Atteso**: labels significativi su pulsanti, struttura heading coerente.

---

## PERF — Performance

### `TC-PERF-001` 🟡 ⚡ Palette su 1000 prompt < 200ms

**Setup**: vault con ≥ 1000 prompt.

**Steps**: apri palette, digita una query.

**Atteso**: risultati appaiono entro 200ms (percepito istantaneo).

---

### `TC-PERF-002` 🟡 ⚡ Compilazione prompt < 100ms

**Setup**: prompt con 5 segnaposti, 2 import.

**Steps**: Ctrl+Enter in Compila.

**Atteso**: < 100ms dalla pressione al toast "Copiato".

---

### `TC-PERF-003` 🔵 ⚡ Avvio app < 2s (no semantic loading)

**Setup**: ricerca semantica OFF.

**Steps**: doppio-click app icon.

**Atteso**: Shell visibile entro 2s.

---

### `TC-PERF-004` 🔵 ⚡ Avvio app < 5s (con semantic loading)

**Setup**: ricerca semantica ON.

**Atteso**: Shell visibile entro 5s (modello caricato in background).

---

## DEPRECATED / Future

Test cases che facevano riferimento a feature non ancora implementate in v0.8.11:

- *(nessuno al momento — aggiornare quando appropriato)*

---

## Appendix — Smoke test "golden path" v1.0

Sequenza minima da eseguire prima di ogni release per validare il flusso utente principale:

1. `TC-ONBOARD-001` (creazione vault primo avvio)
2. `TC-PROMPT-001` (crea prompt)
3. `TC-SEARCH-001` (cerca per titolo)
4. `TC-COMPILE-001` (compila via palette)
5. `TC-HOTKEY-001` (hotkey globale)
6. `TC-IMPORT-001` (import semplice)
7. `TC-EXP-JSON-001` (export JSON)
8. `TC-UPDATER-001` (check update)

Tempo stimato: ~15 minuti. Se tutti i golden path passano, la release è candidabile.
