use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::fuel::{self, FuelConfig, FuelOutput, FuelState};
use super::fluid::{self, CoolantType, FluidConfig, FluidOutput};
use super::power::{self, CycleType, PowerConfig, PowerOutput};
use super::waste::{self, WasteConfig, WasteOutput};

// Full simulation parameters — everything needed to run end-to-end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    // Simulation duration in years.
    pub duration_years: f64,
    // Time step size in days.
    pub time_step_days: f64,

    // ── Fuel config ──
    pub initial_heavy_metal_tonnes: f64,
    pub enrichment_pct: f64,
    pub target_burnup_gwd_t: f64,
    pub thermal_power_mw: f64,
    pub breeding_ratio: f64,

    // ── Fluid config ──
    pub coolant_type: CoolantType,
    pub coolant_inlet_temp_c: f64,
    pub coolant_flow_rate_kg_s: f64,

    // ── Power config ──
    pub cycle_type: CycleType,
    pub rated_electric_power_mw: f64,
}

// The result of a single time step across all modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub time_step: usize,
    pub time_years: f64,
    pub fuel: FuelOutput,
    pub fluid: FluidOutput,
    pub power: PowerOutput,
    pub waste: WasteOutput,
}

// The result of a complete simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationOutput {
    pub steps: Vec<StepResult>,
    pub total_steps: usize,
    pub average_capacity_factor: f64,
}

// Validate simulation parameters before running.
pub fn validate_params(params: &SimulationParams) -> Result<(), String> {
    if params.duration_years <= 0.0 {
        return Err("duration_years must be positive".into());
    }
    if params.time_step_days <= 0.0 {
        return Err("time_step_days must be positive".into());
    }
    if params.thermal_power_mw < 0.0 {
        return Err("thermal_power_mw cannot be negative".into());
    }
    if params.initial_heavy_metal_tonnes < 0.0 {
        return Err("initial_heavy_metal_tonnes cannot be negative".into());
    }
    if params.rated_electric_power_mw < 0.0 {
        return Err("rated_electric_power_mw cannot be negative".into());
    }
    Ok(())
}

// Compute the total number of time steps for a given parameter set.
pub fn compute_total_steps(params: &SimulationParams) -> usize {
    let dt_years = params.time_step_days / 365.25;
    ((params.duration_years / dt_years).ceil() as usize).max(1)
}

