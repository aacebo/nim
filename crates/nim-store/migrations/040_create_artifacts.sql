CREATE TABLE IF NOT EXISTS artifacts (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL DEFAULT 0,
    type            TEXT            NOT NULL,
    name            TEXT            NOT NULL,
    content         JSONB           NOT NULL,
    summary         TEXT,
    embedding       VECTOR(768),
    confidence      REAL            NOT NULL DEFAULT 0.0,
    salience        REAL            NOT NULL DEFAULT 0.0,
    strength        REAL            NOT NULL DEFAULT 0.0,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories_artifacts (
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (memory_id, artifact_id)
);

CREATE TABLE IF NOT EXISTS artifact_annotations (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL DEFAULT 0,
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    label           TEXT            NOT NULL,
    text            TEXT            NOT NULL,
    spans           JSONB           NOT NULL,
    confidence      REAL            NOT NULL,
    embedding       VECTOR(768),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS artifact_annotations_embedding_idx
ON artifact_annotations USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS artifacts_embedding_idx
ON artifacts USING hnsw (embedding vector_cosine_ops);
