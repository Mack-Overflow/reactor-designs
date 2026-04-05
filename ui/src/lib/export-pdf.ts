import { jsPDF } from 'jspdf';
import { Chart } from 'chart.js';

// ── Layout constants ──

const PAGE_W = 210;
const PAGE_H = 297;
const MARGIN = 18;
const COL_W = PAGE_W - MARGIN * 2;
const FONT = 'helvetica';

// ── Color palette ──

const CLR = {
	black: '#000000',
	dark: '#1a1a1a',
	mid: '#666666',
	light: '#999999',
	faint: '#cccccc',
	accent: '#0055ff',
	white: '#ffffff',
	red: '#cc3344',
};

// ── Helpers ──

function addPage(doc: jsPDF): number {
	doc.addPage();
	return MARGIN;
}

function ensureSpace(doc: jsPDF, y: number, need: number): number {
	if (y + need > PAGE_H - MARGIN) return addPage(doc);
	return y;
}

function drawLine(doc: jsPDF, y: number) {
	doc.setDrawColor(CLR.faint);
	doc.setLineWidth(0.2);
	doc.line(MARGIN, y, PAGE_W - MARGIN, y);
}

function heading(doc: jsPDF, y: number, text: string): number {
	y = ensureSpace(doc, y, 12);
	doc.setFont(FONT, 'bold');
	doc.setFontSize(7);
	doc.setTextColor(CLR.light);
	doc.text(text.toUpperCase(), MARGIN, y);
	return y + 6;
}

function kv(doc: jsPDF, y: number, key: string, value: string, xKey = MARGIN, xVal = 80): number {
	y = ensureSpace(doc, y, 5);
	doc.setFont(FONT, 'normal');
	doc.setFontSize(8);
	doc.setTextColor(CLR.mid);
	doc.text(key, xKey, y);
	doc.setTextColor(CLR.dark);
	doc.text(String(value), xVal, y);
	return y + 4.5;
}

function titleBlock(doc: jsPDF, title: string, subtitle: string): number {
	let y = MARGIN;
	doc.setFont(FONT, 'bold');
	doc.setFontSize(6);
	doc.setTextColor(CLR.light);
	doc.text('REACTOR-WIKI', MARGIN, y);
	y += 10;
	doc.setFont(FONT, 'bold');
	doc.setFontSize(16);
	doc.setTextColor(CLR.black);
	doc.text(title, MARGIN, y);
	y += 6;
	doc.setFont(FONT, 'normal');
	doc.setFontSize(8);
	doc.setTextColor(CLR.mid);
	doc.text(subtitle, MARGIN, y);
	y += 4;
	drawLine(doc, y);
	return y + 6;
}

function footer(doc: jsPDF) {
	const pages = doc.getNumberOfPages();
	for (let i = 1; i <= pages; i++) {
		doc.setPage(i);
		doc.setFont(FONT, 'normal');
		doc.setFontSize(6);
		doc.setTextColor(CLR.faint);
		doc.text(`Page ${i} of ${pages}`, PAGE_W - MARGIN, PAGE_H - 10, { align: 'right' });
		doc.text(`Generated ${new Date().toISOString().slice(0, 10)}`, MARGIN, PAGE_H - 10);
	}
}

// ── Chart capture with print-friendly theme swap ──

const PRINT_COLORS = ['#0055ff', '#cc3344', '#118844', '#cc8800', '#6633cc', '#007799'];

function getChartCanvases(): HTMLCanvasElement[] {
	return Array.from(document.querySelectorAll('.charts canvas, .compare-charts canvas'));
}

function applyPrintTheme(chart: Chart) {
	const opts = chart.options;

	// Background
	opts.backgroundColor = '#ffffff';

	// Title
	if (opts.plugins?.title) {
		(opts.plugins.title as Record<string, unknown>).color = '#333333';
	}
	// Legend
	if (opts.plugins?.legend?.labels) {
		(opts.plugins.legend.labels as Record<string, unknown>).color = '#444444';
	}
	// Tooltip — no change needed, not captured in image

	// Scales
	for (const axis of Object.values(opts.scales ?? {})) {
		if (!axis) continue;
		const a = axis as Record<string, unknown>;
		if (a.ticks) (a.ticks as Record<string, unknown>).color = '#666666';
		if (a.grid) (a.grid as Record<string, unknown>).color = '#e0e0e0';
		if (a.border) (a.border as Record<string, unknown>).color = '#cccccc';
		if (a.title) (a.title as Record<string, unknown>).color = '#555555';
	}

	// Dataset line colors
	chart.data.datasets.forEach((ds, i) => {
		(ds as unknown as Record<string, unknown>).borderColor = PRINT_COLORS[i % PRINT_COLORS.length];
		(ds as unknown as Record<string, unknown>).borderWidth = 2;
	});
}

