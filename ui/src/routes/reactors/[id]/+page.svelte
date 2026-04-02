<script lang="ts">
	import { page } from '$app/state';
	import { apiFetch } from '$lib/api';
	import type { ReactorDesign } from '$lib/types';

	let reactor = $state<ReactorDesign | null>(null);
	let loading = $state(true);
	let error = $state('');

	$effect(() => {
		const id = page.params.id;
		apiFetch<ReactorDesign>(`/api/reactors/${id}`)
			.then((data) => (reactor = data))
			.catch((e) => (error = e.message))
			.finally(() => (loading = false));
	});
</script>

{#if loading}
	<p class="status">Loading reactor details...</p>
{:else if error}
	<p class="status error">{error}</p>
{:else if reactor}
	<div class="header">
		<a href="/reactors" class="back">&larr; All Reactors</a>
		<span class="badge">{reactor.design_type}</span>
		<h1>{reactor.name}</h1>
		{#if reactor.vendor}
			<p class="vendor">{reactor.vendor}</p>
		{/if}
	</div>

	<div class="sections">
		<section class="spec-section">
			<h2>Power</h2>
			<dl>
				<dt>Thermal Output</dt>
				<dd>{reactor.thermal_power_mw ?? 'N/A'} MW</dd>
				<dt>Electric Output</dt>
				<dd>{reactor.electric_power_mw ?? 'N/A'} MW</dd>
				{#if reactor.thermal_power_mw && reactor.electric_power_mw}
					<dt>Thermal Efficiency</dt>
					<dd>
						{((Number(reactor.electric_power_mw) / Number(reactor.thermal_power_mw)) * 100).toFixed(
							1
						)}%
					</dd>
				{/if}
			</dl>
		</section>

		<section class="spec-section">
			<h2>Core Design</h2>
			<dl>
				<dt>Coolant</dt>
				<dd>{reactor.coolant_type ?? 'N/A'}</dd>
				<dt>Moderator</dt>
				<dd>{reactor.moderator ?? 'N/A'}</dd>
				<dt>Fuel Type</dt>
				<dd>{reactor.fuel_type ?? 'N/A'}</dd>
				<dt>Enrichment</dt>
				<dd>{reactor.enrichment_pct ?? 'N/A'}%</dd>
			</dl>
		</section>

		{#if reactor.design_metadata && Object.keys(reactor.design_metadata).length > 0}
			<section class="spec-section">
				<h2>Additional Parameters</h2>
				<dl>
					{#each Object.entries(reactor.design_metadata) as [key, value]}
						<dt>{key.replace(/_/g, ' ')}</dt>
						<dd>
							{#if value !== null && typeof value === 'object'}
								<dl class="nested">
									{#each Object.entries(value as Record<string, unknown>) as [k, v]}
										<dt>{k.replace(/_/g, ' ')}</dt>
										<dd>{v}</dd>
									{/each}
								</dl>
							{:else if typeof value === 'boolean'}
								{value ? 'Yes' : 'No'}
							{:else}
								{value ?? 'N/A'}
							{/if}
						</dd>
					{/each}
				</dl>
			</section>
		{/if}
	</div>

	{#if reactor.source_url}
		<p class="source">
			Source: <a href={reactor.source_url} target="_blank" rel="noopener">{reactor.source_url}</a>
		</p>
	{/if}
{/if}

<style>
	.status {
		color: #94a3b8;
	}

	.error {
		color: #f87171;
	}

	.back {
		color: #38bdf8;
		text-decoration: none;
		font-size: 0.85rem;
	}

	.back:hover {
		text-decoration: underline;
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
		margin-top: 1rem;
	}

	h1 {
		font-size: 2rem;
		margin: 0.5rem 0 0.15rem;
	}

	.vendor {
		color: #94a3b8;
		margin: 0;
	}

	.sections {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 1.25rem;
		margin-top: 2rem;
	}

	.spec-section {
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 8px;
		padding: 1.25rem;
	}

	h2 {
		font-size: 1rem;
		color: #38bdf8;
		margin: 0 0 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		font-weight: 600;
	}

	dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 0.4rem 1rem;
		margin: 0;
	}

	dt {
		color: #94a3b8;
		font-size: 0.85rem;
		text-transform: capitalize;
	}

	dd {
		margin: 0;
		font-size: 0.9rem;
	}

	dl.nested {
		margin-top: 0.25rem;
		padding-left: 0.5rem;
		border-left: 2px solid #334155;
	}

	.source {
		margin-top: 2rem;
		font-size: 0.85rem;
		color: #64748b;
	}

	.source a {
		color: #38bdf8;
		text-decoration: none;
		word-break: break-all;
	}

	.source a:hover {
		text-decoration: underline;
	}
</style>
