# Blueprint — Linter personalizzabile (visibilità + tuning)

> Stato: **design, no codice**. Feature in due fasi rilasciabili indipendentemente.
> Fase 3 (regole custom utente) **congelata** per scelta esplicita — vedi §7.
> Origine: discussione 2026-06-14 su come rendere visibili/modificabili le
> regole di diagnosi (`linting.rs`) e permettere all'utente di gestirle.

## 1. Obiettivo

Oggi le 11 regole del linter sono funzioni Rust hardcoded in
`apps/client/src-tauri/src/linting.rs`, con costanti compile-time e nessuna
superficie utente oltre al toggle per-categoria (che esiste in
`PannelloLinter.svelte` ma **non è montato** in Impostazioni).

Vogliamo:

- **Fase 1 — Visibilità**: catalogo in-app delle regole (cosa controlla
  ciascuna, severità, categoria) come singola fonte di verità dal backend, e
  toggle a granularità **singola regola** (non solo categoria).
- **Fase 2 — Tuning**: override di severità (es. declassare un `Error` a
  `Warning`) e soglie editabili (`LEN_MAX_BODY`, `LEN_MIN_BODY`,
  `NGRAM_THRESHOLD`).

Vincoli trasversali: **tutto additivo**, default = comportamento attuale,
zero migrazioni distruttive, nessuna duplicazione delle descrizioni regola
fra Rust e TS.

## 2. Stato attuale (cosa riusiamo)

| Pezzo | File | Note |
| --- | --- | --- |
| Logica 11 regole | `src-tauri/src/linting.rs` | funzioni `regola_*` + `analizza()` / `analizza_completo()` |
| Comando lint | `linting.rs:567` `prompt_lint(body, prompt_id, categorie_disabilitate, state)` | unico caller è il frontend |
| Filtro categorie | `linting.rs:545` `filtra_categorie(issues, &[String])` | filtra per **prefisso alfabetico** del `code` |
| Costanti | `linting.rs:121-126` | `LEN_MAX_BODY=4000`, `LEN_MIN_BODY=30`, `NGRAM_SIZE=3`, `NGRAM_THRESHOLD=4` |
| Pref categorie (TS) | `src/lib/preferenze-linter.ts` | `localStorage` key `pap.linter.categorie_disabilitate`; etichette+descrizioni per categoria |
| UI toggle (non montata) | `src/lib/components/PannelloLinter.svelte` | card per categoria; esportata in `components/index.ts` |
| Tab diagnosi | `src/lib/components/DiagnosiTab.svelte` | invoca `prompt_lint`, debounce 400ms, passa `categorieDisabilitate` |
| Impostazioni | `src/lib/superfici/ImpostazioniModal.svelte` | `subSezioni[]` (riga 331); render accordion per `sub.id` |
| Persistenza globale backend | `src-tauri/src/preferenze.rs` | `Preferenze` struct + `preferenze.json` in data-dir + comandi carica/salva |

**Decisione persistenza**: il modello attuale è *frontend-owns-config →
passa al backend come parametro di `prompt_lint` per chiamata*. Lo
manteniamo: nessuna nuova persistenza backend, la config vive in
`localStorage` e viaggia come parametro. Quando atterrerà *vault-a-cartella*
(`docs/roadmap/vault-a-cartella.md`, ancora design), il blob `localStorage`
migrerà in `.pap/linter.json` per seguire il vault — ma **non** è
prerequisito di questa feature (YAGNI).

## 3. Decisioni di design

1. **Catalogo = singola fonte di verità nel Rust.** Le descrizioni regola
   NON vanno duplicate in TS. Nuovo comando `prompt_lint_regole()` che
   ritorna i metadati. Le mappe `ETICHETTE`/`DESCRIZIONI` in
   `preferenze-linter.ts` vengono **deprecate** a favore dei dati dal
   backend (restano per le 5 *categorie*, non per le regole).
