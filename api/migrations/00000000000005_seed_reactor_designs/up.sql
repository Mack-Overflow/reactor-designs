INSERT INTO reactor_designs (name, design_type, vendor, thermal_power_mw, electric_power_mw, coolant_type, moderator, fuel_type, enrichment_pct, design_metadata, source_url) VALUES
(
    'Aurora Powerhouse',
    'EBR-II',
    'Oklo Inc.',
    62.5,
    75,
    'Liquid Sodium',
    'None',
    'HALEU',
    67.0,
    '{
        "integral_design": true,
        "reactor_type": "fast_spectrum_sodium_cooled",
        "fuel_type": "metallic_U-Pu-Zr",
        "fuel_enrichment_percent": 67,
        "core_life_years": null,
        "refueling_interval_months": 6,
        "online_refueling": false,
        "sealed_core": false,
        "breeding_ratio": 1.2,
        "fuel_cycle": {
            "closed_cycle": true,
            "on_site_reprocessing": true,
            "reprocessing_method": "pyroprocessing"
        },
        "coolant": "liquid_sodium",
        "moderator": null,
        "neutron_spectrum": "fast",
        "passive_safety": {
            "negative_reactivity_coefficients": true,
            "natural_circulation_decay_heat_removal": true
        }
    }',
    'https://oklo.com/technology/default.aspx'
),
(
    'Natrium SFR',
    'SFR',
    'TerraPower',
    840,
    345,
    'Sodium',
    'None',
    'Metallic HALEU',
    15.0,
    '{"thermal_storage": true, "storage_capacity_mwh": 1500, "refueling_interval_months": 18}',
    'https://www.terrapower.com/natrium-technology/'
),
(
    'Xe-100 HTGR',
    'HTGR',
    'X-energy',
    200,
    80,
    'Helium',
    'Graphite',
    'TRISO UCO',
    15.5,
    '{"pebble_bed": true, "outlet_temp_c": 750, "modules_per_plant": 4}',
    'https://x-energy.com/reactors/xe-100'
),
(
    'IMSR-400',
    'MSR',
    'Terrestrial Energy',
    400,
    195,
    'Molten FLiBe salt',
    'Graphite',
    'Low-enriched UF4 in salt',
    4.95,
    '{"integral_design": true, "core_life_years": 7, "online_refueling": false}',
    'https://www.terrestrialenergy.com/technology/imsr/'
),
(
    'Kairos KP-FHR',
    'FHR',
    'Kairos Power',
    320,
    140,
    'Molten FLiBe salt',
    'Graphite',
    'TRISO',
    19.75,
    '{"pebble_bed": true, "outlet_temp_c": 650, "natural_circulation": true}',
    'https://kairospower.com/technology/'
),
(
    'BWRX-300',
    'BWR',
    'GE Hitachi',
    870,
    300,
    'Light water',
    'Light water',
    'UO2',
    4.95,
    '{"natural_circulation": true, "passive_safety": true, "design_simplification_pct": 50}',
    'https://www.gevernova.com/nuclear/carbon-free-power/bwrx-300-small-modular-reactor'
),
(
    'LFR-AS-200',
    'LFR',
    'Westinghouse',
    500,
    200,
    'Lead',
    'None',
    'UO2/MOX',
    11.5,
    '{"pool_type": true, "outlet_temp_c": 510, "seismic_isolation": true}',
    'https://www.westinghousenuclear.com/energy-systems/lead-cooled-fast-reactor'
);
