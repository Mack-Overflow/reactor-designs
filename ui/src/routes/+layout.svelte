<script>
	import '../app.css';
	let { children } = $props();
	let modulesOpen = $state(false);
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

<div class="min-h-screen flex flex-col">
	<nav
		class="flex items-center justify-between px-12 py-5 border-b border-white/8 backdrop-blur-md sticky top-0 z-100 bg-black/85 max-lg:px-5 max-lg:py-4 max-lg:gap-4"
	>
		<a href="/" class="flex items-center gap-2.5 no-underline text-white shrink-0">
			<span class="text-sm font-extrabold tracking-wider">REACTOR-WIKI</span>
		</a>
		<div
			class="flex gap-10 max-lg:gap-5 max-lg:overflow-x-auto max-lg:-webkit-overflow-scrolling-touch max-xs:gap-3.5"
		>
			<a
				href="/reactors"
				class="text-white/45 no-underline text-xs font-medium tracking-nav uppercase transition-colors duration-300 whitespace-nowrap hover:text-white max-lg:text-[0.7rem] max-xs:text-[0.6rem] max-xs:tracking-label"
				>Designs</a
			>
			<a
				href="/simulate"
				class="text-white/45 no-underline text-xs font-medium tracking-nav uppercase transition-colors duration-300 whitespace-nowrap hover:text-white max-lg:text-[0.7rem] max-xs:text-[0.6rem] max-xs:tracking-label"
				>Simulate</a
			>
			<div
				class="relative flex items-center"
				role="navigation"
				onmouseenter={() => (modulesOpen = true)}
				onmouseleave={() => (modulesOpen = false)}
			>
				<button
					type="button"
					class="text-white/45 bg-transparent border-none cursor-pointer text-xs font-medium tracking-nav uppercase transition-colors duration-300 whitespace-nowrap hover:text-white max-lg:text-[0.7rem] max-xs:text-[0.6rem] max-xs:tracking-label font-sans p-0 leading-none"
					onclick={() => (modulesOpen = !modulesOpen)}
				>
					Modules
				</button>
				{#if modulesOpen}
					<div class="absolute top-full left-1/2 -translate-x-1/2 pt-2 z-50">
						<div class="bg-black/95 border border-white/10 backdrop-blur-md flex flex-col min-w-[180px]">
							<a href="/modules/fuel" class="px-5 py-3 text-[0.7rem] font-medium tracking-nav uppercase text-white/45 no-underline transition-colors duration-200 hover:text-white hover:bg-white/4" onclick={() => (modulesOpen = false)}>
								<span class="font-mono text-white/15 mr-2">01</span> Fuel Cycle
							</a>
							<a href="/modules/thermal" class="px-5 py-3 text-[0.7rem] font-medium tracking-nav uppercase text-white/45 no-underline transition-colors duration-200 hover:text-white hover:bg-white/4 border-t border-white/4" onclick={() => (modulesOpen = false)}>
								<span class="font-mono text-white/15 mr-2">02</span> Thermal
							</a>
							<a href="/modules/power" class="px-5 py-3 text-[0.7rem] font-medium tracking-nav uppercase text-white/45 no-underline transition-colors duration-200 hover:text-white hover:bg-white/4 border-t border-white/4" onclick={() => (modulesOpen = false)}>
								<span class="font-mono text-white/15 mr-2">03</span> Power
							</a>
							<a href="/modules/waste" class="px-5 py-3 text-[0.7rem] font-medium tracking-nav uppercase text-white/45 no-underline transition-colors duration-200 hover:text-white hover:bg-white/4 border-t border-white/4" onclick={() => (modulesOpen = false)}>
								<span class="font-mono text-white/15 mr-2">04</span> Waste & Decay
							</a>
						</div>
					</div>
				{/if}
			</div>
			<a
				href="/compare"
				class="text-white/45 no-underline text-xs font-medium tracking-nav uppercase transition-colors duration-300 whitespace-nowrap hover:text-white max-lg:text-[0.7rem] max-xs:text-[0.6rem] max-xs:tracking-label"
				>Compare</a
			>
			<a
				href="/ingest"
				class="text-white/45 no-underline text-xs font-medium tracking-nav uppercase transition-colors duration-300 whitespace-nowrap hover:text-white max-lg:text-[0.7rem] max-xs:text-[0.6rem] max-xs:tracking-label"
				>Ingest</a
			>
		</div>
	</nav>
	<main class="flex-1 max-w-[1400px] w-full mx-auto p-12 max-lg:px-4 max-lg:py-6">
		{@render children()}
	</main>
	<footer class="border-t border-white/6 py-8 px-12 text-center max-lg:py-6 max-lg:px-4">
		<span class="text-[0.7rem] tracking-wide uppercase text-white/20">
			Nuclear Reactor Simulation Platform
		</span>
	</footer>
</div>
