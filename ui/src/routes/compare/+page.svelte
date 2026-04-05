<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign, LaunchResponse, CompareEntry } from '$lib/types';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';

	let reactors = $state<ReactorDesign[]>([]);
	let loading = $state(true);
	let error = $state('');

	let selectedIds = $state<string[]>([]);
	let runMap = $state<Record<string, string>>({});
	let launching = $state(false);
	let comparing = $state(false);
	let compareData = $state<CompareEntry[] | null>(null);

	$effect(() => {
		apiFetch<ReactorDesign[]>('/api/reactors')
			.then((data) => (reactors = data))
			.catch((e) => (error = e.message))
			.finally(() => (loading = false));
	});

	function toggleReactor(id: string) {
		if (selectedIds.includes(id)) {
			selectedIds = selectedIds.filter((s) => s !== id);
		} else if (selectedIds.length < 4) {
			selectedIds = [...selectedIds, id];
		}
	}

	const COOLANT_MAP: Record<string, string> = {
		Sodium: 'Sodium', Lead: 'Lead', Helium: 'Helium',
		'Molten FLiBe salt': 'FLiBe', 'Light water': 'LightWater', 'Liquid Sodium': 'Sodium'
	};
	const CYCLE_MAP: Record<string, string> = {
		Sodium: 'Rankine', Lead: 'Rankine', Helium: 'Brayton', FLiBe: 'Rankine', LightWater: 'Rankine'
	};

	function defaultParams(r: ReactorDesign) {
		const coolantRaw = r.coolant_type ?? 'Sodium';
		const coolantType = COOLANT_MAP[coolantRaw] ?? 'Sodium';
		const cycleType = CYCLE_MAP[coolantType] ?? 'Rankine';
		const thermalMw = Number(r.thermal_power_mw ?? 100);
		const initialHm = thermalMw > 500 ? 60 : 10;
		let inletTemp = 350, flowRate = 4400;
		if (coolantType === 'Helium') { inletTemp = 260; flowRate = 80; }
		else if (coolantType === 'FLiBe') { inletTemp = 550; flowRate = 1350; }
		else if (coolantType === 'Lead') { inletTemp = 400; flowRate = 25000; }
		else if (coolantType === 'LightWater') { inletTemp = 280; flowRate = 5000; }
		return {
			duration_years: 5, time_step_days: 30,
			initial_heavy_metal_tonnes: initialHm,
			enrichment_pct: Number(r.enrichment_pct ?? 5),
			target_burnup_gwd_t: Number(r.enrichment_pct ?? 5) > 10 ? 150 : 60,
			thermal_power_mw: thermalMw,
			breeding_ratio: r.design_type === 'SFR' || r.design_type === 'LFR' ? 0.8 : 0.0,
			coolant_type: coolantType, coolant_inlet_temp_c: inletTemp,
			coolant_flow_rate_kg_s: flowRate, cycle_type: cycleType,
			rated_electric_power_mw: Number(r.electric_power_mw ?? 100)
		};
	}

	async function launchAndCompare() {
		launching = true; error = '';
		const newRunMap: Record<string, string> = {};
		try {
			for (const reactorId of selectedIds) {
				const reactor = reactors.find((r) => r.id === reactorId)!;
				const res = await apiFetch<LaunchResponse>('/api/simulations', {
					method: 'POST',
					body: JSON.stringify({ reactor_id: reactorId, params: defaultParams(reactor) })
				});
				newRunMap[reactorId] = res.id;
			}
			runMap = newRunMap;
			launching = false;
			comparing = true;
			const runIds = Object.values(newRunMap);
			compareData = await apiFetch<CompareEntry[]>('/api/compare', {
				method: 'POST',
				body: JSON.stringify({ run_ids: runIds })
			});
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			launching = false; comparing = false;
		}
	}

	let timeLabels = $derived(
		compareData && compareData.length > 0
			? compareData[0].results.map((r) => Number(r.time_years ?? 0).toFixed(2))
			: []
	);

	function metricDatasets(
		field: keyof import('$lib/types').SimulationResult
	): { label: string; data: number[] }[] {
		if (!compareData) return [];
		return compareData.map((entry) => ({
			label: entry.reactor_name,
			data: entry.results.map((r) => Number(r[field] ?? 0))
		}));
	}
</script>

<h1>Compare Designs</h1>
<p class="subtitle">Select 2-4 reactor designs to run simulations and compare side by side.</p>

