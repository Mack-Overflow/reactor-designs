<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign } from '$lib/types';

	let reactors = $state<ReactorDesign[]>([]);
	let loading = $state(true);

	$effect(() => {
		apiFetch<ReactorDesign[]>('/api/reactors')
			.then((data) => (reactors = data))
			.catch((e) => console.error('Failed to load reactors:', e))
			.finally(() => (loading = false));
	});

	let designTypes = $derived([...new Set(reactors.map((r) => r.design_type))]);
</script>

<section class="flex gap-12 my-8 mb-16 max-lg:flex-col max-lg:gap-6 max-lg:my-6 max-lg:mb-12">
	<!-- Left: hero -->
	<div class="flex-1 pt-16 pb-12 max-lg:pt-8 max-lg:pb-6">
		<p class="text-[0.7rem] font-semibold tracking-wider uppercase text-white/30 mb-6">
			Reactor Simulation Platform
		</p>
		<h1 class="text-[clamp(2.5rem,6vw,4.5rem)] font-extrabold leading-[1.05] mb-6 tracking-tight">
			Model the future<br />
			of <span class="text-gradient">nuclear energy</span>
		</h1>
		<p class="text-lg leading-[1.7] text-white/45 max-w-[560px] mb-10 font-light max-lg:text-[0.95rem]">
			Compare advanced reactor designs. Run physics simulations on fuel burnup, thermal hydraulics,
			power generation, and radioactive waste evolution.
		</p>
		<div class="flex gap-4 max-lg:flex-col max-lg:gap-3">
			<a href="/reactors" class="btn-primary max-lg:text-center">Explore Designs</a>
			<a href="/simulate" class="btn-outline max-lg:text-center">Run Simulation</a>
		</div>
	</div>

	<!-- Right: 2x2 stats grid -->
	<div class="grid grid-cols-2 grid-divider self-center shrink-0">
		<div class="bg-black py-8 px-6 flex flex-col gap-1 max-lg:py-5 max-lg:px-4">
			<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">{loading ? '--' : reactors.length}</span>
			<span class="text-[0.7rem] uppercase tracking-wide text-white/35 font-medium">Reactor Designs</span>
		</div>
		<div class="bg-black py-8 px-6 flex flex-col gap-1 max-lg:py-5 max-lg:px-4">
			<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">{loading ? '--' : designTypes.length}</span>
			<span class="text-[0.7rem] uppercase tracking-wide text-white/35 font-medium">Reactor Types</span>
		</div>
		<div class="bg-black py-8 px-6 flex flex-col gap-1 max-lg:py-5 max-lg:px-4">
			<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">4</span>
			<span class="text-[0.7rem] uppercase tracking-wide text-white/35 font-medium">Physics Modules</span>
		</div>
		<div class="bg-black py-8 px-6 flex flex-col gap-1 max-lg:py-5 max-lg:px-4">
			<span class="text-[2.5rem] font-extrabold font-mono tracking-snug max-lg:text-[1.8rem]">8</span>
			<span class="text-[0.7rem] uppercase tracking-wide text-white/35 font-medium">Tracked Isotopes</span>
		</div>
	</div>
</section>

<section class="grid grid-cols-2 grid-divider mb-16 max-lg:grid-cols-1 max-lg:mb-12">
	<a href="/modules/fuel" class="group bg-black p-10 no-underline text-inherit flex flex-col transition-colors duration-250 hover:bg-white/3 max-lg:p-6">
		<span class="font-mono text-xs text-white/20 block mb-4">01</span>
		<h3 class="text-lg font-bold mb-3 tracking-slight">Fuel Cycle</h3>
		<p class="text-[0.9rem] leading-relaxed text-white/40 m-0 font-light flex-1">
			Track burnup accumulation, fissile depletion, and breeding gain across the full fuel cycle
			with configurable batch strategies.
		</p>
		<span class="inline-block mt-5 text-xs font-semibold tracking-mid text-white/20 transition-colors duration-200 group-hover:text-white/70">Simulate →</span>
	</a>
	<a href="/modules/thermal" class="group bg-black p-10 no-underline text-inherit flex flex-col transition-colors duration-250 hover:bg-white/3 max-lg:p-6">
		<span class="font-mono text-xs text-white/20 block mb-4">02</span>
		<h3 class="text-lg font-bold mb-3 tracking-slight">Thermal Hydraulics</h3>
		<p class="text-[0.9rem] leading-relaxed text-white/40 m-0 font-light flex-1">
			Single-channel coolant models for sodium, lead, helium, FLiBe, and light water with
			temperature rise calculations.
		</p>
		<span class="inline-block mt-5 text-xs font-semibold tracking-mid text-white/20 transition-colors duration-200 group-hover:text-white/70">Simulate →</span>
	</a>
	<a href="/modules/power" class="group bg-black p-10 no-underline text-inherit flex flex-col transition-colors duration-250 hover:bg-white/3 max-lg:p-6">
		<span class="font-mono text-xs text-white/20 block mb-4">03</span>
		<h3 class="text-lg font-bold mb-3 tracking-slight">Power Generation</h3>
		<p class="text-[0.9rem] leading-relaxed text-white/40 m-0 font-light flex-1">
			Rankine and Brayton cycle efficiency curves convert thermal output to electric power with
			capacity factor tracking.
		</p>
		<span class="inline-block mt-5 text-xs font-semibold tracking-mid text-white/20 transition-colors duration-200 group-hover:text-white/70">Simulate →</span>
	</a>
	<a href="/modules/waste" class="group bg-black p-10 no-underline text-inherit flex flex-col transition-colors duration-250 hover:bg-white/3 max-lg:p-6">
		<span class="font-mono text-xs text-white/20 block mb-4">04</span>
		<h3 class="text-lg font-bold mb-3 tracking-slight">Waste & Decay</h3>
		<p class="text-[0.9rem] leading-relaxed text-white/40 m-0 font-light flex-1">
			Bateman equation decay chains track Cs-137, Pu-239, Am-241 and more with activity and mass
			evolution over time.
		</p>
		<span class="inline-block mt-5 text-xs font-semibold tracking-mid text-white/20 transition-colors duration-200 group-hover:text-white/70">Simulate →</span>
	</a>
</section>

{#if !loading && reactors.length > 0}
	<section class="mb-8">
		<div class="flex justify-between items-baseline mb-6">
			<h2 class="section-heading text-white/40">Loaded Designs</h2>
			<a href="/reactors" class="text-[0.8rem] text-white/30 no-underline transition-colors duration-200 hover:text-white">View all &rarr;</a>
		</div>
		<div class="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] grid-divider max-lg:grid-cols-1">
			{#each reactors.slice(0, 4) as r}
				<a href="/reactors/{r.id}" class="bg-black p-6 flex flex-col gap-1 no-underline text-white transition-colors duration-300 hover:bg-white/3">
					<span class="text-[0.65rem] font-bold tracking-wide uppercase text-white/25">{r.design_type}</span>
					<span class="text-base font-semibold">{r.name}</span>
					<span class="font-mono text-[0.85rem] text-white/40">{r.electric_power_mw ?? '?'} MWe</span>
				</a>
			{/each}
		</div>
	</section>
{/if}
