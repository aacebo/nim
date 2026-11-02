CREATE TABLE IF NOT EXISTS messages (
    id              UUID            PRIMARY KEY,
    chat_id         UUID            NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    parent_id       UUID            REFERENCES messages(id) ON DELETE CASCADE,
    version         INT             NOT NULL,
    role            TEXT            NOT NULL,
    content         JSONB           NOT NULL,
    summary         TEXT,
    embedding       VECTOR(384),
    confidence      REAL            NOT NULL DEFAULT 0.0,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS message_annotations (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL DEFAULT 0,
    message_id      UUID            NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    label           TEXT            NOT NULL,
    text            TEXT            NOT NULL,
    spans           JSONB           NOT NULL,
    confidence      REAL            NOT NULL,
    embedding       VECTOR(384),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS messages_tasks (
    message_id      UUID            NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    task_id         UUID            NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (message_id, task_id)
);

ALTER TABLE chats
ADD last_analyzed_id UUID REFERENCES messages(id) ON DELETE SET NULL,
ADD last_analyzed_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS message_annotations_embedding_idx
ON message_annotations USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS messages_embedding_idx
ON messages USING hnsw (embedding vector_cosine_ops);
