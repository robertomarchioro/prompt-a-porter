# Todo Fase 3 — Intelligenza

> **Obiettivo**: rendere il vault navigabile in modo "umano" — l'utente cerca per significato, non per parole esatte. Aggiungere intelligenza assistiva sui prompt (qualità, target, tag) senza creare lock-in cloud: tutto deve girare **client-side** o sul server self-hosted.
>
> **Deliverable finale**: tag release `v0.3.0-fase3`.

---

## Filosofia di Fase

> "Pegaso è bellissimo da vedere, ma vola solo se Bellerofonte sa cavalcarlo."

L'AI in PaP è uno strumento di scoperta e qualità, non di sostituzione. Ogni feature in questa fase deve:

1. **Funzionare offline** sul client desktop (zero dipendenze cloud)
2. Essere **opt-in** (utente attiva esplicitamente in Impostazioni)
3. Mantenere **dato in chiaro** locale, embeddings server-side opzionali
4. Avere **degradazione graziosa**: se l'embedding model non è disponibile, ricerca FTS5 continua a funzionare

---

## Prerequisiti (Step 0)

- [ ] Fase 2 chiusa: `v0.2.0-fase2` taggata, web app + extension funzionanti
- [ ] Modello dati supporta `TargetModel` (anticipato in Fase 1 o Fase 2)
- [ ] Decidere strategia embeddings: **client-side puro** (consigliato) o **server-side opzionale** (per workspace team)
- [ ] Crea branch `fase-3` da `main`

---

## Scope Feature Fase 3

| # | Feature | Modulo |
|---|---------|--------|
| 1 | Ricerca semantica via embeddings | client (server opzionale) |
| 2 | Ricerca ibrida FTS5 + vettoriale | client |
| 3 | Auto-suggerimento tag su nuovo prompt | client |
| 4 | Linting prompt (lunghezza, segnaposti, PII) | client |
| 5 | Modello-target dichiarato + filtri | client + server |
| 6 | Statistiche di qualità prompt | client |

---

## Step 1 — Setup ONNX Runtime + modello embeddings

- [ ] Aggiungi crate `ort` (ONNX Runtime Rust binding) al client desktop
- [ ] Modello scelto: **`bge-small-en-v1.5`** (~33 MB, 384 dim, multilingue accettabile) o **`all-MiniLM-L6-v2`** (~80 MB, 384 dim, classico)
- [ ] **Bundling vs download**:
  - Bundle nel binario: +30-80 MB. Pro: zero setup. Contro: ogni upgrade modello richiede re-release.
  - Download al primo uso: scarica da repo HuggingFace o mirror Anthropic-friendly. Pro: binario snello. Contro: serve connessione iniziale.
  - **Decisione consigliata**: download on-demand con cache in `~/.local/share/prompt-a-porter/models/`
- [ ] Comando Tauri `embeddings_init()` → carica modello in memoria (lazy, primo uso)
- [ ] Comando Tauri `embeddings_compute(text: String)` → returns `Vec<f32>` (384 dim)
- [ ] Performance target: < 50ms per embedding di un prompt medio (1000 token) su CPU moderna
- [ ] Threading: ONNX Runtime su thread dedicato, non blocca main loop
- [ ] Test: 1000 embeddings in batch < 30s su laptop standard

## Step 2 — sqlite-vec integration

- [ ] Aggiungi estensione `sqlite-vec` al setup SQLite client (carica come dynamic library al boot del vault)
- [ ] Migration v3 con nuova tabella virtuale:
  ```sql
  CREATE VIRTUAL TABLE PromptsEmbeddings USING vec0(
      PromptId TEXT PRIMARY KEY,
      Embedding FLOAT[384]
  );
  ```
- [ ] Hook su create/update prompt → ricalcola embedding del concat `Title + Description + Body` (debounced 2s per evitare ricalcolo durante typing)
- [ ] Job di backfill: al primo unlock con feature attivata, calcola embeddings per tutti i prompt esistenti (con progress bar in UI)
- [ ] Storage size estimation: 384 float32 = 1.5 KB per prompt → trascurabile su 10k prompt (15 MB)
- [ ] Hook su delete/tombstone → rimozione corrispondente da `PromptsEmbeddings`

