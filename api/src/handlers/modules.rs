use actix_web::{web, HttpResponse};
use std::collections::HashMap;

use crate::services::simulation::fluid::{compute_fluid, CoolantType, FluidConfig};
use crate::services::simulation::fuel::{step_fuel, FuelConfig, FuelState};
use crate::services::simulation::power::{compute_power, thermal_efficiency, CycleType, PowerConfig};
use crate::services::simulation::waste::{default_isotope_library, step_waste, IsotopeState};

// ── Thermal module ──

#[derive(serde::Deserialize)]
pub struct ThermalRequest {
    pub thermal_power_mw: f64,
    pub coolant_type: CoolantType,
    pub inlet_temp_c: f64,
    pub flow_rate_kg_s: f64,
}

#[derive(serde::Serialize)]
pub struct CoolantResult {
    pub coolant: String,
    pub outlet_temp_c: f64,
    pub delta_t_c: f64,
}

#[derive(serde::Serialize)]
pub struct ThermalResponse {
    pub outlet_temp_c: f64,
    pub delta_t_c: f64,
    pub flow_rate_kg_s: f64,
    // Outlet temps for all 5 coolants at same power/flow/inlet — for comparison
    pub coolant_comparison: Vec<CoolantResult>,
    // Outlet temp across a range of thermal powers (0..rated, 10 points) — for the channel viz
    pub power_curve: Vec<PowerCurvePoint>,
}

#[derive(serde::Serialize)]
pub struct PowerCurvePoint {
    pub thermal_power_mw: f64,
    pub outlet_temp_c: f64,
    pub delta_t_c: f64,
}

// POST /api/modules/thermal
pub async fn run_thermal(body: web::Json<ThermalRequest>) -> HttpResponse {
    let req = body.into_inner();

    let cfg = FluidConfig {
        coolant_type: req.coolant_type,
        inlet_temp_c: req.inlet_temp_c,
        flow_rate_kg_s: req.flow_rate_kg_s,
    };
    let primary = compute_fluid(&cfg, req.thermal_power_mw);

    // Comparison across all 5 coolant types at the same operating point
    let all_coolants = [
        (CoolantType::Sodium, "Sodium"),
        (CoolantType::Lead, "Lead"),
        (CoolantType::Helium, "Helium"),
        (CoolantType::FLiBe, "FLiBe"),
        (CoolantType::LightWater, "Light Water"),
    ];

    let coolant_comparison = all_coolants.iter().map(|(ct, name)| {
        let c = FluidConfig {
            coolant_type: *ct,
            inlet_temp_c: req.inlet_temp_c,
            flow_rate_kg_s: req.flow_rate_kg_s,
        };
        let out = compute_fluid(&c, req.thermal_power_mw);
        CoolantResult {
            coolant: name.to_string(),
            outlet_temp_c: out.outlet_temp_c,
            delta_t_c: out.delta_t_c,
        }
    }).collect();

    // Power curve: outlet temp vs thermal power from 10% to 100% of rated, 12 points
    let power_curve = (1..=12).map(|i| {
        let frac = i as f64 / 12.0;
        let p = req.thermal_power_mw * frac;
        let out = compute_fluid(&cfg, p);
        PowerCurvePoint {
            thermal_power_mw: p,
            outlet_temp_c: out.outlet_temp_c,
            delta_t_c: out.delta_t_c,
        }
    }).collect();

    HttpResponse::Ok().json(ThermalResponse {
        outlet_temp_c: primary.outlet_temp_c,
        delta_t_c: primary.delta_t_c,
        flow_rate_kg_s: primary.flow_rate_kg_s,
        coolant_comparison,
        power_curve,
    })
}

// ── Power module ──

#[derive(serde::Deserialize)]
pub struct PowerRequest {
    pub thermal_power_mw: f64,
    pub outlet_temp_c: f64,
    pub cycle_type: CycleType,
    pub rated_electric_power_mw: f64,
}

#[derive(serde::Serialize)]
pub struct CycleResult {
    pub cycle: String,
    pub efficiency: f64,
    pub electric_power_mw: f64,
    pub capacity_factor: f64,
}

