CREATE TABLE IF NOT EXISTS artifacts (
    id              UUID            PRIMARY KEY,
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    type            TEXT            NOT NULL,
    name            TEXT,
    content         JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