## Step 3 — Ricerca ibrida (semantica + FTS5)

- [ ] Algoritmo: **Reciprocal Rank Fusion (RRF)** combina top-K da FTS5 e top-K da sqlite-vec
- [ ] Comando Tauri `search_hybrid(query, limit, alpha)` dove `alpha` ∈ [0,1] pesa semantico vs lessicale
- [ ] Default `alpha = 0.5` (bilanciato), configurabile in Impostazioni > Ricerca (slider con preset Lessicale/Bilanciato/Semantico)
- [ ] **Fallback automatico**: se embedding model non caricato → solo FTS5 senza errore
- [ ] **UI**: nessun cambio visibile per l'utente nella libreria. La ricerca è la stessa, ma trova "fratelli concettuali"
- [ ] **UI Command Palette**: badge "🧠 ricerca semantica attiva" piccolino se la query usa embeddings
- [ ] Highlight nei risultati: per FTS5 match testuale, per semantico nessun highlight (non c'è una keyword da evidenziare)
- [ ] Test qualitativi: dataset di 50 prompt, query "riscrivi email in tono formale" deve trovare anche prompt "trasforma messaggio in business style"

## Step 4 — Auto-suggerimento tag

Quando l'utente crea un nuovo prompt, suggerire tag pertinenti.

- [ ] Approccio: per ogni tag esistente nel workspace, calcola embedding del nome tag + descrizione (se presente). Confronta con embedding del nuovo prompt. Top-K per cosine similarity.
- [ ] Comando Tauri `tags_suggest(promptId or text)` → lista `[{tag, score}]` ordinata
- [ ] **UI nell'Editor**: sotto al tag picker, sezione "Tag suggeriti" con chip cliccabili (top 5)
- [ ] Fallback: se < 10 tag esistenti nel workspace, ritorna tag più frequenti (no embedding necessario)
- [ ] Soglia minima cosine similarity per suggerire: 0.55 (configurabile)
- [ ] **UX**: i suggerimenti non sono autoritativi, sono suggerimenti. L'utente deve poter ignorare facilmente.

## Step 5 — Linting prompt

Avvisi proattivi che aiutano a scrivere prompt migliori, senza essere paternalistici.

**Categorie di lint**:

| Codice | Descrizione | Severità |
|--------|-------------|----------|
| `LEN001` | Body troppo lungo (> 4000 char) — possibile spreco token | warning |
| `LEN002` | Body troppo corto (< 30 char) — probabilmente incompleto | info |
| `PH001` | Segnaposto malformato (`{nome}` invece di `{{nome}}`) | error |
| `PH002` | Segnaposto definito ma non usato nel body | warning |
| `PH003` | Nome segnaposto contiene caratteri speciali | warning |
| `PII001` | Possibile email rilevata nel body (regex) | warning |
| `PII002` | Possibile codice fiscale/SSN rilevato | warning |
| `PII003` | Possibile numero di carta di credito (Luhn check) | error |
| `PII004` | Possibile API key o token (pattern OpenAI, Anthropic, AWS, GitHub) | error |
| `STY001` | Body con molte ripetizioni (n-gram analysis) | info |
| `STY002` | Mancanza di istruzioni chiare (manca verbo imperativo nelle prime righe) | info |

- [ ] Modulo `lib_lint/` in client con regole pluggable
- [ ] Comando Tauri `prompt_lint(body, segnaposti)` → lista issues `[{code, severity, message, line, col}]`
- [ ] **UI**: pannello "Diagnosi" nell'editor (collapsible, default chiuso se zero issues)
- [ ] Inline marker in CodeMirror 6 (decoration) sui punti incriminati
- [ ] Severità error blocca il salvataggio (override possibile con conferma esplicita)
- [ ] Configurabile in Impostazioni > Linting: abilita/disabilita per categoria
- [ ] Test: dataset di prompt buoni e cattivi, verifica precision/recall delle regole PII

## Step 6 — Modello-target dichiarato

Ogni prompt può dichiarare per quale modello AI è ottimizzato.

- [ ] Migration: aggiunto `TargetModel` in Fase 1 — verificare che sia popolato correttamente
- [ ] Valori predefiniti: `claude-opus`, `claude-sonnet`, `claude-haiku`, `gpt-4`, `gpt-4-mini`, `gemini-pro`, `gemini-flash`, `llama-3`, `generic`
- [ ] **UI Editor**: dropdown nella sidebar destra con preset + opzione "Custom"
- [ ] **UI Libreria**: filtro nella sidebar "Per modello target" (count per modello)
- [ ] **UI Renderer**: badge che mostra il modello target del prompt corrente, warning se l'utente sta per copiarlo per un modello diverso (eventually)
- [ ] **Server**: filtro `?target=claude-opus` su endpoint search

## Step 7 — Statistiche qualità prompt

Aggregazione passiva dei dati d'uso (già raccolti da `UseCount`, `LastUsedAt`).

- [ ] Vista "Insight" in Libreria (icona 📊 nella sidebar)
- [ ] **Top prompt usati** ultimi 30 giorni
- [ ] **Prompt non usati** da > 90 giorni (candidati a cleanup)
- [ ] **Prompt più commentati**
- [ ] **Distribuzione per tag**, **per modello target**, **per autore** (per workspace team)
- [ ] **Lint health**: % prompt senza issues, top 10 categorie issue più frequenti
- [ ] Charts: usa SVG inline + librerie minimal (es. `chart.js` solo se proprio necessario, preferire SVG custom)
- [ ] **Privacy**: nessun dato esce dal vault. Statistiche aggregate locali.

## Step 8 — Quality gate Fase 3

- [ ] Test coverage ≥ 70% su moduli `lib_lint`, `lib_embeddings`, ricerca ibrida
- [ ] Benchmark: ricerca su 10k prompt < 100ms (P95)
- [ ] Memory profiling: embedding model unload dopo 5 min idle (configurabile)
- [ ] Test su workspace con dataset realistico (1000+ prompt, generabile con script)
- [ ] Verifica grace degradation: app funzionante anche senza modello scaricato (toggle off temporaneo)

## Step 9 — Documentazione e release

- [ ] Nuovo `docs/ricerca-semantica.md` con razionali, modello scelto, performance attese
- [ ] Nuovo `docs/linting-regole.md` con catalogo completo regole
- [ ] Aggiorna `docs/schema-dati.md` con `PromptsEmbeddings`
- [ ] Aggiorna `docs/architettura.md` con flusso embeddings
- [ ] Changelog v0.3.0
- [ ] Crea `docs/todo-fase-4.md`
- [ ] Tag `v0.3.0-fase3`

---

## Decisioni discrezionali

1. **Modello embedding**: `bge-small-en-v1.5` (33 MB, multilingue passabile) o **`bge-small-it`** (italiano-specifico se esiste alternativa equivalente) o **`paraphrase-multilingual-MiniLM-L12-v2`** (118 MB, multilingue forte). Considera che l'utenza è italofona ma i prompt sono spesso in inglese.
2. **Cache embeddings server-side per workspace team**: il server ricalcola gli embedding per condividerli? Pro: zero ricalcolo per ogni client. Contro: il server vede testo prompt in chiaro (necessario per il calcolo). Trade-off di privacy.
3. **Linting PII è block-by-default o warn-only?** Per workspace ad alta sensibilità (Fase 5 E2E) sarà block; per ora warn-by-default sembra ragionevole.

---

## Riferimenti

- Fase 2 (precedente): `docs/todo-fase-2.md`
- Fase 4 (prossima): `docs/todo-fase-4.md`
- Modelli embedding: https://huggingface.co/BAAI/bge-small-en-v1.5
- sqlite-vec: https://github.com/asg017/sqlite-vec
