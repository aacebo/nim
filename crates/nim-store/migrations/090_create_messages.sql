CREATE TABLE IF NOT EXISTS messages (
    id              UUID            PRIMARY KEY,
    chat_id         UUID            NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    version         INT             NOT NULL,
    role            TEXT            NOT NULL,
    content         JSONB           NOT NULL,
    summary         TEXT,
    confidence      REAL            NOT NULL,
    embedding       VECTOR(384),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS messages_embedding_idx
ON messages USING hnsw (embedding vector_cosine_ops);
