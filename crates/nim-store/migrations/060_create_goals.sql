CREATE TABLE IF NOT EXISTS goals (
    id          UUID            PRIMARY KEY,
    parent_id   UUID            NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    version     INT             NOT NULL,
    description TEXT            NOT NULL,
    status      TEXT            NOT NUll,
    priority    FLOAT           NOT NULL,
    deadline    TIMESTAMPTZ,
    conditions  JSONB           NOT NULL,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS
ON goals (updated_at DESC);
