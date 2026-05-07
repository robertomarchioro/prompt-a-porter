# Regression testing dei prompt

> Disponibile da `v0.4.0`. **Differenziatore strategico**: nessun
> altro prompt manager esistente offre questa funzione.

I **golden examples** trasformano un prompt da testo a **contratto
comportamentale verificabile**: dato un certo input, ti aspetti un
certo output (con tolleranza). PaP misura nel tempo se il prompt
produce ancora output coerente, soprattutto al cambio del modello AI
sottostante (Claude 4.6 → 4.7, GPT-5 → GPT-6, ecc.).

## Concetti

| Termine | Cosa è |
|---|---|
| **Golden example** | Caso di test salvato sul prompt: `input_vars` JSON + `expected_output` + `similarity_fn` + soglia |
| **Run / Observation** | Esecuzione: chiama il provider, ottiene `actual_output`, calcola similarità vs expected, salva tutto |
| **Provider AI** | Backend che genera l'`actual_output`: Ollama locale, Anthropic, OpenAI, OpenAI-compat |
| **Drift** | Differenza tra similarità media nel periodo e similarità dell'ultima run |

## Setup provider

Prima di eseguire golden, configura almeno un provider in
**Impostazioni** → **Provider AI** (pannello dedicato disponibile da
`v0.5.0`; su `v0.4.x` l'inserimento avveniva via SQL diretto sulla
tabella `ProviderConfig`):

| Provider | Cosa serve | Costo |
|---|---|---|
| **Ollama** | URL `http://localhost:11434` (default) — installa Ollama + scarica un modello (`ollama pull llama3.2`) | 0 € locale |
| **Anthropic** | API key `sk-ant-…` da console.anthropic.com | per token |
| **OpenAI** | API key `sk-…` da platform.openai.com | per token |
| **OpenAI-compat** | URL custom (LM Studio, vLLM) + key generica | varia |
| **Google (Gemini)** | API key da `aistudio.google.com/apikey`; modelli `gemini-2.5-flash` / `gemini-2.5-pro` | per token |

> **Sicurezza**: le API key vivono in plaintext nel DB cifrato
> SQLCipher AES-256. La protezione è quella del vault — niente doppia
> cifratura applicativa. Il command `provider_config_lista` non rinvia
> mai la chiave al frontend.

## Crea un golden

Apri un prompt in **Editor** → tab **Test** (accanto a Cronologia /
Diagnosi). Click **+ Aggiungi golden**:

1. **Etichetta**: nome leggibile (es. `caso comune`, `edge case lungo`).
2. **Input vars** JSON: oggetto con i valori dei segnaposti. Es.
   ```json
   {"contesto":"reclamo cliente", "tono":"formale"}
   ```
   Le `{{var}}` nel body verranno sostituite con questi valori prima
   della chiamata al provider.
3. **Expected output**: il testo che ti aspetti.
4. **Similarity function** (vedi sotto): `cosine` (default), `exact-match`, `regex`, `llm-judge`.
5. **Soglia tolleranza** ∈ [0, 1]: usata da `cosine`/`llm-judge`.

## Similarity functions

| Funzione | Costo | Quando usarla |
|---|---|---|
| **`cosine`** | ~50 ms (riusa MiniLM Fase 3) | Default. Confronto semantico tollerante a parafrasi |
| **`exact-match`** | gratis | Output strutturati (JSON strict). 1.0 se identico, 0.0 altrimenti |
| **`regex`** | gratis | L'`expected` è una regex. Pass se matcha |
| **`llm-judge`** | ~1-3 s + costo provider | Sfumato: meta-prompt al modello giudice "Punteggio 0-1 di aderenza" |

`llm-judge` richiede un secondo provider come "giudice": può essere
lo stesso del run principale (cheap) o un provider più capace per
ridurre il bias del giudice.

## Esegui un golden

Click **Esegui** sul golden. Il client:

