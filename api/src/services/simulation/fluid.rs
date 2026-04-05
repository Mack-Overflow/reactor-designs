use serde::{Deserialize, Serialize};

// Supported coolant types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CoolantType {
    Sodium,
    Lead,
    Helium,
    FLiBe,
    LightWater,
}

// Temperature-dependent specific heat capacity in kJ/(kg·K).
//
// Each coolant uses published correlations:
//   Sodium   – IAEA Nuclear Energy Series NF-T-1.7
//   Lead     – OECD/NEA Lead Handbook (correlation in Kelvin)
//   Helium   – monatomic ideal gas (constant Cp)
//   FLiBe    – Benes & Konings (2012), slight T dependence
//   Water    – IAPWS-IF97 approximation at PWR pressure (~15.5 MPa)
pub fn specific_heat(coolant: CoolantType, temp_c: f64) -> f64 {
    match coolant {
        CoolantType::Sodium => {
            // Cp = 1658 - 0.8479·T + 4.454e-4·T² (J/kg·K), T in °C
            // Valid ~100–1200 °C
            (1658.0 - 0.8479 * temp_c + 4.454e-4 * temp_c.powi(2)) / 1000.0
        }
        CoolantType::Lead => {
            // Cp = 175.1 - 4.961e-2·T_K + 1.985e-5·T_K² (J/kg·K)
            // Valid ~600–1300 K (~327–1027 °C)
            let t_k = temp_c + 273.15;
            (175.1 - 4.961e-2 * t_k + 1.985e-5 * t_k.powi(2)) / 1000.0
        }
        CoolantType::Helium => {
            // Essentially constant for monatomic ideal gas
            5.193
        }
        CoolantType::FLiBe => {
            // Cp ≈ 2386 J/kg·K at 700 °C, slight linear decrease with temperature
            (2386.0 - 0.12 * (temp_c - 700.0)) / 1000.0
        }
        CoolantType::LightWater => {
            // Approximate Cp at PWR pressure (~15.5 MPa).
            // Rises steeply approaching saturation (~345 °C).
            // ~4.5 kJ/kg·K at 280 °C, ~6.5 kJ/kg·K near 340 °C.
            let base = 4.5;
            let rise = 2.0 * ((temp_c - 280.0) / 60.0).clamp(0.0, 1.0).powi(2);
            base + rise
        }
    }
}

// Configuration for the fluid dynamics module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidConfig {
    pub coolant_type: CoolantType,
    // Coolant inlet temperature (°C).
    pub inlet_temp_c: f64,
    // Mass flow rate (kg/s).
    pub flow_rate_kg_s: f64,
}

// Output of a single fluid-dynamics calculation step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidOutput {
    pub inlet_temp_c: f64,
    pub outlet_temp_c: f64,
    pub flow_rate_kg_s: f64,
    pub delta_t_c: f64,
}

