<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign } from '$lib/types';

	let reactors = $state<ReactorDesign[]>([]);
	let loading = $state(true);
	let submitting = $state(false);
	let error = $state('');

	let selectedReactorId = $state('');
	let durationYears = $state(5);
	let timeStepDays = $state(30);
	let initialHeavyMetalTonnes = $state(60);
	let targetBurnupGwdT = $state(150);
	let breedingRatio = $state(0.8);
	let coolantInletTempC = $state(350);
	let coolantFlowRateKgS = $state(4400);
	let ratedElectricPowerMw = $state(345);

	// ── Tooltip state ──────────────────────────────────────
	let activeTooltip = $state<string | null>(null);

	function toggleTip(id: string) {
		activeTooltip = activeTooltip === id ? null : id;
	}

	// ── Param metadata: descriptions, ranges, warnings ─────
	interface ParamMeta {
		short: string;
		detail: string;
		typicalRange: string;
		warn?: (val: number) => string | null;
	}

	const PARAM_META: Record<string, ParamMeta> = {
		durationYears: {
			short: 'Total simulated operational time',
			detail:
				'How many years of reactor operation to model. Longer durations reveal fuel depletion curves and long-lived waste accumulation but increase computation time.',
			typicalRange: '1–40 years',
			warn: (v) => (v > 30 ? 'Durations >30y may exceed single fuel cycle — results assume no refueling unless breeding ratio sustains criticality.' : null)
		},
		timeStepDays: {
			short: 'Interval between simulation calculations',
			detail:
				'Smaller steps improve accuracy for fast-changing isotopes (like I-131, t½ = 8d) but increase result size. 30 days is a good default; use 7 days for detailed short-term waste analysis.',
			typicalRange: '1–90 days',
			warn: (v) => (v > 60 ? 'Steps >60d may miss short-lived fission product dynamics entirely.' : null)
		},
		initialHeavyMetalTonnes: {
			short: 'Total fissile + fertile fuel mass loaded into the core',
			detail:
				'The initial uranium/plutonium inventory. Larger cores (like SFRs) load 40–80t; small modular reactors may load 2–10t. This determines the absolute fission rate needed for rated power.',
			typicalRange: '2–80 tonnes'
		},
		targetBurnupGwdT: {
			short: 'Energy extracted per tonne of fuel before replacement',
			detail:
				'Measured in gigawatt-days per metric ton. Higher burnup means more energy from the same fuel, but requires higher enrichment. TRISO fuel can exceed 150 GWd/t; conventional UO₂ reaches ~50 GWd/t.',
			typicalRange: '40–200 GWd/t',
			warn: (v) =>
				v > 200
					? 'Burnup above 200 GWd/t exceeds demonstrated limits for most fuel forms.'
					: v < 20
						? 'Unusually low — typical only for natural uranium (CANDU) cycles.'
						: null
		},
		breedingRatio: {
			short: 'Rate of new fissile material creation vs. consumption',
			detail:
				'A ratio of 1.0 means the reactor produces as much fissile material as it burns (break-even breeder). Above 1.0 = net fissile production. Zero = once-through cycle with no breeding. Fast-spectrum reactors (SFR, LFR) typically achieve 0.7–1.2.',
			typicalRange: '0.0–1.2',
			warn: (v) => (v > 1.3 ? 'Breeding ratios >1.3 are not physically achievable in current designs.' : null)
		},
		coolantInletTempC: {
			short: 'Coolant temperature entering the reactor core',
			detail:
				'Set by the secondary loop heat exchangers. Sodium: ~350°C, Lead: ~400°C, Helium: ~250°C, FLiBe: ~550°C. The difference between inlet and outlet determines how much heat the coolant carries away.',
			typicalRange: '250–600°C'
		},
		coolantFlowRateKgS: {
			short: 'Mass of coolant passing through the core per second',
			detail:
				'Higher flow removes more heat but requires larger pumps. Sodium and lead are dense liquids with high flow rates (thousands of kg/s). Helium is a gas — much lower mass flow but compensated by high velocity.',
			typicalRange: '50–30,000 kg/s'
		},
		ratedElectricPowerMw: {
			short: 'Design electrical output after thermal-to-electric conversion',
			detail:
				'The nameplate electric capacity. Actual output varies over the fuel cycle as efficiency and capacity factor change. The ratio of electric to thermal power reflects the thermodynamic cycle efficiency (~33–48%).',
			typicalRange: '50–1,000 MWe'
		}
	};

	// ── Derived warnings for current values ────────────────
	let warnings = $derived.by(() => {
		const w: Record<string, string | null> = {};
		for (const [key, meta] of Object.entries(PARAM_META)) {
			if (!meta.warn) continue;
			const val = { durationYears, timeStepDays, initialHeavyMetalTonnes, targetBurnupGwdT, breedingRatio, coolantInletTempC, coolantFlowRateKgS, ratedElectricPowerMw }[key];
			w[key] = val != null ? meta.warn(val) : null;
		}
		return w;
	});

	// ── Efficiency preview (derived from current selections) ──
	let efficiencyPreview = $derived.by(() => {
		if (!selectedReactor) return null;
		const tOut = coolantInletTempC + (Number(selectedReactor.thermal_power_mw ?? 500) / (coolantFlowRateKgS * 1.26)); // rough Cp
		const tSinkK = 300 + 273.15;
		const tSourceK = tOut + 273.15;
		const carnot = 1 - tSinkK / tSourceK;
		const practical = carnot * 0.55;
		return {
			estOutletC: Math.round(tOut),
			carnotPct: (carnot * 100).toFixed(1),
			practicalPct: (practical * 100).toFixed(1)
		};
	});

	const COOLANT_MAP: Record<string, string> = {
		Sodium: 'Sodium',
		Lead: 'Lead',
		Helium: 'Helium',
		'Molten FLiBe salt': 'FLiBe',
		'Light water': 'LightWater',
		'Liquid Sodium': 'Sodium'
	};

	const CYCLE_MAP: Record<string, string> = {
		Sodium: 'Rankine',
		Lead: 'Rankine',
		Helium: 'Brayton',
		FLiBe: 'SCO2Brayton',
		LightWater: 'Rankine'
	};

	let selectedReactor = $derived(reactors.find((r) => r.id === selectedReactorId));

	$effect(() => {
		apiFetch<ReactorDesign[]>('/api/reactors')
			.then((data) => {
				reactors = data;
				const preselect = page.url.searchParams.get('reactor');
				if (preselect && data.some((r) => r.id === preselect)) {
					selectedReactorId = preselect;
				}
			})
			.catch((e) => (error = e.message))
			.finally(() => (loading = false));
	});

	$effect(() => {
		const r = selectedReactor;
		if (!r) return;
		initialHeavyMetalTonnes = Number(r.thermal_power_mw ?? 840) > 500 ? 60 : 10;
		targetBurnupGwdT = Number(r.enrichment_pct ?? 5) > 10 ? 150 : 60;
		breedingRatio = r.design_type === 'SFR' || r.design_type === 'LFR' ? 0.8 : 0.0;
		ratedElectricPowerMw = Number(r.electric_power_mw ?? 100);

		const coolant = r.coolant_type ?? 'Sodium';
		if (coolant.includes('Sodium') || coolant === 'Liquid Sodium') {
			coolantInletTempC = 350;
			coolantFlowRateKgS = 4400;
		} else if (coolant.includes('Helium')) {
			coolantInletTempC = 260;
			coolantFlowRateKgS = 80;
		} else if (coolant.includes('FLiBe') || coolant.includes('salt')) {
			coolantInletTempC = 550;
			coolantFlowRateKgS = 1350;
		} else if (coolant.includes('Lead')) {
			coolantInletTempC = 400;
			coolantFlowRateKgS = 25000;
		} else {
			coolantInletTempC = 280;
			coolantFlowRateKgS = 5000;
		}
	});

	async function launch() {
		if (!selectedReactor) return;
		submitting = true;
		error = '';

		const coolantRaw = selectedReactor.coolant_type ?? 'Sodium';
		const coolantType = COOLANT_MAP[coolantRaw] ?? 'Sodium';
		const cycleType = CYCLE_MAP[coolantType] ?? 'Rankine';

		const launchPayload = {
			reactor_id: selectedReactorId,
			params: {
				duration_years: durationYears,
				time_step_days: timeStepDays,
				initial_heavy_metal_tonnes: initialHeavyMetalTonnes,
				enrichment_pct: Number(selectedReactor.enrichment_pct ?? 5),
				target_burnup_gwd_t: targetBurnupGwdT,
				thermal_power_mw: Number(selectedReactor.thermal_power_mw ?? 100),
				breeding_ratio: breedingRatio,
				coolant_type: coolantType,
				coolant_inlet_temp_c: coolantInletTempC,
				coolant_flow_rate_kg_s: coolantFlowRateKgS,
				cycle_type: cycleType,
				rated_electric_power_mw: ratedElectricPowerMw
			}
		};

		sessionStorage.setItem('sim_launch', JSON.stringify(launchPayload));
		goto('/simulate/live');
		submitting = false;
	}
