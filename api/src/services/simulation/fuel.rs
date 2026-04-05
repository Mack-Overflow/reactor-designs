use serde::{Deserialize, Serialize};

// ── Physical constants ──

// Energy released per fission event: ~200 MeV → joules.
const ENERGY_PER_FISSION_J: f64 = 200.0e6 * 1.602_176_634e-19; // 3.204e-11 J

// Approximate mass of fissile material consumed per GWd of thermal energy.
// 1 GWd ≈ 86400 GJ; at ~200 MeV/fission and ~235 amu/atom → ~1.05 kg consumed.
const FISSILE_CONSUMED_KG_PER_GWD: f64 = 1.05;

// Reactor shuts down (subcritical) when fissile fraction drops below this.
const SHUTDOWN_FISSILE_FRACTION: f64 = 0.02;

// Slight power degradation coefficient as fissile depletes.
// At fissile_fraction = 0.5, power is reduced by ~2.5%.
const POWER_DEGRADATION_COEFF: f64 = 0.05;

// Configuration for the fuel cycle module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelConfig {
    // Initial heavy-metal loading in the core (metric tons).
    pub initial_heavy_metal_tonnes: f64,
    // U-235 enrichment percentage (e.g. 15.0 for 15 %).
    pub enrichment_pct: f64,
    // Target discharge burnup (GWd/t).
    pub target_burnup_gwd_t: f64,
    // Rated thermal power (MW_th).
    pub thermal_power_mw: f64,
    // Breeding ratio (> 1.0 for breeders, ~0 for non-breeders).
    pub breeding_ratio: f64,
}

// Output of a single fuel-cycle time step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelOutput {
    // Cumulative burnup so far (GWd/t).
    pub burnup_gwd_t: f64,
    // Incremental burnup this step (GWd — total, not per tonne).
    pub burnup_delta_gwd: f64,
    // Remaining fissile fraction (0.0–1.0).
    pub fuel_remaining_pct: f64,
    // Actual thermal power this step (MW_th).
    pub effective_thermal_power_mw: f64,
    // Instantaneous fission rate (fissions/second).
    pub fission_rate_per_s: f64,
    // Current fissile fraction relative to initial loading (0.0–1.0).
    pub fissile_fraction: f64,
}

// Mutable state carried across time steps.
#[derive(Debug, Clone)]
pub struct FuelState {
    // Accumulated burnup in GWd/t.
    pub cumulative_burnup_gwd_t: f64,
    // Current fissile mass (kg).
    pub fissile_mass_kg: f64,
    // Initial fissile mass (kg) — set once at construction.
    pub initial_fissile_mass_kg: f64,
    // Whether the reactor has permanently shut down (subcritical).
    pub shutdown: bool,
}

impl FuelState {
    pub fn new(cfg: &FuelConfig) -> Self {
        let initial_fissile =
            cfg.enrichment_pct / 100.0 * cfg.initial_heavy_metal_tonnes * 1000.0;
        Self {
            cumulative_burnup_gwd_t: 0.0,
            fissile_mass_kg: initial_fissile,
            initial_fissile_mass_kg: initial_fissile,
            shutdown: false,
        }
    }
}

