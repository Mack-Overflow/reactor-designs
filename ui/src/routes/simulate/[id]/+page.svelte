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
	<p class="text-[#94a3b8]">Loading simulation results...</p>
{:else if error}
	<p class="text-[#f87171]">{error}</p>
{:else if run}
	<div>
		<a href="/simulate" class="text-[#38bdf8] no-underline text-[0.85rem] hover:underline">&larr; New Simulation</a>
		<h1 class="text-[1.75rem] mt-2 mb-2">Simulation Results</h1>
		<div class="flex gap-4 items-center mb-6">
			<span class="text-[0.7rem] font-bold px-2 py-0.5 rounded uppercase
				{run.status === 'completed' ? 'bg-[#34d399] text-[#0f172a]' : ''}
				{run.status === 'failed' ? 'bg-[#f87171] text-[#0f172a]' : ''}
				{run.status === 'running' ? 'bg-[#facc15] text-[#0f172a]' : ''}">{run.status}</span>
			<span class="text-[0.8rem] text-[#64748b]">Run ID: {run.id.slice(0, 8)}...</span>
			{#if run.completed_at}
				<span class="text-[0.8rem] text-[#64748b]">Completed: {new Date(run.completed_at).toLocaleString()}</span>
			{/if}
		</div>
	</div>

	{#if results.length > 0}
		<div class="grid grid-cols-[repeat(auto-fit,minmax(480px,1fr))] gap-4">
			<TimeSeriesChart labels={timeLabels} datasets={burnupData} title="Fuel Burnup" yLabel="GWd/t" />
			<TimeSeriesChart labels={timeLabels} datasets={tempData} title="Coolant Temperature" yLabel="Temperature (C)" />
			<TimeSeriesChart labels={timeLabels} datasets={powerData} title="Power Output" yLabel="MW" />
			<TimeSeriesChart labels={timeLabels} datasets={capacityData} title="Capacity Factor" yLabel="Factor (0-1)" />
			<TimeSeriesChart labels={timeLabels} datasets={wasteData} title="Waste Inventory" yLabel="Mass (kg)" />
			<TimeSeriesChart labels={timeLabels} datasets={activityData} title="Total Radioactivity" yLabel="Activity (Bq)" logScale={true} />
		</div>

		{#if waste.length > 0}
			<section class="mt-8">
				<h2 class="text-base text-[#38bdf8] uppercase tracking-[0.05em] mb-3">Final Isotope Inventory</h2>
				<table class="w-full border-collapse bg-[#1e293b] border border-[#334155] rounded-lg overflow-hidden">
					<thead>
						<tr>
							<th class="text-left px-4 py-2.5 text-xs uppercase text-[#94a3b8] border-b border-[#334155]">Isotope</th>
							<th class="text-left px-4 py-2.5 text-xs uppercase text-[#94a3b8] border-b border-[#334155]">Mass (kg)</th>
							<th class="text-left px-4 py-2.5 text-xs uppercase text-[#94a3b8] border-b border-[#334155]">Activity (Bq)</th>
							<th class="text-left px-4 py-2.5 text-xs uppercase text-[#94a3b8] border-b border-[#334155]">Half-life (yr)</th>
						</tr>
					</thead>
					<tbody>
						{#each waste as iso}
							<tr class="hover:*:bg-[#334155]">
								<td class="px-4 py-2 text-[0.85rem] border-b border-[#1e293b] font-mono">{iso.isotope}</td>
								<td class="px-4 py-2 text-[0.85rem] border-b border-[#1e293b] font-mono">{Number(iso.mass_kg ?? 0).toExponential(3)}</td>
								<td class="px-4 py-2 text-[0.85rem] border-b border-[#1e293b] font-mono">{Number(iso.activity_bq ?? 0).toExponential(3)}</td>
								<td class="px-4 py-2 text-[0.85rem] border-b border-[#1e293b] font-mono">{Number(iso.half_life_years ?? 0).toExponential(3)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</section>
		{/if}
	{:else}
		<p class="text-[#94a3b8]">No results available for this simulation.</p>
	{/if}
{/if}
