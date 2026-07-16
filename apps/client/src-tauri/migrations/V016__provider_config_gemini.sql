-- V016: aggiunge 'gemini' al CHECK di ProviderConfig.Provider.
--
-- provider_ai.rs (GeminiProvider, PROVIDERS_VALIDI) e la UI
-- (modelli-provider.ts, PannelloProviderConfig) supportano Gemini, ma il
-- CHECK definito in V010 elencava solo ('anthropic','openai','ollama',
-- 'openai-compat'): salvare un provider 'gemini' violava il vincolo.
--
-- SQLite non consente di alterare un CHECK esistente: si ricrea la tabella
-- con il vincolo corretto preservando i dati. Nessuna foreign key referenzia
-- ProviderConfig, quindi drop + rename è sicuro.

CREATE TABLE ProviderConfig_v16 (
    Provider       TEXT PRIMARY KEY
        CHECK (Provider IN ('anthropic','openai','ollama','openai-compat','gemini')),
    ApiKey         TEXT,
    BaseUrl        TEXT,
    DefaultModel   TEXT,
    Abilitato      INTEGER NOT NULL DEFAULT 1,
    CreatedAt      TEXT NOT NULL,
    UpdatedAt      TEXT NOT NULL
);

INSERT INTO ProviderConfig_v16
    (Provider, ApiKey, BaseUrl, DefaultModel, Abilitato, CreatedAt, UpdatedAt)
SELECT Provider, ApiKey, BaseUrl, DefaultModel, Abilitato, CreatedAt, UpdatedAt
FROM ProviderConfig;

DROP TABLE ProviderConfig;
ALTER TABLE ProviderConfig_v16 RENAME TO ProviderConfig;
