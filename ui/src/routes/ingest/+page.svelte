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

<div class="page-header">
	<h1>Ingest Reactors</h1>
	<p class="subtitle">Upload reactor designs via JSON or CSV</p>
</div>

<div class="ingest-layout">
	<div class="upload-section">
		<div
			class="drop-zone"
			class:dragging
			class:has-file={file !== null}
			role="button"
			tabindex="0"
			ondragover={(e) => { e.preventDefault(); dragging = true; }}
			ondragleave={() => (dragging = false)}
			ondrop={handleDrop}
			onclick={() => document.getElementById('file-input')?.click()}
			onkeydown={(e) => { if (e.key === 'Enter') document.getElementById('file-input')?.click(); }}
		>
			{#if file}
				<div class="file-info">
					<span class="file-badge">{fileExt.toUpperCase()}</span>
					<span class="file-name">{file.name}</span>
					<span class="file-size">{(file.size / 1024).toFixed(1)} KB</span>
				</div>
			{:else}
				<div class="drop-prompt">
					<span class="drop-icon">+</span>
					<span>Drop a <strong>.json</strong> or <strong>.csv</strong> file here</span>
					<span class="drop-hint">or click to browse</span>
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

		<div class="actions">
			{#if file}
				<button class="btn-secondary" onclick={clearFile}>Clear</button>
			{/if}
			<button
				class="btn-primary"
				disabled={!file || !validFile || uploading}
				onclick={upload}
			>
				{uploading ? 'Uploading...' : 'Upload & Import'}
			</button>
		</div>

		{#if file && !validFile}
			<p class="status error">Unsupported file type. Please upload a .json or .csv file.</p>
		{/if}

		{#if error}
			<p class="status error">{error}</p>
		{/if}
	</div>

	<div class="format-guide">
		{#if hasPreview}
			<h3>File Preview <span class="preview-count">{parsedRecords.length} record{parsedRecords.length !== 1 ? 's' : ''}</span></h3>

			{#if parseError}
				<div class="parse-error">
					<span class="pe-label">Parse Error</span>
					<span class="pe-msg">{parseError}</span>
				</div>
			{:else}
				{#each parsedRecords as record, idx}
					<div class="record-block">
						{#if parsedRecords.length > 1}
							<h4>Record {idx + 1}{record.name ? ` — ${record.name}` : ''}</h4>
						{:else if record.name}
							<h4>{record.name}</h4>
						{/if}

						{#if missingRequired(record).length > 0}
							<div class="missing-warn">Missing required: {missingRequired(record).join(', ')}</div>
						{/if}

						<pre class="preview-pre"><code>{#each Object.entries(record) as [key, value], i}<span class={classifyField(key) === 'valid' ? 'field-valid' : 'field-extra'}>"{key}"</span>: <span class={classifyField(key) === 'valid' ? 'value-valid' : 'value-extra'}>{typeof value === 'object' && value !== null ? JSON.stringify(value, null, 2).split('\n').map((line, li) => li === 0 ? line : '  ' + line).join('\n') : JSON.stringify(value)}</span>{i < Object.entries(record).length - 1 ? ',' : ''}
{/each}</code></pre>
					</div>
				{/each}

				<div class="legend">
					<span class="legend-item"><span class="legend-dot valid"></span> Known field</span>
					<span class="legend-item"><span class="legend-dot extra"></span> Extra (ignored)</span>
				</div>
			{/if}
		{:else}
			<h3>Expected Formats</h3>

			<div class="format-block">
				<h4>JSON</h4>
				<pre><code>{`[
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

			<div class="format-block">
				<h4>CSV</h4>
				<pre><code>{`name,design_type,vendor,thermal_power_mw,electric_power_mw,coolant_type,moderator,fuel_type,enrichment_pct,source_url
My Reactor,SFR,Acme Nuclear,500,200,Sodium,,MOX,20.0,https://...`}</code></pre>
				<p class="format-note">CSV does not support design_metadata. It can be added after import.</p>
			</div>
		{/if}
	</div>
</div>

{#if result}
	<div class="results">
		<div class="result-summary">
			<div class="stat">
				<span class="stat-value">{result.imported}</span>
				<span class="stat-label">Imported</span>
			</div>
			{#if result.failed > 0}
				<div class="stat stat-error">
					<span class="stat-value">{result.failed}</span>
					<span class="stat-label">Failed</span>
				</div>
			{/if}
		</div>

		{#if result.errors.length > 0}
			<div class="error-list">
				<h4>Errors</h4>
				{#each result.errors as err}
					<p class="error-item">{err}</p>
				{/each}
			</div>
		{/if}

		{#if result.reactors.length > 0}
			<h3>Imported Reactors</h3>
			<div class="table-wrap">
				<div class="table-header">
					<span class="col-type">Type</span>
					<span class="col-name">Name</span>
					<span class="col-vendor">Vendor</span>
					<span class="col-thermal">Thermal</span>
					<span class="col-electric">Electric</span>
					<span class="col-coolant">Coolant</span>
					<span class="col-fuel">Fuel</span>
				</div>
				{#each result.reactors as reactor}
					<a href="/reactors/{reactor.id}" class="table-row">
						<span class="col-type">
							<span class="type-badge">{reactor.design_type}</span>
						</span>
						<span class="col-name">{reactor.name}</span>
						<span class="col-vendor">{reactor.vendor ?? '--'}</span>
						<span class="col-thermal mono">{reactor.thermal_power_mw ?? '--'} <small>MW</small></span>
						<span class="col-electric mono">{reactor.electric_power_mw ?? '--'} <small>MW</small></span>
						<span class="col-coolant">{reactor.coolant_type ?? '--'}</span>
						<span class="col-fuel">{reactor.fuel_type ?? '--'}</span>
					</a>
				{/each}
			</div>
		{/if}
	</div>
{/if}

<style>
	.page-header {
		margin-bottom: 2.5rem;
	}

	h1 {
		font-size: 2rem;
		font-weight: 800;
		letter-spacing: -0.03em;
		margin: 0 0 0.25rem;
	}

	.subtitle {
		color: rgba(255, 255, 255, 0.35);
		font-size: 0.85rem;
		margin: 0;
	}

	.ingest-layout {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 3rem;
		margin-bottom: 3rem;
	}

	/* Drop zone */
	.drop-zone {
		border: 1px dashed rgba(255, 255, 255, 0.15);
		padding: 3rem 2rem;
		text-align: center;
		cursor: pointer;
		transition: all 0.2s;
	}

	.drop-zone:hover,
	.drop-zone.dragging {
		border-color: rgba(255, 255, 255, 0.4);
		background: rgba(255, 255, 255, 0.02);
	}

	.drop-zone.has-file {
		border-style: solid;
		border-color: rgba(255, 255, 255, 0.2);
	}

	.drop-prompt {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		color: rgba(255, 255, 255, 0.4);
		font-size: 0.85rem;
	}

	.drop-icon {
		font-size: 2rem;
		font-weight: 300;
		color: rgba(255, 255, 255, 0.2);
	}

	.drop-hint {
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.2);
	}

	.file-info {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.75rem;
	}

	.file-badge {
		display: inline-block;
		border: 1px solid rgba(255, 255, 255, 0.2);
		padding: 0.2rem 0.5rem;
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.1em;
	}

	.file-name {
		font-weight: 600;
		font-size: 0.9rem;
	}

	.file-size {
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.35);
	}

	/* Actions */
	.actions {
		display: flex;
		gap: 0.75rem;
		margin-top: 1rem;
	}

	.btn-primary,
	.btn-secondary {
		padding: 0.7rem 1.5rem;
		font-size: 0.8rem;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
		font-family: 'Inter', sans-serif;
	}

	.btn-primary {
		background: #fff;
		color: #000;
		flex: 1;
	}

	.btn-primary:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.85);
	}

	.btn-primary:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: transparent;
		color: rgba(255, 255, 255, 0.5);
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	.btn-secondary:hover {
		color: #fff;
		border-color: rgba(255, 255, 255, 0.3);
	}

	/* Format guide */
	.format-guide {
		min-width: 0;
	}

	.format-guide h3 {
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
		margin: 0 0 1.25rem;
		display: flex;
		align-items: baseline;
		gap: 0.75rem;
	}

	.preview-count {
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.65rem;
		font-weight: 400;
		letter-spacing: 0;
		text-transform: none;
		color: rgba(255, 255, 255, 0.2);
	}

	.format-block {
		margin-bottom: 1.5rem;
	}

	.format-block h4 {
		font-size: 0.7rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.5);
		margin: 0 0 0.5rem;
	}

	pre {
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.06);
		padding: 1rem;
		overflow-x: auto;
		margin: 0;
	}

	code {
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.72rem;
		line-height: 1.6;
		color: rgba(255, 255, 255, 0.6);
	}

	.format-note {
		font-size: 0.72rem;
		color: rgba(255, 255, 255, 0.25);
		margin: 0.5rem 0 0;
	}

	/* Preview records */
	.record-block {
		margin-bottom: 1.25rem;
	}

	.record-block h4 {
		font-size: 0.7rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.5);
		margin: 0 0 0.5rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.preview-pre {
		max-height: 280px;
		overflow-y: auto;
	}

	.field-valid {
		color: #64748b;
	}

	.value-valid {
		color: #94a3b8;
	}

	.field-extra {
		color: #ef4444;
	}

	.value-extra {
		color: rgba(239, 68, 68, 0.6);
	}

	.missing-warn {
		font-size: 0.7rem;
		font-family: 'JetBrains Mono', monospace;
		color: #ef4444;
		padding: 0.4rem 0.65rem;
		border: 1px solid rgba(239, 68, 68, 0.2);
		background: rgba(239, 68, 68, 0.05);
		margin-bottom: 0.5rem;
	}

	.parse-error {
		border: 1px solid rgba(239, 68, 68, 0.3);
		background: rgba(239, 68, 68, 0.05);
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.pe-label {
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: #ef4444;
	}

	.pe-msg {
		font-size: 0.8rem;
		font-family: 'JetBrains Mono', monospace;
		color: rgba(239, 68, 68, 0.8);
	}

	.legend {
		display: flex;
		gap: 1.25rem;
		margin-top: 0.75rem;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.65rem;
		color: rgba(255, 255, 255, 0.3);
		letter-spacing: 0.04em;
	}

	.legend-dot {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 1px;
	}

	.legend-dot.valid {
		background: #64748b;
	}

	.legend-dot.extra {
		background: #ef4444;
	}

	/* Status */
	.status {
		margin-top: 1rem;
		font-size: 0.8rem;
	}

	.error {
		color: #ff3366;
	}

	/* Results */
	.results {
		margin-top: 1rem;
	}

	.results h3 {
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.3);
		margin: 2rem 0 1rem;
	}

	.result-summary {
		display: flex;
		gap: 2rem;
		margin-bottom: 1rem;
	}

	.stat {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.stat-value {
		font-family: 'JetBrains Mono', monospace;
		font-size: 2rem;
		font-weight: 700;
	}

	.stat-label {
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		color: rgba(255, 255, 255, 0.35);
	}

	.stat-error .stat-value {
		color: #ff3366;
	}

	.error-list {
		border: 1px solid rgba(255, 51, 102, 0.2);
		padding: 1rem 1.25rem;
		margin-bottom: 1.5rem;
	}

	.error-list h4 {
		font-size: 0.7rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: #ff3366;
		margin: 0 0 0.75rem;
	}

	.error-item {
		font-size: 0.8rem;
		color: rgba(255, 255, 255, 0.5);
		margin: 0.35rem 0;
		font-family: 'JetBrains Mono', monospace;
	}

	/* Table (matches reactors page) */
	.table-wrap {
		border: 1px solid rgba(255, 255, 255, 0.06);
	}

	.table-header {
		display: grid;
		grid-template-columns: 80px 1.5fr 1fr 0.8fr 0.8fr 1fr 1fr;
		gap: 0;
		padding: 0.75rem 1.25rem;
		font-size: 0.65rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.25);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
	}

	.table-row {
		display: grid;
		grid-template-columns: 80px 1.5fr 1fr 0.8fr 0.8fr 1fr 1fr;
		gap: 0;
		padding: 1rem 1.25rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.04);
		text-decoration: none;
		color: #fff;
		font-size: 0.85rem;
		transition: background 0.2s;
		align-items: center;
	}

	.table-row:hover {
		background: rgba(255, 255, 255, 0.03);
	}

	.table-row:last-child {
		border-bottom: none;
	}

	.type-badge {
		display: inline-block;
		border: 1px solid rgba(255, 255, 255, 0.15);
		padding: 0.15rem 0.45rem;
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}

	.col-vendor,
	.col-coolant,
	.col-fuel {
		color: rgba(255, 255, 255, 0.5);
	}

	.mono {
		font-family: 'JetBrains Mono', monospace;
	}

	small {
		color: rgba(255, 255, 255, 0.25);
		font-size: 0.7rem;
	}

	@media (max-width: 900px) {
		.ingest-layout {
			grid-template-columns: 1fr;
		}

		.table-header {
			display: none;
		}

		.table-row {
			display: flex;
			flex-wrap: wrap;
			gap: 0.5rem 1rem;
			padding: 0.85rem;
			align-items: baseline;
		}

		.col-type {
			order: 0;
		}

		.col-name {
			order: 1;
			flex: 1;
			min-width: 0;
			font-weight: 600;
		}

		.col-vendor {
			order: 2;
			width: 100%;
			font-size: 0.75rem;
		}

		.col-thermal,
		.col-electric {
			font-size: 0.8rem;
		}

		.col-coolant,
		.col-fuel {
			font-size: 0.75rem;
		}
	}

	@media (max-width: 600px) {
		.drop-zone {
			padding: 2rem 1rem;
		}

		.col-coolant,
		.col-fuel {
			display: none;
		}

		.stat-value {
			font-size: 1.5rem;
		}
	}
</style>
