export interface ReactorDesign {
	id: string;
	name: string;
	design_type: string;
	vendor: string | null;
	thermal_power_mw: number | null;
	electric_power_mw: number | null;
	coolant_type: string | null;
	moderator: string | null;
	fuel_type: string | null;
	enrichment_pct: number | null;
	design_metadata: Record<string, unknown> | null;
	source_url: string | null;
	created_at: string;
	updated_at: string;
}

export interface SimulationRun {
	id: string;
	reactor_id: string;
	status: string;
	params: SimulationParams | null;
	started_at: string | null;
	completed_at: string | null;
	error_message: string | null;
}

export interface SimulationParams {
	duration_years: number;
	time_step_days: number;
	initial_heavy_metal_tonnes: number;
	enrichment_pct: number;
	target_burnup_gwd_t: number;
	thermal_power_mw: number;
	breeding_ratio: number;
	coolant_type: string;
	coolant_inlet_temp_c: number;
	coolant_flow_rate_kg_s: number;
	cycle_type: 'Rankine' | 'Brayton' | 'SCO2Brayton';
	rated_electric_power_mw: number;
}

export interface SimulationResult {
	id: string;
	run_id: string;
	time_step: number;
	time_years: number | null;
	fuel_burnup_gwd_t: number | null;
	fuel_remaining_pct: number | null;
	coolant_temp_inlet_c: number | null;
	coolant_temp_outlet_c: number | null;
	coolant_flow_rate_kg_s: number | null;
	thermal_power_mw: number | null;
	electric_power_mw: number | null;
	capacity_factor: number | null;
	waste_actinides_kg: number | null;
	waste_fission_products_kg: number | null;
	waste_total_activity_bq: number | null;
	extra_data: Record<string, unknown> | null;
}

export interface WasteIsotope {
	id: string;
	result_id: string;
	isotope: string;
	mass_kg: number | null;
	activity_bq: number | null;
	half_life_years: number | null;
}

export interface LaunchResponse {
	id: string;
	status: string;
	total_steps: number;
	average_capacity_factor: number;
}

export interface CompareEntry {
	run_id: string;
	reactor_name: string;
	reactor_design_type: string;
	results: SimulationResult[];
}

export interface IngestResponse {
	imported: number;
	failed: number;
	errors: string[];
	reactors: ReactorDesign[];
}
