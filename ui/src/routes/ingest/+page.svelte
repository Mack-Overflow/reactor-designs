<script lang="ts">
	import type { IngestResponse } from '$lib/types';

	const API_BASE = import.meta.env.VITE_API_URL ?? 'http://localhost:18080';

	const KNOWN_FIELDS = new Set([
		'name', 'design_type', 'vendor', 'thermal_power_mw', 'electric_power_mw',
		'coolant_type', 'moderator', 'fuel_type', 'enrichment_pct',
		'design_metadata', 'source_url'
	]);
	const REQUIRED_FIELDS = new Set(['name', 'design_type']);

	let file = $state<File | null>(null);
	let parsedRecords = $state<Record<string, unknown>[]>([]);
	let parseError = $state('');
	let dragging = $state(false);
	let uploading = $state(false);
	let result = $state<IngestResponse | null>(null);
	let error = $state('');

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragging = false;
		const dropped = e.dataTransfer?.files[0];
		if (dropped) setFile(dropped);
	}

	function handleFileInput(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files?.[0]) setFile(input.files[0]);
	}

	function setFile(f: File) {
		file = f;
		result = null;
		error = '';
		parseError = '';
		parsedRecords = [];

		const ext = f.name.split('.').pop()?.toLowerCase() ?? '';
		if (ext === 'json') {
			f.text().then((text) => {
					try {
					const parsed = JSON.parse(text);
					parsedRecords = Array.isArray(parsed) ? parsed : [parsed];
				} catch (e: any) {
					parseError = e.message ?? 'Invalid JSON';
				}
			});
		} else if (ext === 'csv') {
			f.text().then((text) => {
					parsedRecords = parseCsvPreview(text);
			});
		}
	}

	function parseCsvPreview(text: string): Record<string, unknown>[] {
		const lines = text.trim().split('\n');
		if (lines.length < 2) return [];
		const headers = lines[0].split(',').map((h) => h.trim());
		const records: Record<string, unknown>[] = [];
		for (let i = 1; i < lines.length; i++) {
			const vals = lines[i].split(',').map((v) => v.trim());
			const record: Record<string, unknown> = {};
			headers.forEach((h, idx) => {
				const v = vals[idx] ?? '';
				if (v === '' || v === 'null') record[h] = null;
				else if (!isNaN(Number(v))) record[h] = Number(v);
				else record[h] = v;
			});
			records.push(record);
		}
		return records;
	}

	function classifyField(key: string): 'valid' | 'extra' {
		return KNOWN_FIELDS.has(key) ? 'valid' : 'extra';
	}

	function missingRequired(record: Record<string, unknown>): string[] {
		return [...REQUIRED_FIELDS].filter((f) => {
			const val = record[f];
			return val === undefined || val === null || val === '';
		});
	}

	function clearFile() {
		file = null;
		parsedRecords = [];
		parseError = '';
		result = null;
		error = '';
	}

	async function upload() {
		if (!file) return;
		uploading = true;
		error = '';
		result = null;

		const formData = new FormData();
		formData.append('file', file);

		try {
			const res = await fetch(`${API_BASE}/api/ingest`, {
				method: 'POST',
				body: formData
			});
			const body = await res.json();
			if (!res.ok) {
				error = body.error ?? `Upload failed: ${res.status}`;
			} else {
				result = body as IngestResponse;
			}
		} catch (e: any) {
			error = e.message ?? 'Upload failed';
		} finally {
			uploading = false;
		}
	}

	let fileExt = $derived(file?.name.split('.').pop()?.toLowerCase() ?? '');
	let validFile = $derived(fileExt === 'json' || fileExt === 'csv');
	let hasPreview = $derived(parsedRecords.length > 0 || parseError);
</script>

<div class="mb-10">
	<h1 class="text-3xl font-extrabold tracking-tight m-0 mb-1">Ingest Reactors</h1>
	<p class="text-white/35 text-[0.85rem] m-0">Upload reactor designs via JSON or CSV</p>
</div>

