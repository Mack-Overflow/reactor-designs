<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign, LaunchResponse, CompareEntry } from '$lib/types';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';
	import { exportComparisonPdf } from '$lib/export-pdf';

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

	function handleExportComparison() {
		if (!compareData) return;
		exportComparisonPdf({
			entries: compareData.map((entry) => {
				const last = entry.results[entry.results.length - 1] ?? null;
				return {
					reactorName: entry.reactor_name,
					reactorType: entry.reactor_design_type,
					finalStep: last ? {
						time_years: Number(last.time_years ?? 0),
						fuel_burnup_gwd_t: Number(last.fuel_burnup_gwd_t ?? 0),
						fuel_remaining_pct: Number(last.fuel_remaining_pct ?? 0),
						coolant_temp_inlet_c: Number(last.coolant_temp_inlet_c ?? 0),
						coolant_temp_outlet_c: Number(last.coolant_temp_outlet_c ?? 0),
						thermal_power_mw: Number(last.thermal_power_mw ?? 0),
						electric_power_mw: Number(last.electric_power_mw ?? 0),
						capacity_factor: Number(last.capacity_factor ?? 0),
						waste_actinides_kg: Number(last.waste_actinides_kg ?? 0),
						waste_fission_products_kg: Number(last.waste_fission_products_kg ?? 0),
						waste_total_activity_bq: Number(last.waste_total_activity_bq ?? 0),
					} : null,
				};
			}),
		});
	}
</script>

<h1 class="text-3xl font-extrabold tracking-tight m-0 mb-2 max-lg:text-2xl">Compare Designs</h1>
<p class="text-white/35 text-[0.95rem] font-light mb-10 max-lg:text-[0.85rem] max-lg:mb-6">Select 2-4 reactor designs to run simulations and compare side by side.</p>

{#if loading}
	<p class="text-white/40">Loading...</p>
{:else}
	<div class="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] grid-divider mb-8 max-lg:grid-cols-1">
		{#each reactors as r, i}
			<button
				class="bg-black border-none p-5 cursor-pointer text-left text-white/50 grid grid-cols-[auto_1fr_auto] gap-4 items-center transition-all duration-200 font-sans relative overflow-hidden hover:bg-white/2 hover:text-white max-lg:p-4 {selectedIds.includes(r.id) ? '!bg-white/4 !text-white' : ''}"
				onclick={() => toggleReactor(r.id)}
			>
				<div class="font-mono text-[0.7rem] text-white/15">{String(i + 1).padStart(2, '0')}</div>
				<div class="flex flex-col gap-0.5">
					<span class="text-[0.6rem] font-bold tracking-wide uppercase opacity-40">{r.design_type}</span>
					<span class="text-[0.9rem] font-semibold">{r.name}</span>
					{#if r.vendor}
						<span class="text-xs opacity-40">{r.vendor}</span>
					{/if}
				</div>
				<div class="font-mono text-[0.85rem]">{r.electric_power_mw ?? '?'} <small class="text-[0.6rem] opacity-40">MWe</small></div>
				{#if selectedIds.includes(r.id)}
					<div class="absolute top-0 left-0 w-[3px] h-full bg-white"></div>
				{/if}
			</button>
		{/each}
	</div>

	{#if selectedIds.length >= 2}
		<button class="btn-primary py-3.5 px-10 text-[0.85rem] font-bold tracking-[0.03em] mb-8 max-lg:w-full max-lg:text-center" onclick={launchAndCompare} disabled={launching || comparing}>
			{#if launching}
				Running {selectedIds.length} simulations...
			{:else if comparing}
				Loading comparison...
			{:else}
				Compare {selectedIds.length} Designs
			{/if}
		</button>
	{:else}
		<p class="font-mono text-xs text-white/20 mb-8">{selectedIds.length}/2 minimum selected</p>
	{/if}

	{#if error}
		<div class="error-box mb-8">{error}</div>
	{/if}

	{#if compareData}
		<div class="mb-6">
			<button class="btn-primary px-6 py-2.5 text-xs font-bold tracking-mid uppercase" onclick={handleExportComparison}>Export PDF</button>
		</div>
		<div class="chart-grid-auto max-lg:grid-cols-1">
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('fuel_burnup_gwd_t')} title="Fuel Burnup" yLabel="GWd/t" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('coolant_temp_outlet_c')} title="Outlet Temperature" yLabel="deg C" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('electric_power_mw')} title="Electric Power" yLabel="MW" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('capacity_factor')} title="Capacity Factor" yLabel="0-1" />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('waste_total_activity_bq')} title="Radioactivity" yLabel="Bq" logScale={true} />
			<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('waste_actinides_kg')} title="Actinide Mass" yLabel="kg" />
		</div>
	{/if}
{/if}
