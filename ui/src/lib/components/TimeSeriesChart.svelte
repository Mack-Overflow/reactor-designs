<script lang="ts">
	import { Chart, registerables } from 'chart.js';
	import { onMount } from 'svelte';

	Chart.register(...registerables);

	interface Dataset {
		label: string;
		data: number[];
		borderColor?: string;
		backgroundColor?: string;
	}

	let {
		labels,
		datasets,
		title = '',
		yLabel = '',
		logScale = false
	}: {
		labels: string[];
		datasets: Dataset[];
		title?: string;
		yLabel?: string;
		logScale?: boolean;
	} = $props();

	let canvas: HTMLCanvasElement;
	let chart: Chart | undefined;

	const COLORS = ['#fff', '#ff3366', '#00ff88', '#ffcc00', '#66ccff', '#cc66ff'];

	function buildChart() {
		if (chart) chart.destroy();

		chart = new Chart(canvas, {
			type: 'line',
			data: {
				labels,
				datasets: datasets.map((ds, i) => ({
					label: ds.label,
					data: ds.data,
					borderColor: ds.borderColor ?? COLORS[i % COLORS.length],
					backgroundColor: 'transparent',
					tension: 0.4,
					pointRadius: 0,
					borderWidth: 1.5
				}))
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				animation: { duration: 300 },
				plugins: {
					title: title
						? {
								display: true,
								text: title.toUpperCase(),
								color: 'rgba(255,255,255,0.4)',
								font: { size: 10, weight: 'bold', family: 'Inter' },
								padding: { bottom: 16 }
							}
						: { display: false },
					legend: {
						labels: {
							color: 'rgba(255,255,255,0.5)',
							font: { size: 11, family: 'Inter' },
							boxWidth: 12,
							boxHeight: 2,
							padding: 16
						}
					},
					tooltip: {
						backgroundColor: '#fff',
						titleColor: '#000',
						bodyColor: '#000',
						borderColor: 'rgba(0,0,0,0.1)',
						borderWidth: 1,
						cornerRadius: 4,
						titleFont: { family: 'JetBrains Mono', size: 11 },
						bodyFont: { family: 'JetBrains Mono', size: 11 },
						padding: 10
					}
				},
				scales: {
					x: {
						title: {
							display: true,
							text: 'Time (Years)',
							color: 'rgba(255,255,255,0.3)',
							font: { size: 10, family: 'Inter' }
						},
						ticks: {
							color: 'rgba(255,255,255,0.2)',
							maxTicksLimit: 8,
							font: { size: 10, family: 'JetBrains Mono' }
						},
						grid: { color: 'rgba(255,255,255,0.04)' },
						border: { color: 'rgba(255,255,255,0.08)' }
					},
					y: {
						type: logScale ? 'logarithmic' : 'linear',
						title: yLabel
							? {
									display: true,
									text: yLabel,
									color: 'rgba(255,255,255,0.3)',
									font: { size: 10, family: 'Inter' }
								}
							: { display: false },
						ticks: {
							color: 'rgba(255,255,255,0.2)',
							font: { size: 10, family: 'JetBrains Mono' },
							...(logScale ? {
								maxTicksLimit: 8,
								callback: (value: number | string) => {
									const v = Number(value);
									if (v <= 0) return '';
									const exp = Math.round(Math.log10(v));
									if (Math.abs(v - Math.pow(10, exp)) / Math.pow(10, exp) < 0.01) {
										return `1e${exp}`;
									}
									return '';
								}
							} : {})
						},
						grid: { color: 'rgba(255,255,255,0.04)' },
						border: { color: 'rgba(255,255,255,0.08)' }
					}
				}
			}
		});
	}

	onMount(() => {
		buildChart();
		return () => chart?.destroy();
	});

	$effect(() => {
		if (canvas && labels && datasets) {
			buildChart();
		}
	});
</script>

<div class="relative h-[320px] bg-white/2 border border-white/6 rounded-sm p-5">
	<canvas bind:this={canvas}></canvas>
</div>
