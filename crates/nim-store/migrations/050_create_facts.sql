CREATE TABLE IF NOT EXISTS facts (
    id          UUID            PRIMARY KEY,
    description TEXT            NOT NULL,
    confidence  FLOAT           NOT NULL,
    embedding   VECTOR(384),
    recalls     BIGINT          NOT NULL,
    recalled_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories_facts (
    memory_id   UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    fact_id     UUID            NOT NULL REFERENCES facts(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (memory_id, fact_id)
);

CREATE TABLE IF NOT EXISTS entities_facts (
    entity_id   UUID            NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    fact_id     UUID            NOT NULL REFERENCES facts(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (entity_id, fact_id)
);

CREATE INDEX IF NOT EXISTS facts_updated_at_idx
ON facts (updated_at DESC);
