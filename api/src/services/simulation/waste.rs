use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const LN2: f64 = std::f64::consts::LN_2;
const AMU_KG: f64 = 1.660_539_07e-27;

// ── Isotope metadata ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsotopeInfo {
    pub half_life_years: f64,
    // Cumulative fission yield (atoms per fission) for fission products.
    // Zero for actinides (produced via capture, not fission).
    pub fission_yield: f64,
    // Base capture-production rate (atoms per fission) for actinides.
    // Scaled by breeding_ratio factor in the step function.
    // Zero for fission products.
    pub capture_yield_base: f64,
    // Mass number (e.g. 137 for Cs-137).
    pub mass_number: u32,
    // Daughter isotope from decay (if tracked).
    pub daughter: Option<String>,
    pub is_actinide: bool,
}

// Per-isotope state exposed per time step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsotopeState {
    pub mass_kg: f64,
    pub activity_bq: f64,
    pub half_life_years: f64,
}

// Aggregate waste output for a single time step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteOutput {
    pub isotopes: HashMap<String, IsotopeState>,
    pub total_actinides_kg: f64,
    pub total_fission_products_kg: f64,
    pub total_activity_bq: f64,
}

// Configuration for the waste module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteConfig {
    pub isotope_library: HashMap<String, IsotopeInfo>,
}

// Build the default isotope library with decay chains.
//
// Fission products: produced proportional to fission rate with cumulative yields.
// Actinides: produced via neutron capture chains, rate scaled by breeding ratio.
//
// Tracked decay chains:
//   Pu-241 →(β) Am-241 →(α) Np-237
//   Cm-244 →(α) Pu-240
pub fn default_isotope_library() -> HashMap<String, IsotopeInfo> {
    let mut lib = HashMap::new();

    // ── Fission products ──

    // Short-lived (dominate early cooling)
    lib.insert("I-131".into(), IsotopeInfo {
        half_life_years: 8.02 / 365.25,
        fission_yield: 0.029,
        capture_yield_base: 0.0,
        mass_number: 131,
        daughter: None, // Xe-131 (stable)
        is_actinide: false,
    });
    lib.insert("Xe-133".into(), IsotopeInfo {
        half_life_years: 5.243 / 365.25,
        fission_yield: 0.067,
        capture_yield_base: 0.0,
        mass_number: 133,
        daughter: None, // Cs-133 (stable)
        is_actinide: false,
    });
    lib.insert("Ba-140".into(), IsotopeInfo {
        half_life_years: 12.75 / 365.25,
        fission_yield: 0.062,
        capture_yield_base: 0.0,
        mass_number: 140,
        daughter: None, // La-140 → Ce-140 (fast chain)
        is_actinide: false,
    });
    lib.insert("Xe-135".into(), IsotopeInfo {
        half_life_years: 9.14 / 24.0 / 365.25,
        fission_yield: 0.065,
        capture_yield_base: 0.0,
        mass_number: 135,
        daughter: None, // Cs-135 (2.3 My, effectively stable here)
        is_actinide: false,
    });

    // Medium-lived (dominate 1–100 year storage)
    lib.insert("Cs-137".into(), IsotopeInfo {
        half_life_years: 30.17,
        fission_yield: 0.061,
        capture_yield_base: 0.0,
        mass_number: 137,
        daughter: None, // Ba-137m (fast)
        is_actinide: false,
    });
    lib.insert("Sr-90".into(), IsotopeInfo {
        half_life_years: 28.8,
        fission_yield: 0.058,
        capture_yield_base: 0.0,
        mass_number: 90,
        daughter: None, // Y-90 → Zr-90 (fast)
        is_actinide: false,
    });
    lib.insert("Kr-85".into(), IsotopeInfo {
        half_life_years: 10.756,
        fission_yield: 0.003,
        capture_yield_base: 0.0,
        mass_number: 85,
        daughter: None, // Rb-85 (stable)
        is_actinide: false,
    });

    // Long-lived
    lib.insert("Tc-99".into(), IsotopeInfo {
        half_life_years: 211_100.0,
        fission_yield: 0.061,
        capture_yield_base: 0.0,
        mass_number: 99,
        daughter: None,
        is_actinide: false,
    });

    // ── Actinides (produced by neutron capture, not fission) ──

    lib.insert("Pu-239".into(), IsotopeInfo {
        half_life_years: 24_110.0,
        fission_yield: 0.0,
        capture_yield_base: 0.80,  // primary capture product from U-238
        mass_number: 239,
        daughter: None, // U-235 (α, t½=24 ky — negligible on simulation timescale)
        is_actinide: true,
    });
    lib.insert("Pu-240".into(), IsotopeInfo {
        half_life_years: 6_561.0,
        fission_yield: 0.0,
        capture_yield_base: 0.05,  // from Pu-239 capture without fission
        mass_number: 240,
        daughter: None, // U-236 (α, very long-lived)
        is_actinide: true,
    });
    lib.insert("Pu-241".into(), IsotopeInfo {
        half_life_years: 14.35,
        fission_yield: 0.0,
        capture_yield_base: 0.03,  // from Pu-240 capture
        mass_number: 241,
        daughter: Some("Am-241".into()),  // β decay
        is_actinide: true,
    });
    lib.insert("Am-241".into(), IsotopeInfo {
        half_life_years: 432.2,
        fission_yield: 0.0,
        capture_yield_base: 0.0,  // produced only from Pu-241 decay
        mass_number: 241,
        daughter: Some("Np-237".into()),  // α decay
        is_actinide: true,
    });
    lib.insert("Np-237".into(), IsotopeInfo {
        half_life_years: 2.144e6,
        fission_yield: 0.0,
        capture_yield_base: 0.0,  // produced only from Am-241 decay
        mass_number: 237,
        daughter: None, // Pa-233 (very long chain)
        is_actinide: true,
    });
    lib.insert("Cm-244".into(), IsotopeInfo {
        half_life_years: 18.1,
        fission_yield: 0.0,
        capture_yield_base: 0.005,  // from successive capture chain
        mass_number: 244,
        daughter: Some("Pu-240".into()),  // α decay
        is_actinide: true,
    });

    lib
}