2. **Granularità per-regola riusa il filtro esistente.** Generalizziamo
   `filtra_categorie`: un token disabilitato matcha un issue se è uguale al
   `code` completo (`"PII001"`) **oppure** al prefisso alfabetico
   (`"PII"`). Retrocompatibile: `["PII"]` continua a disabilitare la
   famiglia.
3. **Config come un solo oggetto.** Fase 2 sostituisce il parametro
   `categorie_disabilitate: Option<Vec<String>>` con
   `config: Option<ConfigLinter>` (superset). Il caller è solo il frontend →
   nessun consumatore esterno rotto.
4. **Override severità in post-pass.** Non tocchiamo le singole `regola_*`:
   applichiamo gli override in una passata finale su `Vec<Issue>`.
5. **Soglie con default = costanti attuali.** `SoglieLinter::default()`
   riproduce 4000/30/4 → assenza di config ⇒ comportamento identico a oggi
   (regression-safe).

## 4. Fase 1 — Visibilità + toggle per-regola

### 4.1 Backend (`linting.rs`)

Nuovo tipo metadati + registro statico:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RegolaMeta {
    pub code: &'static str,          // "LEN001"
    pub categoria: &'static str,     // "LEN"
    pub severita_default: Severita,  // riusa enum esistente
    pub titolo: &'static str,        // breve, una riga
    pub descrizione: &'static str,   // cosa controlla / perché
    pub configurabile: bool,         // ha soglie tunabili (Fase 2)
}

/// Catalogo completo. Unica fonte di verità per UI e doc.
pub fn regole_catalogo() -> Vec<RegolaMeta> {
    vec![
        RegolaMeta { code: "LEN001", categoria: "LEN", severita_default: Severita::Warning,
            titolo: "Body troppo lungo",
            descrizione: "Avvisa se il body supera la soglia massima (spreco di token).",
            configurabile: true },
        RegolaMeta { code: "LEN002", categoria: "LEN", severita_default: Severita::Info,
            titolo: "Body troppo corto",
            descrizione: "Segnala body sotto la soglia minima: probabilmente incompleto.",
            configurabile: true },
        // PH001, PH003, PII001, PII003, PII004, STY001, IMP001, IMP002, IMP003, IMP004 …
    ]
}

