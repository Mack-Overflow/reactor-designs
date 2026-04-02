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
</script>

<h1>Nuclear Reactor Simulation Platform</h1>

<div class="stats">
	<div class="stat-card">
		<span class="stat-value">{loading ? '...' : reactors.length}</span>
		<span class="stat-label">Reactor Designs</span>
	</div>
</div>

<p class="subtitle">
	Compare advanced nuclear reactor designs and run physics simulations on fuel usage, fluid
	processes, power generation, and waste generation.
</p>

<div class="actions">
	<a href="/reactors" class="btn">Browse Reactors</a>
</div>

<style>
	h1 {
		font-size: 2rem;
		margin-bottom: 0.5rem;
	}

	.subtitle {
		color: #94a3b8;
		max-width: 600px;
		line-height: 1.6;
	}

	.stats {
		display: flex;
		gap: 1rem;
		margin: 1.5rem 0;
	}

	.stat-card {
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 8px;
		padding: 1.25rem 1.5rem;
		display: flex;
		flex-direction: column;
		min-width: 140px;
	}

	.stat-value {
		font-size: 2rem;
		font-weight: 700;
		color: #38bdf8;
	}

	.stat-label {
		font-size: 0.85rem;
		color: #94a3b8;
		margin-top: 0.25rem;
	}

	.actions {
		margin-top: 2rem;
	}

	.btn {
		display: inline-block;
		background: #38bdf8;
		color: #0f172a;
		padding: 0.6rem 1.25rem;
		border-radius: 6px;
		text-decoration: none;
		font-weight: 600;
		transition: background 0.2s;
	}

	.btn:hover {
		background: #7dd3fc;
	}
</style>
