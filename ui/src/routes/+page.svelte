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

<section class="hero">
	<p class="eyebrow">Nuclear Simulation Platform</p>
	<h1>
		Model the future<br />
		of <span class="highlight">nuclear energy</span>
	</h1>
	<p class="subtitle">
		Compare advanced reactor designs. Run physics simulations on fuel burnup, thermal hydraulics,
		power generation, and radioactive waste evolution.
	</p>
	<div class="hero-actions">
		<a href="/reactors" class="btn-primary">Explore Designs</a>
		<a href="/simulate" class="btn-outline">Run Simulation</a>
	</div>
</section>

<section class="stats-row">
	<div class="stat">
		<span class="stat-number">{loading ? '--' : reactors.length}</span>
		<span class="stat-label">Reactor Designs</span>
	</div>
	<div class="stat">
		<span class="stat-number">{loading ? '--' : designTypes.length}</span>
		<span class="stat-label">Reactor Types</span>
	</div>
	<div class="stat">
		<span class="stat-number">4</span>
		<span class="stat-label">Physics Modules</span>
	</div>
	<div class="stat">
		<span class="stat-number">8</span>
		<span class="stat-label">Tracked Isotopes</span>
	</div>
</section>

<section class="features">
	<div class="feature">
		<span class="feature-number">01</span>
		<h3>Fuel Cycle</h3>
		<p>
			Track burnup accumulation, fissile depletion, and breeding gain across the full fuel cycle
			with configurable batch strategies.
		</p>
	</div>
	<div class="feature">
		<span class="feature-number">02</span>
		<h3>Thermal Hydraulics</h3>
		<p>
			Single-channel coolant models for sodium, lead, helium, FLiBe, and light water with
			temperature rise calculations.
		</p>
	</div>
	<div class="feature">
		<span class="feature-number">03</span>
		<h3>Power Generation</h3>
		<p>
			Rankine and Brayton cycle efficiency curves convert thermal output to electric power with
			capacity factor tracking.
		</p>
	</div>
	<div class="feature">
		<span class="feature-number">04</span>
		<h3>Waste & Decay</h3>
		<p>
			Bateman equation decay chains track Cs-137, Pu-239, Am-241 and more with activity and mass
			evolution over time.
		</p>
	</div>
</section>

{#if !loading && reactors.length > 0}
	<section class="reactor-preview">
		<div class="section-header">
			<h2>Loaded Designs</h2>
			<a href="/reactors" class="see-all">View all &rarr;</a>
		</div>
		<div class="reactor-strip">
			{#each reactors.slice(0, 4) as r}
				<a href="/reactors/{r.id}" class="reactor-pill">
					<span class="pill-type">{r.design_type}</span>
					<span class="pill-name">{r.name}</span>
					<span class="pill-power">{r.electric_power_mw ?? '?'} MWe</span>
				</a>
			{/each}
		</div>
	</section>
{/if}

<style>
	.hero {
		padding: 4rem 0 3rem;
	}

	.eyebrow {
		font-size: 0.7rem;
		font-weight: 600;
		letter-spacing: 0.2em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
		margin: 0 0 1.5rem;
	}

	h1 {
		font-size: clamp(2.5rem, 6vw, 4.5rem);
		font-weight: 800;
		line-height: 1.05;
		margin: 0 0 1.5rem;
		letter-spacing: -0.03em;
	}

	.highlight {
		background: linear-gradient(135deg, #fff 0%, rgba(255, 255, 255, 0.5) 100%);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
	}

	.subtitle {
		font-size: 1.1rem;
		line-height: 1.7;
		color: rgba(255, 255, 255, 0.45);
		max-width: 560px;
		margin: 0 0 2.5rem;
		font-weight: 300;
	}

	.hero-actions {
		display: flex;
		gap: 1rem;
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
		letter-spacing: 0.02em;
	}

	.btn-primary:hover {
		background: rgba(255, 255, 255, 0.85);
		transform: translateY(-1px);
	}

	.btn-outline {
		display: inline-block;
		border: 1px solid rgba(255, 255, 255, 0.2);
		color: #fff;
		padding: 0.75rem 2rem;
		font-size: 0.85rem;
		font-weight: 500;
		text-decoration: none;
		transition: all 0.3s;
		letter-spacing: 0.02em;
	}

	.btn-outline:hover {
		border-color: rgba(255, 255, 255, 0.5);
		background: rgba(255, 255, 255, 0.05);
	}

	.stats-row {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		margin: 2rem 0 4rem;
	}

	.stat {
		background: #000;
		padding: 2rem 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.stat-number {
		font-size: 2.5rem;
		font-weight: 800;
		font-family: 'JetBrains Mono', monospace;
		letter-spacing: -0.02em;
	}

	.stat-label {
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.15em;
		color: rgba(255, 255, 255, 0.35);
		font-weight: 500;
	}

	.features {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		margin-bottom: 4rem;
	}

	.feature {
		background: #000;
		padding: 2.5rem;
	}

	.feature-number {
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.2);
		display: block;
		margin-bottom: 1rem;
	}

	.feature h3 {
		font-size: 1.15rem;
		font-weight: 700;
		margin: 0 0 0.75rem;
		letter-spacing: -0.01em;
	}

	.feature p {
		font-size: 0.9rem;
		line-height: 1.6;
		color: rgba(255, 255, 255, 0.4);
		margin: 0;
		font-weight: 300;
	}

	.reactor-preview {
		margin-bottom: 2rem;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		margin-bottom: 1.5rem;
	}

	h2 {
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.4);
		margin: 0;
	}

	.see-all {
		font-size: 0.8rem;
		color: rgba(255, 255, 255, 0.3);
		text-decoration: none;
		transition: color 0.2s;
	}

	.see-all:hover {
		color: #fff;
	}

	.reactor-strip {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
	}

	.reactor-pill {
		background: #000;
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
		text-decoration: none;
		color: #fff;
		transition: background 0.3s;
	}

	.reactor-pill:hover {
		background: rgba(255, 255, 255, 0.03);
	}

	.pill-type {
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.25);
	}

	.pill-name {
		font-size: 1rem;
		font-weight: 600;
	}

	.pill-power {
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.85rem;
		color: rgba(255, 255, 255, 0.4);
	}

	@media (max-width: 768px) {
		.hero {
			padding: 2rem 0 1.5rem;
		}

		.subtitle {
			font-size: 0.95rem;
		}

		.hero-actions {
			flex-direction: column;
			gap: 0.75rem;
		}

		.hero-actions a {
			text-align: center;
		}

		.stats-row {
			grid-template-columns: repeat(2, 1fr);
			margin: 1.5rem 0 3rem;
		}

		.stat {
			padding: 1.25rem 1rem;
		}

		.stat-number {
			font-size: 1.8rem;
		}

		.features {
			grid-template-columns: 1fr;
			margin-bottom: 3rem;
		}

		.feature {
			padding: 1.5rem;
		}

		.reactor-strip {
			grid-template-columns: 1fr;
		}
	}
</style>
