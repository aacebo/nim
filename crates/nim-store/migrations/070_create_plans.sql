CREATE TABLE IF NOT EXISTS plans (
    id          UUID            PRIMARY KEY,
    title       TEXT            NOT NULL,
    status      TEXT            NOT NULL,
    description TEXT            NOT NULL,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS steps (
    id          UUID            PRIMARY KEY,
    plan_id     UUID            NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    position    INT             NOT NULL,
    name        TEXT            NOT NULL,
    status      TEXT            NOT NUll,
    about       TEXT,
    action      JSONB           NOT NULL,
    condition   JSONB,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS
ON plans (updated_at DESC);

CREATE INDEX IF NOT EXISTS
ON steps (plan_id, position ASC);
