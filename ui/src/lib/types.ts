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
