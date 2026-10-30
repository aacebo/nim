CREATE TABLE IF NOT EXISTS models (
    id              UUID            PRIMARY KEY,
    version         INT             NOT NULL,
    type            TEXT            NOT NULL,           -- bert, bart, t5, ...
    name            TEXT            NOT NULL UNIQUE,    -- sentence-transformers/all-MiniLM-L12-v2
    tasks           TEXT[]          NOT NULL,           -- text_summarize, audio_classify, ...
    architectures   TEXT[]          NOT NULL,           -- ForTokenClassification, ForSequenceClassification, ...
    source          JSONB           NOT NULL,
    weights         JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
