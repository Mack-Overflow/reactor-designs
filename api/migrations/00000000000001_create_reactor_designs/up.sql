CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE reactor_designs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255)    NOT NULL,
    design_type     VARCHAR(100)    NOT NULL,
    vendor          VARCHAR(255),
    thermal_power_mw    DECIMAL,
    electric_power_mw   DECIMAL,
    coolant_type    VARCHAR(100),
    moderator       VARCHAR(100),
    fuel_type       VARCHAR(100),
    enrichment_pct  DECIMAL,
    design_metadata JSONB,
    source_url      TEXT,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reactor_designs_design_type ON reactor_designs (design_type);
