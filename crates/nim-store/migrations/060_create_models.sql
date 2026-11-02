CREATE TABLE IF NOT EXISTS models (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL,
    type            TEXT            NOT NULL,           -- bert, bart, t5, ...
    name            TEXT            NOT NULL UNIQUE,    -- sentence-transformers/all-MiniLM-L12-v2
    tasks           TEXT[]          NOT NULL,           -- text_summarize, audio_classify, ...
    architectures   TEXT[]          NOT NULL,           -- ForTokenClassification, ForSequenceClassification, ...
    source          JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS model_weights (
    id              UUID            PRIMARY KEY,
    model_id        UUID            NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    version         INT             NOT NULL,
    name            TEXT            NOT NULL,
    content         JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
