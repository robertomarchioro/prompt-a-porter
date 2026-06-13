# Vault demo per screenshot

`demo-vault.json` è una libreria di esempio (17 prompt — di cui 1 variante e 1 fork —, 7 cartelle, 8 tag, 3 versioni storiche, 4 segnaposti globali) pensata per popolare l'app prima di catturare gli screenshot del sito. È nel formato di export v1 (vedi [`../utente/formato-export-json.md`](../utente/formato-export-json.md)).

## Come usarlo

1. Crea (o usa) un vault **vuoto** dedicato alle demo.
2. **Impostazioni → Dati → Importa JSON** → seleziona `demo-vault.json` → modalità **`skip`**.
3. Otterrai una libreria completa: tag colorati, preferiti, conteggi d'uso, modelli target e segnaposti globali già impostati.

## Cosa mostra (già pronto dopo l'import)

- **Cartelle** già organizzate (con un livello di nesting `Scrittura/Email`): `Scrittura`, `Sviluppo`, `Marketing`, `Produttività`, `Traduzione`, `Ruoli`. I prompt sono già nelle cartelle giuste.
- **Libreria popolata** con descrizioni, badge modello (`claude-sonnet`, `claude-opus`, `gpt-4`, …), preferiti e conteggi d'uso → ordinamenti "popolari"/"recenti".
- **Tag colorati** in sidebar e filtro per tag.
- **Viste**: alcuni prompt sono `workspace` (vista Team) e altri `private`.
- **Segnaposti `{{nome}}`** in quasi tutti i body → modale Compila.
- **Import fra prompt** `{{import "..."}}`: `Email cold outreach`, `Code review strutturata` e `Genera test unitari` importano i due prompt "Ruolo …". L'highlighting del token e l'anteprima risolta funzionano subito.
- **Cronologia**: `Email professionale parametrica` ha 3 versioni → tab Cronologia + diff.
- **Varianti e fork**: `Email professionale parametrica` ha una **variante B**; `Code review strutturata` ha un **fork** ("Code review strutturata (fork)", adattato a Rust). Le relazioni sono ricreate automaticamente dall'import.
- **Segnaposti globali** (Impostazioni → Segnaposti globali): `autore`, `ruolo`, `azienda`, `email` seminati automaticamente dall'import. Visibili subito nel prompt `Firma email standard` e in `Email professionale parametrica`.

## Manutenzione

Il file è coperto dai test `import_export::test::demo_vault_importa_pulito` e `import_export::test::demo_vault_semina_global_placeholders`: deserializza come `ExportV1` v1 e si importa senza errori. Se cambi lo schema di export, i test rompono e va rigenerato il demo.

## Licenza dei contenuti

I prompt sono originali del progetto (riprendono le ricette in [`../utente/casi-uso/`](../utente/casi-uso/README.md)), quindi sotto la stessa licenza del repo. Non includono materiale di terze parti.