// Compute coolant temperature rise across the core for a single time step.
//
// Uses an iterative solve because Cp depends on temperature:
//   ΔT = Q / (ṁ × Cp(T_avg))
//   T_avg = (T_in + T_out) / 2
//   T_out = T_in + ΔT
//
// Converges in 5–10 iterations for all coolant types.
pub fn compute_fluid(cfg: &FluidConfig, thermal_power_mw: f64) -> FluidOutput {
    if cfg.flow_rate_kg_s <= 0.0 || thermal_power_mw <= 0.0 {
        return FluidOutput {
            inlet_temp_c: cfg.inlet_temp_c,
            outlet_temp_c: cfg.inlet_temp_c,
            flow_rate_kg_s: cfg.flow_rate_kg_s,
            delta_t_c: 0.0,
        };
    }

    let q_kw = thermal_power_mw * 1000.0;

    // Initial guess using inlet Cp
    let cp_inlet = specific_heat(cfg.coolant_type, cfg.inlet_temp_c);
    let mut t_out = cfg.inlet_temp_c + q_kw / (cfg.flow_rate_kg_s * cp_inlet);

    // Iterate to converge on outlet temperature
    for _ in 0..10 {
        let avg_temp = (cfg.inlet_temp_c + t_out) / 2.0;
        let cp = specific_heat(cfg.coolant_type, avg_temp);
        if cp <= 0.0 {
            break;
        }
        t_out = cfg.inlet_temp_c + q_kw / (cfg.flow_rate_kg_s * cp);
    }

    let delta_t = t_out - cfg.inlet_temp_c;

    FluidOutput {
        inlet_temp_c: cfg.inlet_temp_c,
        outlet_temp_c: t_out,
        flow_rate_kg_s: cfg.flow_rate_kg_s,
        delta_t_c: delta_t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sodium_sfr_delta_t() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::Sodium,
            inlet_temp_c: 350.0,
            flow_rate_kg_s: 4400.0,
        };
        let out = compute_fluid(&cfg, 840.0);
        // With temperature-dependent Cp (~1.38 kJ/kg·K at avg ~420 °C):
        // ΔT ≈ 840_000 / (4400 * 1.38) ≈ 138 °C → outlet ~488 °C
        assert!(out.delta_t_c > 100.0 && out.delta_t_c < 200.0,
            "sodium delta_t={}", out.delta_t_c);
        assert!(out.outlet_temp_c > 430.0 && out.outlet_temp_c < 560.0,
            "sodium outlet={}", out.outlet_temp_c);
    }

    #[test]
    fn helium_htgr_delta_t() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::Helium,
            inlet_temp_c: 260.0,
            flow_rate_kg_s: 80.0,
        };
        let out = compute_fluid(&cfg, 200.0);
        // ΔT = 200_000 / (80 * 5.193) ≈ 481 °C → outlet ~741 °C
        assert!(out.delta_t_c > 400.0 && out.delta_t_c < 600.0,
            "helium delta_t={}", out.delta_t_c);
        assert!(out.outlet_temp_c > 650.0 && out.outlet_temp_c < 850.0,
            "helium outlet={}", out.outlet_temp_c);
    }

    #[test]
    fn flibe_fhr_delta_t() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::FLiBe,
            inlet_temp_c: 550.0,
            flow_rate_kg_s: 1350.0,
        };
        let out = compute_fluid(&cfg, 320.0);
        // Cp ~2.40 kJ/kg·K at ~600 °C:
        // ΔT = 320_000 / (1350 * 2.40) ≈ 98.8 °C → outlet ~649 °C
        assert!(out.delta_t_c > 50.0 && out.delta_t_c < 200.0,
            "flibe delta_t={}", out.delta_t_c);
        assert!(out.outlet_temp_c > 600.0 && out.outlet_temp_c < 750.0,
            "flibe outlet={}", out.outlet_temp_c);
    }

    #[test]
    fn lead_lfr_delta_t() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::Lead,
            inlet_temp_c: 400.0,
            flow_rate_kg_s: 25000.0,
        };
        let out = compute_fluid(&cfg, 500.0);
        // Lead Cp ~0.147 kJ/kg·K → ΔT = 500_000 / (25000 * 0.147) ≈ 136 °C
        assert!(out.delta_t_c > 80.0 && out.delta_t_c < 200.0,
            "lead delta_t={}", out.delta_t_c);
        assert!(out.outlet_temp_c > 480.0 && out.outlet_temp_c < 600.0,
            "lead outlet={}", out.outlet_temp_c);
    }

    #[test]
    fn light_water_pwr_delta_t() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::LightWater,
            inlet_temp_c: 280.0,
            flow_rate_kg_s: 15000.0,
        };
        let out = compute_fluid(&cfg, 870.0);
        // Cp ~4.5 kJ/kg·K at 280 °C (rising near saturation):
        // ΔT ≈ 870_000 / (15000 * 4.6) ≈ 12.6 °C
        assert!(out.delta_t_c > 5.0 && out.delta_t_c < 30.0,
            "water delta_t={}", out.delta_t_c);
    }

    #[test]
    fn temperature_dependent_cp_differs_from_constant() {
        // Sodium at 350 °C vs 550 °C should have different Cp
        let cp_low = specific_heat(CoolantType::Sodium, 350.0);
        let cp_high = specific_heat(CoolantType::Sodium, 550.0);
        assert!((cp_low - cp_high).abs() > 0.01,
            "Cp should vary with temperature: cp_350={cp_low}, cp_550={cp_high}");
    }

    #[test]
    fn zero_flow_no_panic() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::Sodium,
            inlet_temp_c: 350.0,
            flow_rate_kg_s: 0.0,
        };
        let out = compute_fluid(&cfg, 840.0);
        assert_eq!(out.delta_t_c, 0.0);
        assert_eq!(out.outlet_temp_c, 350.0);
    }

    #[test]
    fn zero_power_no_heating() {
        let cfg = FluidConfig {
            coolant_type: CoolantType::LightWater,
            inlet_temp_c: 280.0,
            flow_rate_kg_s: 5000.0,
        };
        let out = compute_fluid(&cfg, 0.0);
        assert_eq!(out.delta_t_c, 0.0);
        assert_eq!(out.outlet_temp_c, 280.0);
    }
}
