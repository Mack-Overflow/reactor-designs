const API_BASE = import.meta.env.VITE_API_URL ?? 'http://localhost:18080';

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const headers: Record<string, string> = {};
	if (init?.body) {
		headers['Content-Type'] = 'application/json';
	}
	const res = await fetch(`${API_BASE}${path}`, {
		...init,
		headers: { ...headers, ...init?.headers }
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({}));
		throw new Error(body.error ?? `API error: ${res.status}`);
	}
	if (res.status === 204) return undefined as T;
	return res.json();
}
