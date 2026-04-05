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

<div class="page-header">
	<h1>Reactor Designs</h1>
	<p class="count">{loading ? '...' : filtered.length} designs</p>
</div>

<div class="filters">
	<input type="text" placeholder="Search..." bind:value={search} />
	<select bind:value={typeFilter}>
		<option value="">All types</option>
		{#each designTypes as dt}
			<option value={dt}>{dt}</option>
		{/each}
	</select>
</div>

{#if loading}
	<p class="status">Loading...</p>
{:else if error}
	<p class="status error">{error}</p>
{:else if filtered.length === 0}
	<p class="status">No reactors found.</p>
{:else}
	<div class="table-wrap">
		<div class="table-header">
			<span class="col-type">Type</span>
			<span class="col-name">Name</span>
			<span class="col-vendor">Vendor</span>
			<span class="col-thermal">Thermal</span>
			<span class="col-electric">Electric</span>
			<span class="col-coolant">Coolant</span>
			<span class="col-fuel">Fuel</span>
		</div>
		{#each filtered as reactor}
			<a href="/reactors/{reactor.id}" class="table-row">
				<span class="col-type">
					<span class="type-badge">{reactor.design_type}</span>
				</span>
				<span class="col-name">{reactor.name}</span>
				<span class="col-vendor">{reactor.vendor ?? '--'}</span>
				<span class="col-thermal mono">{reactor.thermal_power_mw ?? '--'} <small>MW</small></span>
				<span class="col-electric mono"
					>{reactor.electric_power_mw ?? '--'} <small>MW</small></span
				>
				<span class="col-coolant">{reactor.coolant_type ?? '--'}</span>
				<span class="col-fuel">{reactor.fuel_type ?? '--'}</span>
			</a>
		{/each}
	</div>
{/if}

<style>
	.page-header {
		display: flex;
		align-items: baseline;
		gap: 1rem;
		margin-bottom: 2rem;
	}

	h1 {
		font-size: 2rem;
		font-weight: 800;
		letter-spacing: -0.03em;
		margin: 0;
	}

	.count {
		font-size: 0.8rem;
		color: rgba(255, 255, 255, 0.3);
		margin: 0;
		font-family: 'JetBrains Mono', monospace;
	}

	.filters {
		display: flex;
		gap: 1px;
		background: rgba(255, 255, 255, 0.1);
		margin-bottom: 2rem;
	}

	input,
	select {
		background: #000;
		border: none;
		color: #fff;
		padding: 0.8rem 1rem;
		font-size: 0.85rem;
		font-family: 'Inter', sans-serif;
		outline: none;
	}

	input {
		flex: 1;
	}

	input::placeholder {
		color: rgba(255, 255, 255, 0.25);
	}

	select {
		min-width: 140px;
		cursor: pointer;
	}

	.status {
		color: rgba(255, 255, 255, 0.4);
	}

	.error {
		color: #ff3366;
	}

	.table-wrap {
		border: 1px solid rgba(255, 255, 255, 0.06);
	}

	.table-header {
		display: grid;
		grid-template-columns: 80px 1.5fr 1fr 0.8fr 0.8fr 1fr 1fr;
		gap: 0;
		padding: 0.75rem 1.25rem;
		font-size: 0.65rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.25);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
	}

	.table-row {
		display: grid;
		grid-template-columns: 80px 1.5fr 1fr 0.8fr 0.8fr 1fr 1fr;
		gap: 0;
		padding: 1rem 1.25rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.04);
		text-decoration: none;
		color: #fff;
		font-size: 0.85rem;
		transition: background 0.2s;
		align-items: center;
	}

	.table-row:hover {
		background: rgba(255, 255, 255, 0.03);
	}

	.table-row:last-child {
		border-bottom: none;
	}

	.type-badge {
		display: inline-block;
		border: 1px solid rgba(255, 255, 255, 0.15);
		padding: 0.15rem 0.45rem;
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}

	.col-vendor,
	.col-coolant,
	.col-fuel {
		color: rgba(255, 255, 255, 0.5);
	}

	.mono,
	:global(.mono) {
		font-family: 'JetBrains Mono', monospace;
	}

	small {
		color: rgba(255, 255, 255, 0.25);
		font-size: 0.7rem;
	}

	@media (max-width: 900px) {
		.table-header {
			display: none;
		}

		.table-row {
			display: flex;
			flex-wrap: wrap;
			gap: 0.5rem 1rem;
			padding: 1rem;
			align-items: baseline;
		}

		.col-type {
			order: 0;
		}

		.col-name {
			order: 1;
			flex: 1;
			min-width: 0;
			font-weight: 600;
		}

		.col-vendor {
			order: 2;
			width: 100%;
			font-size: 0.75rem;
		}

		.col-thermal,
		.col-electric {
			font-size: 0.8rem;
		}

		.col-coolant,
		.col-fuel {
			font-size: 0.75rem;
		}
	}

	@media (max-width: 600px) {
		.page-header {
			flex-direction: column;
			gap: 0.25rem;
			margin-bottom: 1.25rem;
		}

		h1 {
			font-size: 1.5rem;
		}

		.filters {
			flex-direction: column;
			gap: 0;
			margin-bottom: 1.25rem;
		}

		select {
			min-width: auto;
		}

		.table-row {
			padding: 0.85rem;
		}

		.col-coolant,
		.col-fuel {
			display: none;
		}
	}
</style>
