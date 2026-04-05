<script>
	let { children } = $props();
</script>

<svelte:head>
	<title>REACTOR-WIKI</title>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;700&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div class="app">
	<nav>
		<a href="/" class="logo">
			<!-- <span class="logo-icon">&#9762;</span> -->
			<span class="logo-text">REACTOR-WIKI</span>
		</a>
		<div class="nav-links">
			<a href="/reactors">Designs</a>
			<a href="/simulate">Simulate</a>
			<a href="/compare">Compare</a>
			<a href="/ingest">Ingest</a>
		</div>
	</nav>
	<main>
		{@render children()}
	</main>
	<footer>
		<span>Nuclear Reactor Simulation Platform</span>
	</footer>
</div>

<style>
	:global(*) {
		box-sizing: border-box;
	}

	:global(body) {
		margin: 0;
		font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
		background: #000;
		color: #fff;
		-webkit-font-smoothing: antialiased;
	}

	:global(::selection) {
		background: #fff;
		color: #000;
	}

	.app {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.25rem 3rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.08);
		backdrop-filter: blur(12px);
		position: sticky;
		top: 0;
		z-index: 100;
		background: rgba(0, 0, 0, 0.85);
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		text-decoration: none;
		color: #fff;
		flex-shrink: 0;
	}

	.logo-icon {
		font-size: 1.4rem;
	}

	.logo-text {
		font-size: 0.85rem;
		font-weight: 800;
		letter-spacing: 0.2em;
	}

	.nav-links {
		display: flex;
		gap: 2.5rem;
	}

	.nav-links a {
		color: rgba(255, 255, 255, 0.45);
		text-decoration: none;
		font-size: 0.8rem;
		font-weight: 500;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		transition: color 0.3s;
		white-space: nowrap;
	}

	.nav-links a:hover {
		color: #fff;
	}

	main {
		flex: 1;
		max-width: 1400px;
		width: 100%;
		margin: 0 auto;
		padding: 3rem;
	}

	footer {
		border-top: 1px solid rgba(255, 255, 255, 0.06);
		padding: 2rem 3rem;
		text-align: center;
	}

	footer span {
		font-size: 0.7rem;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.2);
	}

	@media (max-width: 768px) {
		nav {
			padding: 1rem 1.25rem;
			gap: 1rem;
		}

		.nav-links {
			gap: 1.25rem;
			overflow-x: auto;
			-webkit-overflow-scrolling: touch;
		}

		.nav-links a {
			font-size: 0.7rem;
		}

		main {
			padding: 1.5rem 1rem;
		}

		footer {
			padding: 1.5rem 1rem;
		}
	}

	@media (max-width: 400px) {
		.logo-text {
			font-size: 0.7rem;
			letter-spacing: 0.12em;
		}

		.nav-links {
			gap: 0.85rem;
		}

		.nav-links a {
			font-size: 0.6rem;
			letter-spacing: 0.04em;
		}
	}
</style>
