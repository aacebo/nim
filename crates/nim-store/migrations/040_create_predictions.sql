CREATE TABLE IF NOT EXISTS predictions (
    id          UUID            PRIMARY KEY,
    version     INT             NOT NULL,
    hypothesis  TEXT            NOT NULL,
    confidence  FLOAT           NOT NULL,
    deadline    TIMESTAMPTZ,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS predictions_memories (
    prediction_id   UUID            NOT NULL REFERENCES predictions(id) ON DELETE CASCADE,
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (prediction_id, memory_id)
);

CREATE INDEX IF NOT EXISTS
ON predictions (updated_at DESC);