#[tauri::command]
pub fn prompt_lint_regole() -> Vec<RegolaMeta> {
    regole_catalogo()
}
```

Generalizzazione del filtro (rinominato per chiarezza, vecchia firma
mantenuta come wrapper se serve compat interna):

```rust
/// Un token disabilita un issue se uguale al code completo O al prefisso
/// alfabetico. `["PII"]` → famiglia; `["PII001"]` → singola regola.
pub(crate) fn filtra_disabilitate(issues: Vec<Issue>, disabilitate: &[String]) -> Vec<Issue> {
    if disabilitate.is_empty() { return issues; }
    let set: HashSet<&str> = disabilitate.iter().map(String::as_str).collect();
    issues.into_iter().filter(|i| {
        let prefisso: String = i.code.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
        !set.contains(i.code) && !set.contains(prefisso.as_str())
    }).collect()
}
```

Registrare il nuovo comando in `lib.rs` (`invoke_handler`). **Coerenza
test**: aggiungere un test che verifica `regole_catalogo().len()` == numero
regole effettive e che ogni `code` sia unico e abbia un `categoria` valido
(LEN/PH/PII/STY/IMP).

### 4.2 Frontend

**`preferenze-linter.ts`** — estendere a per-regola mantenendo retrocompat:

- Nuova key `pap.linter.regole_disabilitate` (lista di code/prefissi).
- Migrazione one-shot: se esiste la vecchia key
  `pap.linter.categorie_disabilitate` e la nuova no, copiala e mantienila
  (lettura difensiva come l'attuale).
- `leggiRegoleDisabilitate(): string[]`, `salvaRegoleDisabilitate(string[])`,
  `toggleRegola(code, lista)` (immutabile, come l'attuale `toggleCategoria`).
- Tenere `CATEGORIE_LINTER` + etichette categoria (servono per il
  raggruppamento UI); le descrizioni *per regola* arrivano dal backend.

**`PannelloLinter.svelte`** — rifacimento del corpo (la shell resta):

- `onMount`: `const catalogo = await invoke<RegolaMeta[]>("prompt_lint_regole")`
  (cache in modulo: è pura, non cambia a runtime).
- Render raggruppato per `categoria` (header categoria con toggle "tutta la
  famiglia" = comportamento attuale) → sotto, una riga per regola con
  `titolo`, `code`, `descrizione`, `severita_default` (badge) e `Switch`
  per-regola.
- Toggle famiglia = aggiunge/rimuove il prefisso; toggle regola = aggiunge/
  rimuove il code. Un issue è nascosto se famiglia O regola disabilitata
  (stessa semantica del backend).

**`ImpostazioniModal.svelte`** — montare la sotto-sezione:

- Aggiungere a `subSezioni[]` (riga 331) una voce `{ id: "linter", label:
  "Linter", … }` sotto il cluster "avanzate" (vicino a `globali`).
- Nel blocco accordion (intorno a riga 2105, `sub.id === "globali"`)
  aggiungere `{:else if sub.id === "linter"}<PannelloLinter />`.
- Icona lucide coerente (es. `ShieldCheck` o `ListChecks`) nella mappa icone
  (riga ~1878).

**`DiagnosiTab.svelte`** — sostituire `leggiCategorieDisabilitate()` con
`leggiRegoleDisabilitate()` nel payload `categorieDisabilitate` (il nome del
parametro backend resta finché non arriva Fase 2; il valore ora può
contenere code completi).

### 4.3 Test Fase 1

- **Rust**: catalogo (conteggio/unicità/categorie valide);
  `filtra_disabilitate` con code esatto (disabilita solo `PII001`, lascia
  `PII003`), con prefisso (disabilita famiglia), retrocompat lista vuota.
- **TS (vitest)**: migrazione key vecchia→nuova; `toggleRegola` immutabile;
  lettura difensiva su JSON malformato.

## 5. Fase 2 — Tuning (severità + soglie)

### 5.1 Backend

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SoglieLinter {
    pub len_max_body: usize,    // default 4000
    pub len_min_body: usize,    // default 30
    pub ngram_threshold: usize, // default 4
}
impl Default for SoglieLinter {
    fn default() -> Self { Self { len_max_body: 4000, len_min_body: 30, ngram_threshold: 4 } }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ConfigLinter {
    pub disabilitate: Vec<String>,                  // code o prefissi (Fase 1)
    pub severita_override: HashMap<String, Severita>, // code → severità forzata
    pub soglie: SoglieLinter,
}
```

- `analizza(body)` → `analizza(body, &ConfigLinter)`; le `regola_len*` e
  `regola_sty001` leggono `config.soglie` invece delle `const`. Le costanti
  diventano i default di `SoglieLinter`.
- **Validazione soglie** (system boundary): clamp difensivo —
  `len_min_body < len_max_body` (altrimenti ignora min), `ngram_threshold >=
  2`, valori `usize` già non-negativi. Fallire piano: input invalido →
  default, non panico.
- **Override severità**: passata finale
  `applica_override(&mut Vec<Issue>, &HashMap)` che, per ogni issue con
  `code` presente nella mappa, sostituisce `severita`. Nessuna modifica alle
  `regola_*`.
- Comando aggiornato:

```rust
#[tauri::command]
pub fn prompt_lint(
    body: String,
    prompt_id: Option<String>,
    config: Option<ConfigLinter>,   // sostituisce categorie_disabilitate
    state: State<'_, VaultState>,
) -> Result<Vec<Issue>, PapErrore> {
    let cfg = config.unwrap_or_default();
    let mut out = analizza(&body, &cfg);
    // … regole_imp invariate …
    applica_override(&mut out, &cfg.severita_override);
    Ok(filtra_disabilitate(out, &cfg.disabilitate))
}
```