function captureChartsAsImages(): string[] {
	const canvases = getChartCanvases();
	const images: string[] = [];

	for (const canvas of canvases) {
		const chart = Chart.getChart(canvas);
		if (!chart) continue;

		// Snapshot current config
		const origOpts = JSON.stringify(chart.options);
		const origDatasets = chart.data.datasets.map((ds) => ({
			borderColor: (ds as unknown as Record<string, unknown>).borderColor,
			borderWidth: (ds as unknown as Record<string, unknown>).borderWidth,
		}));

		// Apply light theme, render, capture
		applyPrintTheme(chart);
		chart.update('none');

		// Draw white background behind chart content
		const ctx = canvas.getContext('2d');
		if (ctx) {
			const img = ctx.getImageData(0, 0, canvas.width, canvas.height);
			ctx.fillStyle = '#ffffff';
			ctx.fillRect(0, 0, canvas.width, canvas.height);
			ctx.putImageData(img, 0, 0);
		}

		images.push(canvas.toDataURL('image/png', 0.95));

		// Restore original theme
		Object.assign(chart.options, JSON.parse(origOpts));
		chart.data.datasets.forEach((ds, i) => {
			(ds as unknown as Record<string, unknown>).borderColor = origDatasets[i].borderColor;
			(ds as unknown as Record<string, unknown>).borderWidth = origDatasets[i].borderWidth;
		});
		chart.update('none');
	}

	return images;
}

function addChartPages(doc: jsPDF, images: string[], canvases: HTMLCanvasElement[]) {
	for (let i = 0; i < images.length; i += 3) {
		let y = addPage(doc);
		y = heading(doc, y, 'Simulation Charts');
		y += 2;

		for (let j = i; j < images.length && j < i + 3; j++) {
			y = ensureSpace(doc, y, 75);
			const canvas = canvases[j];
			const aspect = canvas.width / canvas.height;
			const w = COL_W;
			const h = w / aspect;
			doc.addImage(images[j], 'PNG', MARGIN, y, w, Math.min(h, 70));
			y += Math.min(h, 70) + 6;
		}
	}
}

// ── Types for export data ──

export interface SimStepData {
	time_years: number;
	fuel_burnup_gwd_t: number;
	fuel_remaining_pct: number;
	coolant_temp_inlet_c: number;
	coolant_temp_outlet_c: number;
	thermal_power_mw: number;
	electric_power_mw: number;
	capacity_factor: number;
	waste_actinides_kg: number;
	waste_fission_products_kg: number;
	waste_total_activity_bq: number;
}

export interface SimExportData {
	reactorName: string;
	reactorType: string;
	runId: string;
	averageCf: number;
	params: Record<string, unknown>;
	steps: SimStepData[];
}

export interface CompareExportData {
	entries: {
		reactorName: string;
		reactorType: string;
		finalStep: SimStepData | null;
	}[];
}

// ── Export: Simulation ──