<div class="grid grid-cols-2 gap-12 mb-12 max-xl:grid-cols-1">
	<div>
		<div
			class="border border-dashed border-white/15 py-12 px-8 text-center cursor-pointer transition-all duration-200 hover:border-white/40 hover:bg-white/2 max-md:py-8 max-md:px-4 {dragging ? 'border-white/40 bg-white/2' : ''} {file ? 'border-solid border-white/20' : ''}"
			role="button"
			tabindex="0"
			ondragover={(e) => { e.preventDefault(); dragging = true; }}
			ondragleave={() => (dragging = false)}
			ondrop={handleDrop}
			onclick={() => document.getElementById('file-input')?.click()}
			onkeydown={(e) => { if (e.key === 'Enter') document.getElementById('file-input')?.click(); }}
		>
			{#if file}
				<div class="flex items-center justify-center gap-3">
					<span class="inline-block border border-white/20 px-2 py-1 text-[0.65rem] font-bold tracking-[0.1em]">{fileExt.toUpperCase()}</span>
					<span class="font-semibold text-[0.9rem]">{file.name}</span>
					<span class="font-mono text-xs text-white/35">{(file.size / 1024).toFixed(1)} KB</span>
				</div>
			{:else}
				<div class="flex flex-col items-center gap-2 text-white/40 text-[0.85rem]">
					<span class="text-3xl font-light text-white/20">+</span>
					<span>Drop a <strong>.json</strong> or <strong>.csv</strong> file here</span>
					<span class="text-[0.7rem] text-white/20">or click to browse</span>
				</div>
			{/if}
		</div>

		<input
			id="file-input"
			type="file"
			accept=".json,.csv"
			onchange={handleFileInput}
			style="display:none"
		/>

		<div class="flex gap-3 mt-4">
			{#if file}
				<button class="btn-outline px-6 py-3 text-[0.8rem] font-semibold tracking-mid uppercase" onclick={clearFile}>Clear</button>
			{/if}
			<button
				class="btn-primary flex-1 px-6 py-3 text-[0.8rem] font-semibold tracking-mid uppercase"
				disabled={!file || !validFile || uploading}
				onclick={upload}
			>
				{uploading ? 'Uploading...' : 'Upload & Import'}
			</button>
		</div>

		{#if file && !validFile}
			<p class="text-error mt-4 text-[0.8rem]">Unsupported file type. Please upload a .json or .csv file.</p>
		{/if}

		{#if error}
			<p class="text-error mt-4 text-[0.8rem]">{error}</p>
		{/if}
	</div>

	<div class="min-w-0">
		{#if hasPreview}
			<h3 class="section-heading flex items-baseline gap-3 mb-5">
				File Preview
				<span class="font-mono text-[0.65rem] font-normal tracking-normal normal-case text-white/20">{parsedRecords.length} record{parsedRecords.length !== 1 ? 's' : ''}</span>
			</h3>

			{#if parseError}
				<div class="border border-[rgba(239,68,68,0.3)] bg-[rgba(239,68,68,0.05)] p-4 flex flex-col gap-1.5">
					<span class="text-[0.65rem] font-bold tracking-[0.1em] uppercase text-[#ef4444]">Parse Error</span>
					<span class="text-[0.8rem] font-mono text-[rgba(239,68,68,0.8)]">{parseError}</span>
				</div>
			{:else}
				{#each parsedRecords as record, idx}
					<div class="mb-5">
						{#if parsedRecords.length > 1}
							<h4 class="text-[0.7rem] font-bold tracking-[0.1em] uppercase text-white/50 m-0 mb-2 whitespace-nowrap overflow-hidden text-ellipsis">Record {idx + 1}{record.name ? ` — ${record.name}` : ''}</h4>
						{:else if record.name}
							<h4 class="text-[0.7rem] font-bold tracking-[0.1em] uppercase text-white/50 m-0 mb-2">{record.name}</h4>
						{/if}

						{#if missingRequired(record).length > 0}
							<div class="text-[0.7rem] font-mono text-[#ef4444] px-2.5 py-1.5 border border-[rgba(239,68,68,0.2)] bg-[rgba(239,68,68,0.05)] mb-2">Missing required: {missingRequired(record).join(', ')}</div>
						{/if}

						<pre class="bg-white/3 border border-white/6 p-4 overflow-x-auto m-0 max-h-[280px] overflow-y-auto"><code class="font-mono text-[0.72rem] leading-relaxed text-white/60">{#each Object.entries(record) as [key, value], i}<span class="{classifyField(key) === 'valid' ? 'text-[#64748b]' : 'text-[#ef4444]'}">"{key}"</span>: <span class="{classifyField(key) === 'valid' ? 'text-[#94a3b8]' : 'text-[rgba(239,68,68,0.6)]'}">{typeof value === 'object' && value !== null ? JSON.stringify(value, null, 2).split('\n').map((line, li) => li === 0 ? line : '  ' + line).join('\n') : JSON.stringify(value)}</span>{i < Object.entries(record).length - 1 ? ',' : ''}
{/each}</code></pre>
					</div>
				{/each}

				<div class="flex gap-5 mt-3">
					<span class="flex items-center gap-1.5 text-[0.65rem] text-white/30 tracking-label">
						<span class="inline-block w-2 h-2 rounded-[1px] bg-[#64748b]"></span> Known field
					</span>
					<span class="flex items-center gap-1.5 text-[0.65rem] text-white/30 tracking-label">
						<span class="inline-block w-2 h-2 rounded-[1px] bg-[#ef4444]"></span> Extra (ignored)
					</span>
				</div>
			{/if}
		{:else}
			<h3 class="section-heading mb-5">Expected Formats</h3>

			<div class="mb-6">
				<h4 class="text-[0.7rem] font-bold tracking-[0.1em] uppercase text-white/50 m-0 mb-2">JSON</h4>
				<pre class="bg-white/3 border border-white/6 p-4 overflow-x-auto m-0"><code class="font-mono text-[0.72rem] leading-relaxed text-white/60">{`[
  {
    "name": "My Reactor",
    "design_type": "SFR",
    "vendor": "Acme Nuclear",
    "thermal_power_mw": 500,
    "electric_power_mw": 200,
    "coolant_type": "Sodium",
    "moderator": null,
    "fuel_type": "MOX",
    "enrichment_pct": 20.0,
    "source_url": "https://...",
    "design_metadata": {
      "breeding_ratio": 1.2,
      "cycle_type": "Rankine"
    }
  }
]`}</code></pre>
			</div>

			<div class="mb-6">
				<h4 class="text-[0.7rem] font-bold tracking-[0.1em] uppercase text-white/50 m-0 mb-2">CSV</h4>
				<pre class="bg-white/3 border border-white/6 p-4 overflow-x-auto m-0"><code class="font-mono text-[0.72rem] leading-relaxed text-white/60">{`name,design_type,vendor,thermal_power_mw,electric_power_mw,coolant_type,moderator,fuel_type,enrichment_pct,source_url
My Reactor,SFR,Acme Nuclear,500,200,Sodium,,MOX,20.0,https://...`}</code></pre>
				<p class="text-[0.72rem] text-white/25 mt-2 mb-0">CSV does not support design_metadata. It can be added after import.</p>
			</div>
		{/if}
	</div>
</div>

{#if result}
	<div class="mt-4">
		<div class="flex gap-8 mb-4">
			<div class="flex flex-col gap-1">
				<span class="font-mono text-3xl font-bold max-md:text-2xl">{result.imported}</span>
				<span class="text-[0.7rem] uppercase tracking-[0.1em] text-white/35">Imported</span>
			</div>
			{#if result.failed > 0}
				<div class="flex flex-col gap-1">
					<span class="font-mono text-3xl font-bold text-error max-md:text-2xl">{result.failed}</span>
					<span class="text-[0.7rem] uppercase tracking-[0.1em] text-white/35">Failed</span>
				</div>
			{/if}
		</div>

		{#if result.errors.length > 0}
			<div class="border border-[rgba(255,51,102,0.2)] px-5 py-4 mb-6">
				<h4 class="text-[0.7rem] font-bold tracking-[0.1em] uppercase text-error m-0 mb-3">Errors</h4>
				{#each result.errors as err}
					<p class="text-[0.8rem] text-white/50 my-1.5 font-mono">{err}</p>
				{/each}
			</div>
		{/if}

		{#if result.reactors.length > 0}
			<h3 class="section-heading mt-8 mb-4">Imported Reactors</h3>
			<div class="border border-white/6">
				<div class="table-header max-xl:hidden">
					<span>Type</span>
					<span>Name</span>
					<span>Vendor</span>
					<span>Thermal</span>
					<span>Electric</span>
					<span>Coolant</span>
					<span>Fuel</span>
				</div>
				{#each result.reactors as reactor}
					<a href="/reactors/{reactor.id}" class="table-row max-xl:flex max-xl:flex-wrap max-xl:gap-x-4 max-xl:gap-y-2 max-xl:p-3.5 max-xl:items-baseline">
						<span class="max-xl:order-0">
							<span class="type-badge">{reactor.design_type}</span>
						</span>
						<span class="max-xl:order-1 max-xl:flex-1 max-xl:min-w-0 max-xl:font-semibold">{reactor.name}</span>
						<span class="text-white/50 max-xl:order-2 max-xl:w-full max-xl:text-xs">{reactor.vendor ?? '--'}</span>
						<span class="font-mono max-xl:text-[0.8rem]">{reactor.thermal_power_mw ?? '--'} <small class="text-white/25 text-[0.7rem]">MW</small></span>
						<span class="font-mono max-xl:text-[0.8rem]">{reactor.electric_power_mw ?? '--'} <small class="text-white/25 text-[0.7rem]">MW</small></span>
						<span class="text-white/50 max-xl:text-xs max-md:hidden">{reactor.coolant_type ?? '--'}</span>
						<span class="text-white/50 max-xl:text-xs max-md:hidden">{reactor.fuel_type ?? '--'}</span>
					</a>
				{/each}
			</div>
		{/if}
	</div>
{/if}
