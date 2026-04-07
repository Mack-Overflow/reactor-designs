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
	let mobilePhase = $state<1 | 2>(1);
	let durationYears = $state(5);
	let timeStepDays = $state(30);
	let initialHeavyMetalTonnes = $state(60);
	let targetBurnupGwdT = $state(150);
	let breedingRatio = $state(0.8);
	let coolantInletTempC = $state(350);
	let coolantFlowRateKgS = $state(4400);
	let ratedElectricPowerMw = $state(345);

	// ── Tooltip state ──────────────────────────────────
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

	// ── Derived warnings for current values ────────────
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
					mobilePhase = 2;
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
				reactor_name: selectedReactor.name,
				reactor_type: selectedReactor.design_type,
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

<h1 class="text-3xl font-extrabold tracking-tight m-0 mb-2 max-lg:text-2xl">Configure Simulation</h1>
<p class="text-white/35 text-[0.95rem] font-light mb-12 max-lg:text-[0.85rem] max-lg:mb-8">Select a reactor and configure physics parameters for the simulation run.</p>

{#if loading}
	<p class="text-white/40">Loading reactors...</p>
{:else}
	<form onsubmit={(e) => { e.preventDefault(); launch(); }}>
		<div class:max-lg:hidden={mobilePhase !== 1}>
			<section class="mb-10 max-lg:mb-8">
				<h2 class="section-heading mb-2">01 &mdash; Select Reactor</h2>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] grid-divider bg-white/8">
					{#each reactors as r}
						<button
							type="button"
							class="bg-black border-none p-5 text-left text-white/50 cursor-pointer flex flex-col gap-1 transition-all duration-200 font-sans hover:bg-white/3 hover:text-white max-lg:p-4 {selectedReactorId === r.id ? '!bg-white !text-black' : ''}"
							onclick={() => { selectedReactorId = r.id; mobilePhase = 2; }}
						>
							<span class="text-[0.6rem] font-bold tracking-wide uppercase opacity-40 {selectedReactorId === r.id ? '!opacity-50' : ''}">{r.design_type}</span>
							<span class="text-[0.9rem] font-semibold">{r.name}</span>
							<span class="text-[0.7rem] opacity-45 leading-snug">
								{r.coolant_type ?? '—'} &middot; {r.fuel_type ?? '—'} &middot; {r.enrichment_pct ?? '?'}% enriched
							</span>
							<span class="text-[0.8rem] opacity-60 mt-0.5 font-mono">{r.thermal_power_mw ?? '?'} MWth &rarr; {r.electric_power_mw ?? '?'} MWe</span>
						</button>
					{/each}
				</div>
			</section>
		</div>

		{#if selectedReactor}
		<div class:max-lg:hidden={mobilePhase !== 2}>
			<button type="button" class="hidden max-lg:inline-flex max-lg:items-center max-lg:gap-1 bg-none border-none text-white/40 text-[0.8rem] font-sans cursor-pointer p-0 mb-6 transition-colors duration-200 hover:text-white" onclick={() => (mobilePhase = 1)}>
				&larr; Change Reactor
			</button>
			<!-- Reactor context banner -->
			<div class="flex flex-wrap items-baseline gap-y-2 gap-x-4 px-5 py-4 mb-10 border-l-2 border-white/15 bg-white/2">
				<span class="text-[0.6rem] font-semibold tracking-caps uppercase text-white/30">Modeling</span>
				<span class="text-base font-bold text-white">{selectedReactor.name}</span>
				<span class="text-xs text-white/35 w-full">
					{selectedReactor.coolant_type} cooled &middot;
					{selectedReactor.fuel_type} fuel &middot;
					{selectedReactor.enrichment_pct}% enrichment &middot;
					{selectedReactor.design_type} spectrum
				</span>
			</div>

			<section class="mb-10 max-lg:mb-8">
				<h2 class="section-heading mb-2">02 &mdash; Simulation Parameters</h2>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-5 max-lg:grid-cols-1 max-lg:gap-4">
					{@render paramField('durationYears', 'Duration', durationYears, (v) => durationYears = v, 0.1, undefined, 'years')}
					{@render paramField('timeStepDays', 'Time Step', timeStepDays, (v) => timeStepDays = v, 1, undefined, 'days')}
				</div>
			</section>

			<section class="mb-10 max-lg:mb-8">
				<h2 class="section-heading mb-2">03 &mdash; Fuel Configuration</h2>
				<p class="text-white/40 text-[0.8rem] leading-relaxed mb-5 max-w-[60ch]">
					Controls how fuel is consumed over the reactor's operating life. Burnup and breeding ratio
					are the primary differentiators between once-through and closed fuel cycles.
				</p>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-5 max-lg:grid-cols-1 max-lg:gap-4">
					{@render paramField('initialHeavyMetalTonnes', 'Heavy Metal Loading', initialHeavyMetalTonnes, (v) => initialHeavyMetalTonnes = v, 0.1, undefined, 'tonnes')}
					{@render paramField('targetBurnupGwdT', 'Target Burnup', targetBurnupGwdT, (v) => targetBurnupGwdT = v, 1, undefined, 'GWd/t')}
					{@render paramField('breedingRatio', 'Breeding Ratio', breedingRatio, (v) => breedingRatio = v, 0, 2)}
				</div>
			</section>

			<section class="mb-10 max-lg:mb-8">
				<h2 class="section-heading mb-2">04 &mdash; Coolant & Power Cycle</h2>
				<p class="text-white/40 text-[0.8rem] leading-relaxed mb-5 max-w-[60ch]">
					Coolant properties determine heat removal from the core. The temperature difference
					between inlet and outlet directly sets thermodynamic cycle efficiency.
				</p>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-5 max-lg:grid-cols-1 max-lg:gap-4">
					{@render paramField('coolantInletTempC', 'Inlet Temperature', coolantInletTempC, (v) => coolantInletTempC = v, undefined, undefined, '°C')}
					{@render paramField('coolantFlowRateKgS', 'Flow Rate', coolantFlowRateKgS, (v) => coolantFlowRateKgS = v, 1, undefined, 'kg/s')}
					{@render paramField('ratedElectricPowerMw', 'Rated Electric Power', ratedElectricPowerMw, (v) => ratedElectricPowerMw = v, 0, undefined, 'MWe')}
				</div>
			</section>

			<!-- Live efficiency estimate -->
			{#if efficiencyPreview}
				<div class="mb-10 p-5 border border-white/6 bg-white/2">
					<h2 class="section-heading mb-2">05 &mdash; Estimated Output</h2>
					<p class="text-white/40 text-[0.8rem] leading-relaxed mb-5 max-w-[60ch]">Derived from your current parameter selections. These are starting estimates. Actual values evolve over the simulation as fuel depletes and temperatures shift.</p>
					<div class="flex gap-8 flex-wrap max-lg:gap-5">
						<div class="flex flex-col gap-1">
							<span class="text-[1.4rem] font-bold text-white font-mono">{efficiencyPreview.estOutletC}°C</span>
							<span class="text-[0.65rem] font-medium uppercase tracking-[0.1em] text-white/25">Est. outlet temp</span>
						</div>
						<div class="flex flex-col gap-1">
							<span class="text-[1.4rem] font-bold text-white font-mono">{efficiencyPreview.carnotPct}%</span>
							<span class="text-[0.65rem] font-medium uppercase tracking-[0.1em] text-white/25">Carnot limit</span>
						</div>
						<div class="flex flex-col gap-1">
							<span class="text-[1.4rem] font-bold text-white font-mono">{efficiencyPreview.practicalPct}%</span>
							<span class="text-[0.65rem] font-medium uppercase tracking-[0.1em] text-white/25">Est. cycle efficiency</span>
						</div>
					</div>
				</div>
			{/if}

			{#if error}
				<p class="text-error mb-4">{error}</p>
			{/if}

			<button type="submit" class="btn-primary py-3.5 px-10 text-[0.85rem] font-bold tracking-[0.03em] max-lg:w-full max-lg:text-center" disabled={submitting}>
				{submitting ? 'Launching...' : 'Launch Simulation'}
			</button>
		</div>
		{/if}
	</form>
{/if}

{#snippet paramField(key: string, label: string, value: number, setValue: (v: number) => void, min?: number, max?: number, unit?: string)}
	<label class="flex flex-col gap-1">
		<div class="flex items-center gap-1.5">
			<span class="text-[0.7rem] font-medium tracking-nav uppercase text-white/35">{label}</span>
			<button type="button" class="w-4 h-4 rounded-full border border-white/15 bg-transparent text-white/25 text-[0.55rem] font-bold cursor-pointer inline-flex items-center justify-center shrink-0 transition-all duration-150 p-0 font-sans hover:border-white/35 hover:text-white/60" onclick={() => toggleTip(key)} aria-label="More info">?</button>
		</div>
		<span class="text-[0.72rem] text-white/35 leading-snug">{PARAM_META[key].short}</span>
		{#if activeTooltip === key}
			<div class="text-xs leading-relaxed text-white/40 px-3 py-2.5 bg-white/3 border border-white/6">
				{PARAM_META[key].detail}
				<span class="block mt-1.5 font-mono text-[0.65rem] text-white/25">Typical: {PARAM_META[key].typicalRange}</span>
			</div>
		{/if}
		<div class="flex items-center border border-white/10 transition-colors duration-200 focus-within:border-white/30">
			<input type="number" {value} oninput={(e) => setValue(Number((e.target as HTMLInputElement).value))} min={min} max={max} step="any" required class="flex-1 bg-transparent border-none text-white py-3 px-3.5 text-[0.9rem] font-mono outline-none min-w-0" />
			{#if unit}
				<span class="pr-3.5 text-[0.7rem] text-white/25 tracking-[0.05em] whitespace-nowrap">{unit}</span>
			{/if}
		</div>
		<span class="text-[0.65rem] text-white/35 font-mono">Typical: {PARAM_META[key].typicalRange}</span>
		{#if warnings[key]}
			<span class="text-[0.7rem] text-[#f0993b] leading-snug py-1.5 px-2 bg-[rgba(240,153,59,0.08)] border-l-2 border-[rgba(240,153,59,0.3)]">{warnings[key]}</span>
		{/if}
	</label>
{/snippet}
