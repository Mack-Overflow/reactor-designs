<script lang="ts">
	import { page } from '$app/state';
	import { apiFetch } from '$lib/api';
	import type { SimulationRun, SimulationResult, WasteIsotope } from '$lib/types';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';

	let run = $state<SimulationRun | null>(null);
	let results = $state<SimulationResult[]>([]);
	let waste = $state<WasteIsotope[]>([]);
	let loading = $state(true);
	let error = $state('');

	$effect(() => {
		const id = page.params.id;
		Promise.all([
			apiFetch<SimulationRun>(`/api/simulations/${id}`),
			apiFetch<SimulationResult[]>(`/api/simulations/${id}/results`),
			apiFetch<WasteIsotope[]>(`/api/simulations/${id}/waste`)
		])
			.then(([r, res, w]) => {
				run = r;
				results = res;
				waste = w;
			})
			.catch((e) => (error = e.message))
			.finally(() => (loading = false));
	});

	let timeLabels = $derived(results.map((r) => Number(r.time_years ?? 0).toFixed(2)));

	let burnupData = $derived([
		{ label: 'Burnup (GWd/t)', data: results.map((r) => Number(r.fuel_burnup_gwd_t ?? 0)) }
	]);

	let tempData = $derived([
		{ label: 'Inlet (C)', data: results.map((r) => Number(r.coolant_temp_inlet_c ?? 0)) },
		{ label: 'Outlet (C)', data: results.map((r) => Number(r.coolant_temp_outlet_c ?? 0)) }
	]);

	let powerData = $derived([
		{ label: 'Thermal (MW)', data: results.map((r) => Number(r.thermal_power_mw ?? 0)) },
		{ label: 'Electric (MW)', data: results.map((r) => Number(r.electric_power_mw ?? 0)) }
	]);

	let capacityData = $derived([
		{
			label: 'Capacity Factor',
			data: results.map((r) => Number(r.capacity_factor ?? 0))
		}
	]);

	let wasteData = $derived([
		{
			label: 'Actinides (kg)',
			data: results.map((r) => Number(r.waste_actinides_kg ?? 0))
		},
		{
			label: 'Fission Products (kg)',
			data: results.map((r) => Number(r.waste_fission_products_kg ?? 0))
		}
	]);

	let activityData = $derived([
		{
			label: 'Total Activity (Bq)',
			data: results.map((r) => Math.max(Number(r.waste_total_activity_bq ?? 0), 1))
		}
	]);
</script>

{#if loading}
	<p class="status">Loading simulation results...</p>
{:else if error}
	<p class="status error">{error}</p>
{:else if run}
	<div class="header">
		<a href="/simulate" class="back">&larr; New Simulation</a>
		<h1>Simulation Results</h1>
		<div class="meta">
			<span class="badge {run.status}">{run.status}</span>
			<span class="meta-item">Run ID: {run.id.slice(0, 8)}...</span>
			{#if run.completed_at}
				<span class="meta-item">Completed: {new Date(run.completed_at).toLocaleString()}</span>
			{/if}
		</div>
	</div>

	{#if results.length > 0}
		<div class="charts">
			<TimeSeriesChart
				labels={timeLabels}
				datasets={burnupData}
				title="Fuel Burnup"
				yLabel="GWd/t"
			/>

			<TimeSeriesChart
				labels={timeLabels}
				datasets={tempData}
				title="Coolant Temperature"
				yLabel="Temperature (C)"
			/>

			<TimeSeriesChart
				labels={timeLabels}
				datasets={powerData}
				title="Power Output"
				yLabel="MW"
			/>

			<TimeSeriesChart
				labels={timeLabels}
				datasets={capacityData}
				title="Capacity Factor"
				yLabel="Factor (0-1)"
			/>

			<TimeSeriesChart
				labels={timeLabels}
				datasets={wasteData}
				title="Waste Inventory"
				yLabel="Mass (kg)"
			/>

			<TimeSeriesChart
				labels={timeLabels}
				datasets={activityData}
				title="Total Radioactivity"
				yLabel="Activity (Bq)"
				logScale={true}
			/>
		</div>

		{#if waste.length > 0}
			<section class="waste-section">
				<h2>Final Isotope Inventory</h2>
				<table>
					<thead>
						<tr>
							<th>Isotope</th>
							<th>Mass (kg)</th>
							<th>Activity (Bq)</th>
							<th>Half-life (yr)</th>
						</tr>
					</thead>
					<tbody>
						{#each waste as iso}
							<tr>
								<td>{iso.isotope}</td>
								<td>{Number(iso.mass_kg ?? 0).toExponential(3)}</td>
								<td>{Number(iso.activity_bq ?? 0).toExponential(3)}</td>
								<td>{Number(iso.half_life_years ?? 0).toExponential(3)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</section>
		{/if}
	{:else}
		<p class="status">No results available for this simulation.</p>
	{/if}
{/if}

<style>
	.status {
		color: #94a3b8;
	}

	.error {
		color: #f87171;
	}

	.back {
		color: #38bdf8;
		text-decoration: none;
		font-size: 0.85rem;
	}

	.back:hover {
		text-decoration: underline;
	}

	h1 {
		font-size: 1.75rem;
		margin: 0.5rem 0;
	}

	.meta {
		display: flex;
		gap: 1rem;
		align-items: center;
		margin-bottom: 1.5rem;
	}

	.badge {
		font-size: 0.7rem;
		font-weight: 700;
		padding: 0.15rem 0.5rem;
		border-radius: 4px;
		text-transform: uppercase;
	}

	.badge.completed {
		background: #34d399;
		color: #0f172a;
	}

	.badge.failed {
		background: #f87171;
		color: #0f172a;
	}

	.badge.running {
		background: #facc15;
		color: #0f172a;
	}

	.meta-item {
		font-size: 0.8rem;
		color: #64748b;
	}

	.charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(480px, 1fr));
		gap: 1rem;
	}

	.waste-section {
		margin-top: 2rem;
	}

	h2 {
		font-size: 1rem;
		color: #38bdf8;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-bottom: 0.75rem;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 8px;
		overflow: hidden;
	}

	th {
		text-align: left;
		padding: 0.6rem 1rem;
		font-size: 0.75rem;
		text-transform: uppercase;
		color: #94a3b8;
		border-bottom: 1px solid #334155;
	}

	td {
		padding: 0.5rem 1rem;
		font-size: 0.85rem;
		border-bottom: 1px solid #1e293b;
		font-family: monospace;
	}

	tr:hover td {
		background: #334155;
	}
</style>
