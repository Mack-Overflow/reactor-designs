<script>
	let { children } = $props();
</script>

<svelte:head>
	<title>Reactor Designs</title>
</svelte:head>

<div class="app">
	<nav>
		<a href="/">Dashboard</a>
		<a href="/reactors">Reactors</a>
	</nav>
	<main>
		{@render children()}
	</main>
</div>

<style>
	:global(body) {
		margin: 0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		background: #0f172a;
		color: #e2e8f0;
	}

	.app {
		min-height: 100vh;
	}

	nav {
		background: #1e293b;
		padding: 1rem 2rem;
		display: flex;
		gap: 1.5rem;
		border-bottom: 1px solid #334155;
	}

	nav a {
		color: #94a3b8;
		text-decoration: none;
		font-weight: 500;
		transition: color 0.2s;
	}

	nav a:hover {
		color: #38bdf8;
	}

	main {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
	}
</style>
