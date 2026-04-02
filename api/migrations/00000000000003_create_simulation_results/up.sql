CREATE TABLE simulation_results (
    id                          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id                      UUID            NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    time_step                   INTEGER         NOT NULL,
    time_years                  DECIMAL,
    fuel_burnup_gwd_t           DECIMAL,
    fuel_remaining_pct          DECIMAL,
    coolant_temp_inlet_c        DECIMAL,
    coolant_temp_outlet_c       DECIMAL,
    coolant_flow_rate_kg_s      DECIMAL,
    thermal_power_mw            DECIMAL,
    electric_power_mw           DECIMAL,
    capacity_factor             DECIMAL,
    waste_actinides_kg          DECIMAL,
    waste_fission_products_kg   DECIMAL,
    waste_total_activity_bq     DECIMAL,
    extra_data                  JSONB
);

CREATE INDEX idx_simulation_results_run_time ON simulation_results (run_id, time_step);
