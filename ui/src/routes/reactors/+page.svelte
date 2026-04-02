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

<h1>Reactor Designs</h1>

<div class="filters">
	<input type="text" placeholder="Search by name, vendor, coolant..." bind:value={search} />
	<select bind:value={typeFilter}>
		<option value="">All types</option>
		{#each designTypes as dt}
			<option value={dt}>{dt}</option>
		{/each}
	</select>
</div>

{#if loading}
	<p class="status">Loading reactors...</p>
{:else if error}
	<p class="status error">{error}</p>
{:else if filtered.length === 0}
	<p class="status">No reactors found.</p>
{:else}
	<div class="grid">
		{#each filtered as reactor}
			<a href="/reactors/{reactor.id}" class="card">
				<div class="card-header">
					<span class="badge">{reactor.design_type}</span>
					<h2>{reactor.name}</h2>
					{#if reactor.vendor}
						<span class="vendor">{reactor.vendor}</span>
					{/if}
				</div>
				<div class="specs">
					{#if reactor.thermal_power_mw}
						<div class="spec">
							<span class="spec-label">Thermal</span>
							<span class="spec-value">{reactor.thermal_power_mw} MW</span>
						</div>
					{/if}
					{#if reactor.electric_power_mw}
						<div class="spec">
							<span class="spec-label">Electric</span>
							<span class="spec-value">{reactor.electric_power_mw} MW</span>
						</div>
					{/if}
					{#if reactor.coolant_type}
						<div class="spec">
							<span class="spec-label">Coolant</span>
							<span class="spec-value">{reactor.coolant_type}</span>
						</div>
					{/if}
					{#if reactor.fuel_type}
						<div class="spec">
							<span class="spec-label">Fuel</span>
							<span class="spec-value">{reactor.fuel_type}</span>
						</div>
					{/if}
				</div>
			</a>
		{/each}
	</div>
{/if}

<style>
	h1 {
		font-size: 1.75rem;
		margin-bottom: 1.5rem;
	}

	.filters {
		display: flex;
		gap: 0.75rem;
		margin-bottom: 1.5rem;
	}

	input,
	select {
		background: #1e293b;
		border: 1px solid #334155;
		color: #e2e8f0;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		font-size: 0.9rem;
	}

	input {
		flex: 1;
	}

	input::placeholder {
		color: #64748b;
	}

	.status {
		color: #94a3b8;
	}

	.error {
		color: #f87171;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
	}

	.card {
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 8px;
		padding: 1.25rem;
		text-decoration: none;
		color: inherit;
		transition:
			border-color 0.2s,
			transform 0.2s;
	}

	.card:hover {
		border-color: #38bdf8;
		transform: translateY(-2px);
	}

	.card-header {
		margin-bottom: 1rem;
	}

	.badge {
		display: inline-block;
		background: #38bdf8;
		color: #0f172a;
		font-size: 0.7rem;
		font-weight: 700;
		padding: 0.15rem 0.5rem;
		border-radius: 4px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	h2 {
		font-size: 1.15rem;
		margin: 0.5rem 0 0.15rem;
	}

	.vendor {
		font-size: 0.85rem;
		color: #94a3b8;
	}

	.specs {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.5rem;
	}

	.spec {
		display: flex;
		flex-direction: column;
	}

	.spec-label {
		font-size: 0.7rem;
		color: #64748b;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.spec-value {
		font-size: 0.9rem;
		color: #e2e8f0;
	}
</style>
