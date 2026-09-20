-- V018: aggiunge 'openrouter' al CHECK di ProviderConfig.Provider.
--
-- provider_ai.rs (OpenRouterProvider, PROVIDERS_VALIDI) e la UI
-- (modelli-provider.ts, PannelloProviderConfig) supportano OpenRouter come
-- provider di prima classe (#668), ma il CHECK definito in V016 elencava
-- solo ('anthropic','openai','ollama','openai-compat','gemini'): salvare un
-- provider 'openrouter' violava il vincolo.
--
-- SQLite non consente di alterare un CHECK esistente: si ricrea la tabella
-- con il vincolo corretto preservando i dati. Nessuna foreign key referenzia
-- ProviderConfig, quindi drop + rename è sicuro (come V016).

CREATE TABLE ProviderConfig_v18 (
    Provider       TEXT PRIMARY KEY
        CHECK (Provider IN ('anthropic','openai','ollama','openai-compat','gemini','openrouter')),
    ApiKey         TEXT,
    BaseUrl        TEXT,
    DefaultModel   TEXT,
    Abilitato      INTEGER NOT NULL DEFAULT 1,
    CreatedAt      TEXT NOT NULL,
    UpdatedAt      TEXT NOT NULL
);

INSERT INTO ProviderConfig_v18
    (Provider, ApiKey, BaseUrl, DefaultModel, Abilitato, CreatedAt, UpdatedAt)
SELECT Provider, ApiKey, BaseUrl, DefaultModel, Abilitato, CreatedAt, UpdatedAt
FROM ProviderConfig;

DROP TABLE ProviderConfig;
ALTER TABLE ProviderConfig_v18 RENAME TO ProviderConfig;
