-- V010: configurazione provider AI per il regression testing (Fase 4 Step 8f).
--
-- Una row per provider noto. `ApiKey` è il segreto (header
-- `x-api-key` per Anthropic, `Authorization: Bearer` per OpenAI). Vive
-- nel vault SQLite cifrato SQLCipher AES-256: nessuna doppia cifratura
-- applicativa, la protezione è quella del vault stesso.
--
-- `BaseUrl` permette override per provider OpenAI-compatibili (LM
-- Studio, vLLM) o per Ollama remoto.
--
-- `Abilitato = 0` rende il provider invisibile ai dropdown UI senza
-- doverlo cancellare (preserva la API key per riattivazione veloce).

CREATE TABLE IF NOT EXISTS ProviderConfig (
    Provider       TEXT PRIMARY KEY
        CHECK (Provider IN ('anthropic','openai','ollama','openai-compat')),
    ApiKey         TEXT,                    -- NULL per Ollama (no key)
    BaseUrl        TEXT,                    -- NULL = default ufficiale del provider
    DefaultModel   TEXT,                    -- ultimo modello scelto, suggerimento UI
    Abilitato      INTEGER NOT NULL DEFAULT 1,
    CreatedAt      TEXT NOT NULL,
    UpdatedAt      TEXT NOT NULL
);
