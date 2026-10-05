CREATE TABLE IF NOT EXISTS observations (
    id          UUID            PRIMARY KEY,
    source      TEXT            NOT NULL,
    description TEXT            NOT NULL,
    embedding   VECTOR(384),
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS observations_memories (
    observation_id  UUID            NOT NULL REFERENCES observations(id) ON DELETE CASCADE,
    memory_id       UUID            NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (observation_id, memory_id)
);

CREATE INDEX IF NOT EXISTS
ON observations (updated_at DESC);
