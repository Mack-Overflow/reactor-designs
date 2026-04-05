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

<a href="/simulate" class="back">&larr; New Simulation</a>

<div class="hero">
	<h1>
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
		<div class="progress-track">
			<div class="progress-bar" style="width: {status === 'completed' ? 100 : progressPct}%"></div>
		</div>
		<div class="progress-meta">
			<span class="mono">{steps.length} / {totalSteps}</span>
			<span>steps</span>
			{#if status === 'completed'}
				<span class="separator">&bull;</span>
				<span class="mono">{averageCf.toFixed(3)}</span>
				<span>avg capacity factor</span>
			{/if}
			{#if runId}
				<span class="separator">&bull;</span>
				<a href="/simulate/{runId}" class="run-link">{runId.slice(0, 8)}</a>
			{/if}
		</div>
	{/if}

	{#if status === 'completed'}
		<div class="completed-actions">
			<button class="btn-export" onclick={handleExport}>Export PDF</button>
			<a href="/simulate" class="btn-again">Run Again</a>
		</div>
	{/if}
</div>

{#if status === 'error'}
	<div class="error-box">{errorMsg}</div>
{/if}

{#if latest}
	<div class="live-stats">
		<div class="live-stat">
			<span class="ls-value mono">{latest.thermal_power_mw.toFixed(0)}</span>
			<span class="ls-label">MW thermal</span>
		</div>
		<div class="live-stat">
			<span class="ls-value mono">{latest.electric_power_mw.toFixed(0)}</span>
			<span class="ls-label">MW electric</span>
		</div>
		<div class="live-stat">
			<span class="ls-value mono">{latest.coolant_temp_outlet_c.toFixed(0)}&deg;</span>
			<span class="ls-label">outlet temp</span>
		</div>
		<div class="live-stat">
			<span class="ls-value mono">{latest.fuel_burnup_gwd_t.toFixed(1)}</span>
			<span class="ls-label">GWd/t burnup</span>
		</div>
		<div class="live-stat">
			<span class="ls-value mono">{(latest.fuel_remaining_pct * 100).toFixed(1)}%</span>
			<span class="ls-label">fuel remaining</span>
		</div>
		<div class="live-stat">
			<span class="ls-value mono">{latest.waste_total_activity_bq.toExponential(2)}</span>
			<span class="ls-label">Bq activity</span>
		</div>
	</div>
{/if}

{#if steps.length > 0}
	<div class="charts">
		<TimeSeriesChart labels={timeLabels} datasets={burnupData} title="Fuel Burnup" yLabel="GWd/t" />
		<TimeSeriesChart labels={timeLabels} datasets={tempData} title="Coolant Temperature" yLabel="deg C" />
		<TimeSeriesChart labels={timeLabels} datasets={powerData} title="Power Output" yLabel="MW" />
		<TimeSeriesChart labels={timeLabels} datasets={capacityData} title="Capacity Factor" yLabel="0-1" />
		<TimeSeriesChart labels={timeLabels} datasets={wasteData} title="Waste Mass" yLabel="kg" />
		<TimeSeriesChart labels={timeLabels} datasets={activityData} title="Radioactivity" yLabel="Bq" logScale={true} />
	</div>
{/if}

<style>
	.back {
		font-size: 0.8rem;
		color: rgba(255, 255, 255, 0.3);
		text-decoration: none;
		transition: color 0.2s;
	}

	.back:hover {
		color: #fff;
	}

	.hero {
		margin: 2rem 0 2.5rem;
	}

	h1 {
		font-size: clamp(1.5rem, 4vw, 2.5rem);
		font-weight: 800;
		letter-spacing: -0.03em;
		margin: 0 0 1.5rem;
	}

	.progress-track {
		height: 2px;
		background: rgba(255, 255, 255, 0.06);
		margin-bottom: 0.75rem;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		background: #fff;
		transition: width 0.4s ease;
	}

	.progress-meta {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.35);
	}

	.mono {
		font-family: 'JetBrains Mono', monospace;
		color: #fff;
	}

	.separator {
		color: rgba(255, 255, 255, 0.15);
	}

	.run-link {
		color: rgba(255, 255, 255, 0.5);
		text-decoration: none;
		font-family: 'JetBrains Mono', monospace;
	}

	.run-link:hover {
		color: #fff;
	}

	.completed-actions {
		display: flex;
		gap: 0.75rem;
		margin-top: 1.25rem;
	}

	.btn-export {
		background: #fff;
		color: #000;
		border: none;
		padding: 0.65rem 1.5rem;
		font-size: 0.75rem;
		font-weight: 700;
		font-family: 'Inter', sans-serif;
		cursor: pointer;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		transition: all 0.2s;
	}

	.btn-export:hover {
		background: rgba(255, 255, 255, 0.85);
	}

	.btn-again {
		display: inline-flex;
		align-items: center;
		border: 1px solid rgba(255, 255, 255, 0.15);
		color: rgba(255, 255, 255, 0.6);
		padding: 0.65rem 1.5rem;
		font-size: 0.75rem;
		font-weight: 600;
		font-family: 'Inter', sans-serif;
		text-decoration: none;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		transition: all 0.2s;
	}

	.btn-again:hover {
		color: #fff;
		border-color: rgba(255, 255, 255, 0.4);
	}

	.error-box {
		background: rgba(255, 51, 102, 0.1);
		border: 1px solid rgba(255, 51, 102, 0.2);
		color: #ff3366;
		padding: 1rem 1.25rem;
		font-size: 0.85rem;
		margin-bottom: 2rem;
	}

	.live-stats {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		margin-bottom: 3rem;
	}

	.live-stat {
		background: #000;
		padding: 1.25rem 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	.ls-value {
		font-size: 1.25rem;
		font-weight: 700;
	}

	.ls-label {
		font-size: 0.6rem;
		text-transform: uppercase;
		letter-spacing: 0.12em;
		color: rgba(255, 255, 255, 0.25);
		font-weight: 500;
	}

	.charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(480px, 100%), 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.04);
	}

	@media (max-width: 768px) {
		.live-stats {
			grid-template-columns: repeat(3, 1fr);
		}

		.ls-value {
			font-size: 1rem;
		}

		.live-stat {
			padding: 1rem 0.75rem;
		}

		.progress-meta {
			flex-wrap: wrap;
			gap: 0.35rem;
		}

		.charts {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 480px) {
		.live-stats {
			grid-template-columns: repeat(2, 1fr);
		}

		.ls-value {
			font-size: 0.9rem;
		}
	}
</style>