// Classify an isotope as actinide (true) or fission product (false).
fn is_actinide(name: &str) -> bool {
    let prefix = name.split('-').next().unwrap_or("");
    matches!(prefix, "Pu" | "Am" | "Cm" | "Np" | "Bk" | "Cf" | "U")
}

// Decay constant λ = ln(2) / t_half.
fn decay_constant_per_s(half_life_years: f64) -> f64 {
    if half_life_years <= 0.0 {
        return 0.0;
    }
    LN2 / (half_life_years * 365.25 * 86400.0)
}

// Convert atom count to mass in kg.
fn atoms_to_kg(atoms: f64, mass_number: u32) -> f64 {
    atoms * (mass_number as f64) * AMU_KG
}

// Activity in Bq = λ * N (decay constant × atom count).
fn activity_bq(atoms: f64, lambda: f64) -> f64 {
    atoms * lambda
}

// Perform one time-step update of the waste inventory using analytical Bateman solutions.
//
// For each isotope, the equation over interval dt with constant source rate S:
//   N(t+dt) = N(t)·exp(-λ·dt) + (S/λ)·(1 - exp(-λ·dt))
//
// For very long-lived / stable isotopes (λ → 0):
//   N(t+dt) = N(t) + S·dt
//
// Source rate S includes:
//   - Direct fission production (fission products)
//   - Neutron capture production (actinides, scaled by breeding ratio)
//   - Decay contribution from parent isotope
//
// Parameters:
//   - `current`: mutable isotope atom-count inventory.
//   - `fission_rate_per_s`: instantaneous fission rate from the fuel module.
//   - `breeding_ratio`: reactor breeding ratio (scales actinide production).
//   - `dt_seconds`: duration of this time step in seconds.
//   - `library`: isotope data.
pub fn step_waste(
    current: &mut HashMap<String, f64>,
    fission_rate_per_s: f64,
    breeding_ratio: f64,
    dt_seconds: f64,
    library: &HashMap<String, IsotopeInfo>,
) -> WasteOutput {
    // Snapshot beginning-of-step atom counts for parent decay contributions.
    let snapshot: HashMap<String, f64> = current.clone();

    // Actinide production scaling: even BR=0 reactors produce some actinides
    // from parasitic capture, but breeders produce much more.
    let actinide_scale = 0.3 + 0.7 * breeding_ratio.max(0.0);

    let mut isotopes = HashMap::new();
    let mut total_actinides_kg = 0.0;
    let mut total_fission_products_kg = 0.0;
    let mut total_activity_bq = 0.0;

    for (name, info) in library {
        let lambda = decay_constant_per_s(info.half_life_years);

        // Source term: direct production (atoms/second)
        let direct_production = if info.fission_yield > 0.0 {
            // Fission product
            fission_rate_per_s * info.fission_yield
        } else if info.capture_yield_base > 0.0 {
            // Actinide from capture
            fission_rate_per_s * info.capture_yield_base * actinide_scale
        } else {
            0.0
        };

        // Decay-chain contribution from parent isotopes.
        // Find all isotopes in the library whose daughter is this isotope.
        let parent_contribution: f64 = library.iter()
            .filter(|(_, p_info)| p_info.daughter.as_deref() == Some(name.as_str()))
            .map(|(p_name, p_info)| {
                let parent_atoms = snapshot.get(p_name).copied().unwrap_or(0.0);
                let p_lambda = decay_constant_per_s(p_info.half_life_years);
                // Decay rate of parent = λ_parent × N_parent
                p_lambda * parent_atoms
            })
            .sum();

        let source = direct_production + parent_contribution;

        // Current atom count
        let n_old = current.get(name).copied().unwrap_or(0.0);

        // Analytical Bateman solution for constant source rate over dt
        let exp_term = (-lambda * dt_seconds).exp();
        let n_new = if lambda > 1e-30 {
            n_old * exp_term + (source / lambda) * (1.0 - exp_term)
        } else {
            // Stable or extremely long-lived: just accumulate
            n_old + source * dt_seconds
        };

        let n_new = n_new.max(0.0);
        current.insert(name.clone(), n_new);

        // Convert to mass and activity
        let mass_kg = atoms_to_kg(n_new, info.mass_number);
        let act = activity_bq(n_new, lambda);

        if info.is_actinide || is_actinide(name) {
            total_actinides_kg += mass_kg;
        } else {
            total_fission_products_kg += mass_kg;
        }
        total_activity_bq += act;

        isotopes.insert(name.clone(), IsotopeState {
            mass_kg,
            activity_bq: act,
            half_life_years: info.half_life_years,
        });
    }

    WasteOutput {
        isotopes,
        total_actinides_kg,
        total_fission_products_kg,
        total_activity_bq,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_library() -> HashMap<String, IsotopeInfo> {
        default_isotope_library()
    }

    // Helper: typical fission rate for a 840 MW reactor
    fn natrium_fission_rate() -> f64 {
        840.0e6 / (200.0e6 * 1.602_176_634e-19)
    }

    #[test]
    fn zero_fission_rate_no_waste() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;
        let out = step_waste(&mut inv, 0.0, 0.8, dt_s, &lib);
        assert!(out.total_actinides_kg < 1e-15);
        assert!(out.total_fission_products_kg < 1e-15);
    }

    #[test]
    fn nonzero_fission_produces_waste() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;
        let out = step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        assert!(out.total_actinides_kg > 0.0);
        assert!(out.total_fission_products_kg > 0.0);
        assert!(out.total_activity_bq > 0.0);
    }

    #[test]
    fn higher_fission_rate_produces_more() {
        let lib = test_library();
        let dt_s = 30.0 * 86400.0;
        let mut inv_lo = HashMap::new();
        let mut inv_hi = HashMap::new();
        let out_lo = step_waste(&mut inv_lo, natrium_fission_rate() * 0.5, 0.8, dt_s, &lib);
        let out_hi = step_waste(&mut inv_hi, natrium_fission_rate(), 0.8, dt_s, &lib);
        let mass_lo: f64 = out_lo.isotopes.values().map(|s| s.mass_kg).sum();
        let mass_hi: f64 = out_hi.isotopes.values().map(|s| s.mass_kg).sum();
        assert!(mass_hi > mass_lo);
    }

    #[test]
    fn mass_accumulates_over_steps() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;
        let out1 = step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        let out2 = step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        let mass1: f64 = out1.isotopes.values().map(|s| s.mass_kg).sum();
        let mass2: f64 = out2.isotopes.values().map(|s| s.mass_kg).sum();
        assert!(mass2 > mass1, "mass should accumulate: step1={mass1}, step2={mass2}");
    }

    #[test]
    fn short_lived_reach_equilibrium() {
        // Xe-135 (t½ ~9h) should reach secular equilibrium within a few steps.
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 86400.0; // 1 day
        let rate = natrium_fission_rate();
        // Run many steps to reach equilibrium
        let mut xe135_mass = Vec::new();
        for _ in 0..30 {
            let out = step_waste(&mut inv, rate, 0.8, dt_s, &lib);
            xe135_mass.push(out.isotopes.get("Xe-135").unwrap().mass_kg);
        }
        // Last few values should be within 1% of each other (equilibrium)
        let last = xe135_mass[29];
        let prev = xe135_mass[25];
        let rel_diff = (last - prev).abs() / last;
        assert!(rel_diff < 0.01,
            "Xe-135 should reach equilibrium: day25={prev:.6e}, day30={last:.6e}");
    }

    #[test]
    fn short_lived_decay_after_shutdown() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;

        // Produce isotopes during operation
        for _ in 0..12 {
            step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        }
        let i131_during = inv.get("I-131").copied().unwrap_or(0.0);

        // Shut down: fission_rate = 0, only decay for 1 year
        step_waste(&mut inv, 0.0, 0.8, 365.25 * 86400.0, &lib);
        let i131_after = inv.get("I-131").copied().unwrap_or(0.0);

        // I-131 (t½=8d) should be almost gone after 1 year
        assert!(i131_after < i131_during * 0.001,
            "I-131 should decay after shutdown");
    }

    #[test]
    fn long_lived_isotopes_persist() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;

        // Produce Pu-239
        step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        let pu_after_prod = inv.get("Pu-239").copied().unwrap_or(0.0);

        // Wait 10 years with no production
        step_waste(&mut inv, 0.0, 0.8, 10.0 * 365.25 * 86400.0, &lib);
        let pu_after_wait = inv.get("Pu-239").copied().unwrap_or(0.0);

        // Pu-239 (t½=24110y) should barely decay in 10 years
        assert!(pu_after_wait > pu_after_prod * 0.99,
            "Pu-239 should persist: before={pu_after_prod:.3e}, after={pu_after_wait:.3e}");
    }

    #[test]
    fn decay_chain_pu241_to_am241() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;

        // Produce Pu-241 during operation
        for _ in 0..24 {
            step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        }
        let pu241_during = inv.get("Pu-241").copied().unwrap_or(0.0);
        assert!(pu241_during > 0.0, "Pu-241 should be produced");

        // Shut down and wait ~30 years (>2 half-lives of Pu-241)
        for _ in 0..30 {
            step_waste(&mut inv, 0.0, 0.8, 365.25 * 86400.0, &lib);
        }

        // Am-241 should have grown from Pu-241 decay
        let am241 = inv.get("Am-241").copied().unwrap_or(0.0);
        assert!(am241 > 0.0, "Am-241 should grow from Pu-241 decay");
    }

    #[test]
    fn decay_chain_cm244_to_pu240() {
        let lib = test_library();
        let mut inv = HashMap::new();
        let dt_s = 30.0 * 86400.0;

        // Produce Cm-244
        for _ in 0..24 {
            step_waste(&mut inv, natrium_fission_rate(), 0.8, dt_s, &lib);
        }
        let pu240_during = inv.get("Pu-240").copied().unwrap_or(0.0);

        // Wait ~50 years (several Cm-244 half-lives = 18.1y)
        for _ in 0..50 {
            step_waste(&mut inv, 0.0, 0.8, 365.25 * 86400.0, &lib);
        }

        // Pu-240 should have grown from Cm-244 decay
        let pu240_after = inv.get("Pu-240").copied().unwrap_or(0.0);
        assert!(pu240_after > pu240_during,
            "Pu-240 should grow from Cm-244 decay: during={pu240_during:.3e}, after={pu240_after:.3e}");
    }

    #[test]
    fn breeding_ratio_affects_actinide_production() {
        let lib = test_library();
        let dt_s = 30.0 * 86400.0;
        let rate = natrium_fission_rate();

        let mut inv_low = HashMap::new();
        let mut inv_high = HashMap::new();
        step_waste(&mut inv_low, rate, 0.0, dt_s, &lib);
        step_waste(&mut inv_high, rate, 1.0, dt_s, &lib);

        let pu_low = inv_low.get("Pu-239").copied().unwrap_or(0.0);
        let pu_high = inv_high.get("Pu-239").copied().unwrap_or(0.0);
        assert!(pu_high > pu_low,
            "Higher BR should produce more Pu-239: BR0={pu_low:.3e}, BR1={pu_high:.3e}");
    }

    #[test]
    fn actinide_classification() {
        assert!(is_actinide("Pu-239"));
        assert!(is_actinide("Am-241"));
        assert!(is_actinide("Cm-244"));
        assert!(is_actinide("Np-237"));
        assert!(!is_actinide("Cs-137"));
        assert!(!is_actinide("Sr-90"));
        assert!(!is_actinide("I-131"));
    }

    #[test]
    fn activity_sanity_check() {
        // Cs-137: known specific activity ~3.215 TBq/g = 3.215e15 Bq/kg
        // 1 kg → atoms = 1000 / 137 * 6.022e23 = 4.395e24 atoms
        // λ = ln2 / (30.17 * 365.25 * 86400) = 7.282e-10 /s
        // A = N * λ = 4.395e24 * 7.282e-10 = 3.20e15 Bq
        let atoms_1kg = 1.0 / (137.0 * AMU_KG);
        let lambda = decay_constant_per_s(30.17);
        let act = activity_bq(atoms_1kg, lambda);
        assert!(act > 3.0e15 && act < 3.5e15,
            "Cs-137 1 kg activity should be ~3.2e15 Bq, got {act:.3e}");
    }

    #[test]
    fn new_isotopes_tracked() {
        let lib = test_library();
        assert!(lib.contains_key("Xe-133"), "should track Xe-133");
        assert!(lib.contains_key("Ba-140"), "should track Ba-140");
        assert!(lib.contains_key("Kr-85"), "should track Kr-85");
        assert!(lib.contains_key("Pu-241"), "should track Pu-241");
        assert!(lib.contains_key("Np-237"), "should track Np-237");
        assert!(lib.contains_key("Pu-240"), "should track Pu-240");
    }
}
