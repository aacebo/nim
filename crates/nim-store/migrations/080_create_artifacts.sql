CREATE TABLE IF NOT EXISTS artifacts (
    id              UUID            PRIMARY KEY,
    type            TEXT            NOT NULL,
    name            TEXT,
    content         JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories_artifacts (
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (memory_id, artifact_id)
);