// Run a simulation, calling `on_step` after each time step completes.
//
// Module coupling per step:
//   1. fuel.step()     → fissile_fraction, fission_rate, effective_thermal_power
//   2. fluid.step()    → coolant temperatures (depends on thermal_power)
//   3. power.compute() → electric output and efficiency (depends on outlet_temp)
//   4. waste.step()    → isotope inventory (depends on fission_rate + breeding_ratio)
//
// If the fuel module reports shutdown (fissile exhausted), subsequent steps
// continue with zero fission but waste decay still proceeds.
pub fn run_simulation_streaming<F>(
    params: &SimulationParams,
    mut on_step: F,
) -> Result<SimulationOutput, String>
where
    F: FnMut(&StepResult, usize),
{
    validate_params(params)?;

    let dt_years = params.time_step_days / 365.25;
    let dt_seconds = params.time_step_days * 86400.0;
    let total_steps = compute_total_steps(params);

    let fuel_cfg = FuelConfig {
        initial_heavy_metal_tonnes: params.initial_heavy_metal_tonnes,
        enrichment_pct: params.enrichment_pct,
        target_burnup_gwd_t: params.target_burnup_gwd_t,
        thermal_power_mw: params.thermal_power_mw,
        breeding_ratio: params.breeding_ratio,
    };

    let fluid_cfg = FluidConfig {
        coolant_type: params.coolant_type,
        inlet_temp_c: params.coolant_inlet_temp_c,
        flow_rate_kg_s: params.coolant_flow_rate_kg_s,
    };

    let waste_cfg = WasteConfig {
        isotope_library: waste::default_isotope_library(),
    };

    let mut fuel_state = FuelState::new(&fuel_cfg);
    let mut waste_inventory: HashMap<String, f64> = HashMap::new();
    let mut steps = Vec::with_capacity(total_steps);
    let mut electric_powers = Vec::with_capacity(total_steps);

    for i in 0..total_steps {
        let time_years = (i as f64 + 1.0) * dt_years;

        // 1. Fuel: fissile depletion, power, fission rate
        let fuel_out = fuel::step_fuel(&mut fuel_state, &fuel_cfg, dt_years);

        // 2. Fluid: coolant temperatures from thermal power
        let fluid_out = fluid::compute_fluid(&fluid_cfg, fuel_out.effective_thermal_power_mw);

        // 3. Power: efficiency from outlet temp, electric output
        let power_cfg = PowerConfig {
            thermal_power_mw: fuel_out.effective_thermal_power_mw,
            coolant_outlet_temp_c: fluid_out.outlet_temp_c,
            cycle_type: params.cycle_type,
            rated_electric_power_mw: params.rated_electric_power_mw,
        };
        let power_out = power::compute_power(&power_cfg);

        // 4. Waste: isotope production (from fission_rate) + decay
        let waste_out = waste::step_waste(
            &mut waste_inventory,
            fuel_out.fission_rate_per_s,
            params.breeding_ratio,
            dt_seconds,
            &waste_cfg.isotope_library,
        );

        electric_powers.push(power_out.electric_power_mw);

        let step = StepResult {
            time_step: i + 1,
            time_years,
            fuel: fuel_out,
            fluid: fluid_out,
            power: power_out,
            waste: waste_out,
        };

        on_step(&step, total_steps);
        steps.push(step);
    }

    let average_capacity_factor =
        power::average_capacity_factor(&electric_powers, params.rated_electric_power_mw);

    Ok(SimulationOutput {
        steps,
        total_steps,
        average_capacity_factor,
    })
}