</script>

<h1>Configure Simulation</h1>
<p class="subtitle">Select a reactor and configure physics parameters for the simulation run.</p>

{#if loading}
	<p class="status">Loading reactors…</p>
{:else}
	<form onsubmit={(e) => { e.preventDefault(); launch(); }}>
		<section class="section">
			<h2>01 &mdash; Select Reactor</h2>
			<div class="reactor-selector">
				{#each reactors as r}
					<button
						type="button"
						class="reactor-option"
						class:selected={selectedReactorId === r.id}
						onclick={() => (selectedReactorId = r.id)}
					>
						<span class="ro-type">{r.design_type}</span>
						<span class="ro-name">{r.name}</span>
						<span class="ro-detail">
							{r.coolant_type ?? '—'} &middot; {r.fuel_type ?? '—'} &middot; {r.enrichment_pct ?? '?'}% enriched
						</span>
						<span class="ro-power mono">{r.thermal_power_mw ?? '?'} MWth &rarr; {r.electric_power_mw ?? '?'} MWe</span>
					</button>
				{/each}
			</div>
		</section>

		{#if selectedReactor}
			<!-- Reactor context banner -->
			<div class="reactor-context">
				<span class="rc-label">Modeling</span>
				<span class="rc-name">{selectedReactor.name}</span>
				<span class="rc-specs">
					{selectedReactor.coolant_type} cooled &middot;
					{selectedReactor.fuel_type} fuel &middot;
					{selectedReactor.enrichment_pct}% enrichment &middot;
					{selectedReactor.design_type} spectrum
				</span>
			</div>

			<section class="section">
				<h2>02 &mdash; Simulation Parameters</h2>
				<div class="param-grid">
					<label>
						<div class="param-header">
							<span class="param-label">Duration</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('durationYears')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.durationYears.short}</span>
						{#if activeTooltip === 'durationYears'}
							<div class="tip-expanded">{PARAM_META.durationYears.detail}<span class="tip-range">Typical: {PARAM_META.durationYears.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={durationYears} min="0.1" step="any" required />
							<span class="input-unit">years</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.durationYears.typicalRange}</span>
						{#if warnings.durationYears}
							<span class="param-warn">{warnings.durationYears}</span>
						{/if}
					</label>
					<label>
						<div class="param-header">
							<span class="param-label">Time Step</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('timeStepDays')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.timeStepDays.short}</span>
						{#if activeTooltip === 'timeStepDays'}
							<div class="tip-expanded">{PARAM_META.timeStepDays.detail}<span class="tip-range">Typical: {PARAM_META.timeStepDays.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={timeStepDays} min="1" step="1" required />
							<span class="input-unit">days</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.timeStepDays.typicalRange}</span>
						{#if warnings.timeStepDays}
							<span class="param-warn">{warnings.timeStepDays}</span>
						{/if}
					</label>
				</div>
			</section>

			<section class="section">
				<h2>03 &mdash; Fuel Configuration</h2>
				<p class="section-desc">
					Controls how fuel is consumed over the reactor's operating life. Burnup and breeding ratio
					are the primary differentiators between once-through and closed fuel cycles.
				</p>
				<div class="param-grid">
					<label>
						<div class="param-header">
							<span class="param-label">Heavy Metal Loading</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('initialHeavyMetalTonnes')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.initialHeavyMetalTonnes.short}</span>
						{#if activeTooltip === 'initialHeavyMetalTonnes'}
							<div class="tip-expanded">{PARAM_META.initialHeavyMetalTonnes.detail}<span class="tip-range">Typical: {PARAM_META.initialHeavyMetalTonnes.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={initialHeavyMetalTonnes} min="0.1" step="0.1" required />
							<span class="input-unit">tonnes</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.initialHeavyMetalTonnes.typicalRange}</span>
					</label>
					<label>
						<div class="param-header">
							<span class="param-label">Target Burnup</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('targetBurnupGwdT')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.targetBurnupGwdT.short}</span>
						{#if activeTooltip === 'targetBurnupGwdT'}
							<div class="tip-expanded">{PARAM_META.targetBurnupGwdT.detail}<span class="tip-range">Typical: {PARAM_META.targetBurnupGwdT.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={targetBurnupGwdT} min="1" step="1" required />
							<span class="input-unit">GWd/t</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.targetBurnupGwdT.typicalRange}</span>
						{#if warnings.targetBurnupGwdT}
							<span class="param-warn">{warnings.targetBurnupGwdT}</span>
						{/if}
					</label>
					<label>
						<div class="param-header">
							<span class="param-label">Breeding Ratio</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('breedingRatio')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.breedingRatio.short}</span>
						{#if activeTooltip === 'breedingRatio'}
							<div class="tip-expanded">{PARAM_META.breedingRatio.detail}<span class="tip-range">Typical: {PARAM_META.breedingRatio.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={breedingRatio} min="0" max="2" step="0.1" required />
						</div>
						<span class="param-range">Typical: {PARAM_META.breedingRatio.typicalRange}</span>
						{#if warnings.breedingRatio}
							<span class="param-warn">{warnings.breedingRatio}</span>
						{/if}
					</label>
				</div>
			</section>

			<section class="section">
				<h2>04 &mdash; Coolant & Power Cycle</h2>
				<p class="section-desc">
					Coolant properties determine heat removal from the core. The temperature difference
					between inlet and outlet directly sets thermodynamic cycle efficiency.
				</p>
				<div class="param-grid">
					<label>
						<div class="param-header">
							<span class="param-label">Inlet Temperature</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('coolantInletTempC')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.coolantInletTempC.short}</span>
						{#if activeTooltip === 'coolantInletTempC'}
							<div class="tip-expanded">{PARAM_META.coolantInletTempC.detail}<span class="tip-range">Typical: {PARAM_META.coolantInletTempC.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={coolantInletTempC} step="1" required />
							<span class="input-unit">&deg;C</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.coolantInletTempC.typicalRange}</span>
					</label>
					<label>
						<div class="param-header">
							<span class="param-label">Flow Rate</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('coolantFlowRateKgS')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.coolantFlowRateKgS.short}</span>
						{#if activeTooltip === 'coolantFlowRateKgS'}
							<div class="tip-expanded">{PARAM_META.coolantFlowRateKgS.detail}<span class="tip-range">Typical: {PARAM_META.coolantFlowRateKgS.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={coolantFlowRateKgS} min="1" step="1" required />
							<span class="input-unit">kg/s</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.coolantFlowRateKgS.typicalRange}</span>
					</label>
					<label>
						<div class="param-header">
							<span class="param-label">Rated Electric Power</span>
							<button type="button" class="tip-btn" onclick={() => toggleTip('ratedElectricPowerMw')} aria-label="More info">?</button>
						</div>
						<span class="param-hint">{PARAM_META.ratedElectricPowerMw.short}</span>
						{#if activeTooltip === 'ratedElectricPowerMw'}
							<div class="tip-expanded">{PARAM_META.ratedElectricPowerMw.detail}<span class="tip-range">Typical: {PARAM_META.ratedElectricPowerMw.typicalRange}</span></div>
						{/if}
						<div class="input-wrap">
							<input type="number" bind:value={ratedElectricPowerMw} min="0" step="1" required />
							<span class="input-unit">MWe</span>
						</div>
						<span class="param-range">Typical: {PARAM_META.ratedElectricPowerMw.typicalRange}</span>
					</label>
				</div>
			</section>

			<!-- Live efficiency estimate -->
			{#if efficiencyPreview}
				<div class="efficiency-preview">
					<h2>05 &mdash; Estimated Output</h2>
					<p class="section-desc">Derived from your current parameter selections. These are starting estimates. Actual values evolve over the simulation as fuel depletes and temperatures shift.</p>
					<div class="ep-grid">
						<div class="ep-item">
							<span class="ep-value mono">{efficiencyPreview.estOutletC}°C</span>
							<span class="ep-label">Est. outlet temp</span>
						</div>
						<div class="ep-item">
							<span class="ep-value mono">{efficiencyPreview.carnotPct}%</span>
							<span class="ep-label">Carnot limit</span>
						</div>
						<div class="ep-item">
							<span class="ep-value mono">{efficiencyPreview.practicalPct}%</span>
							<span class="ep-label">Est. cycle efficiency</span>
						</div>
					</div>
				</div>
			{/if}

			{#if error}
				<p class="error">{error}</p>
			{/if}

			<button type="submit" class="btn-launch" disabled={submitting}>
				{submitting ? 'Launching…' : 'Launch Simulation'}
			</button>
		{/if}
	</form>
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
		margin: 0 0 3rem;
	}

	.section {
		margin-bottom: 2.5rem;
	}

	h2 {
		font-size: 0.7rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
		margin: 0 0 0.5rem;
	}

	.section-desc {
		color: rgba(255, 255, 255, 0.25);
		font-size: 0.8rem;
		line-height: 1.5;
		margin: 0 0 1.25rem;
		max-width: 60ch;
	}

	/* ── Reactor selector ────────────────────────────── */

	.reactor-selector {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.08);
	}

	.reactor-option {
		background: #000;
		border: none;
		padding: 1.25rem;
		text-align: left;
		color: rgba(255, 255, 255, 0.5);
		cursor: pointer;
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		transition: all 0.2s;
		font-family: 'Inter', sans-serif;
	}

	.reactor-option:hover {
		background: rgba(255, 255, 255, 0.03);
		color: #fff;
	}

	.reactor-option.selected {
		background: #fff;
		color: #000;
	}

	.ro-type {
		font-size: 0.6rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		opacity: 0.4;
	}

	.reactor-option.selected .ro-type { opacity: 0.5; }

	.ro-name {
		font-size: 0.9rem;
		font-weight: 600;
	}

	.ro-detail {
		font-size: 0.7rem;
		opacity: 0.45;
		line-height: 1.4;
	}

	.ro-power {
		font-size: 0.8rem;
		opacity: 0.6;
		margin-top: 0.15rem;
	}

	.mono { font-family: 'JetBrains Mono', monospace; }

	/* ── Reactor context banner ──────────────────────── */

	.reactor-context {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0.5rem 1rem;
		padding: 1rem 1.25rem;
		margin-bottom: 2.5rem;
		border-left: 2px solid rgba(255, 255, 255, 0.15);
		background: rgba(255, 255, 255, 0.02);
	}

	.rc-label {
		font-size: 0.6rem;
		font-weight: 600;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
	}

	.rc-name {
		font-size: 1rem;
		font-weight: 700;
		color: #fff;
	}

	.rc-specs {
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.35);
		width: 100%;
	}

	/* ── Parameter inputs ────────────────────────────── */

	.param-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 1.25rem;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
	}

	.param-header {
		display: flex;
		align-items: center;
		gap: 0.4rem;
	}

	.param-label {
		font-size: 0.7rem;
		font-weight: 500;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.35);
	}

	.param-hint {
		font-size: 0.72rem;
		color: rgba(255, 255, 255, 0.2);
		line-height: 1.4;
	}

	.param-range {
		font-size: 0.65rem;
		color: rgba(255, 255, 255, 0.15);
		font-family: 'JetBrains Mono', monospace;
	}

	.param-warn {
		font-size: 0.7rem;
		color: #f0993b;
		line-height: 1.4;
		padding: 0.35rem 0.5rem;
		background: rgba(240, 153, 59, 0.08);
		border-left: 2px solid rgba(240, 153, 59, 0.3);
	}

	/* ── Tooltip button & expanded detail ────────────── */

	.tip-btn {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 1px solid rgba(255, 255, 255, 0.15);
		background: transparent;
		color: rgba(255, 255, 255, 0.25);
		font-size: 0.55rem;
		font-weight: 700;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		transition: all 0.15s;
		padding: 0;
		font-family: 'Inter', sans-serif;
	}

	.tip-btn:hover {
		border-color: rgba(255, 255, 255, 0.35);
		color: rgba(255, 255, 255, 0.6);
	}

	.tip-expanded {
		font-size: 0.75rem;
		line-height: 1.55;
		color: rgba(255, 255, 255, 0.4);
		padding: 0.6rem 0.75rem;
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.06);
	}

	.tip-range {
		display: block;
		margin-top: 0.35rem;
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.65rem;
		color: rgba(255, 255, 255, 0.25);
	}

	/* ── Input fields ────────────────────────────────── */

	.input-wrap {
		display: flex;
		align-items: center;
		border: 1px solid rgba(255, 255, 255, 0.1);
		transition: border-color 0.2s;
	}

	.input-wrap:focus-within {
		border-color: rgba(255, 255, 255, 0.3);
	}

	input {
		flex: 1;
		background: transparent;
		border: none;
		color: #fff;
		padding: 0.7rem 0.85rem;
		font-size: 0.9rem;
		font-family: 'JetBrains Mono', monospace;
		outline: none;
		min-width: 0;
	}

	.input-unit {
		padding-right: 0.85rem;
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.25);
		letter-spacing: 0.05em;
		white-space: nowrap;
	}

	/* ── Efficiency preview ──────────────────────────── */

	.efficiency-preview {
		margin-bottom: 2.5rem;
		padding: 1.25rem;
		border: 1px solid rgba(255, 255, 255, 0.06);
		background: rgba(255, 255, 255, 0.02);
	}

	.ep-grid {
		display: flex;
		gap: 2rem;
		flex-wrap: wrap;
	}

	.ep-item {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	.ep-value {
		font-size: 1.4rem;
		font-weight: 700;
		color: #fff;
	}

	.ep-label {
		font-size: 0.65rem;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		color: rgba(255, 255, 255, 0.25);
	}

	/* ── Actions ─────────────────────────────────────── */

	.btn-launch {
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
	}

	.btn-launch:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.85);
		transform: translateY(-1px);
	}

	.btn-launch:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.status { color: rgba(255, 255, 255, 0.4); }
	.error { color: #ff3366; margin-bottom: 1rem; }

	/* ── Responsive ──────────────────────────────────── */

	@media (max-width: 768px) {
		h1 { font-size: 1.5rem; }
		.subtitle { font-size: 0.85rem; margin-bottom: 2rem; }
		.reactor-selector { grid-template-columns: 1fr; }
		.reactor-option {
			padding: 1rem;
		}
		.param-grid { grid-template-columns: 1fr; gap: 1rem; }
		.btn-launch { width: 100%; text-align: center; }
		.section { margin-bottom: 2rem; }
		.ep-grid { gap: 1.25rem; }
	}
</style>