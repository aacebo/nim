CREATE TABLE IF NOT EXISTS memories (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL DEFAULT 0,
    type            TEXT            NOT NULL,
    contexts        JSONB           NOT NULL,
    salience        FLOAT           NOT NULL DEFAULT 0.0,
    strength        FLOAT           NOT NULL DEFAULT 0.0,
    confidence      FLOAT           NOT NULL DEFAULT 0.0,
    recalls         INT             NOT NULL DEFAULT 0,
    description     TEXT            NOT NULL,
    summary         TEXT,
    embedding       VECTOR(384),
    recalled_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories_relations (
    source_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    target_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (source_id, target_id)
);

CREATE INDEX IF NOT EXISTS memories_type_idx
ON memories (type);

CREATE INDEX IF NOT EXISTS memories_updated_at_idx
ON memories (updated_at DESC);

CREATE INDEX IF NOT EXISTS memories_embedding_idx
ON memories USING hnsw (embedding vector_cosine_ops);