// Run a complete simulation (non-streaming). Collects all results at once.
pub fn run_simulation(params: &SimulationParams) -> Result<SimulationOutput, String> {
    run_simulation_streaming(params, |_, _| {})
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build physically-consistent Natrium SFR params.
    fn natrium_params() -> SimulationParams {
        let thermal_power_mw = 840.0;
        let initial_hm_t = 60.0;
        let burnup_rate_per_day = thermal_power_mw / (initial_hm_t * 1000.0);
        let target_burnup = 150.0;
        let cycle_life_years = target_burnup / (burnup_rate_per_day * 365.25);
        let duration = cycle_life_years * 0.17; // ~5 years

        SimulationParams {
            duration_years: duration,
            time_step_days: 30.0,
            initial_heavy_metal_tonnes: initial_hm_t,
            enrichment_pct: 15.0,
            target_burnup_gwd_t: target_burnup,
            thermal_power_mw,
            breeding_ratio: 0.8,
            coolant_type: CoolantType::Sodium,
            coolant_inlet_temp_c: 350.0,
            coolant_flow_rate_kg_s: 4400.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 345.0,
        }
    }

    // Build physically-consistent Xe-100 HTGR params.
    fn xe100_params() -> SimulationParams {
        let thermal_power_mw = 200.0;
        let initial_hm_t = 7.0;
        let burnup_rate_per_day = thermal_power_mw / (initial_hm_t * 1000.0);
        let target_burnup = burnup_rate_per_day * 365.25 * 16.0;
        let duration = 3.0;

        SimulationParams {
            duration_years: duration,
            time_step_days: 30.0,
            initial_heavy_metal_tonnes: initial_hm_t,
            enrichment_pct: 15.5,
            target_burnup_gwd_t: target_burnup,
            thermal_power_mw,
            breeding_ratio: 0.0,
            coolant_type: CoolantType::Helium,
            coolant_inlet_temp_c: 260.0,
            coolant_flow_rate_kg_s: 80.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 80.0,
        }
    }

    // ── Validation tests ──

    #[test]
    fn rejects_zero_duration() {
        let mut p = natrium_params();
        p.duration_years = 0.0;
        assert!(run_simulation(&p).is_err());
    }

    #[test]
    fn rejects_negative_step() {
        let mut p = natrium_params();
        p.time_step_days = -1.0;
        assert!(run_simulation(&p).is_err());
    }

    // ── Natrium SFR integration ──

    #[test]
    fn natrium_runs_to_completion() {
        let result = run_simulation(&natrium_params()).unwrap();
        assert_eq!(result.total_steps, result.steps.len());
        assert!(result.total_steps > 50);
    }

    #[test]
    fn natrium_burnup_increases_monotonically() {
        let result = run_simulation(&natrium_params()).unwrap();
        for w in result.steps.windows(2) {
            assert!(w[1].fuel.burnup_gwd_t >= w[0].fuel.burnup_gwd_t);
        }
    }

    #[test]
    fn natrium_produces_electric_power() {
        let result = run_simulation(&natrium_params()).unwrap();
        let first = &result.steps[0];
        assert!(first.power.electric_power_mw > 250.0,
            "electric power = {}", first.power.electric_power_mw);
        assert!(first.power.efficiency > 0.30);
    }

    #[test]
    fn natrium_outlet_temp_reasonable() {
        let result = run_simulation(&natrium_params()).unwrap();
        let first = &result.steps[0];
        assert!(first.fluid.outlet_temp_c > 400.0 && first.fluid.outlet_temp_c < 600.0,
            "outlet temp = {}", first.fluid.outlet_temp_c);
    }

    #[test]
    fn natrium_waste_accumulates() {
        let result = run_simulation(&natrium_params()).unwrap();
        let last = result.steps.last().unwrap();
        assert!(last.waste.total_actinides_kg > 0.0);
        assert!(last.waste.total_fission_products_kg > 0.0);
        assert!(last.waste.total_activity_bq > 0.0);
    }

    #[test]
    fn natrium_capacity_factor_reasonable() {
        let result = run_simulation(&natrium_params()).unwrap();
        assert!(result.average_capacity_factor > 0.5 && result.average_capacity_factor <= 1.0,
            "avg CF = {}", result.average_capacity_factor);
    }

    #[test]
    fn natrium_fissile_fraction_decreases() {
        let result = run_simulation(&natrium_params()).unwrap();
        let first = &result.steps[0];
        let last = result.steps.last().unwrap();
        // With BR=0.8, fissile still decreases (net consumption = 20% of total)
        assert!(last.fuel.fissile_fraction < first.fuel.fissile_fraction,
            "fissile should decrease: first={}, last={}",
            first.fuel.fissile_fraction, last.fuel.fissile_fraction);
    }

    // ── Xe-100 HTGR integration ──

    #[test]
    fn xe100_runs_to_completion() {
        let result = run_simulation(&xe100_params()).unwrap();
        assert!(result.total_steps > 30);
    }

    #[test]
    fn xe100_rankine_efficiency() {
        let result = run_simulation(&xe100_params()).unwrap();
        let first = &result.steps[0];
        assert!(first.power.efficiency > 0.30 && first.power.efficiency < 0.44,
            "xe100 efficiency = {}", first.power.efficiency);
    }

    #[test]
    fn xe100_helium_outlet_temp() {
        let result = run_simulation(&xe100_params()).unwrap();
        let first = &result.steps[0];
        assert!(first.fluid.outlet_temp_c > 650.0 && first.fluid.outlet_temp_c < 850.0,
            "xe100 outlet = {}", first.fluid.outlet_temp_c);
    }

    // ── Cross-module coupling ──

    #[test]
    fn power_drops_when_fuel_exhausted() {
        let thermal_power_mw = 300.0;
        let initial_hm_t = 5.0;
        let target_burnup = 40.0;
        let burnup_rate = thermal_power_mw / (initial_hm_t * 1000.0);
        let cycle_life_years = target_burnup / (burnup_rate * 365.25);
        let duration = cycle_life_years * 2.7;

        let params = SimulationParams {
            duration_years: duration,
            time_step_days: 30.0,
            initial_heavy_metal_tonnes: initial_hm_t,
            enrichment_pct: 5.0,
            target_burnup_gwd_t: target_burnup,
            thermal_power_mw,
            breeding_ratio: 0.0,
            coolant_type: CoolantType::LightWater,
            coolant_inlet_temp_c: 280.0,
            coolant_flow_rate_kg_s: 5000.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 100.0,
        };
        let result = run_simulation(&params).unwrap();
        let last = result.steps.last().unwrap();
        assert!(last.fuel.effective_thermal_power_mw < 1.0,
            "thermal power should be ~0 after fuel exhaustion, got {}",
            last.fuel.effective_thermal_power_mw);
        assert!(last.power.electric_power_mw < 1.0);
        assert!(last.fluid.delta_t_c < 0.1);
    }

    #[test]
    fn waste_continues_decaying_after_shutdown() {
        let params = SimulationParams {
            duration_years: 10.0,
            time_step_days: 30.0,
            initial_heavy_metal_tonnes: 5.0,
            enrichment_pct: 5.0,
            target_burnup_gwd_t: 20.0,
            thermal_power_mw: 300.0,
            breeding_ratio: 0.0,
            coolant_type: CoolantType::LightWater,
            coolant_inlet_temp_c: 280.0,
            coolant_flow_rate_kg_s: 5000.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 100.0,
        };
        let result = run_simulation(&params).unwrap();

        // Find first shutdown step (zero power)
        let shutdown_idx = result.steps.iter()
            .position(|s| s.fuel.effective_thermal_power_mw < 1.0);
        assert!(shutdown_idx.is_some(), "reactor should shut down");

        // After shutdown, waste activity should still be tracked and declining
        let last = result.steps.last().unwrap();
        assert!(last.waste.total_activity_bq > 0.0,
            "waste should still have activity after shutdown");
    }

    #[test]
    fn waste_tracks_isotope_detail() {
        let result = run_simulation(&natrium_params()).unwrap();
        let last = result.steps.last().unwrap();
        assert!(last.waste.isotopes.contains_key("Cs-137"));
        assert!(last.waste.isotopes.contains_key("Pu-239"));
        assert!(last.waste.isotopes.contains_key("Am-241"));
        let cs = &last.waste.isotopes["Cs-137"];
        assert!(cs.mass_kg > 0.0);
        assert!(cs.activity_bq > 0.0);
    }

    // ── Conservation of mass/energy ──

    #[test]
    fn energy_conservation_across_steps() {
        let result = run_simulation(&natrium_params()).unwrap();
        for step in &result.steps {
            if step.fuel.effective_thermal_power_mw < 1e-6 {
                continue;
            }
            let expected_electric = step.fuel.effective_thermal_power_mw * step.power.efficiency;
            let rel_err = (step.power.electric_power_mw - expected_electric).abs()
                / expected_electric.max(1e-12);
            assert!(
                rel_err < 1e-6,
                "Energy not conserved at step {}: electric={:.4} expected={:.4}",
                step.time_step, step.power.electric_power_mw, expected_electric
            );
        }
    }

    #[test]
    fn coolant_energy_balance() {
        // Verify Q = m_dot * Cp(T_avg) * delta_T matches thermal power
        let result = run_simulation(&natrium_params()).unwrap();
        for step in &result.steps {
            if step.fuel.effective_thermal_power_mw < 1e-6 {
                continue;
            }
            let avg_temp = (step.fluid.inlet_temp_c + step.fluid.outlet_temp_c) / 2.0;
            let cp = fluid::specific_heat(CoolantType::Sodium, avg_temp);
            let q_mw = step.fluid.flow_rate_kg_s * cp * step.fluid.delta_t_c / 1000.0;
            let rel_err = (q_mw - step.fuel.effective_thermal_power_mw).abs()
                / step.fuel.effective_thermal_power_mw.max(1e-12);
            assert!(
                rel_err < 0.01,
                "Coolant energy balance at step {}: fluid_q={:.4} MW, thermal={:.4} MW (Cp={:.4})",
                step.time_step, q_mw, step.fuel.effective_thermal_power_mw, cp
            );
        }
    }

    // ── Time step stability ──

    #[test]
    fn time_step_stability() {
        let mut params_coarse = natrium_params();
        params_coarse.time_step_days = 30.0;
        let mut params_fine = natrium_params();
        params_fine.time_step_days = 7.0;

        let result_coarse = run_simulation(&params_coarse).unwrap();
        let result_fine = run_simulation(&params_fine).unwrap();

        let burnup_coarse = result_coarse.steps.last().unwrap().fuel.burnup_gwd_t;
        let burnup_fine = result_fine.steps.last().unwrap().fuel.burnup_gwd_t;
        let rel_burnup = (burnup_coarse - burnup_fine).abs() / burnup_fine;
        assert!(
            rel_burnup < 0.05,
            "Burnup diverges: coarse={burnup_coarse:.4}, fine={burnup_fine:.4}, rel={rel_burnup:.4}"
        );

        let dt_coarse = params_coarse.time_step_days / 365.25;
        let energy_coarse: f64 = result_coarse.steps.iter()
            .map(|s| s.power.electric_power_mw * dt_coarse)
            .sum();
        let dt_fine = params_fine.time_step_days / 365.25;
        let energy_fine: f64 = result_fine.steps.iter()
            .map(|s| s.power.electric_power_mw * dt_fine)
            .sum();
        let rel_energy = (energy_coarse - energy_fine).abs() / energy_fine;
        assert!(
            rel_energy < 0.05,
            "Energy diverges: coarse={energy_coarse:.4}, fine={energy_fine:.4}, rel={rel_energy:.4}"
        );
    }

    // ── Efficiency bounded by Carnot ──

    #[test]
    fn natrium_efficiency_bounded_by_carnot() {
        let result = run_simulation(&natrium_params()).unwrap();
        let t_cold = 35.0 + 273.15;
        for step in &result.steps {
            if step.fuel.effective_thermal_power_mw < 1e-6 {
                continue;
            }
            let t_hot = step.fluid.outlet_temp_c + 273.15;
            let carnot = 1.0 - (t_cold / t_hot);
            assert!(
                step.power.efficiency <= carnot + 1e-10,
                "Efficiency {:.4} exceeds Carnot limit {:.4} at step {}",
                step.power.efficiency, carnot, step.time_step
            );
        }
    }

    #[test]
    fn xe100_efficiency_bounded_by_carnot() {
        let result = run_simulation(&xe100_params()).unwrap();
        let t_cold = 35.0 + 273.15;
        for step in &result.steps {
            if step.fuel.effective_thermal_power_mw < 1e-6 {
                continue;
            }
            let t_hot = step.fluid.outlet_temp_c + 273.15;
            let carnot = 1.0 - (t_cold / t_hot);
            assert!(
                step.power.efficiency <= carnot + 1e-10,
                "Efficiency {:.4} exceeds Carnot limit {:.4} at step {}",
                step.power.efficiency, carnot, step.time_step
            );
        }
    }

    // ── SCO2 Brayton integration ──

    #[test]
    fn msr_sco2_brayton_runs() {
        let params = SimulationParams {
            duration_years: 2.0,
            time_step_days: 30.0,
            initial_heavy_metal_tonnes: 30.0,
            enrichment_pct: 7.0,
            target_burnup_gwd_t: 100.0,
            thermal_power_mw: 400.0,
            breeding_ratio: 0.0,
            coolant_type: CoolantType::FLiBe,
            coolant_inlet_temp_c: 600.0,
            coolant_flow_rate_kg_s: 1350.0,
            cycle_type: CycleType::SCO2Brayton,
            rated_electric_power_mw: 185.0,
        };
        let result = run_simulation(&params).unwrap();
        let first = &result.steps[0];
        // FLiBe outlet ~700 °C, SCO2 at ~700 °C → ~46% efficiency
        assert!(first.power.efficiency > 0.40 && first.power.efficiency < 0.50,
            "MSR sCO2 efficiency = {}", first.power.efficiency);
        assert!(first.fluid.outlet_temp_c > 650.0 && first.fluid.outlet_temp_c < 800.0,
            "FLiBe outlet = {}", first.fluid.outlet_temp_c);
    }
}
