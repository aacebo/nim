CREATE TABLE IF NOT EXISTS annotations (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL DEFAULT 0,
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    label           TEXT            NOT NULL,
    text            TEXT            NOT NULL,
    spans           JSONB           NOT NULL,
    confidence      REAL            NOT NULL,
    embedding       VECTOR(384),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS annotations_artifact_idx
ON annotations (artifact_id, created_at DESC);

CREATE INDEX IF NOT EXISTS annotations_embedding_idx
ON annotations USING hnsw (embedding vector_cosine_ops);

CREATE UNIQUE INDEX IF NOT EXISTS annotations_artifact_id_label_text_idx
ON annotations (artifact_id, label, text);