{#if loading}
	<p class="status">Loading...</p>
{:else}
	<div class="reactor-grid">
		{#each reactors as r, i}
			<button
				class="reactor-card"
				class:selected={selectedIds.includes(r.id)}
				onclick={() => toggleReactor(r.id)}
			>
				<div class="card-index mono">{String(i + 1).padStart(2, '0')}</div>
				<div class="card-body">
					<span class="card-type">{r.design_type}</span>
					<span class="card-name">{r.name}</span>
					{#if r.vendor}
						<span class="card-vendor">{r.vendor}</span>
					{/if}
				</div>
				<div class="card-power mono">{r.electric_power_mw ?? '?'} <small>MWe</small></div>
				{#if selectedIds.includes(r.id)}
					<div class="selected-indicator"></div>
				{/if}
			</button>
		{/each}
	</div>

	{#if selectedIds.length >= 2}
		<button class="btn-compare" onclick={launchAndCompare} disabled={launching || comparing}>
			{#if launching}
				Running {selectedIds.length} simulations...
			{:else if comparing}
				Loading comparison...
			{:else}
				Compare {selectedIds.length} Designs
			{/if}
		</button>
	{:else}
		<p class="hint mono">{selectedIds.length}/2 minimum selected</p>
	{/if}

	{#if error}
		<div class="error-box">{error}</div>
	{/if}

	{#if compareData}
		<div class="charts">
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('fuel_burnup_gwd_t')} title="Fuel Burnup" yLabel="GWd/t" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('coolant_temp_outlet_c')} title="Outlet Temperature" yLabel="deg C" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('electric_power_mw')} title="Electric Power" yLabel="MW" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('capacity_factor')} title="Capacity Factor" yLabel="0-1" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('waste_total_activity_bq')} title="Radioactivity" yLabel="Bq" logScale={true} />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('waste_actinides_kg')} title="Actinide Mass" yLabel="kg" />
		</div>
	{/if}
{/if}

<style>
	h1 {
		font-size: 2rem;
		font-weight: 800;
		letter-spacing: -0.03em;
		margin: 0 0 0.5rem;
	}

	.subtitle {
		color: rgba(255, 255, 255, 0.35);
		font-size: 0.95rem;
		font-weight: 300;
		margin: 0 0 2.5rem;
	}

	.reactor-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		margin-bottom: 2rem;
	}

	.reactor-card {
		background: #000;
		border: none;
		padding: 1.25rem;
		cursor: pointer;
		text-align: left;
		color: rgba(255, 255, 255, 0.5);
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 1rem;
		align-items: center;
		transition: all 0.2s;
		font-family: 'Inter', sans-serif;
		position: relative;
		overflow: hidden;
	}

	.reactor-card:hover {
		background: rgba(255, 255, 255, 0.02);
		color: #fff;
	}

	.reactor-card.selected {
		background: rgba(255, 255, 255, 0.04);
		color: #fff;
	}

	.selected-indicator {
		position: absolute;
		top: 0;
		left: 0;
		width: 3px;
		height: 100%;
		background: #fff;
	}

	.card-index {
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.15);
	}

	.card-body {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}

	.card-type {
		font-size: 0.6rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		opacity: 0.4;
	}

	.card-name {
		font-size: 0.9rem;
		font-weight: 600;
	}

	.card-vendor {
		font-size: 0.75rem;
		opacity: 0.4;
	}

	.card-power {
		font-size: 0.85rem;
	}

	.card-power small {
		font-size: 0.6rem;
		opacity: 0.4;
	}

	.mono {
		font-family: 'JetBrains Mono', monospace;
	}

	.btn-compare {
		background: #fff;
		color: #000;
		border: none;
		padding: 0.85rem 2.5rem;
		font-size: 0.85rem;
		font-weight: 700;
		font-family: 'Inter', sans-serif;
		cursor: pointer;
		letter-spacing: 0.03em;
		transition: all 0.3s;
		margin-bottom: 2rem;
	}

	.btn-compare:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.85);
		transform: translateY(-1px);
	}

	.btn-compare:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.hint {
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.2);
		margin-bottom: 2rem;
	}

	.error-box {
		background: rgba(255, 51, 102, 0.1);
		border: 1px solid rgba(255, 51, 102, 0.2);
		color: #ff3366;
		padding: 1rem 1.25rem;
		font-size: 0.85rem;
		margin-bottom: 2rem;
	}

	.status {
		color: rgba(255, 255, 255, 0.4);
	}

	.charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(480px, 100%), 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.04);
	}

	@media (max-width: 768px) {
		h1 {
			font-size: 1.5rem;
		}

		.subtitle {
			font-size: 0.85rem;
			margin-bottom: 1.5rem;
		}

		.reactor-grid {
			grid-template-columns: 1fr;
		}

		.reactor-card {
			grid-template-columns: auto 1fr auto;
			padding: 1rem;
		}

		.btn-compare {
			width: 100%;
			text-align: center;
		}

		.charts {
			grid-template-columns: 1fr;
		}
	}
</style>
