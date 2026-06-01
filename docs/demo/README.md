# Vault demo per screenshot

`demo-vault.json` è una libreria di esempio (15 prompt, 8 tag, 3 versioni storiche) pensata per popolare l'app prima di catturare gli screenshot del sito. È nel formato di export v1 (vedi [`../utente/formato-export-json.md`](../utente/formato-export-json.md)).

## Come usarlo

1. Crea (o usa) un vault **vuoto** dedicato alle demo.
2. **Impostazioni → Dati → Importa JSON** → seleziona `demo-vault.json` → modalità **`skip`**.
3. Otterrai una libreria con tag colorati, preferiti, conteggi d'uso e modelli target.

## Cosa mostra (già pronto dopo l'import)

- **Libreria popolata** con descrizioni, badge modello (`claude-sonnet`, `claude-opus`, `gpt-4`, …), preferiti e conteggi d'uso → ordinamenti "popolari"/"recenti".
- **Tag colorati** in sidebar e filtro per tag.
- **Viste**: alcuni prompt sono `workspace` (vista Team) e altri `private`.
- **Segnaposti `{{nome}}`** in quasi tutti i body → modale Compila.
- **Import fra prompt** `{{import "..."}}`: `Email cold outreach`, `Code review strutturata` e `Genera test unitari` importano i due prompt "Ruolo …". L'highlighting del token e l'anteprima risolta funzionano subito.
- **Cronologia**: `Email professionale parametrica` ha 3 versioni → tab Cronologia + diff.

## Setup manuale per screenshot completi (2 minuti)

L'import JSON v1 **non** ripristina cartelle e segnaposti globali (vivono fuori dallo schema di export). Per gli screenshot che li mostrano:

- **Cartelle**: crea ~5 cartelle e trascina i prompt:
  - `Scrittura` → email professionale, riscrivi tono, spiega concetto
  - `Sviluppo` → code review, commit, test unitari, ruolo engineer
  - `Marketing` → cold outreach, nomi prodotto, ruolo marketing
  - `Produttività` → summarize, brainstorm, note riunione
  - `Traduzione` → traduzione tecnica
- **Segnaposti globali** (Impostazioni → Segnaposti globali), usati da `Firma email standard` e `Email professionale parametrica`:
  - `autore` = `Mario Rossi`
  - `ruolo` = `Product Manager`
  - `azienda` = `Acme S.r.l.`
  - `email` = `mario.rossi@acme.example`

> **Varianti / fork**: non sono rappresentabili nell'export v1 (manca `ParentPromptId`/`ForkOfPromptId`). Per screenshot di varianti A/B, creale a mano da un prompt dopo l'import.

## Manutenzione

Il file è coperto dal test `import_export::test::demo_vault_importa_pulito`: deserializza come `ExportV1` v1 e si importa senza errori. Se cambi lo schema di export, il test rompe e va rigenerato il demo.

## Licenza dei contenuti

I prompt sono originali del progetto (riprendono le ricette in [`../utente/casi-uso/`](../utente/casi-uso/README.md)), quindi sotto la stessa licenza del repo. Non includono materiale di terze parti.
