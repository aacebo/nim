CREATE TABLE IF NOT EXISTS chats (
    id                  UUID            PRIMARY KEY,
    version             INT             NOT NULL,
    summary             TEXT,
    embedding           VECTOR(768),
    confidence          REAL            NOT NULL DEFAULT 0.0,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chats_entities (
    chat_id         UUID            NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    entity_id       UUID            NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (chat_id, entity_id)
);

CREATE TABLE IF NOT EXISTS chats_artifacts (
    chat_id         UUID            NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (chat_id, artifact_id)
);

CREATE TABLE IF NOT EXISTS chats_tasks (
    chat_id         UUID            NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    task_id         UUID            NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (chat_id, task_id)
);

CREATE INDEX IF NOT EXISTS chats_embedding_idx
ON chats USING hnsw (embedding vector_cosine_ops);
