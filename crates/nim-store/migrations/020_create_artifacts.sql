CREATE TABLE IF NOT EXISTS artifacts (
    id              UUID            PRIMARY KEY,
    type            TEXT            NOT NULL,
    name            TEXT            NOT NULL,
    content         JSONB           NOT NULL,
    summary         TEXT,
    embedding       VECTOR(384),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories_artifacts (
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (memory_id, artifact_id)
);

CREATE INDEX IF NOT EXISTS artifacts_embedding_idx
ON artifacts USING hnsw (embedding vector_cosine_ops);
