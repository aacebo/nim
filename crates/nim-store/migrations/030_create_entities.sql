CREATE TABLE IF NOT EXISTS entities (
    id          UUID            PRIMARY KEY,
    type        TEXT            NOT NULL,
    version     INT             NOT NULL,
    name        TEXT            NOT NULL,
    summary     TEXT,
    confidence  REAL            NOT NULL,
    embedding   VECTOR(384),
    recalls     BIGINT          NOT NULL,
    recalled_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS entities_memories (
    entity_id       UUID            NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (entity_id, memory_id)
);

CREATE TABLE IF NOT EXISTS entities_relations (
    source_id       UUID            NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    target_id       UUID            NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    type            TEXT            NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (source_id, target_id, type)
);

CREATE INDEX IF NOT EXISTS entities_updated_at_idx
ON entities (updated_at DESC);

CREATE INDEX IF NOT EXISTS entities_embedding_idx
ON entities USING hnsw (embedding vector_cosine_ops);
