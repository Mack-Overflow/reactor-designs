use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::*;

// ── reactor_designs ──

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = reactor_designs)]
pub struct ReactorDesign {
    pub id: Uuid,
    pub name: String,
    pub design_type: String,
    pub vendor: Option<String>,
    pub thermal_power_mw: Option<BigDecimal>,
    pub electric_power_mw: Option<BigDecimal>,
    pub coolant_type: Option<String>,
    pub moderator: Option<String>,
    pub fuel_type: Option<String>,
    pub enrichment_pct: Option<BigDecimal>,
    pub design_metadata: Option<serde_json::Value>,
    pub source_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = reactor_designs)]
pub struct NewReactorDesign {
    pub name: String,
    pub design_type: String,
    pub vendor: Option<String>,
    pub thermal_power_mw: Option<BigDecimal>,
    pub electric_power_mw: Option<BigDecimal>,
    pub coolant_type: Option<String>,
    pub moderator: Option<String>,
    pub fuel_type: Option<String>,
    pub enrichment_pct: Option<BigDecimal>,
    pub design_metadata: Option<serde_json::Value>,
    pub source_url: Option<String>,
}

// ── simulation_runs ──

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = simulation_runs)]
pub struct SimulationRun {
    pub id: Uuid,
    pub reactor_id: Uuid,
    pub status: String,
    pub params: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = simulation_runs)]
pub struct NewSimulationRun {
    pub reactor_id: Uuid,
    pub status: String,
    pub params: Option<serde_json::Value>,
}

// ── simulation_results ──

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = simulation_results)]
pub struct SimulationResult {
    pub id: Uuid,
    pub run_id: Uuid,
    pub time_step: i32,
    pub time_years: Option<BigDecimal>,
    pub fuel_burnup_gwd_t: Option<BigDecimal>,
    pub fuel_remaining_pct: Option<BigDecimal>,
    pub coolant_temp_inlet_c: Option<BigDecimal>,
    pub coolant_temp_outlet_c: Option<BigDecimal>,
    pub coolant_flow_rate_kg_s: Option<BigDecimal>,
    pub thermal_power_mw: Option<BigDecimal>,
    pub electric_power_mw: Option<BigDecimal>,
    pub capacity_factor: Option<BigDecimal>,
    pub waste_actinides_kg: Option<BigDecimal>,
    pub waste_fission_products_kg: Option<BigDecimal>,
    pub waste_total_activity_bq: Option<BigDecimal>,
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = simulation_results)]
pub struct NewSimulationResult {
    pub run_id: Uuid,
    pub time_step: i32,
    pub time_years: Option<BigDecimal>,
    pub fuel_burnup_gwd_t: Option<BigDecimal>,
    pub fuel_remaining_pct: Option<BigDecimal>,
    pub coolant_temp_inlet_c: Option<BigDecimal>,
    pub coolant_temp_outlet_c: Option<BigDecimal>,
    pub coolant_flow_rate_kg_s: Option<BigDecimal>,
    pub thermal_power_mw: Option<BigDecimal>,
    pub electric_power_mw: Option<BigDecimal>,
    pub capacity_factor: Option<BigDecimal>,
    pub waste_actinides_kg: Option<BigDecimal>,
    pub waste_fission_products_kg: Option<BigDecimal>,
    pub waste_total_activity_bq: Option<BigDecimal>,
    pub extra_data: Option<serde_json::Value>,
}

// ── waste_isotope_inventory ──

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = waste_isotope_inventory)]
pub struct WasteIsotopeInventory {
    pub id: Uuid,
    pub result_id: Uuid,
    pub isotope: String,
    pub mass_kg: Option<BigDecimal>,
    pub activity_bq: Option<BigDecimal>,
    pub half_life_years: Option<BigDecimal>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = waste_isotope_inventory)]
pub struct NewWasteIsotopeInventory {
    pub result_id: Uuid,
    pub isotope: String,
    pub mass_kg: Option<BigDecimal>,
    pub activity_bq: Option<BigDecimal>,
    pub half_life_years: Option<BigDecimal>,
}
