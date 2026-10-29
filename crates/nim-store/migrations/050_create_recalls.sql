CREATE TABLE IF NOT EXISTS recalls (
    id              UUID            PRIMARY KEY,
    trigger         TEXT            NOT NULL,
    source          TEXT            NOT NULL,
    query           TEXT,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories_recalls (
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    recall_id       UUID            NOT NULL REFERENCES recalls(id) ON DELETE CASCADE,
    score           REAL            NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (memory_id, recall_id)
);
