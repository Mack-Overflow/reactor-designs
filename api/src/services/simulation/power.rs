use serde::{Deserialize, Serialize};

// Thermodynamic cycle type for power conversion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CycleType {
    // Steam (Rankine) — LWR, SFR, LFR
    Rankine,
    // Gas turbine (Brayton) — HTGR, KP-FHR
    Brayton,
    // Supercritical CO₂ Brayton — MSR, advanced SFR/LFR
    SCO2Brayton,
}

// Heat sink (condenser / environment) temperature for Carnot bound.
const HEAT_SINK_TEMP_C: f64 = 35.0;

// Inputs for the power generation module at a single time step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    // Instantaneous thermal power output (MW_th).
    pub thermal_power_mw: f64,
    // Coolant outlet temperature (°C) — determines cycle efficiency.
    pub coolant_outlet_temp_c: f64,
    // Thermodynamic cycle.
    pub cycle_type: CycleType,
    // Rated (nameplate) electric capacity (MW_e) for capacity-factor calculation.
    pub rated_electric_power_mw: f64,
}

// Output of a single power-generation calculation step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerOutput {
    // Thermal-to-electric conversion efficiency (0.0–1.0).
    pub efficiency: f64,
    // Gross electric power produced this step (MW_e).
    pub electric_power_mw: f64,
    // Capacity factor relative to rated electric output (0.0–1.0).
    pub capacity_factor: f64,
}

// ── Cycle efficiency models ──
//
// Each uses a piecewise-linear fit to published data, then is capped by Carnot.

// Rankine (steam): fitted to LWR / SFR / LFR data.
//   ≤ 260 °C → 30 % (saturated-steam LWR floor)
//   260–550 °C → linear 30 %–42 % (superheated / supercritical steam)
//   > 550 °C → 42 % ceiling (materials limit)
fn rankine_efficiency(outlet_temp_c: f64) -> f64 {
    const T_LOW: f64 = 260.0;
    const T_HIGH: f64 = 550.0;
    const EFF_LOW: f64 = 0.30;
    const EFF_HIGH: f64 = 0.42;

    if outlet_temp_c <= T_LOW {
        EFF_LOW
    } else if outlet_temp_c >= T_HIGH {
        EFF_HIGH
    } else {
        EFF_LOW + (EFF_HIGH - EFF_LOW) * (outlet_temp_c - T_LOW) / (T_HIGH - T_LOW)
    }
}

// Brayton (gas-turbine): fitted to helium-cooled HTGR / open-air turbine data.
//   ≤ 600 °C → 35 % (low-end recuperated Brayton)
//   600–950 °C → linear 35 %–48 %
//   > 950 °C → 48 % ceiling (turbine materials limit)
fn brayton_efficiency(outlet_temp_c: f64) -> f64 {
    const T_LOW: f64 = 600.0;
    const T_HIGH: f64 = 950.0;
    const EFF_LOW: f64 = 0.35;
    const EFF_HIGH: f64 = 0.48;

    if outlet_temp_c <= T_LOW {
        EFF_LOW
    } else if outlet_temp_c >= T_HIGH {
        EFF_HIGH
    } else {
        EFF_LOW + (EFF_HIGH - EFF_LOW) * (outlet_temp_c - T_LOW) / (T_HIGH - T_LOW)
    }
}

// Supercritical CO₂ Brayton: higher efficiency than steam at moderate temps.
//   ≤ 500 °C → 38 % (sCO₂ near critical point advantage)
//   500–750 °C → linear 38 %–48 %
//   > 750 °C → 48 % ceiling
fn sco2_brayton_efficiency(outlet_temp_c: f64) -> f64 {
    const T_LOW: f64 = 500.0;
    const T_HIGH: f64 = 750.0;
    const EFF_LOW: f64 = 0.38;
    const EFF_HIGH: f64 = 0.48;

    if outlet_temp_c <= T_LOW {
        EFF_LOW
    } else if outlet_temp_c >= T_HIGH {
        EFF_HIGH
    } else {
        EFF_LOW + (EFF_HIGH - EFF_LOW) * (outlet_temp_c - T_LOW) / (T_HIGH - T_LOW)
    }
}

// Carnot efficiency: theoretical ceiling for a heat engine.
fn carnot_efficiency(outlet_temp_c: f64) -> f64 {
    let t_hot_k = outlet_temp_c + 273.15;
    let t_cold_k = HEAT_SINK_TEMP_C + 273.15;
    if t_hot_k > t_cold_k {
        1.0 - t_cold_k / t_hot_k
    } else {
        0.0
    }
}

// Look up thermal-to-electric efficiency, bounded by Carnot limit.
pub fn thermal_efficiency(cycle: CycleType, outlet_temp_c: f64) -> f64 {
    let cycle_eff = match cycle {
        CycleType::Rankine => rankine_efficiency(outlet_temp_c),
        CycleType::Brayton => brayton_efficiency(outlet_temp_c),
        CycleType::SCO2Brayton => sco2_brayton_efficiency(outlet_temp_c),
    };
    // Enforce Carnot bound
    let carnot = carnot_efficiency(outlet_temp_c);
    cycle_eff.min(carnot)
}

// Run the power-generation calculation for a single simulation time step.
pub fn compute_power(cfg: &PowerConfig) -> PowerOutput {
    let efficiency = thermal_efficiency(cfg.cycle_type, cfg.coolant_outlet_temp_c);
    let electric_power_mw = cfg.thermal_power_mw * efficiency;
    let capacity_factor = if cfg.rated_electric_power_mw > 0.0 {
        (electric_power_mw / cfg.rated_electric_power_mw).min(1.0)
    } else {
        0.0
    };

    PowerOutput {
        efficiency,
        electric_power_mw,
        capacity_factor,
    }
}