// Run one fuel-cycle time step.
//
// Physics model:
//   1. Compute fissile fraction = current_fissile / initial_fissile.
//   2. If fraction < SHUTDOWN_FISSILE_FRACTION, reactor is subcritical → zero power.
//   3. Otherwise, power degrades slightly: P_eff = P_rated * (1 - k*(1 - frac)).
//   4. Burnup accrues: dB = P_eff * dt / M_hm, capped at target burnup.
//   5. Fissile inventory updated: consumed = dE * 1.05 kg/GWd, bred = consumed * BR.
//   6. Fission rate derived from effective power: R = P / E_per_fission.
pub fn step_fuel(
    state: &mut FuelState,
    cfg: &FuelConfig,
    dt_years: f64,
) -> FuelOutput {
    let dt_days = dt_years * 365.25;

    // Current fissile fraction
    let fissile_fraction = if state.initial_fissile_mass_kg > 0.0 {
        (state.fissile_mass_kg / state.initial_fissile_mass_kg).max(0.0)
    } else {
        0.0
    };

    // Check shutdown: once subcritical, stays off
    if state.shutdown || fissile_fraction < SHUTDOWN_FISSILE_FRACTION {
        state.shutdown = true;
        return FuelOutput {
            burnup_gwd_t: state.cumulative_burnup_gwd_t,
            burnup_delta_gwd: 0.0,
            fuel_remaining_pct: fissile_fraction,
            effective_thermal_power_mw: 0.0,
            fission_rate_per_s: 0.0,
            fissile_fraction,
        };
    }

    // Reactivity-limited power: slight degradation as fissile depletes
    let power_factor =
        (1.0 - POWER_DEGRADATION_COEFF * (1.0 - fissile_fraction)).clamp(0.0, 1.0);
    let effective_power = cfg.thermal_power_mw * power_factor;

    // Burnup rate in GWd/t per day at current power
    let burnup_rate = if cfg.initial_heavy_metal_tonnes > 0.0 {
        effective_power / (cfg.initial_heavy_metal_tonnes * 1000.0)
    } else {
        0.0
    };

    // Remaining capacity before hitting target
    let remaining_burnup = (cfg.target_burnup_gwd_t - state.cumulative_burnup_gwd_t).max(0.0);

    let potential_burnup = burnup_rate * dt_days;
    let actual_burnup = potential_burnup.min(remaining_burnup);

    // Power derate if we'd overshoot burnup limit within this step
    let power_derate = if potential_burnup > 0.0 {
        (actual_burnup / potential_burnup).min(1.0)
    } else {
        0.0
    };
    let final_thermal_power = effective_power * power_derate;

    // Update cumulative burnup
    state.cumulative_burnup_gwd_t += actual_burnup;

    // Total thermal energy this step (GWd)
    let total_burnup_gwd = actual_burnup * cfg.initial_heavy_metal_tonnes;

    // Fissile inventory: consumption and breeding
    let fissile_consumed = total_burnup_gwd * FISSILE_CONSUMED_KG_PER_GWD;
    let fissile_bred = fissile_consumed * cfg.breeding_ratio;
    state.fissile_mass_kg = (state.fissile_mass_kg - fissile_consumed + fissile_bred).max(0.0);

    // Updated fissile fraction after this step
    let new_fissile_fraction = if state.initial_fissile_mass_kg > 0.0 {
        (state.fissile_mass_kg / state.initial_fissile_mass_kg).max(0.0)
    } else {
        0.0
    };

    // Fission rate from effective thermal power
    let fission_rate = if ENERGY_PER_FISSION_J > 0.0 {
        final_thermal_power * 1.0e6 / ENERGY_PER_FISSION_J
    } else {
        0.0
    };

    FuelOutput {
        burnup_gwd_t: state.cumulative_burnup_gwd_t,
        burnup_delta_gwd: total_burnup_gwd,
        fuel_remaining_pct: new_fissile_fraction,
        effective_thermal_power_mw: final_thermal_power,
        fission_rate_per_s: fission_rate,
        fissile_fraction: new_fissile_fraction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn natrium_config() -> FuelConfig {
        FuelConfig {
            initial_heavy_metal_tonnes: 60.0,
            enrichment_pct: 15.0,
            target_burnup_gwd_t: 150.0,
            thermal_power_mw: 840.0,
            breeding_ratio: 0.8,
        }
    }

    #[test]
    fn burnup_increases_over_time() {
        let cfg = natrium_config();
        let mut state = FuelState::new(&cfg);
        let out = step_fuel(&mut state, &cfg, 1.0);
        assert!(out.burnup_gwd_t > 0.0);
        assert!(out.burnup_delta_gwd > 0.0);
    }

    #[test]
    fn fuel_depletes_over_cycle() {
        let cfg = natrium_config();
        let mut state = FuelState::new(&cfg);
        let mut last = step_fuel(&mut state, &cfg, 1.0);
        for _ in 1..10 {
            last = step_fuel(&mut state, &cfg, 1.0);
        }
        assert!(last.fuel_remaining_pct < 1.0);
        assert!(last.fuel_remaining_pct > 0.0);
    }

    #[test]
    fn fissile_fraction_tracks_depletion() {
        let cfg = FuelConfig {
            breeding_ratio: 0.0,
            ..natrium_config()
        };
        let mut state = FuelState::new(&cfg);
        let initial = state.fissile_mass_kg;
        step_fuel(&mut state, &cfg, 1.0);
        assert!(state.fissile_mass_kg < initial,
            "fissile should decrease with BR=0");
    }

    #[test]
    fn fission_rate_proportional_to_power() {
        let cfg = natrium_config();
        let mut state = FuelState::new(&cfg);
        let out = step_fuel(&mut state, &cfg, 0.01);
        // At ~840 MW: R = P / E_fission = 840e6 / 3.204e-11 ≈ 2.62e19 fissions/s
        assert!(out.fission_rate_per_s > 2.0e19,
            "fission rate too low: {:.3e}", out.fission_rate_per_s);
        assert!(out.fission_rate_per_s < 3.5e19,
            "fission rate too high: {:.3e}", out.fission_rate_per_s);
    }

    #[test]
    fn shutdown_when_fissile_exhausted() {
        // Small core, no breeding, aggressive burnup
        let cfg = FuelConfig {
            initial_heavy_metal_tonnes: 5.0,
            enrichment_pct: 5.0,
            target_burnup_gwd_t: 100.0,
            thermal_power_mw: 500.0,
            breeding_ratio: 0.0,
        };
        let mut state = FuelState::new(&cfg);
        // Initial fissile = 250 kg. Consumption ~1.05 kg/GWd, ~9.13 GWd/yr at 500 MW
        // → ~9.6 kg/yr consumed. Should shut down after ~25 years.
        let mut last = FuelOutput {
            burnup_gwd_t: 0.0,
            burnup_delta_gwd: 0.0,
            fuel_remaining_pct: 1.0,
            effective_thermal_power_mw: 0.0,
            fission_rate_per_s: 0.0,
            fissile_fraction: 1.0,
        };
        for _ in 0..50 {
            last = step_fuel(&mut state, &cfg, 1.0);
        }
        assert!(state.shutdown, "reactor should be shut down");
        assert!(last.effective_thermal_power_mw < 1.0);
        assert!(last.fission_rate_per_s == 0.0);
    }

    #[test]
    fn power_drops_at_target_burnup() {
        let cfg = FuelConfig {
            initial_heavy_metal_tonnes: 10.0,
            enrichment_pct: 5.0,
            target_burnup_gwd_t: 50.0,
            thermal_power_mw: 500.0,
            breeding_ratio: 0.0,
        };
        let mut state = FuelState::new(&cfg);
        for _ in 0..20 {
            step_fuel(&mut state, &cfg, 1.0);
        }
        let out = step_fuel(&mut state, &cfg, 1.0);
        // Either burnup target reached or fissile exhausted — power should be zero
        assert!(out.effective_thermal_power_mw < 1.0);
    }

    #[test]
    fn breeder_preserves_fuel() {
        let cfg_breeder = FuelConfig {
            breeding_ratio: 1.0,
            ..natrium_config()
        };
        let cfg_burner = FuelConfig {
            breeding_ratio: 0.0,
            ..natrium_config()
        };

        let mut state_b = FuelState::new(&cfg_breeder);
        let mut state_r = FuelState::new(&cfg_burner);

        for _ in 0..5 {
            step_fuel(&mut state_b, &cfg_breeder, 1.0);
            step_fuel(&mut state_r, &cfg_burner, 1.0);
        }
        let out_b = step_fuel(&mut state_b, &cfg_breeder, 1.0);
        let out_r = step_fuel(&mut state_r, &cfg_burner, 1.0);

        assert!(out_b.fuel_remaining_pct > out_r.fuel_remaining_pct,
            "breeder ({}) should have more fuel remaining than burner ({})",
            out_b.fuel_remaining_pct, out_r.fuel_remaining_pct);
    }

    #[test]
    fn breeder_ratio_one_conserves_fissile() {
        let cfg = FuelConfig {
            breeding_ratio: 1.0,
            ..natrium_config()
        };
        let mut state = FuelState::new(&cfg);
        let initial = state.fissile_mass_kg;
        for _ in 0..5 {
            step_fuel(&mut state, &cfg, 1.0);
        }
        let diff = (state.fissile_mass_kg - initial).abs();
        assert!(diff < 1.0,
            "BR=1.0 should roughly conserve fissile, diff={diff:.2} kg");
    }

    #[test]
    fn zero_heavy_metal_no_panic() {
        let cfg = FuelConfig {
            initial_heavy_metal_tonnes: 0.0,
            ..natrium_config()
        };
        let mut state = FuelState::new(&cfg);
        let out = step_fuel(&mut state, &cfg, 1.0);
        assert_eq!(out.burnup_gwd_t, 0.0);
        assert_eq!(out.effective_thermal_power_mw, 0.0);
    }

    #[test]
    fn power_degrades_slightly_with_depletion() {
        let cfg = FuelConfig {
            breeding_ratio: 0.0,
            ..natrium_config()
        };
        let mut state = FuelState::new(&cfg);
        let out_first = step_fuel(&mut state, &cfg, 0.1);
        for _ in 0..30 {
            step_fuel(&mut state, &cfg, 1.0);
        }
        if !state.shutdown {
            let out_late = step_fuel(&mut state, &cfg, 0.1);
            assert!(out_late.effective_thermal_power_mw < out_first.effective_thermal_power_mw,
                "power should degrade as fuel depletes");
        }
    }
}
