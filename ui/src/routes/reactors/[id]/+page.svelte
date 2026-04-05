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
		<a href="/simulate?reactor={reactor.id}" class="btn-primary">Simulate Reactor</a>
	</div>

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
		margin-bottom: 2rem;
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

		.btn-primary {
			display: block;
			text-align: center;
			width: 100%;
		}
	}
</style>