// Compute the average capacity factor over an entire fuel cycle.
pub fn average_capacity_factor(step_electric_powers_mw: &[f64], rated_electric_power_mw: f64) -> f64 {
    if step_electric_powers_mw.is_empty() || rated_electric_power_mw <= 0.0 {
        return 0.0;
    }
    let sum: f64 = step_electric_powers_mw
        .iter()
        .map(|p| (p / rated_electric_power_mw).min(1.0))
        .sum();
    sum / step_electric_powers_mw.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Efficiency curve tests ──

    #[test]
    fn rankine_below_range() {
        let eff = thermal_efficiency(CycleType::Rankine, 200.0);
        assert!(eff > 0.28 && eff < 0.32);
    }

    #[test]
    fn rankine_above_range() {
        let eff = thermal_efficiency(CycleType::Rankine, 600.0);
        assert!(eff > 0.40 && eff < 0.44);
    }

    #[test]
    fn rankine_mid_range() {
        let eff = thermal_efficiency(CycleType::Rankine, 400.0);
        assert!(eff > 0.33 && eff < 0.39);
    }

    #[test]
    fn brayton_below_range() {
        let eff = thermal_efficiency(CycleType::Brayton, 600.0);
        assert!(eff > 0.33 && eff < 0.37);
    }

    #[test]
    fn brayton_above_range() {
        let eff = thermal_efficiency(CycleType::Brayton, 950.0);
        assert!(eff > 0.46 && eff < 0.50);
    }

    #[test]
    fn brayton_mid_range() {
        let eff = thermal_efficiency(CycleType::Brayton, 750.0);
        assert!(eff > 0.38 && eff < 0.44);
    }

    #[test]
    fn sco2_below_range() {
        let eff = thermal_efficiency(CycleType::SCO2Brayton, 450.0);
        assert!(eff > 0.36 && eff < 0.40,
            "SCO2 at 450°C should be ~38%, got {eff}");
    }

    #[test]
    fn sco2_mid_range() {
        let eff = thermal_efficiency(CycleType::SCO2Brayton, 625.0);
        assert!(eff > 0.40 && eff < 0.46,
            "SCO2 at 625°C should be ~43%, got {eff}");
    }

    #[test]
    fn sco2_above_range() {
        let eff = thermal_efficiency(CycleType::SCO2Brayton, 800.0);
        assert!(eff > 0.46 && eff < 0.50,
            "SCO2 at 800°C should be ~48%, got {eff}");
    }

    #[test]
    fn all_cycles_bounded_by_carnot() {
        let temps = [200.0, 400.0, 600.0, 800.0, 1000.0];
        for t in temps {
            let carnot = carnot_efficiency(t);
            for cycle in [CycleType::Rankine, CycleType::Brayton, CycleType::SCO2Brayton] {
                let eff = thermal_efficiency(cycle, t);
                assert!(eff <= carnot + 1e-10,
                    "{cycle:?} at {t}°C: eff {eff} exceeds Carnot {carnot}");
            }
        }
    }

    // ── compute_power tests ──

    #[test]
    fn natrium_sfr_power() {
        let cfg = PowerConfig {
            thermal_power_mw: 840.0,
            coolant_outlet_temp_c: 500.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 345.0,
        };
        let out = compute_power(&cfg);
        assert!(out.efficiency > 0.30 && out.efficiency < 0.42);
        assert!(out.electric_power_mw > 280.0 && out.electric_power_mw < 360.0);
        assert!(out.capacity_factor > 0.0 && out.capacity_factor <= 1.0);
    }

    #[test]
    fn xe100_htgr_power() {
        let cfg = PowerConfig {
            thermal_power_mw: 200.0,
            coolant_outlet_temp_c: 750.0,
            cycle_type: CycleType::Brayton,
            rated_electric_power_mw: 80.0,
        };
        let out = compute_power(&cfg);
        assert!(out.efficiency > 0.35 && out.efficiency < 0.48);
        assert!(out.electric_power_mw > 70.0 && out.electric_power_mw < 100.0);
    }

    #[test]
    fn msr_sco2_power() {
        let cfg = PowerConfig {
            thermal_power_mw: 400.0,
            coolant_outlet_temp_c: 700.0,
            cycle_type: CycleType::SCO2Brayton,
            rated_electric_power_mw: 185.0,
        };
        let out = compute_power(&cfg);
        assert!(out.efficiency > 0.40 && out.efficiency < 0.50,
            "MSR sCO2 efficiency = {}", out.efficiency);
        assert!(out.electric_power_mw > 160.0 && out.electric_power_mw < 200.0);
    }

    #[test]
    fn zero_rated_capacity_gives_zero_cf() {
        let cfg = PowerConfig {
            thermal_power_mw: 100.0,
            coolant_outlet_temp_c: 400.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 0.0,
        };
        let out = compute_power(&cfg);
        assert_eq!(out.capacity_factor, 0.0);
    }

    // ── average_capacity_factor tests ──

    #[test]
    fn average_cf_constant_full_power() {
        let powers = vec![345.0; 100];
        let avg = average_capacity_factor(&powers, 345.0);
        assert!((avg - 1.0).abs() < 1e-9);
    }

    #[test]
    fn average_cf_half_power() {
        let powers = vec![172.5; 50];
        let avg = average_capacity_factor(&powers, 345.0);
        assert!((avg - 0.5).abs() < 1e-9);
    }

    #[test]
    fn average_cf_empty() {
        assert_eq!(average_capacity_factor(&[], 345.0), 0.0);
    }

    #[test]
    fn average_cf_clamped_above_rated() {
        let powers = vec![400.0; 10];
        let avg = average_capacity_factor(&powers, 345.0);
        assert!((avg - 1.0).abs() < 1e-9);
    }
}
