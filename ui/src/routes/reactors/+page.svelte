<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign } from '$lib/types';

	let reactors = $state<ReactorDesign[]>([]);
	let filtered = $state<ReactorDesign[]>([]);
	let loading = $state(true);
	let error = $state('');
	let search = $state('');
	let typeFilter = $state('');

	$effect(() => {
		apiFetch<ReactorDesign[]>('/api/reactors')
			.then((data) => {
				reactors = data;
				filtered = data;
			})
			.catch((e) => (error = e.message))
			.finally(() => (loading = false));
	});

	$effect(() => {
		const s = search.toLowerCase();
		filtered = reactors.filter((r) => {
			const matchesSearch =
				!s ||
				r.name.toLowerCase().includes(s) ||
				r.vendor?.toLowerCase().includes(s) ||
				r.coolant_type?.toLowerCase().includes(s);
			const matchesType = !typeFilter || r.design_type === typeFilter;
			return matchesSearch && matchesType;
		});
	});

	let designTypes = $derived([...new Set(reactors.map((r) => r.design_type))].sort());
</script>

<div class="flex items-baseline gap-4 mb-8 max-md:flex-col max-md:gap-1 max-md:mb-5">
	<h1 class="text-3xl font-extrabold tracking-tight m-0">Reactor Designs</h1>
	<p class="text-[0.8rem] text-white/30 m-0 font-mono">{loading ? '...' : filtered.length} designs</p>
</div>

<div class="flex grid-divider mb-8 max-md:flex-col max-md:gap-0 max-md:mb-5">
	<input
		type="text"
		placeholder="Search..."
		bind:value={search}
		class="flex-1 bg-black border-none text-white py-3 px-4 text-[0.85rem] font-sans outline-none placeholder:text-white/25"
	/>
	<select bind:value={typeFilter} class="bg-black border-none text-white py-3 px-4 text-[0.85rem] font-sans outline-none min-w-[140px] cursor-pointer max-md:min-w-0">
		<option value="">All types</option>
		{#each designTypes as dt}
			<option value={dt}>{dt}</option>
		{/each}
	</select>
</div>

{#if loading}
	<p class="text-white/40">Loading...</p>
{:else if error}
	<p class="text-error">{error}</p>
{:else if filtered.length === 0}
	<p class="text-white/40">No reactors found.</p>
{:else}
	<div class="border border-white/6">
		<div class="table-header max-xl:hidden">
			<span>Type</span>
			<span>Name</span>
			<span>Vendor</span>
			<span>Thermal</span>
			<span>Electric</span>
			<span>Coolant</span>
			<span>Fuel</span>
		</div>
		{#each filtered as reactor}
			<a href="/reactors/{reactor.id}" class="table-row max-xl:flex max-xl:flex-wrap max-xl:gap-x-4 max-xl:gap-y-2 max-xl:p-4 max-xl:items-baseline max-md:p-3.5">
				<span class="max-xl:order-0">
					<span class="type-badge">{reactor.design_type}</span>
				</span>
				<span class="max-xl:order-1 max-xl:flex-1 max-xl:min-w-0 max-xl:font-semibold">{reactor.name}</span>
				<span class="text-white/50 max-xl:order-2 max-xl:w-full max-xl:text-xs">{reactor.vendor ?? '--'}</span>
				<span class="font-mono max-xl:text-[0.8rem]">{reactor.thermal_power_mw ?? '--'} <small class="text-white/25 text-[0.7rem]">MW</small></span>
				<span class="font-mono max-xl:text-[0.8rem]">{reactor.electric_power_mw ?? '--'} <small class="text-white/25 text-[0.7rem]">MW</small></span>
				<span class="text-white/50 max-xl:text-xs max-md:hidden">{reactor.coolant_type ?? '--'}</span>
				<span class="text-white/50 max-xl:text-xs max-md:hidden">{reactor.fuel_type ?? '--'}</span>
			</a>
		{/each}
	</div>
{/if}
