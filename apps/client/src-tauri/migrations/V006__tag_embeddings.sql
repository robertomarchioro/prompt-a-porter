-- V006: tabella virtuale vec0 per embeddings dei tag.
--
-- Auto-suggerimento tag (Fase 3 Step 4): per un nuovo prompt, suggeriamo
-- i top-K tag esistenti il cui embedding del nome è più vicino all'embedding
-- del body del prompt. Pattern simmetrico a PromptsEmbeddings.
--
-- Modello e dimensioni: stesso del PromptsEmbeddings (vedi
-- docs/architettura/decisioni/embedding-model.md).

CREATE VIRTUAL TABLE IF NOT EXISTS TagsEmbeddings USING vec0(
    TagId TEXT PRIMARY KEY,
    Embedding FLOAT[384]
);