#[derive(serde::Serialize)]
pub struct PowerResponse {
    pub efficiency: f64,
    pub electric_power_mw: f64,
    pub capacity_factor: f64,
    pub carnot_efficiency: f64,
    // All 3 cycles at the same operating point — for T-S diagram comparison
    pub cycle_comparison: Vec<CycleResult>,
    // Efficiency vs outlet temp curve for the selected cycle, 400°C–950°C, 20 points
    pub efficiency_curve: Vec<EfficiencyCurvePoint>,
}

#[derive(serde::Serialize)]
pub struct EfficiencyCurvePoint {
    pub outlet_temp_c: f64,
    pub efficiency: f64,
    pub carnot: f64,
}

// POST /api/modules/power
pub async fn run_power(body: web::Json<PowerRequest>) -> HttpResponse {
    let req = body.into_inner();

    let cfg = PowerConfig {
        thermal_power_mw: req.thermal_power_mw,
        coolant_outlet_temp_c: req.outlet_temp_c,
        cycle_type: req.cycle_type,
        rated_electric_power_mw: req.rated_electric_power_mw,
    };
    let primary = compute_power(&cfg);

    // Carnot limit at this outlet temp
    let t_hot_k = req.outlet_temp_c + 273.15;
    let t_cold_k = 35.0 + 273.15;
    let carnot = if t_hot_k > t_cold_k { 1.0 - t_cold_k / t_hot_k } else { 0.0 };

    // Comparison: all 3 cycle types at this outlet temp
    let cycles = [
        (CycleType::Rankine, "Rankine"),
        (CycleType::Brayton, "Brayton"),
        (CycleType::SCO2Brayton, "SCO₂ Brayton"),
    ];

    let cycle_comparison = cycles.iter().map(|(ct, name)| {
        let c = PowerConfig {
            thermal_power_mw: req.thermal_power_mw,
            coolant_outlet_temp_c: req.outlet_temp_c,
            cycle_type: *ct,
            rated_electric_power_mw: req.rated_electric_power_mw,
        };
        let out = compute_power(&c);
        CycleResult {
            cycle: name.to_string(),
            efficiency: out.efficiency,
            electric_power_mw: out.electric_power_mw,
            capacity_factor: out.capacity_factor,
        }
    }).collect();

    // Efficiency curve: 200°C to 950°C in 25 steps for selected cycle
    let efficiency_curve = (0..=25).map(|i| {
        let t = 200.0 + i as f64 * (950.0 - 200.0) / 25.0;
        let eff = thermal_efficiency(req.cycle_type, t);
        let t_hot = t + 273.15;
        let c = if t_hot > t_cold_k { 1.0 - t_cold_k / t_hot } else { 0.0 };
        EfficiencyCurvePoint { outlet_temp_c: t, efficiency: eff, carnot: c }
    }).collect();

    HttpResponse::Ok().json(PowerResponse {
        efficiency: primary.efficiency,
        electric_power_mw: primary.electric_power_mw,
        capacity_factor: primary.capacity_factor,
        carnot_efficiency: carnot,
        cycle_comparison,
        efficiency_curve,
    })
}

// ── Fuel module ──

#[derive(serde::Deserialize)]
pub struct FuelRequest {
    pub initial_heavy_metal_tonnes: f64,
    pub enrichment_pct: f64,
    pub target_burnup_gwd_t: f64,
    pub thermal_power_mw: f64,
    pub breeding_ratio: f64,
    pub duration_years: f64,
    pub time_step_days: f64,
}

#[derive(serde::Serialize)]
pub struct FuelStep {
    pub time_years: f64,
    pub burnup_gwd_t: f64,
    pub fuel_remaining_pct: f64,
    pub effective_thermal_power_mw: f64,
    pub fissile_fraction: f64,
    pub fission_rate_per_s: f64,
}

#[derive(serde::Serialize)]
pub struct FuelResponse {
    pub steps: Vec<FuelStep>,
    pub total_steps: usize,
    pub shutdown_year: Option<f64>,
    pub final_burnup_gwd_t: f64,
}

