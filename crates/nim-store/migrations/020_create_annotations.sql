CREATE TABLE IF NOT EXISTS annotations (
    id              UUID            PRIMARY KEY,
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    label           TEXT            NOT NULL,
    text            TEXT            NOT NULL,
    spans           JSONB           NOT NULL,
    confidence      FLOAT           NOT NULL,
    embedding       VECTOR(384),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS annotations_memory_idx
ON annotations (memory_id, created_at DESC);