### 5.2 Frontend

- `preferenze-linter.ts`: nuovo blob unico `pap.linter.config` (JSON
  `ConfigLinter`-shaped) con migrazione dalle key Fase 1; helper
  `leggiConfig()/salvaConfig()`. `DiagnosiTab` passa `config` invece di
  `categorieDisabilitate`.
- `PannelloLinter.svelte`: per ogni regola `configurabile`, accanto al
  toggle un controllo severità (select Error/Warning/Info, default =
  `severita_default`) e, per le regole con soglia (LEN001/LEN002/STY001), un
  input numerico. Pulsante "Ripristina default" per regola e globale.

### 5.3 Backward compat / regression

- `ConfigLinter::default()` + `SoglieLinter::default()` = identico a oggi.
- Test regressione: `analizza(body, &ConfigLinter::default())` produce gli
  stessi issue dei test attuali (riusare gli assert esistenti).
- Test tuning: `len_max_body` abbassato fa scattare LEN001 su body corto;
  `severita_override{"PII001":"Error"}` cambia la severità; soglie invalide
  → clamp.

## 6. Edge case da coprire

- Override su `code` inesistente → no-op silenzioso.
- Disabilitare famiglia E settare override sulla stessa regola → l'issue è
  filtrato comunque (filtro dopo override: ordine corretto già nel comando).
- `severita_override` deserializza solo valori validi dell'enum (serde
  fallisce su stringhe ignote → l'intero `config` invalido va gestito:
  frontend valida prima di salvare; backend usa default su errore di parse a
  livello comando? No — Tauri rifiuta il comando; quindi il **frontend deve
  garantire un payload valido**).
- `ngram_threshold = 1` degenerebbe (ogni 3-gram conta) → clamp a ≥2.
- IMP004/IMP002 ecc. con override severità: ammesso (utente può alzare
  IMP004 da Info a Warning) — nessun caso speciale.
- Catalogo e regole effettive devono restare allineati: il test di conteggio
  in §4.3 è il guard-rail anti-drift.

## 7. Fuori scope (Fase 3 — congelata)

Regole **custom definite dall'utente** (regex dichiarative o DSL) NON sono
incluse. Scelta del 2026-06-14: si decide il tetto di espressività più
avanti. Nessun motore generico, nessuna persistenza di regole-come-dato,
nessuna esecuzione di pattern forniti dall'utente in questo lavoro.

## 8. Breakdown PR proposto

| PR | Scope | Rischio |
| --- | --- | --- |
| **PR-1** | Backend Fase 1: `RegolaMeta` + `regole_catalogo` + comando `prompt_lint_regole` + generalizzazione filtro per-regola + test | basso |
| **PR-2** | Frontend Fase 1: `preferenze-linter` per-regola + migrazione + rifacimento `PannelloLinter` (fetch catalogo) + mount in `ImpostazioniModal` + `DiagnosiTab` | basso-medio |
| **PR-3** | Backend Fase 2: `ConfigLinter`/`SoglieLinter` + threading soglie + override severità + comando aggiornato + test regressione/tuning | medio |
| **PR-4** | Frontend Fase 2: blob `config` + UI severità/soglie + migrazione + test | medio |

Ogni PR è verde-su-CI e mergeabile da sola. Fase 1 (PR-1+2) consegna già
valore visibile; Fase 2 (PR-3+4) si può rinviare senza debito.

## 9. Doc utente da aggiornare

- `docs/utente/linting-regole.md`: aggiungere sezione "Personalizzare il
  linter" (dove trovarlo in Impostazioni, toggle per-regola, in Fase 2
  severità/soglie). Il catalogo resta descritto lì, ma la **fonte
  autorevole** dei metadati diventa `regole_catalogo()` nel backend.