// POST /api/modules/fuel
pub async fn run_fuel(body: web::Json<FuelRequest>) -> HttpResponse {
    let req = body.into_inner();

    if req.duration_years <= 0.0 || req.time_step_days <= 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "duration_years and time_step_days must be positive"
        }));
    }

    let cfg = FuelConfig {
        initial_heavy_metal_tonnes: req.initial_heavy_metal_tonnes,
        enrichment_pct: req.enrichment_pct,
        target_burnup_gwd_t: req.target_burnup_gwd_t,
        thermal_power_mw: req.thermal_power_mw,
        breeding_ratio: req.breeding_ratio,
    };
    let mut state = FuelState::new(&cfg);

    let dt_years = req.time_step_days / 365.25;
    let total_steps = ((req.duration_years / dt_years).ceil() as usize).max(1);
    let mut steps = Vec::with_capacity(total_steps);
    let mut shutdown_year: Option<f64> = None;

    for i in 0..total_steps {
        let time_years = (i as f64 + 1.0) * dt_years;
        let out = step_fuel(&mut state, &cfg, dt_years);

        if state.shutdown && shutdown_year.is_none() {
            shutdown_year = Some(time_years);
        }

        steps.push(FuelStep {
            time_years,
            burnup_gwd_t: out.burnup_gwd_t,
            fuel_remaining_pct: out.fuel_remaining_pct,
            effective_thermal_power_mw: out.effective_thermal_power_mw,
            fissile_fraction: out.fissile_fraction,
            fission_rate_per_s: out.fission_rate_per_s,
        });
    }

    let final_burnup = steps.last().map(|s| s.burnup_gwd_t).unwrap_or(0.0);

    HttpResponse::Ok().json(FuelResponse {
        total_steps: steps.len(),
        shutdown_year,
        final_burnup_gwd_t: final_burnup,
        steps,
    })
}

// ── Waste module ──

#[derive(serde::Deserialize)]
pub struct WasteRequest {
    pub thermal_power_mw: f64,
    pub breeding_ratio: f64,
    pub duration_years: f64,
    pub time_step_days: f64,
}

#[derive(serde::Serialize)]
pub struct WasteStep {
    pub time_years: f64,
    pub total_actinides_kg: f64,
    pub total_fission_products_kg: f64,
    pub total_activity_bq: f64,
    pub isotopes: HashMap<String, IsotopeState>,
}

#[derive(serde::Serialize)]
pub struct WasteResponse {
    pub steps: Vec<WasteStep>,
    pub total_steps: usize,
}

// POST /api/modules/waste
pub async fn run_waste(body: web::Json<WasteRequest>) -> HttpResponse {
    let req = body.into_inner();

    if req.duration_years <= 0.0 || req.time_step_days <= 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "duration_years and time_step_days must be positive"
        }));
    }

    let library = default_isotope_library();

    // Derive fission rate from thermal power: R = P / E_per_fission
    let energy_per_fission_j: f64 = 200.0e6 * 1.602_176_634e-19;
    let fission_rate_per_s = req.thermal_power_mw * 1.0e6 / energy_per_fission_j;

    let dt_years = req.time_step_days / 365.25;
    let dt_seconds = req.time_step_days * 86400.0;
    let total_steps = ((req.duration_years / dt_years).ceil() as usize).max(1);

    let mut inventory: HashMap<String, f64> = HashMap::new();
    let mut steps = Vec::with_capacity(total_steps);

    for i in 0..total_steps {
        let time_years = (i as f64 + 1.0) * dt_years;
        let out = step_waste(&mut inventory, fission_rate_per_s, req.breeding_ratio, dt_seconds, &library);

        steps.push(WasteStep {
            time_years,
            total_actinides_kg: out.total_actinides_kg,
            total_fission_products_kg: out.total_fission_products_kg,
            total_activity_bq: out.total_activity_bq,
            isotopes: out.isotopes,
        });
    }

    HttpResponse::Ok().json(WasteResponse {
        total_steps: steps.len(),
        steps,
    })
}