export function exportSimulationPdf(data: SimExportData) {
	const doc = new jsPDF({ unit: 'mm', format: 'a4' });
	let y = titleBlock(doc, `Simulation Report`, `${data.reactorName} (${data.reactorType})`);

	// Run info
	y = heading(doc, y, 'Run Information');
	y = kv(doc, y, 'Run ID', data.runId.slice(0, 24) + '...');
	y = kv(doc, y, 'Reactor', data.reactorName);
	y = kv(doc, y, 'Type', data.reactorType);
	y = kv(doc, y, 'Total Steps', String(data.steps.length));
	y = kv(doc, y, 'Avg Capacity Factor', data.averageCf.toFixed(4));
	y += 3;
	drawLine(doc, y);
	y += 6;

	// Parameters
	y = heading(doc, y, 'Simulation Parameters');
	const paramLabels: Record<string, string> = {
		duration_years: 'Duration (years)',
		time_step_days: 'Time Step (days)',
		initial_heavy_metal_tonnes: 'Heavy Metal (t)',
		enrichment_pct: 'Enrichment (%)',
		target_burnup_gwd_t: 'Target Burnup (GWd/t)',
		thermal_power_mw: 'Thermal Power (MW)',
		breeding_ratio: 'Breeding Ratio',
		coolant_type: 'Coolant',
		coolant_inlet_temp_c: 'Inlet Temp (C)',
		coolant_flow_rate_kg_s: 'Flow Rate (kg/s)',
		cycle_type: 'Cycle Type',
		rated_electric_power_mw: 'Rated Electric (MWe)',
	};
	for (const [k, label] of Object.entries(paramLabels)) {
		const val = data.params[k];
		if (val !== undefined) {
			y = kv(doc, y, label, String(val));
		}
	}
	y += 3;
	drawLine(doc, y);
	y += 6;

	// Final state
	const last = data.steps[data.steps.length - 1];
	if (last) {
		y = heading(doc, y, 'Final State');
		y = kv(doc, y, 'Burnup', `${last.fuel_burnup_gwd_t.toFixed(2)} GWd/t`);
		y = kv(doc, y, 'Fuel Remaining', `${(last.fuel_remaining_pct * 100).toFixed(1)}%`);
		y = kv(doc, y, 'Outlet Temp', `${last.coolant_temp_outlet_c.toFixed(0)} C`);
		y = kv(doc, y, 'Electric Power', `${last.electric_power_mw.toFixed(1)} MW`);
		y = kv(doc, y, 'Capacity Factor', last.capacity_factor.toFixed(4));
		y = kv(doc, y, 'Actinides', `${last.waste_actinides_kg.toFixed(3)} kg`);
		y = kv(doc, y, 'Fission Products', `${last.waste_fission_products_kg.toFixed(3)} kg`);
		y = kv(doc, y, 'Total Activity', `${last.waste_total_activity_bq.toExponential(3)} Bq`);
	}

	// Charts
	const canvases = getChartCanvases();
	const images = captureChartsAsImages();
	addChartPages(doc, images, canvases);

	footer(doc);
	doc.save(`simulation_${data.reactorName.replace(/\s+/g, '_')}_${data.runId.slice(0, 8)}.pdf`);
}

// ── Export: Comparison ──

export function exportComparisonPdf(data: CompareExportData) {
	const doc = new jsPDF({ unit: 'mm', format: 'a4' });
	const names = data.entries.map((e) => e.reactorName).join(' vs ');
	let y = titleBlock(doc, 'Comparison Report', names);

	// Summary table
	y = heading(doc, y, 'Reactor Summary');

	for (const entry of data.entries) {
		y = ensureSpace(doc, y, 30);
		doc.setFont(FONT, 'bold');
		doc.setFontSize(9);
		doc.setTextColor(CLR.dark);
		doc.text(entry.reactorName, MARGIN, y);
		doc.setFont(FONT, 'normal');
		doc.setFontSize(7);
		doc.setTextColor(CLR.light);
		doc.text(entry.reactorType, MARGIN + doc.getTextWidth(entry.reactorName) + 4, y);
		y += 5;

		if (entry.finalStep) {
			const s = entry.finalStep;
			y = kv(doc, y, 'Final Burnup', `${s.fuel_burnup_gwd_t.toFixed(2)} GWd/t`);
			y = kv(doc, y, 'Electric Power', `${s.electric_power_mw.toFixed(1)} MW`);
			y = kv(doc, y, 'Capacity Factor', s.capacity_factor.toFixed(4));
			y = kv(doc, y, 'Outlet Temp', `${s.coolant_temp_outlet_c.toFixed(0)} C`);
			y = kv(doc, y, 'Total Activity', `${s.waste_total_activity_bq.toExponential(3)} Bq`);
		}
		y += 4;
		drawLine(doc, y);
		y += 5;
	}

	// Charts
	const canvases = getChartCanvases();
	const images = captureChartsAsImages();
	addChartPages(doc, images, canvases);

	footer(doc);
	const slug = data.entries.map((e) => e.reactorName.replace(/\s+/g, '_')).join('_vs_');
	doc.save(`comparison_${slug}.pdf`);
}
