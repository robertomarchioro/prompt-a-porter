-- V005: tabella virtuale per embeddings vettoriali via sqlite-vec.
--
-- Compatibilità con SQLCipher verificata nello Spike 1 (vedi
-- docs/architettura/decisioni/sqlite-vec-sqlcipher.md): vec0 funziona dentro
-- DB cifrato, le pagine vec0 vengono cifrate insieme al resto del vault.
--
-- Modello di riferimento: paraphrase-multilingual-MiniLM-L12-v2 (384 dim,
-- f32 normalizzato L2). Vedi docs/architettura/decisioni/embedding-model.md.
--
-- Distance metric: L2 (default vec0). Per cosine similarity su embeddings
-- L2-normalized, L2 distance e cosine sono monotonicamente equivalenti per
-- il ranking (cos = 1 - L2²/2), quindi non serve metric esplicito.
--
-- Niente foreign key constraint: vec0 non supporta FK SQLite. La pulizia
-- a cancellazione prompt avviene application-side via DELETE esplicito
-- da rust (vedi modulo embeddings_store.rs).

CREATE VIRTUAL TABLE IF NOT EXISTS PromptsEmbeddings USING vec0(
    PromptId TEXT PRIMARY KEY,
    Embedding FLOAT[384]
);
