diesel::table! {
    reactor_designs (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 100]
        design_type -> Varchar,
        #[max_length = 255]
        vendor -> Nullable<Varchar>,
        thermal_power_mw -> Nullable<Numeric>,
        electric_power_mw -> Nullable<Numeric>,
        #[max_length = 100]
        coolant_type -> Nullable<Varchar>,
        #[max_length = 100]
        moderator -> Nullable<Varchar>,
        #[max_length = 100]
        fuel_type -> Nullable<Varchar>,
        enrichment_pct -> Nullable<Numeric>,
        design_metadata -> Nullable<Jsonb>,
        source_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    simulation_runs (id) {
        id -> Uuid,
        reactor_id -> Uuid,
        #[max_length = 50]
        status -> Varchar,
        params -> Nullable<Jsonb>,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
    }
}

diesel::table! {
    simulation_results (id) {
        id -> Uuid,
        run_id -> Uuid,
        time_step -> Int4,
        time_years -> Nullable<Numeric>,
        fuel_burnup_gwd_t -> Nullable<Numeric>,
        fuel_remaining_pct -> Nullable<Numeric>,
        coolant_temp_inlet_c -> Nullable<Numeric>,
        coolant_temp_outlet_c -> Nullable<Numeric>,
        coolant_flow_rate_kg_s -> Nullable<Numeric>,
        thermal_power_mw -> Nullable<Numeric>,
        electric_power_mw -> Nullable<Numeric>,
        capacity_factor -> Nullable<Numeric>,
        waste_actinides_kg -> Nullable<Numeric>,
        waste_fission_products_kg -> Nullable<Numeric>,
        waste_total_activity_bq -> Nullable<Numeric>,
        extra_data -> Nullable<Jsonb>,
    }
}

diesel::table! {
    waste_isotope_inventory (id) {
        id -> Uuid,
        result_id -> Uuid,
        #[max_length = 20]
        isotope -> Varchar,
        mass_kg -> Nullable<Numeric>,
        activity_bq -> Nullable<Numeric>,
        half_life_years -> Nullable<Numeric>,
    }
}

diesel::joinable!(simulation_runs -> reactor_designs (reactor_id));
diesel::joinable!(simulation_results -> simulation_runs (run_id));
diesel::joinable!(waste_isotope_inventory -> simulation_results (result_id));

diesel::allow_tables_to_appear_in_same_query!(
    reactor_designs,
    simulation_runs,
    simulation_results,
    waste_isotope_inventory,
);
