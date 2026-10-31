CREATE TABLE IF NOT EXISTS tasks (
    id              UUID            PRIMARY KEY,
    parent_id       UUID            REFERENCES tasks(id) ON DELETE SET NULL,
    version         INT             NOT NULL,
    name            TEXT            NOT NULL,           -- text_classify, audio_caption, ...
    source          TEXT            NOT NULL,           -- audio, vision, text, encode...
    status          JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS models_tasks (
    model_id        UUID            NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    task_id         UUID            NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (model_id, task_id)
);

CREATE TABLE IF NOT EXISTS artifacts_tasks (
    artifact_id     UUID            NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    task_id         UUID            NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY (artifact_id, task_id)
);
