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
	<p class="status">Loading...</p>
{:else if error}
	<p class="status error">{error}</p>
{:else if reactor}
	<a href="/reactors" class="back">&larr; All Designs</a>

	<div class="hero">
		<span class="type-badge">{reactor.design_type}</span>
		<h1>{reactor.name}</h1>
		{#if reactor.vendor}
			<p class="vendor">{reactor.vendor}</p>
		{/if}
	</div>

	<div class="power-strip">
		<div class="power-item">
			<span class="power-value mono">{reactor.thermal_power_mw ?? 'N/A'}</span>
			<span class="power-unit">MW thermal</span>
		</div>
		<div class="power-divider"></div>
		<div class="power-item">
			<span class="power-value mono">{reactor.electric_power_mw ?? 'N/A'}</span>
			<span class="power-unit">MW electric</span>
		</div>
		<div class="power-divider"></div>
		<div class="power-item">
			{#if reactor.thermal_power_mw && reactor.electric_power_mw}
				<span class="power-value mono"
					>{(
						(Number(reactor.electric_power_mw) / Number(reactor.thermal_power_mw)) *
						100
					).toFixed(1)}</span
				>
				<span class="power-unit">% efficiency</span>
			{:else}
				<span class="power-value mono">--</span>
				<span class="power-unit">% efficiency</span>
			{/if}
		</div>
	</div>

	<div class="spec-grid">
		<section class="spec-card">
			<h2>Core Design</h2>
			<div class="spec-rows">
				<div class="spec-row">
					<span class="spec-key">Coolant</span>
					<span class="spec-val">{reactor.coolant_type ?? 'N/A'}</span>
				</div>
				<div class="spec-row">
					<span class="spec-key">Moderator</span>
					<span class="spec-val">{reactor.moderator ?? 'None'}</span>
				</div>
				<div class="spec-row">
					<span class="spec-key">Fuel Type</span>
					<span class="spec-val">{reactor.fuel_type ?? 'N/A'}</span>
				</div>
				<div class="spec-row">
					<span class="spec-key">Enrichment</span>
					<span class="spec-val mono">{reactor.enrichment_pct ?? 'N/A'}%</span>
				</div>
			</div>
		</section>

		{#if reactor.design_metadata && Object.keys(reactor.design_metadata).length > 0}
			<section class="spec-card">
				<h2>Parameters</h2>
				<div class="spec-rows">
					{#each Object.entries(reactor.design_metadata) as [key, value]}
						<div class="spec-row">
							<span class="spec-key">{key.replace(/_/g, ' ')}</span>
							<span class="spec-val">
								{#if value !== null && typeof value === 'object'}
									<div class="nested-params">
										{#each Object.entries(value as Record<string, unknown>) as [k, v]}
											<div class="nested-row">
												<span class="nested-key">{k.replace(/_/g, ' ')}</span>
												<span class="nested-val">{v}</span>
											</div>
										{/each}
									</div>
								{:else if typeof value === 'boolean'}
									<span class="bool-badge" class:yes={value}>{value ? 'Yes' : 'No'}</span>
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

	<div class="actions">
		<a href="/simulate" class="btn-primary">Simulate Reactor</a>
	</div>

	<!-- Quick Compare -->
	{#if otherReactors.length > 0}
		<section class="compare-section">
			<h2>Quick Compare</h2>
			<p class="compare-hint">Run a side-by-side simulation of {reactor.name} against another design.</p>

			<div class="compare-controls">
				<div class="compare-pills">
					<div class="compare-pill current">
						<span class="cp-type">{reactor.design_type}</span>
						<span class="cp-name">{reactor.name}</span>
					</div>
					<span class="compare-vs">vs</span>
					<select class="compare-select" bind:value={compareTargetId}>
						<option value="">Select a reactor...</option>
						{#each otherReactors as r}
							<option value={r.id}>{r.design_type} — {r.name} ({r.electric_power_mw ?? '?'} MWe)</option>
						{/each}
					</select>
				</div>
				<button
					class="btn-compare"
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
				<div class="compare-preview">
					<div class="cp-card">
						<span class="cp-card-label">This reactor</span>
						<span class="cp-card-name">{reactor.name}</span>
						<div class="cp-card-specs">
							<span>{reactor.thermal_power_mw ?? '?'} MWth</span>
							<span>{reactor.coolant_type ?? '?'}</span>
							<span>{reactor.enrichment_pct ?? '?'}%</span>
						</div>
					</div>
					<div class="cp-card">
						<span class="cp-card-label">Compare with</span>
						<span class="cp-card-name">{compareTarget.name}</span>
						<div class="cp-card-specs">
							<span>{compareTarget.thermal_power_mw ?? '?'} MWth</span>
							<span>{compareTarget.coolant_type ?? '?'}</span>
							<span>{compareTarget.enrichment_pct ?? '?'}%</span>
						</div>
					</div>
				</div>
			{/if}

			{#if compareError}
				<div class="error-box">{compareError}</div>
			{/if}

			{#if compareData}
				<div class="compare-charts">
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
		<p class="source">
			Source: <a href={reactor.source_url} target="_blank" rel="noopener"
				>{reactor.source_url}</a
			>
		</p>
	{/if}
{/if}

<style>
	.status {
		color: rgba(255, 255, 255, 0.4);
	}

	.error {
		color: #ff3366;
	}

	.back {
		font-size: 0.8rem;
		color: rgba(255, 255, 255, 0.35);
		text-decoration: none;
		letter-spacing: 0.02em;
		transition: color 0.2s;
	}

	.back:hover {
		color: #fff;
	}

	.hero {
		margin: 2rem 0 3rem;
	}

	.type-badge {
		display: inline-block;
		border: 1px solid rgba(255, 255, 255, 0.2);
		padding: 0.2rem 0.6rem;
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		margin-bottom: 1rem;
	}

	h1 {
		font-size: clamp(2rem, 5vw, 3.5rem);
		font-weight: 800;
		letter-spacing: -0.03em;
		margin: 0 0 0.25rem;
		line-height: 1.1;
	}

	.vendor {
		font-size: 1rem;
		color: rgba(255, 255, 255, 0.35);
		margin: 0.5rem 0 0;
		font-weight: 300;
	}

	.power-strip {
		display: flex;
		align-items: center;
		gap: 2rem;
		padding: 2rem 0;
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		margin-bottom: 3rem;
	}

	.power-item {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	.power-value {
		font-size: 2.5rem;
		font-weight: 800;
		font-family: 'JetBrains Mono', monospace;
		letter-spacing: -0.02em;
	}

	.power-unit {
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.15em;
		color: rgba(255, 255, 255, 0.3);
		font-weight: 500;
	}

	.power-divider {
		width: 1px;
		height: 3rem;
		background: rgba(255, 255, 255, 0.08);
	}

	.spec-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		margin-bottom: 3rem;
	}

	.spec-card {
		background: #000;
		padding: 2rem;
	}

	h2 {
		font-size: 0.7rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
		margin: 0 0 1.25rem;
	}

	.spec-rows {
		display: flex;
		flex-direction: column;
	}

	.spec-row {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		padding: 0.6rem 0;
		border-bottom: 1px solid rgba(255, 255, 255, 0.04);
	}

	.spec-row:last-child {
		border-bottom: none;
	}

	.spec-key {
		font-size: 0.85rem;
		color: rgba(255, 255, 255, 0.4);
		text-transform: capitalize;
	}

	.spec-val {
		font-size: 0.85rem;
		text-align: right;
	}

	.mono {
		font-family: 'JetBrains Mono', monospace;
	}

	.nested-params {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.nested-row {
		display: flex;
		gap: 1rem;
		justify-content: flex-end;
	}

	.nested-key {
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.3);
		text-transform: capitalize;
	}

	.nested-val {
		font-size: 0.75rem;
		font-family: 'JetBrains Mono', monospace;
	}

	.bool-badge {
		font-size: 0.7rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
	}

	.bool-badge.yes {
		color: #00ff88;
	}

	.actions {
		margin-bottom: 3rem;
		display: flex;
		gap: 1rem;
	}

	.btn-primary {
		display: inline-block;
		background: #fff;
		color: #000;
		padding: 0.75rem 2rem;
		font-size: 0.85rem;
		font-weight: 600;
		text-decoration: none;
		transition: all 0.3s;
	}

	.btn-primary:hover {
		background: rgba(255, 255, 255, 0.85);
		transform: translateY(-1px);
	}

	/* ── Quick Compare ── */

	.compare-section {
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		padding-top: 2.5rem;
		margin-bottom: 3rem;
	}

	.compare-hint {
		font-size: 0.85rem;
		color: rgba(255, 255, 255, 0.35);
		font-weight: 300;
		margin: 0 0 1.5rem;
	}

	.compare-controls {
		display: flex;
		gap: 1rem;
		align-items: stretch;
		margin-bottom: 1.5rem;
	}

	.compare-pills {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 0;
		border: 1px solid rgba(255, 255, 255, 0.08);
		min-width: 0;
	}

	.compare-pill.current {
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
		padding: 0.6rem 1rem;
		background: rgba(255, 255, 255, 0.04);
		white-space: nowrap;
		flex-shrink: 0;
	}

	.cp-type {
		font-size: 0.55rem;
		font-weight: 700;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
	}

	.cp-name {
		font-size: 0.8rem;
		font-weight: 600;
	}

	.compare-vs {
		padding: 0 0.75rem;
		font-size: 0.7rem;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.2);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		flex-shrink: 0;
		align-self: center;
	}

	.compare-select {
		flex: 1;
		min-width: 0;
		background: transparent;
		border: none;
		border-left: 1px solid rgba(255, 255, 255, 0.08);
		color: #fff;
		padding: 0.6rem 1rem;
		font-size: 0.8rem;
		font-family: 'Inter', sans-serif;
		cursor: pointer;
		outline: none;
	}

	.compare-select option {
		background: #111;
		color: #fff;
	}

	.btn-compare {
		background: #fff;
		color: #000;
		border: none;
		padding: 0 2rem;
		font-size: 0.75rem;
		font-weight: 700;
		font-family: 'Inter', sans-serif;
		cursor: pointer;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		transition: all 0.2s;
		white-space: nowrap;
	}

	.btn-compare:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.85);
	}

	.btn-compare:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.compare-preview {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		margin-bottom: 1.5rem;
	}

	.cp-card {
		background: #000;
		padding: 1.25rem;
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
	}

	.cp-card-label {
		font-size: 0.6rem;
		font-weight: 600;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.25);
	}

	.cp-card-name {
		font-size: 1rem;
		font-weight: 700;
	}

	.cp-card-specs {
		display: flex;
		gap: 1rem;
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.4);
		margin-top: 0.25rem;
	}

	.error-box {
		background: rgba(255, 51, 102, 0.1);
		border: 1px solid rgba(255, 51, 102, 0.2);
		color: #ff3366;
		padding: 1rem 1.25rem;
		font-size: 0.85rem;
		margin-bottom: 1.5rem;
	}

	.compare-charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(480px, 100%), 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.04);
	}

	.source {
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.2);
	}

	.source a {
		color: rgba(255, 255, 255, 0.35);
		text-decoration: none;
		word-break: break-all;
		transition: color 0.2s;
	}

	.source a:hover {
		color: #fff;
	}

	@media (max-width: 768px) {
		.hero {
			margin: 1.5rem 0 2rem;
		}

		.power-strip {
			flex-direction: column;
			gap: 0;
			padding: 0;
			margin-bottom: 2rem;
		}

		.power-item {
			padding: 1.25rem 0;
			flex-direction: row;
			align-items: baseline;
			gap: 0.75rem;
		}

		.power-value {
			font-size: 1.8rem;
		}

		.power-divider {
			width: 100%;
			height: 1px;
		}

		.spec-grid {
			grid-template-columns: 1fr;
			margin-bottom: 2rem;
		}

		.spec-card {
			padding: 1.25rem;
		}

		.actions {
			flex-direction: column;
		}

		.btn-primary {
			display: block;
			text-align: center;
			width: 100%;
		}

		.compare-controls {
			flex-direction: column;
		}

		.compare-pills {
			flex-direction: column;
			align-items: stretch;
		}

		.compare-pill.current {
			border-bottom: 1px solid rgba(255, 255, 255, 0.08);
		}

		.compare-vs {
			padding: 0.5rem 0;
			text-align: center;
		}

		.compare-select {
			border-left: none;
			border-top: 1px solid rgba(255, 255, 255, 0.08);
		}

		.btn-compare {
			padding: 0.85rem 2rem;
			width: 100%;
			text-align: center;
		}

		.compare-preview {
			grid-template-columns: 1fr;
		}

		.compare-charts {
			grid-template-columns: 1fr;
		}
	}
</style>
