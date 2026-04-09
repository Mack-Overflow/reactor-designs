<script lang="ts">
	import { page } from '$app/stores';
	import { apiFetch } from '$lib/api';
	import { Chart, registerables } from 'chart.js';
	import { onMount } from 'svelte';
	import type {
		ReactorDesign,
		CoolantType,
		CycleTypeEnum,
		ThermalRequest,
		ThermalResponse,
		PowerRequest,
		PowerResponse,
		FuelRequest,
		FuelResponse,
		WasteRequest,
		WasteResponse
	} from '$lib/types';

	Chart.register(...registerables);

	// ── Module metadata ──

	type ModuleKey = 'thermal' | 'power' | 'fuel' | 'waste';

	const MODULE_META: Record<ModuleKey, { number: string; title: string; desc: string }> = {
		thermal: {
			number: '02',
			title: 'Thermal Hydraulics',
			desc: 'Single-channel coolant temperature rise across the core — ΔT = Q / (ṁ × Cp)'
		},
		power: {
			number: '03',
			title: 'Power Generation',
			desc: 'Thermodynamic cycle efficiency vs coolant outlet temperature, bounded by the Carnot limit'
		},
		fuel: {
			number: '01',
			title: 'Fuel Cycle',
			desc: 'Burnup accumulation, fissile depletion and breeding gain across the full fuel cycle'
		},
		waste: {
			number: '04',
			title: 'Waste & Decay',
			desc: 'Bateman equation decay chains — fission products and actinide inventory over time'
		}
	};

	const module = $derived($page.params.module as ModuleKey);
	const meta = $derived(MODULE_META[module] ?? MODULE_META['thermal']);

	// ── Reactor list ──

	let reactors = $state<ReactorDesign[]>([]);
	let selectedReactorId = $state('');
	let loadingReactors = $state(true);

	$effect(() => {
		apiFetch<ReactorDesign[]>('/api/reactors')
			.then((d) => {
				reactors = d;
				if (d.length > 0) {
					selectedReactorId = d[0].id;
					applyReactorDefaults(d[0]);
				}
			})
			.catch(console.error)
			.finally(() => (loadingReactors = false));
	});

	function coolantTypeFromString(s: string | null): CoolantType {
		if (!s) return 'LightWater';
		const l = s.toLowerCase();
		if (l.includes('sodium')) return 'Sodium';
		if (l.includes('lead')) return 'Lead';
		if (l.includes('helium')) return 'Helium';
		if (l.includes('flibe') || l.includes('salt')) return 'FLiBe';
		return 'LightWater';
	}

	function cycleTypeFromDesign(designType: string): CycleTypeEnum {
		const t = designType.toUpperCase();
		if (t === 'HTGR') return 'Brayton';
		if (t === 'MSR' || t === 'FHR') return 'SCO2Brayton';
		return 'Rankine';
	}

	// Default inlet temps (°C) and nominal ΔT for flow rate estimation
	const COOLANT_INLET: Record<CoolantType, number> = {
		Sodium: 350,
		Lead: 400,
		Helium: 260,
		FLiBe: 550,
		LightWater: 280
	};
	const COOLANT_DELTA_T: Record<CoolantType, number> = {
		Sodium: 150,
		Lead: 150,
		Helium: 450,
		FLiBe: 100,
		LightWater: 15
	};
	const COOLANT_CP: Record<CoolantType, number> = {
		Sodium: 1.35,
		Lead: 0.147,
		Helium: 5.193,
		FLiBe: 2.4,
		LightWater: 4.5
	};

	function estimateFlowRate(ct: CoolantType, thermalMw: number): number {
		return Math.round((thermalMw * 1000) / (COOLANT_CP[ct] * COOLANT_DELTA_T[ct]));
	}

	// ── Per-module form state ──

	// Thermal
	let th_power = $state(840);
	let th_coolant = $state<CoolantType>('Sodium');
	let th_inlet = $state(350);
	let th_flow = $state(4400);

	// Power
	let pw_thermal = $state(840);
	let pw_outlet = $state(500);
	let pw_cycle = $state<CycleTypeEnum>('Rankine');
	let pw_rated = $state(345);

	// Fuel
	let fu_hm = $state(60);
	let fu_enrich = $state(15);
	let fu_burnup = $state(150);
	let fu_power = $state(840);
	let fu_br = $state(0.8);
	let fu_duration = $state(15);
	let fu_timestep = $state(30);

	// Waste
	let wa_power = $state(840);
	let wa_br = $state(0.8);
	let wa_duration = $state(15);
	let wa_timestep = $state(30);

	function applyReactorDefaults(r: ReactorDesign) {
		const ct = coolantTypeFromString(r.coolant_type);
		const thermalMw = r.thermal_power_mw ?? 840;
		const electricMw = r.electric_power_mw ?? 345;
		const enrich = r.enrichment_pct ?? 15;
		const flow = estimateFlowRate(ct, thermalMw);
		const cycle = cycleTypeFromDesign(r.design_type);

		th_power = thermalMw;
		th_coolant = ct;
		th_inlet = COOLANT_INLET[ct];
		th_flow = flow;

		pw_thermal = thermalMw;
		pw_outlet = COOLANT_INLET[ct] + COOLANT_DELTA_T[ct];
		pw_cycle = cycle;
		pw_rated = electricMw;

		fu_hm = 60;
		fu_enrich = enrich;
		fu_power = thermalMw;

		wa_power = thermalMw;
	}

	$effect(() => {
		const r = reactors.find((x) => x.id === selectedReactorId);
		if (r) applyReactorDefaults(r);
	});

	// ── Results state ──

	let thermalResult = $state<ThermalResponse | null>(null);
	let powerResult = $state<PowerResponse | null>(null);
	let fuelResult = $state<FuelResponse | null>(null);
	let wasteResult = $state<WasteResponse | null>(null);
	let running = $state(false);
	let error = $state('');

	async function runModule() {
		running = true;
		error = '';
		try {
			if (module === 'thermal') {
				const req: ThermalRequest = {
					thermal_power_mw: Number(th_power),
					coolant_type: th_coolant,
					inlet_temp_c: Number(th_inlet),
					flow_rate_kg_s: Number(th_flow)
				};
				thermalResult = await apiFetch<ThermalResponse>('/api/modules/thermal', {
					method: 'POST',
					body: JSON.stringify(req)
				});
			} else if (module === 'power') {
				const req: PowerRequest = {
					thermal_power_mw: Number(pw_thermal),
					outlet_temp_c: Number(pw_outlet),
					cycle_type: pw_cycle,
					rated_electric_power_mw: Number(pw_rated)
				};
				powerResult = await apiFetch<PowerResponse>('/api/modules/power', {
					method: 'POST',
					body: JSON.stringify(req)
				});
			} else if (module === 'fuel') {
				const req: FuelRequest = {
					initial_heavy_metal_tonnes: Number(fu_hm),
					enrichment_pct: Number(fu_enrich),
					target_burnup_gwd_t: Number(fu_burnup),
					thermal_power_mw: Number(fu_power),
					breeding_ratio: Number(fu_br),
					duration_years: Number(fu_duration),
					time_step_days: Number(fu_timestep)
				};
				fuelResult = await apiFetch<FuelResponse>('/api/modules/fuel', {
					method: 'POST',
					body: JSON.stringify(req)
				});
			} else if (module === 'waste') {
				const req: WasteRequest = {
					thermal_power_mw: Number(wa_power),
					breeding_ratio: Number(wa_br),
					duration_years: Number(wa_duration),
					time_step_days: Number(wa_timestep)
				};
				wasteResult = await apiFetch<WasteResponse>('/api/modules/waste', {
					method: 'POST',
					body: JSON.stringify(req)
				});
			}
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			running = false;
		}
	}

	// ── Chart canvases ──

	let chartA: HTMLCanvasElement;
	let chartB: HTMLCanvasElement;
	let chartC: HTMLCanvasElement;
	let chartInstA: Chart | undefined;
	let chartInstB: Chart | undefined;
	let chartInstC: Chart | undefined;

	const COLORS = ['#fff', '#ff3366', '#00ff88', '#ffcc00', '#66ccff', '#cc66ff'];

	const BASE_CHART_OPTIONS = {
		responsive: true,
		maintainAspectRatio: false,
		animation: { duration: 300 },
		plugins: {
			legend: {
				labels: {
					color: 'rgba(255,255,255,0.5)',
					font: { size: 11, family: 'Inter' },
					boxWidth: 12,
					boxHeight: 2,
					padding: 16
				}
			},
			tooltip: {
				backgroundColor: '#fff',
				titleColor: '#000',
				bodyColor: '#000',
				titleFont: { family: 'JetBrains Mono', size: 11 },
				bodyFont: { family: 'JetBrains Mono', size: 11 },
				padding: 10,
				cornerRadius: 4
			}
		},
		scales: {
			x: {
				ticks: { color: 'rgba(255,255,255,0.2)', font: { size: 10, family: 'JetBrains Mono' } },
				grid: { color: 'rgba(255,255,255,0.04)' },
				border: { color: 'rgba(255,255,255,0.08)' }
			},
			y: {
				ticks: { color: 'rgba(255,255,255,0.2)', font: { size: 10, family: 'JetBrains Mono' } },
				grid: { color: 'rgba(255,255,255,0.04)' },
				border: { color: 'rgba(255,255,255,0.08)' }
			}
		}
	};

	function destroyCharts() {
		chartInstA?.destroy();
		chartInstB?.destroy();
		chartInstC?.destroy();
	}

	$effect(() => {
		if (thermalResult && module === 'thermal') buildThermalCharts(thermalResult);
		if (powerResult && module === 'power') buildPowerCharts(powerResult);
		if (fuelResult && module === 'fuel') buildFuelCharts(fuelResult);
		if (wasteResult && module === 'waste') buildWasteCharts(wasteResult);
	});

	function buildThermalCharts(r: ThermalResponse) {
		destroyCharts();
		// Chart A: coolant comparison bar chart
		if (chartA) {
			chartInstA = new Chart(chartA, {
				type: 'bar',
				data: {
					labels: r.coolant_comparison.map((c) => c.coolant),
					datasets: [
						{
							label: 'Outlet Temp (°C)',
							data: r.coolant_comparison.map((c) => c.outlet_temp_c),
							backgroundColor: COLORS.map((c) => c + '44'),
							borderColor: COLORS,
							borderWidth: 1
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'OUTLET TEMPERATURE BY COOLANT',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					}
				} as never
			});
		}
		// Chart B: power curve line chart
		if (chartB) {
			chartInstB = new Chart(chartB, {
				type: 'line',
				data: {
					labels: r.power_curve.map((p) => p.thermal_power_mw.toFixed(0) + ' MW'),
					datasets: [
						{
							label: 'Outlet Temp (°C)',
							data: r.power_curve.map((p) => p.outlet_temp_c),
							borderColor: '#fff',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						},
						{
							label: 'ΔT (°C)',
							data: r.power_curve.map((p) => p.delta_t_c),
							borderColor: '#ff3366',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'TEMPERATURE vs THERMAL POWER',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					}
				} as never
			});
		}
	}

	function buildPowerCharts(r: PowerResponse) {
		destroyCharts();
		// Chart A: cycle comparison bar chart
		if (chartA) {
			chartInstA = new Chart(chartA, {
				type: 'bar',
				data: {
					labels: r.cycle_comparison.map((c) => c.cycle),
					datasets: [
						{
							label: 'Efficiency (%)',
							data: r.cycle_comparison.map((c) => +(c.efficiency * 100).toFixed(1)),
							backgroundColor: COLORS.map((c) => c + '44'),
							borderColor: COLORS,
							borderWidth: 1
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'CYCLE EFFICIENCY COMPARISON',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					}
				} as never
			});
		}
		// Chart B: efficiency vs outlet temp
		if (chartB) {
			chartInstB = new Chart(chartB, {
				type: 'line',
				data: {
					labels: r.efficiency_curve.map((p) => p.outlet_temp_c.toFixed(0) + '°C'),
					datasets: [
						{
							label: 'Cycle Efficiency (%)',
							data: r.efficiency_curve.map((p) => +(p.efficiency * 100).toFixed(2)),
							borderColor: '#fff',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						},
						{
							label: 'Carnot Limit (%)',
							data: r.efficiency_curve.map((p) => +(p.carnot * 100).toFixed(2)),
							borderColor: 'rgba(255,255,255,0.25)',
							backgroundColor: 'transparent',
							borderDash: [4, 4],
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'EFFICIENCY vs OUTLET TEMPERATURE',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					}
				} as never
			});
		}
	}

	function buildFuelCharts(r: FuelResponse) {
		destroyCharts();
		const labels = r.steps.map((s) => s.time_years.toFixed(2));
		// Chart A: burnup over time
		if (chartA) {
			chartInstA = new Chart(chartA, {
				type: 'line',
				data: {
					labels,
					datasets: [
						{
							label: 'Burnup (GWd/t)',
							data: r.steps.map((s) => s.burnup_gwd_t),
							borderColor: '#fff',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'BURNUP ACCUMULATION (GWd/t)',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					},
					scales: {
						...BASE_CHART_OPTIONS.scales,
						x: { ...BASE_CHART_OPTIONS.scales.x, title: { display: true, text: 'Time (Years)', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } } }
					}
				} as never
			});
		}
		// Chart B: fissile fraction + thermal power
		if (chartB) {
			chartInstB = new Chart(chartB, {
				type: 'line',
				data: {
					labels,
					datasets: [
						{
							label: 'Fissile Fraction',
							data: r.steps.map((s) => +s.fissile_fraction.toFixed(4)),
							borderColor: '#ff3366',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5,
							yAxisID: 'y'
						},
						{
							label: 'Thermal Power (MWth)',
							data: r.steps.map((s) => s.effective_thermal_power_mw),
							borderColor: '#ffcc00',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5,
							yAxisID: 'y2'
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'FISSILE FRACTION & THERMAL POWER',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					},
					scales: {
						x: { ...BASE_CHART_OPTIONS.scales.x, title: { display: true, text: 'Time (Years)', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } } },
						y: {
							...BASE_CHART_OPTIONS.scales.y,
							position: 'left',
							title: { display: true, text: 'Fissile Fraction', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } }
						},
						y2: {
							...BASE_CHART_OPTIONS.scales.y,
							position: 'right',
							grid: { drawOnChartArea: false },
							title: { display: true, text: 'Power (MWth)', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } }
						}
					}
				} as never
			});
		}
	}

	function buildWasteCharts(r: WasteResponse) {
		destroyCharts();
		const labels = r.steps.map((s) => s.time_years.toFixed(2));
		// Chart A: stacked area — actinides vs fission products
		if (chartA) {
			chartInstA = new Chart(chartA, {
				type: 'line',
				data: {
					labels,
					datasets: [
						{
							label: 'Actinides (kg)',
							data: r.steps.map((s) => s.total_actinides_kg),
							borderColor: '#ff3366',
							backgroundColor: 'rgba(255,51,102,0.1)',
							fill: true,
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						},
						{
							label: 'Fission Products (kg)',
							data: r.steps.map((s) => s.total_fission_products_kg),
							borderColor: '#ffcc00',
							backgroundColor: 'rgba(255,204,0,0.1)',
							fill: true,
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'WASTE MASS BY CATEGORY (kg)',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					},
					scales: {
						x: { ...BASE_CHART_OPTIONS.scales.x, title: { display: true, text: 'Time (Years)', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } } },
						y: { ...BASE_CHART_OPTIONS.scales.y, stacked: true }
					}
				} as never
			});
		}
		// Chart B: total activity log scale
		if (chartB) {
			chartInstB = new Chart(chartB, {
				type: 'line',
				data: {
					labels,
					datasets: [
						{
							label: 'Total Activity (Bq)',
							data: r.steps.map((s) => s.total_activity_bq),
							borderColor: '#fff',
							backgroundColor: 'transparent',
							tension: 0.3,
							pointRadius: 0,
							borderWidth: 1.5
						}
					]
				},
				options: {
					...BASE_CHART_OPTIONS,
					plugins: {
						...BASE_CHART_OPTIONS.plugins,
						title: {
							display: true,
							text: 'TOTAL ACTIVITY (Bq, log scale)',
							color: 'rgba(255,255,255,0.4)',
							font: { size: 10, weight: 'bold', family: 'Inter' },
							padding: { bottom: 16 }
						}
					},
					scales: {
						x: { ...BASE_CHART_OPTIONS.scales.x, title: { display: true, text: 'Time (Years)', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } } },
						y: {
							...BASE_CHART_OPTIONS.scales.y,
							type: 'logarithmic',
							title: { display: true, text: 'Activity (Bq)', color: 'rgba(255,255,255,0.3)', font: { size: 10, family: 'Inter' } },
							ticks: {
								color: 'rgba(255,255,255,0.2)',
								font: { size: 10, family: 'JetBrains Mono' },
								callback: (v: number | string) => {
									const n = Number(v);
									if (n <= 0) return '';
									const exp = Math.round(Math.log10(n));
									return Math.abs(n - Math.pow(10, exp)) / Math.pow(10, exp) < 0.01 ? `1e${exp}` : '';
								}
							}
						}
					}
				} as never
			});
		}
	}

	onMount(() => {
		return () => destroyCharts();
	});

	// ── Derived display values ──

	// Thermal channel viz
	const channelOutlet = $derived(thermalResult?.outlet_temp_c ?? th_inlet + 150);
	const channelDelta = $derived(thermalResult?.delta_t_c ?? 150);

	// Color for channel gradient: blue at inlet, red-orange at outlet based on delta
	function tempColor(frac: number): string {
		// 0=inlet (blue) → 1=outlet (orange-red)
		const r = Math.round(30 + frac * 225);
		const g = Math.round(130 - frac * 100);
		const b = Math.round(255 - frac * 220);
		return `rgb(${r},${g},${b})`;
	}

	// Power T-S diagram values
	const tsEfficiency = $derived(powerResult?.efficiency ?? 0.38);
	const tsCarnot = $derived(powerResult?.carnot_efficiency ?? 0.6);
	const tsElectric = $derived(powerResult?.electric_power_mw ?? 0);

	// Fuel burnup arc
	const fuelLastStep = $derived(fuelResult ? fuelResult.steps[fuelResult.steps.length - 1] : null);
	const burnupFrac = $derived(fuelLastStep ? fuelLastStep.burnup_gwd_t / fu_burnup : 0);
	const fissileRem = $derived(fuelLastStep ? fuelLastStep.fissile_fraction : 1);

	function arcPath(frac: number, cx: number, cy: number, r: number): string {
		const clamped = Math.min(frac, 0.999);
		const angle = clamped * 2 * Math.PI - Math.PI / 2;
		const x = cx + r * Math.cos(angle);
		const y = cy + r * Math.sin(angle);
		const largeArc = clamped > 0.5 ? 1 : 0;
		return `M ${cx} ${cy - r} A ${r} ${r} 0 ${largeArc} 1 ${x} ${y}`;
	}

	// Waste summary
	const wasteLastStep = $derived(wasteResult ? wasteResult.steps[wasteResult.steps.length - 1] : null);
</script>

<div class="py-8 pb-16">
	<header class="mb-10">
		<a href="/" class="text-xs text-white/30 no-underline tracking-[0.05em] inline-block mb-6 transition-colors duration-200 hover:text-white">← Home</a>
		<div class="flex items-start gap-6">
			<span class="font-mono text-xs text-white/15 pt-1">{meta.number}</span>
			<div>
				<h1 class="text-[clamp(1.6rem,3.5vw,2.5rem)] font-extrabold m-0 mb-1.5 tracking-tight">{meta.title}</h1>
				<p class="text-sm text-white/40 m-0 max-w-[520px] leading-relaxed font-light">{meta.desc}</p>
			</div>
		</div>
	</header>

	<div class="grid grid-cols-[280px_1fr] grid-divider items-start max-xl:grid-cols-1">
		<!-- ── Left: config panel ── -->
		<aside class="bg-black p-8 sticky top-4 max-xl:static">
			<section class="flex flex-col gap-2">
				<label class="text-[0.65rem] uppercase tracking-caps text-white/30 font-semibold">Model</label>
				{#if loadingReactors}
					<div class="text-xs text-white/30 py-2">Loading...</div>
				{:else}
					<select bind:value={selectedReactorId} class="module-select">
						<option value="">— Custom —</option>
						{#each reactors as r}
							<option value={r.id}>{r.name} ({r.design_type})</option>
						{/each}
					</select>
				{/if}
			</section>

			<div class="h-px bg-white/6 my-5"></div>

			<!-- Thermal form -->
			{#if module === 'thermal'}
				<section class="flex flex-col gap-2">
					<label class="module-field-label mt-0">Thermal Power (MW<sub>th</sub>)</label>
					<input type="number" class="module-input" bind:value={th_power} min="1" max="5000" />

					<label class="module-field-label">Coolant Type</label>
					<select bind:value={th_coolant} class="module-select">
						<option value="Sodium">Sodium — 370–900°C range</option>
						<option value="Lead">Lead — 400–1000°C range</option>
						<option value="Helium">Helium — 260–950°C range</option>
						<option value="FLiBe">FLiBe — 460–900°C range</option>
						<option value="LightWater">Light Water — 280–345°C range</option>
					</select>

					<label class="module-field-label">Inlet Temp (°C)</label>
					<input type="number" class="module-input" bind:value={th_inlet} min="50" max="700" />

					<label class="module-field-label">Flow Rate (kg/s)</label>
					<input type="number" class="module-input" bind:value={th_flow} min="1" max="100000" />
				</section>
			{/if}

			<!-- Power form -->
			{#if module === 'power'}
				<section class="flex flex-col gap-2">
					<label class="module-field-label mt-0">Thermal Power (MW<sub>th</sub>)</label>
					<input type="number" class="module-input" bind:value={pw_thermal} min="1" max="5000" />

					<label class="module-field-label">Coolant Outlet Temp (°C)</label>
					<input type="number" class="module-input" bind:value={pw_outlet} min="100" max="1000" />

					<label class="module-field-label">Thermodynamic Cycle</label>
					<select bind:value={pw_cycle} class="module-select">
						<option value="Rankine">Rankine (steam) — LWR, SFR, LFR</option>
						<option value="Brayton">Brayton (gas) — HTGR</option>
						<option value="SCO2Brayton">sCO₂ Brayton — MSR, FHR</option>
					</select>

					<label class="module-field-label">Rated Electric Power (MW<sub>e</sub>)</label>
					<input type="number" class="module-input" bind:value={pw_rated} min="1" max="2000" />
				</section>
			{/if}

			<!-- Fuel form -->
			{#if module === 'fuel'}
				<section class="flex flex-col gap-2">
					<label class="module-field-label mt-0">Thermal Power (MW<sub>th</sub>)</label>
					<input type="number" class="module-input" bind:value={fu_power} min="1" max="5000" />

					<label class="module-field-label">Heavy Metal Loading (tonnes)</label>
					<input type="number" class="module-input" bind:value={fu_hm} min="0.1" max="200" step="0.1" />

					<label class="module-field-label">Enrichment (%)</label>
					<input type="number" class="module-input" bind:value={fu_enrich} min="0.1" max="100" step="0.1" />

					<label class="module-field-label">Target Burnup (GWd/t)</label>
					<input type="number" class="module-input" bind:value={fu_burnup} min="1" max="300" />

					<label class="module-field-label">Breeding Ratio</label>
					<input type="number" class="module-input" bind:value={fu_br} min="0" max="2" step="0.05" />

					<label class="module-field-label">Duration (years)</label>
					<input type="number" class="module-input" bind:value={fu_duration} min="0.1" max="60" step="0.1" />

					<label class="module-field-label">Time Step (days)</label>
					<input type="number" class="module-input" bind:value={fu_timestep} min="1" max="180" />
				</section>
			{/if}

			<!-- Waste form -->
			{#if module === 'waste'}
				<section class="flex flex-col gap-2">
					<label class="module-field-label mt-0">Thermal Power (MW<sub>th</sub>)</label>
					<input type="number" class="module-input" bind:value={wa_power} min="1" max="5000" />

					<label class="module-field-label">Breeding Ratio</label>
					<input type="number" class="module-input" bind:value={wa_br} min="0" max="2" step="0.05" />

					<label class="module-field-label">Duration (years)</label>
					<input type="number" class="module-input" bind:value={wa_duration} min="0.1" max="60" step="0.1" />

					<label class="module-field-label">Time Step (days)</label>
					<input type="number" class="module-input" bind:value={wa_timestep} min="1" max="180" />
				</section>
			{/if}

			<button class="btn-primary w-full mt-6 py-3 px-4 text-[0.85rem] font-bold tracking-[0.05em]" onclick={runModule} disabled={running}>
				{running ? 'Running...' : 'Run Simulation'}
			</button>

			{#if error}
				<p class="text-xs text-error mt-3 p-2 border border-[rgba(255,51,102,0.3)]">{error}</p>
			{/if}
		</aside>

		<!-- ── Right: visualizations ── -->
		<main class="bg-black p-8 flex flex-col gap-8">

			<!-- ══ THERMAL ══ -->
			{#if module === 'thermal'}
				<div class="p-8 border border-white/6">
					<div class="text-[0.65rem] font-bold tracking-wide uppercase text-white/20 mb-5">CORE CHANNEL — {th_coolant.toUpperCase()}</div>
					<div class="flex items-center gap-10 max-xl:flex-col max-xl:items-stretch">
						<svg viewBox="0 0 200 340" class="w-[140px] shrink-0 max-xl:w-full max-xl:max-w-[200px] max-xl:mx-auto" aria-label="Reactor coolant channel">
							<defs>
								<linearGradient id="channelGrad" x1="0" y1="1" x2="0" y2="0">
									<stop offset="0%" stop-color="{tempColor(0)}" />
									<stop offset="100%" stop-color="{tempColor(1)}" />
								</linearGradient>
							</defs>
							<rect x="60" y="30" width="80" height="260" rx="4" fill="rgba(255,255,255,0.03)" stroke="rgba(255,255,255,0.15)" stroke-width="1"/>
							<rect x="65" y="35" width="70" height="250" rx="2" fill="url(#channelGrad)" opacity="0.7"/>
							<line x1="100" y1="310" x2="100" y2="295" stroke="rgba(255,255,255,0.5)" stroke-width="1.5"/>
							<polygon points="100,285 95,298 105,298" fill="rgba(255,255,255,0.5)"/>
							<text x="155" y="290" fill="rgba(255,255,255,0.6)" font-size="12" font-family="JetBrains Mono">{Math.round(th_inlet)}°C</text>
							<line x1="145" y1="282" x2="135" y2="280" stroke="rgba(255,255,255,0.2)" stroke-width="0.5"/>
							<text x="155" y="50" fill="rgba(255,255,255,0.9)" font-size="12" font-family="JetBrains Mono">{Math.round(channelOutlet)}°C</text>
							<line x1="145" y1="46" x2="135" y2="46" stroke="rgba(255,255,255,0.2)" stroke-width="0.5"/>
							<text x="5" y="163" fill="rgba(255,255,255,0.4)" font-size="10" font-family="Inter">ΔT</text>
							<text x="5" y="178" fill="rgba(255,255,255,0.7)" font-size="12" font-family="JetBrains Mono">+{Math.round(channelDelta)}°C</text>
						</svg>
						<div class="grid grid-cols-2 grid-divider flex-1">
							{@render metricCard('Outlet Temp', `${Math.round(channelOutlet)}°C`)}
							{@render metricCard('Temperature Rise', `+${Math.round(channelDelta)}°C`)}
							{@render metricCard('Thermal Power', `${th_power} MWth`)}
							{@render metricCard('Flow Rate', `${th_flow.toLocaleString()} kg/s`)}
						</div>
					</div>
				</div>

				{#if thermalResult}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartA} class="!w-full !h-full"></canvas></div>
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartB} class="!w-full !h-full"></canvas></div>
					</div>
				{:else}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartA} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see coolant comparison</span></div>
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartB} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see power curve</span></div>
					</div>
				{/if}
			{/if}

			<!-- ══ POWER ══ -->
			{#if module === 'power'}
				<div class="p-8 border border-white/6">
					<div class="text-[0.65rem] font-bold tracking-wide uppercase text-white/20 mb-5">T-S DIAGRAM — {pw_cycle.toUpperCase()}</div>
					<div class="flex items-center gap-8 max-xl:flex-col max-xl:items-stretch">
						<svg viewBox="0 0 320 260" class="w-[280px] shrink-0 max-xl:w-full max-xl:max-w-[200px] max-xl:mx-auto" aria-label="Temperature-entropy diagram">
							<line x1="40" y1="220" x2="290" y2="220" stroke="rgba(255,255,255,0.2)" stroke-width="1"/>
							<line x1="40" y1="20" x2="40" y2="220" stroke="rgba(255,255,255,0.2)" stroke-width="1"/>
							<text x="155" y="240" fill="rgba(255,255,255,0.3)" font-size="9" font-family="Inter" text-anchor="middle">Entropy →</text>
							<text x="12" y="120" fill="rgba(255,255,255,0.3)" font-size="9" font-family="Inter" transform="rotate(-90, 12, 120)">Temperature →</text>
							<polygon points="70,{220 - tsCarnot * 180} 220,{220 - tsCarnot * 180} 220,220" fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.1)" stroke-dasharray="4 4" stroke-width="1"/>
							<text x="225" y="{220 - tsCarnot * 180 + 5}" fill="rgba(255,255,255,0.25)" font-size="9" font-family="JetBrains Mono">Carnot</text>
							{#if pw_cycle === 'Rankine'}
								<path d="M 80 {220 - tsEfficiency * 180} Q 110 {220 - tsEfficiency * 200} 140 {220 - tsEfficiency * 195} Q 170 {220 - tsEfficiency * 185} 200 {220 - tsEfficiency * 180} L 200 210 Q 140 215 80 210 Z" fill="rgba(255,255,255,0.08)" stroke="#fff" stroke-width="1.5"/>
							{:else if pw_cycle === 'Brayton'}
								<path d="M 70 {220 - tsEfficiency * 180} L 210 {220 - tsEfficiency * 175} L 230 210 L 65 215 Z" fill="rgba(255,51,102,0.08)" stroke="#ff3366" stroke-width="1.5"/>
							{:else}
								<path d="M 80 {220 - tsEfficiency * 180} L 210 {220 - tsEfficiency * 178} L 215 210 L 75 213 Z" fill="rgba(0,255,136,0.08)" stroke="#00ff88" stroke-width="1.5"/>
							{/if}
							<text x="44" y="{220 - tsEfficiency * 180 + 4}" fill="rgba(255,255,255,0.6)" font-size="9" font-family="JetBrains Mono">{Math.round(pw_outlet)}°C</text>
							<text x="44" y="217" fill="rgba(255,255,255,0.4)" font-size="9" font-family="JetBrains Mono">35°C</text>
							<text x="140" y="{220 - tsEfficiency * 90}" fill="rgba(255,255,255,0.5)" font-size="10" font-family="JetBrains Mono" text-anchor="middle">η = {(tsEfficiency * 100).toFixed(1)}%</text>
						</svg>
						<div class="grid grid-cols-2 grid-divider flex-1">
							{@render metricCard('Cycle Efficiency', `${(tsEfficiency * 100).toFixed(1)}%`, true)}
							{@render metricCard('Carnot Limit', `${(tsCarnot * 100).toFixed(1)}%`)}
							{@render metricCard('Electric Power', `${Math.round(tsElectric)} MWe`)}
							{@render metricCard('Utilisation of Carnot', `${tsCarnot > 0 ? (tsEfficiency / tsCarnot * 100).toFixed(0) : '—'}%`)}
						</div>
					</div>
				</div>

				{#if powerResult}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartA} class="!w-full !h-full"></canvas></div>
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartB} class="!w-full !h-full"></canvas></div>
					</div>
				{:else}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartA} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see cycle comparison</span></div>
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartB} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see efficiency curve</span></div>
					</div>
				{/if}
			{/if}

			<!-- ══ FUEL ══ -->
			{#if module === 'fuel'}
				<div class="p-8 border border-white/6">
					<div class="text-[0.65rem] font-bold tracking-wide uppercase text-white/20 mb-5">FUEL CYCLE STATUS</div>
					<div class="flex items-center gap-8 max-xl:flex-col max-xl:items-stretch">
						<svg viewBox="0 0 200 200" class="w-[180px] shrink-0 max-xl:w-full max-xl:max-w-[200px] max-xl:mx-auto" aria-label="Burnup progress arc">
							<circle cx="100" cy="100" r="80" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="12"/>
							{#if burnupFrac > 0}
								<path d="{arcPath(burnupFrac, 100, 100, 80)}" fill="none" stroke="#fff" stroke-width="12" stroke-linecap="round"/>
							{/if}
							<text x="100" y="92" fill="white" font-size="22" font-family="JetBrains Mono" font-weight="bold" text-anchor="middle">{fuelLastStep ? fuelLastStep.burnup_gwd_t.toFixed(1) : '—'}</text>
							<text x="100" y="112" fill="rgba(255,255,255,0.4)" font-size="10" font-family="Inter" text-anchor="middle">GWd/t</text>
							<text x="100" y="130" fill="rgba(255,255,255,0.25)" font-size="9" font-family="JetBrains Mono" text-anchor="middle">of {fu_burnup} target</text>
						</svg>
						<div class="flex flex-col grid-divider flex-1">
							<div class="bg-black p-5 flex flex-col gap-1">
								<span class="text-[0.62rem] uppercase tracking-caps text-white/30 font-semibold">Fissile Remaining</span>
								<div class="h-[3px] bg-white/10 my-1">
									<div class="h-full bg-white transition-[width] duration-500 ease-out" style="width: {(fissileRem * 100).toFixed(0)}%"></div>
								</div>
								<span class="font-mono text-[1.05rem] font-bold text-white">{(fissileRem * 100).toFixed(1)}%</span>
							</div>
							{#if fuelResult?.shutdown_year}
								<div class="bg-black p-5 flex flex-col gap-1 border-l-2 border-[#ff3366]">
									<span class="text-[0.62rem] uppercase tracking-caps text-white/30 font-semibold">Shutdown</span>
									<span class="font-mono text-[1.05rem] font-bold text-white">Year {fuelResult.shutdown_year.toFixed(1)}</span>
								</div>
							{/if}
							{@render metricCard('Duration', `${fu_duration} yr`)}
							{@render metricCard('Breeding Ratio', `${fu_br}`)}
						</div>
					</div>
				</div>

				{#if fuelResult}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartA} class="!w-full !h-full"></canvas></div>
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartB} class="!w-full !h-full"></canvas></div>
					</div>
				{:else}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartA} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see burnup curve</span></div>
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartB} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see fissile depletion</span></div>
					</div>
				{/if}
			{/if}

			<!-- ══ WASTE ══ -->
			{#if module === 'waste'}
				<div class="p-8 border border-white/6">
					<div class="text-[0.65rem] font-bold tracking-wide uppercase text-white/20 mb-5">WASTE INVENTORY SUMMARY</div>
					<div class="grid grid-cols-3 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-6">
							<div class="flex items-center gap-2 text-[0.65rem] uppercase tracking-caps text-white/40 font-semibold mb-3">
								<span class="w-1.5 h-1.5 rounded-full bg-[#ffcc00]"></span>
								<span>Fission Products</span>
							</div>
							<span class="block font-mono text-lg font-bold text-white mb-2">{wasteLastStep ? wasteLastStep.total_fission_products_kg.toFixed(2) : '—'} kg</span>
							<p class="text-xs text-white/30 leading-relaxed m-0">Short–medium lived (days to centuries). Cs-137, Sr-90, I-131…</p>
						</div>
						<div class="bg-black p-6">
							<div class="flex items-center gap-2 text-[0.65rem] uppercase tracking-caps text-white/40 font-semibold mb-3">
								<span class="w-1.5 h-1.5 rounded-full bg-[#ff3366]"></span>
								<span>Actinides</span>
							</div>
							<span class="block font-mono text-lg font-bold text-white mb-2">{wasteLastStep ? wasteLastStep.total_actinides_kg.toFixed(2) : '—'} kg</span>
							<p class="text-xs text-white/30 leading-relaxed m-0">Long-lived (thousands–millions of years). Pu-239, Am-241, Np-237…</p>
						</div>
						<div class="bg-black p-6">
							<div class="flex items-center gap-2 text-[0.65rem] uppercase tracking-caps text-white/40 font-semibold mb-3">
								<span class="w-1.5 h-1.5 rounded-full bg-white/30"></span>
								<span>Total Activity</span>
							</div>
							<span class="block font-mono text-lg font-bold text-white mb-2">{wasteLastStep ? wasteLastStep.total_activity_bq.toExponential(2) : '—'} Bq</span>
							<p class="text-xs text-white/30 leading-relaxed m-0">Instantaneous radioactivity across all tracked isotopes</p>
						</div>
					</div>
				</div>

				{#if wasteResult}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartA} class="!w-full !h-full"></canvas></div>
						<div class="bg-black p-5 relative h-[280px]"><canvas bind:this={chartB} class="!w-full !h-full"></canvas></div>
					</div>
				{:else}
					<div class="grid grid-cols-2 grid-divider max-xl:grid-cols-1">
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartA} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see waste accumulation</span></div>
						<div class="bg-black p-5 relative h-[280px] flex items-center justify-center"><canvas bind:this={chartB} class="!w-full !h-full"></canvas><span class="text-xs text-white/15 absolute">Run simulation to see activity evolution</span></div>
					</div>
				{/if}
			{/if}

		</main>
	</div>
</div>

{#snippet metricCard(label: string, value: string, highlight?: boolean)}
	<div class="bg-black p-5 flex flex-col gap-1 {highlight ? 'border-l-2 border-white/40' : ''}">
		<span class="text-[0.62rem] uppercase tracking-caps text-white/30 font-semibold">{label}</span>
		<span class="font-mono text-[1.05rem] font-bold text-white">{value}</span>
	</div>
{/snippet}

<style>
	/* Module-specific form controls — kept as scoped styles since they're unique to this page */
	.module-input,
	.module-select {
		background: rgba(255, 255, 255, 0.04);
		border: 1px solid rgba(255, 255, 255, 0.1);
		color: #fff;
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.85rem;
		padding: 0.5rem 0.75rem;
		width: 100%;
		outline: none;
		transition: border-color 0.2s;
		appearance: none;
	}
	.module-input:focus,
	.module-select:focus {
		border-color: rgba(255, 255, 255, 0.4);
	}
	.module-select option { background: #111; }

	.module-field-label {
		font-size: 0.65rem;
		text-transform: uppercase;
		letter-spacing: 0.12em;
		color: rgba(255, 255, 255, 0.3);
		font-weight: 600;
		margin-top: 0.75rem;
	}
</style>
