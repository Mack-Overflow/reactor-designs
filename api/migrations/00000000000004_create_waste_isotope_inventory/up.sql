CREATE TABLE waste_isotope_inventory (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    result_id       UUID            NOT NULL REFERENCES simulation_results(id) ON DELETE CASCADE,
    isotope         VARCHAR(20)     NOT NULL,
    mass_kg         DECIMAL,
    activity_bq     DECIMAL,
    half_life_years DECIMAL
);

CREATE INDEX idx_waste_isotope_result_isotope ON waste_isotope_inventory (result_id, isotope);
