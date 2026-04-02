use serde::{Deserialize, Serialize};

/// Thermodynamic cycle type for power conversion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CycleType {
    /// Steam
    Rankine,
    /// Helium or Super-crit co2
    Brayton,
}

/// Inputs for the power generation module at a single time step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    /// Instantaneous thermal power output (MW_th).
    pub thermal_power_mw: f64,
    /// Coolant outlet temperature (°C).
    pub coolant_outlet_temp_c: f64,
    /// Thermodynamic cycle.
    pub cycle_type: CycleType,
    /// Rated (nameplate) electric capacity (MW_e) for capacity-factor calculation.
    pub rated_electric_power_mw: f64,
}

/// Output of a single power-generation calculation step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerOutput {
    /// Thermal-to-electric conversion efficiency (0.0–1.0).
    pub efficiency: f64,
    /// Gross electric power produced this step (MW_e).
    pub electric_power_mw: f64,
    /// Capacity factor relative to rated electric output (0.0–1.0).
    pub capacity_factor: f64,
}

/// Compute Rankine (steam) cycle efficiency as a function of outlet temperature.
///
/// Uses a piecewise-linear model fitted to typical LWR / SFR / LFR data:
///   - ≤ 260 °C  →  30 % (saturated-steam LWR floor)
///   - 260–550 °C → linear 30 %–42 % (superheated / supercritical steam)
///   - > 550 °C   →  42 % ceiling (materials limit for steam plant)
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

/// Compute Brayton (gas-turbine) cycle efficiency as a function of outlet temperature.
///
/// Uses a piecewise-linear model fitted to helium-cooled HTGR data:
///   - ≤ 600 °C  →  35 % (low-end recuperated Brayton)
///   - 600–950 °C → linear 35 %–48 % (higher turbine inlet → higher Carnot fraction)
///   - > 950 °C   →  48 % ceiling (turbine materials limit)
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

/// Look up thermal-to-electric efficiency for the given cycle and outlet temperature.
pub fn thermal_efficiency(cycle: CycleType, outlet_temp_c: f64) -> f64 {
    match cycle {
        CycleType::Rankine => rankine_efficiency(outlet_temp_c),
        CycleType::Brayton => brayton_efficiency(outlet_temp_c),
    }
}

/// Run the power-generation calculation for a single simulation time step.
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

/// Compute the average capacity factor over an entire fuel cycle given a series of
/// per-step electric power outputs and the rated capacity.
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

    // ── compute_power tests ──

    #[test]
    fn natrium_sfr_power() {
        // Natrium: 840 MWth, sodium outlet ~500 °C, Rankine cycle, rated 345 MWe
        let cfg = PowerConfig {
            thermal_power_mw: 840.0,
            coolant_outlet_temp_c: 500.0,
            cycle_type: CycleType::Rankine,
            rated_electric_power_mw: 345.0,
        };
        let out = compute_power(&cfg);
        assert!(out.efficiency > 0.30 && out.efficiency < 0.42);
        assert!(out.electric_power_mw > 300.0 && out.electric_power_mw < 360.0);
        assert!(out.capacity_factor > 0.0 && out.capacity_factor <= 1.0);
    }

    #[test]
    fn xe100_htgr_power() {
        // Xe-100: 200 MWth, helium outlet 750 °C, Brayton cycle, rated 80 MWe
        let cfg = PowerConfig {
            thermal_power_mw: 200.0,
            coolant_outlet_temp_c: 750.0,
            cycle_type: CycleType::Brayton,
            rated_electric_power_mw: 80.0,
        };
        let out = compute_power(&cfg);
        assert!(out.efficiency > 0.35 && out.efficiency < 0.48);
        assert!(out.electric_power_mw > 70.0 && out.electric_power_mw < 100.0);
        assert!(out.capacity_factor > 0.0 && out.capacity_factor <= 1.0);
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
        // Even if electric output exceeds rated, CF is capped at 1.0
        let powers = vec![400.0; 10];
        let avg = average_capacity_factor(&powers, 345.0);
        assert!((avg - 1.0).abs() < 1e-9);
    }
}
