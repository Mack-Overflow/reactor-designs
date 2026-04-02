CREATE TABLE simulation_runs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reactor_id      UUID            NOT NULL REFERENCES reactor_designs(id) ON DELETE CASCADE,
    status          VARCHAR(50)     NOT NULL DEFAULT 'pending',
    params          JSONB,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    error_message   TEXT
);
