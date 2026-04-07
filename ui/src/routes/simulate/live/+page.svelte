<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';
	import { exportSimulationPdf } from '$lib/export-pdf';

	const API_BASE = import.meta.env.VITE_API_URL ?? 'http://localhost:18080';

	interface StepData {
		time_step: number;
		total_steps: number;
		time_years: number;
		fuel_burnup_gwd_t: number;
		fuel_remaining_pct: number;
		coolant_temp_inlet_c: number;
		coolant_temp_outlet_c: number;
		coolant_flow_rate_kg_s: number;
		thermal_power_mw: number;
		electric_power_mw: number;
		capacity_factor: number;
		waste_actinides_kg: number;
		waste_fission_products_kg: number;
		waste_total_activity_bq: number;
	}

	let steps = $state<StepData[]>([]);
	let totalSteps = $state(0);
	let runId = $state('');
	let status = $state<'connecting' | 'streaming' | 'completed' | 'error'>('connecting');
	let errorMsg = $state('');
	let averageCf = $state(0);
	let launchPayload = $state<Record<string, unknown>>({});

	let timeLabels = $derived(steps.map((s) => s.time_years.toFixed(2)));

	let burnupData = $derived([
		{ label: 'Burnup (GWd/t)', data: steps.map((s) => s.fuel_burnup_gwd_t) }
	]);
	let tempData = $derived([
		{ label: 'Inlet', data: steps.map((s) => s.coolant_temp_inlet_c) },
		{ label: 'Outlet', data: steps.map((s) => s.coolant_temp_outlet_c) }
	]);
	let powerData = $derived([
		{ label: 'Thermal (MW)', data: steps.map((s) => s.thermal_power_mw) },
		{ label: 'Electric (MW)', data: steps.map((s) => s.electric_power_mw) }
	]);
	let capacityData = $derived([
		{ label: 'Capacity Factor', data: steps.map((s) => s.capacity_factor) }
	]);
	let wasteData = $derived([
		{ label: 'Actinides', data: steps.map((s) => s.waste_actinides_kg) },
		{ label: 'Fission Products', data: steps.map((s) => s.waste_fission_products_kg) }
	]);
	let activityData = $derived([
		{
			label: 'Total Activity (Bq)',
			data: steps.map((s) => Math.max(s.waste_total_activity_bq, 1))
		}
	]);

	let progressPct = $derived(totalSteps > 0 ? Math.round((steps.length / totalSteps) * 100) : 0);

	// Derive live stats from latest step
	let latest = $derived(steps.length > 0 ? steps[steps.length - 1] : null);

	function handleExport() {
		const params = launchPayload.params as Record<string, unknown> ?? {};
		exportSimulationPdf({
			reactorName: String(params.reactor_name ?? 'Reactor'),
			reactorType: String(params.reactor_type ?? ''),
			runId,
			averageCf,
			params,
			steps,
		});
	}

	onMount(() => {
		const raw = sessionStorage.getItem('sim_launch');
		if (!raw) {
			goto('/simulate');
			return;
		}
		sessionStorage.removeItem('sim_launch');
		const payload = JSON.parse(raw);
		launchPayload = payload;
		startStream(payload);
	});

	async function startStream(payload: unknown) {
		try {
			const res = await fetch(`${API_BASE}/api/simulations/stream`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(payload)
			});

			if (!res.ok || !res.body) {
				const body = await res.json().catch(() => ({}));
				errorMsg = body.error ?? `API error: ${res.status}`;
				status = 'error';
				return;
			}

			const reader = res.body.getReader();
			const decoder = new TextDecoder();
			let buffer = '';
			status = 'streaming';

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				buffer += decoder.decode(value, { stream: true });
				const parts = buffer.split('\n\n');
				buffer = parts.pop() ?? '';

				for (const part of parts) {
					const lines = part.split('\n');
					let eventType = 'message';
					let data = '';
					for (const line of lines) {
						if (line.startsWith('event: ')) eventType = line.slice(7);
						else if (line.startsWith('data: ')) data = line.slice(6);
					}
					if (!data) continue;
					const parsed = JSON.parse(data);
					if (eventType === 'started') {
						runId = parsed.id;
					} else if (eventType === 'message') {
						const step = parsed as StepData;
						if (step.total_steps) totalSteps = step.total_steps;
						steps = [...steps, step];
					} else if (eventType === 'done') {
						runId = parsed.id;
						averageCf = parsed.average_capacity_factor;
						status = 'completed';
					} else if (eventType === 'error') {
						errorMsg = parsed.error;
						status = 'error';
					}
				}
			}
			if (status === 'streaming') status = 'completed';
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : String(e);
			status = 'error';
		}
	}
</script>

<a href="/simulate" class="text-[0.8rem] text-white/30 no-underline transition-colors duration-200 hover:text-white">&larr; New Simulation</a>

