<script lang="ts">
	import { page } from '$app/state';
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign, LaunchResponse, CompareEntry, SimulationResult } from '$lib/types';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';

	let reactor = $state<ReactorDesign | null>(null);
	let loading = $state(true);
	let error = $state('');

	// Quick compare state
	let allReactors = $state<ReactorDesign[]>([]);
	let compareTargetId = $state('');
	let compareStatus = $state<'idle' | 'running' | 'done' | 'error'>('idle');
	let compareError = $state('');
	let compareData = $state<CompareEntry[] | null>(null);

	$effect(() => {
		const id = page.params.id;
		apiFetch<ReactorDesign>(`/api/reactors/${id}`)
			.then((data) => (reactor = data))
			.catch((e) => (error = e.message))
			.finally(() => (loading = false));

		apiFetch<ReactorDesign[]>('/api/reactors')
			.then((data) => (allReactors = data));
	});

	let otherReactors = $derived(
		allReactors.filter((r) => r.id !== reactor?.id)
	);

	let compareTarget = $derived(
		allReactors.find((r) => r.id === compareTargetId) ?? null
	);

	const COOLANT_MAP: Record<string, string> = {
		Sodium: 'Sodium', Lead: 'Lead', Helium: 'Helium',
		'Molten FLiBe salt': 'FLiBe', 'Light water': 'LightWater', 'Liquid Sodium': 'Sodium'
	};
	const CYCLE_MAP: Record<string, string> = {
		Sodium: 'Rankine', Lead: 'Rankine', Helium: 'Brayton',
		FLiBe: 'SCO2Brayton', LightWater: 'Rankine'
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

	async function runCompare() {
		if (!reactor || !compareTargetId) return;
		compareStatus = 'running';
		compareError = '';
		compareData = null;

		try {
			const targets = [reactor, allReactors.find((r) => r.id === compareTargetId)!];
			const runIds: string[] = [];
			for (const r of targets) {
				const res = await apiFetch<LaunchResponse>('/api/simulations', {
					method: 'POST',
					body: JSON.stringify({ reactor_id: r.id, params: defaultParams(r) })
				});
				runIds.push(res.id);
			}
			compareData = await apiFetch<CompareEntry[]>('/api/compare', {
				method: 'POST',
				body: JSON.stringify({ run_ids: runIds })
			});
			compareStatus = 'done';
		} catch (e: unknown) {
			compareError = e instanceof Error ? e.message : String(e);
			compareStatus = 'error';
		}
	}

	let timeLabels = $derived(
		compareData && compareData.length > 0
			? compareData[0].results.map((r) => Number(r.time_years ?? 0).toFixed(2))
			: []
	);

	function metricDatasets(
		field: keyof SimulationResult
	): { label: string; data: number[] }[] {
		if (!compareData) return [];
		return compareData.map((entry) => ({
			label: entry.reactor_name,
			data: entry.results.map((r) => Number(r[field] ?? 0))
		}));
	}
</script>

{#if loading}
	<p class="text-white/40">Loading...</p>
{:else if error}
	<p class="text-error">{error}</p>
{:else if reactor}
	<a href="/reactors" class="text-[0.8rem] text-white/35 no-underline tracking-label transition-colors duration-200 hover:text-white">&larr; All Designs</a>

	<div class="mt-8 mb-12 max-lg:mt-6 max-lg:mb-8">
		<span class="type-badge mb-4 inline-block border-white/20 px-2.5 py-1 text-[0.65rem] tracking-caps">{reactor.design_type}</span>
		<h1 class="text-[clamp(2rem,5vw,3.5rem)] font-extrabold tracking-tight m-0 mb-1 leading-[1.1]">{reactor.name}</h1>
		{#if reactor.vendor}
			<p class="text-base text-white/35 mt-2 mb-0 font-light">{reactor.vendor}</p>
		{/if}
	</div>

	<div class="flex items-center gap-8 py-8 border-t border-b border-white/6 mb-12 max-lg:flex-col max-lg:gap-0 max-lg:p-0 max-lg:mb-8">
		<div class="flex flex-col gap-1 max-lg:py-5 max-lg:flex-row max-lg:items-baseline max-lg:gap-3">
			<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">{reactor.thermal_power_mw ?? 'N/A'}</span>
			<span class="text-[0.7rem] uppercase tracking-wide text-white/30 font-medium">MW thermal</span>
		</div>
		<div class="w-px h-12 bg-white/8 max-lg:w-full max-lg:h-px"></div>
		<div class="flex flex-col gap-1 max-lg:py-5 max-lg:flex-row max-lg:items-baseline max-lg:gap-3">
			<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">{reactor.electric_power_mw ?? 'N/A'}</span>
			<span class="text-[0.7rem] uppercase tracking-wide text-white/30 font-medium">MW electric</span>
		</div>
		<div class="w-px h-12 bg-white/8 max-lg:w-full max-lg:h-px"></div>
		<div class="flex flex-col gap-1 max-lg:py-5 max-lg:flex-row max-lg:items-baseline max-lg:gap-3">
			{#if reactor.thermal_power_mw && reactor.electric_power_mw}
				<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">{((Number(reactor.electric_power_mw) / Number(reactor.thermal_power_mw)) * 100).toFixed(1)}</span>
				<span class="text-[0.7rem] uppercase tracking-wide text-white/30 font-medium">% efficiency</span>
			{:else}
				<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">--</span>
				<span class="text-[0.7rem] uppercase tracking-wide text-white/30 font-medium">% efficiency</span>
			{/if}
		</div>
	</div>

	<div class="grid grid-cols-[repeat(auto-fit,minmax(350px,1fr))] grid-divider mb-12 max-lg:grid-cols-1 max-lg:mb-8">
		<section class="bg-black p-8 max-lg:p-5">
			<h2 class="section-heading mb-5">Core Design</h2>
			<div class="flex flex-col">
				<div class="flex justify-between items-start py-2.5 border-b border-white/4 last:border-b-0">
					<span class="text-[0.85rem] text-white/40 capitalize">Coolant</span>
					<span class="text-[0.85rem] text-right">{reactor.coolant_type ?? 'N/A'}</span>
				</div>
				<div class="flex justify-between items-start py-2.5 border-b border-white/4 last:border-b-0">
					<span class="text-[0.85rem] text-white/40 capitalize">Moderator</span>
					<span class="text-[0.85rem] text-right">{reactor.moderator ?? 'None'}</span>
				</div>
				<div class="flex justify-between items-start py-2.5 border-b border-white/4 last:border-b-0">
					<span class="text-[0.85rem] text-white/40 capitalize">Fuel Type</span>
					<span class="text-[0.85rem] text-right">{reactor.fuel_type ?? 'N/A'}</span>
				</div>
				<div class="flex justify-between items-start py-2.5 border-b-0">
					<span class="text-[0.85rem] text-white/40 capitalize">Enrichment</span>
					<span class="text-[0.85rem] text-right font-mono">{reactor.enrichment_pct ?? 'N/A'}%</span>
				</div>
			</div>
		</section>

		{#if reactor.design_metadata && Object.keys(reactor.design_metadata).length > 0}
			<section class="bg-black p-8 max-lg:p-5">
				<h2 class="section-heading mb-5">Parameters</h2>
				<div class="flex flex-col">
					{#each Object.entries(reactor.design_metadata) as [key, value]}
						<div class="flex justify-between items-start py-2.5 border-b border-white/4 last:border-b-0">
							<span class="text-[0.85rem] text-white/40 capitalize">{key.replace(/_/g, ' ')}</span>
							<span class="text-[0.85rem] text-right">
								{#if value !== null && typeof value === 'object'}
									<div class="flex flex-col gap-1">
										{#each Object.entries(value as Record<string, unknown>) as [k, v]}
											<div class="flex gap-4 justify-end">
												<span class="text-xs text-white/30 capitalize">{k.replace(/_/g, ' ')}</span>
												<span class="text-xs font-mono">{v}</span>
											</div>
										{/each}
									</div>
								{:else if typeof value === 'boolean'}
									<span class="text-[0.7rem] font-semibold tracking-[0.05em] uppercase {value ? 'text-[#00ff88]' : 'text-white/30'}">{value ? 'Yes' : 'No'}</span>
								{:else}
									{value ?? 'N/A'}
								{/if}
							</span>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	</div>

	<div class="mb-12 flex gap-4 max-lg:flex-col max-lg:mb-8">
		<a href="/simulate?reactor={reactor.id}" class="btn-primary max-lg:block max-lg:text-center max-lg:w-full">Simulate Reactor</a>
	</div>

	<!-- Quick Compare -->
	{#if otherReactors.length > 0}
		<section class="border-t border-white/6 pt-10 mb-12">
			<h2 class="section-heading mb-2">Quick Compare</h2>
			<p class="text-[0.85rem] text-white/35 font-light mb-6">Run a side-by-side simulation of {reactor.name} against another design.</p>

			<div class="flex gap-4 items-stretch mb-6 max-lg:flex-col">
				<div class="flex-1 flex items-center border border-white/8 min-w-0 max-lg:flex-col max-lg:items-stretch">
					<div class="flex flex-col gap-0.5 px-4 py-2.5 bg-white/4 whitespace-nowrap shrink-0 max-lg:border-b max-lg:border-white/8">
						<span class="text-[0.55rem] font-bold tracking-caps uppercase text-white/30">{reactor.design_type}</span>
						<span class="text-[0.8rem] font-semibold">{reactor.name}</span>
					</div>
					<span class="px-3 text-[0.7rem] font-semibold text-white/20 tracking-[0.1em] uppercase shrink-0 self-center max-lg:py-2 max-lg:text-center">vs</span>
					<select bind:value={compareTargetId} class="flex-1 min-w-0 bg-transparent border-none border-l border-white/8 text-white px-4 py-2.5 text-[0.8rem] font-sans cursor-pointer outline-none max-lg:border-l-0 max-lg:border-t max-lg:border-white/8">
						<option value="" class="bg-[#111]">Select a reactor...</option>
						{#each otherReactors as r}
							<option value={r.id} class="bg-[#111]">{r.design_type} — {r.name} ({r.electric_power_mw ?? '?'} MWe)</option>
						{/each}
					</select>
				</div>
				<button
					class="btn-primary px-8 text-xs font-bold tracking-mid uppercase whitespace-nowrap max-lg:w-full max-lg:py-3.5 max-lg:text-center"
					disabled={!compareTargetId || compareStatus === 'running'}
					onclick={runCompare}
				>
					{#if compareStatus === 'running'}
						Running simulations...
					{:else}
						Compare
					{/if}
				</button>
			</div>

			{#if compareTarget && compareTargetId}
				<div class="grid grid-cols-2 grid-divider mb-6 max-lg:grid-cols-1">
					<div class="bg-black p-5 flex flex-col gap-1">
						<span class="text-[0.6rem] font-semibold tracking-caps uppercase text-white/25">This reactor</span>
						<span class="text-base font-bold">{reactor.name}</span>
						<div class="flex gap-4 font-mono text-xs text-white/40 mt-1">
							<span>{reactor.thermal_power_mw ?? '?'} MWth</span>
							<span>{reactor.coolant_type ?? '?'}</span>
							<span>{reactor.enrichment_pct ?? '?'}%</span>
						</div>
					</div>
					<div class="bg-black p-5 flex flex-col gap-1">
						<span class="text-[0.6rem] font-semibold tracking-caps uppercase text-white/25">Compare with</span>
						<span class="text-base font-bold">{compareTarget.name}</span>
						<div class="flex gap-4 font-mono text-xs text-white/40 mt-1">
							<span>{compareTarget.thermal_power_mw ?? '?'} MWth</span>
							<span>{compareTarget.coolant_type ?? '?'}</span>
							<span>{compareTarget.enrichment_pct ?? '?'}%</span>
						</div>
					</div>
				</div>
			{/if}

			{#if compareError}
				<div class="error-box mb-6">{compareError}</div>
			{/if}

			{#if compareData}
				<div class="chart-grid-auto">
					<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('fuel_burnup_gwd_t')} title="Fuel Burnup" yLabel="GWd/t" />
					<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('coolant_temp_outlet_c')} title="Outlet Temperature" yLabel="deg C" />
					<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('electric_power_mw')} title="Electric Power" yLabel="MW" />
					<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('capacity_factor')} title="Capacity Factor" yLabel="0-1" />
					<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('waste_total_activity_bq')} title="Radioactivity" yLabel="Bq" logScale={true} />
					<TimeSeriesChart labels={timeLabels} datasets={metricDatasets('waste_actinides_kg')} title="Actinide Mass" yLabel="kg" />
				</div>
			{/if}
		</section>
	{/if}

	{#if reactor.source_url}
		<p class="text-xs text-white/20">
			Source: <a href={reactor.source_url} target="_blank" rel="noopener" class="text-white/35 no-underline break-all transition-colors duration-200 hover:text-white">{reactor.source_url}</a>
		</p>
	{/if}
{/if}