1. Compila il body con `input_vars` (sostituisce `{{var}}`).
2. Chiama il provider scelto col modello indicato.
3. Calcola `similarità` tra `expected` e `actual` con la funzione del
   golden.
4. Salva una `Observation` con: `Similarità`, `Passed` (`>= soglia`),
   `LatenzaMs`, `TokensUsed`, `Provider`, `Model`, `RanAt`.

L'icona di stato accanto al golden cambia: ○ idle → … running → ✓ ok
/ ✗ ko. Click su "Output ricevuto" espande il pannello con il testo
prodotto.

Errori del provider (HTTP, rate limit, modello sconosciuto) sono
catturati: l'observation viene comunque salvata con `Errore`
valorizzato e `Passed = false`. Errori DB invece propagano (es.
golden inesistente).

## Vista "Regressioni" in Libreria

Sidebar Libreria → **Regressioni**. Mostra tabella drift aggregata
per (prompt × provider × model) negli ultimi N giorni (filtro
periodo dropdown 7/30/90/365):

| Prompt | Provider · Model | Run | Pass | Fail | Sim. media | Sim. ultima | Drift | Ultima |
|---|---|---|---|---|---|---|---|---|
| ... | anthropic · claude-sonnet-4.6 | 12 | 11 | 1 | 0.892 | 0.853 | +4.4% | … |

- **Drift positivo** = peggioramento (la similarità ultima è scesa).
- **Drift negativo** = miglioramento.
- Border-left colorato per stato (✓ tutti pass / ⚠ misto / ✗ tutti
  fail).

Click **Esporta CSV** per scaricare il report (RFC 4180, escape
virgole/quote/newline).

## Schema dati

Vedi `docs/architettura/schema-dati.md` § V008-V010. Riassunto:

- **`PromptGoldens`** (V008): definizione del test
- **`PromptRunObservations`** (V009): storia delle run, indicizzata
  per `(PromptVersionId, RanAt DESC)` e `(Provider, Model, Passed)`
- **`ProviderConfig`** (V010): API key + base URL per provider

## Comandi Tauri esposti

| Comando | Cosa fa |
|---|---|
| `golden_crea/aggiorna/elimina/lista(prompt_id)` | CRUD golden |
| `golden_esegui(golden_id, provider_kind, model, base_url?, judge_provider?, judge_model?)` | Run end-to-end |
| `regression_report(giorni)` | Aggregato drift per (prompt × provider × model) |
| `regression_report_csv(giorni)` | Stesso in CSV per export |
| `provider_config_lista/salva/elimina` | Gestione provider (API key NON esposta nella lista) |
| `provider_ollama_genera(prompt, model, base_url?)` | Smoke test diretto |

## Limiti noti / roadmap

- ✅ **"Esegui tutti i golden" batch** atterrato in `v0.5.0`:
  bottone "Esegui tutti (N)" nel pannello Test dell'Editor.
  Esecuzione sequenziale (no parallel per rate limit), progress
  inline `Esecuzione X/Y…`, summary finale `✓ N passed · ✗ M failed`.
- **CLI integration** `pap test <promptId>` per CI/CD —
  manca subcommand in `apps/cli`.
- **MCP integration** `pap_test_prompt` come tool per agenti —
  Fase 5 con MCP HTTP/SSE.
- ✅ **Provider Google (Gemini)** atterrato in `v0.5.0`: 5/5
  provider pianificati ora implementati (Anthropic, OpenAI,
  OpenAI-compat, Ollama, Gemini).
- ✅ **UI Impostazioni Provider** — atterrato in `v0.5.0`
  (pannello dedicato in **Impostazioni** → **Provider AI**).

## Riferimenti

- Implementazione: `apps/client/src-tauri/src/{regression,provider_ai,similarity}.rs`
- Schema: `docs/architettura/schema-dati.md` § V008-V010
- Spec roadmap: `docs/roadmap/fase-4-workflow.md` Step 8