<div class="mt-8 mb-10">
	<h1 class="text-[clamp(1.5rem,4vw,2.5rem)] font-extrabold tracking-tight m-0 mb-6">
		{#if status === 'streaming'}
			Simulation in progress
		{:else if status === 'completed'}
			Simulation complete
		{:else if status === 'error'}
			Simulation failed
		{:else}
			Connecting...
		{/if}
	</h1>

	{#if status === 'streaming' || status === 'completed'}
		<div class="h-0.5 bg-white/6 mb-3 overflow-hidden">
			<div class="h-full bg-white transition-[width] duration-400 ease-out" style="width: {status === 'completed' ? 100 : progressPct}%"></div>
		</div>
		<div class="flex gap-2 items-center text-xs text-white/35 flex-wrap max-lg:gap-1.5">
			<span class="font-mono text-white">{steps.length} / {totalSteps}</span>
			<span>steps</span>
			{#if status === 'completed'}
				<span class="text-white/15">&bull;</span>
				<span class="font-mono text-white">{averageCf.toFixed(3)}</span>
				<span>avg capacity factor</span>
			{/if}
			{#if runId}
				<span class="text-white/15">&bull;</span>
				<a href="/simulate/{runId}" class="text-white/50 no-underline font-mono hover:text-white">{runId.slice(0, 8)}</a>
			{/if}
		</div>
	{/if}

	{#if status === 'completed'}
		<div class="flex gap-3 mt-5">
			<button class="btn-primary px-6 py-2.5 text-xs font-bold tracking-mid uppercase" onclick={handleExport}>Export PDF</button>
			<a href="/simulate" class="btn-outline px-6 py-2.5 text-xs font-semibold tracking-mid uppercase inline-flex items-center">Run Again</a>
		</div>
	{/if}
</div>

{#if status === 'error'}
	<div class="error-box mb-8">{errorMsg}</div>
{/if}

{#if latest}
	<div class="grid grid-cols-6 grid-divider mb-12 max-lg:grid-cols-3 max-sm:grid-cols-2">
		<div class="bg-black py-5 px-4 flex flex-col gap-1 max-lg:py-4 max-lg:px-3">
			<span class="font-mono text-[1.25rem] font-bold text-white max-lg:text-base max-sm:text-[0.9rem]">{latest.thermal_power_mw.toFixed(0)}</span>
			<span class="text-[0.6rem] uppercase tracking-caps text-white/25 font-medium">MW thermal</span>
		</div>
		<div class="bg-black py-5 px-4 flex flex-col gap-1 max-lg:py-4 max-lg:px-3">
			<span class="font-mono text-[1.25rem] font-bold text-white max-lg:text-base max-sm:text-[0.9rem]">{latest.electric_power_mw.toFixed(0)}</span>
			<span class="text-[0.6rem] uppercase tracking-caps text-white/25 font-medium">MW electric</span>
		</div>
		<div class="bg-black py-5 px-4 flex flex-col gap-1 max-lg:py-4 max-lg:px-3">
			<span class="font-mono text-[1.25rem] font-bold text-white max-lg:text-base max-sm:text-[0.9rem]">{latest.coolant_temp_outlet_c.toFixed(0)}&deg;</span>
			<span class="text-[0.6rem] uppercase tracking-caps text-white/25 font-medium">outlet temp</span>
		</div>
		<div class="bg-black py-5 px-4 flex flex-col gap-1 max-lg:py-4 max-lg:px-3">
			<span class="font-mono text-[1.25rem] font-bold text-white max-lg:text-base max-sm:text-[0.9rem]">{latest.fuel_burnup_gwd_t.toFixed(1)}</span>
			<span class="text-[0.6rem] uppercase tracking-caps text-white/25 font-medium">GWd/t burnup</span>
		</div>
		<div class="bg-black py-5 px-4 flex flex-col gap-1 max-lg:py-4 max-lg:px-3">
			<span class="font-mono text-[1.25rem] font-bold text-white max-lg:text-base max-sm:text-[0.9rem]">{(latest.fuel_remaining_pct * 100).toFixed(1)}%</span>
			<span class="text-[0.6rem] uppercase tracking-caps text-white/25 font-medium">fuel remaining</span>
		</div>
		<div class="bg-black py-5 px-4 flex flex-col gap-1 max-lg:py-4 max-lg:px-3">
			<span class="font-mono text-[1.25rem] font-bold text-white max-lg:text-base max-sm:text-[0.9rem]">{latest.waste_total_activity_bq.toExponential(2)}</span>
			<span class="text-[0.6rem] uppercase tracking-caps text-white/25 font-medium">Bq activity</span>
		</div>
	</div>
{/if}

{#if steps.length > 0}
	<div class="chart-grid-auto max-lg:grid-cols-1">
		<TimeSeriesChart labels={timeLabels} datasets={burnupData} title="Fuel Burnup" yLabel="GWd/t" />
		<TimeSeriesChart labels={timeLabels} datasets={tempData} title="Coolant Temperature" yLabel="deg C" />
		<TimeSeriesChart labels={timeLabels} datasets={powerData} title="Power Output" yLabel="MW" />
		<TimeSeriesChart labels={timeLabels} datasets={capacityData} title="Capacity Factor" yLabel="0-1" />
		<TimeSeriesChart labels={timeLabels} datasets={wasteData} title="Waste Mass" yLabel="kg" />
		<TimeSeriesChart labels={timeLabels} datasets={activityData} title="Radioactivity" yLabel="Bq" logScale={true} />
	</div>
{/if}
