import type { APIRequestContext } from '@playwright/test';

export const AUTH_SERVER_URL = process.env.REVO_AUTH_SERVER_URL ?? 'http://localhost:8787';

/**
 * Ping the auth server's health endpoint. Returns true only if it responds OK.
 * Tests call this and `test.skip(!ok, 'auth server not running')` — failing
 * soft is preferable when the Docker image isn't booted.
 */
export async function isAuthServerUp(request: APIRequestContext): Promise<boolean> {
	try {
		const response = await request.get(`${AUTH_SERVER_URL}/health`, { timeout: 2_000 });
		return response.ok();
	} catch {
		return false;
	}
}

export function randomEmail(prefix = 'pw'): string {
	const suffix = Math.random().toString(36).slice(2, 10);
	return `${prefix}-${Date.now()}-${suffix}@example.test`;
}

/**
 * Attempt to read the latest outbound email for `recipient` from the server's
 * admin inspector endpoint (mock transport in dev). Returns null if the
 * endpoint is unavailable or no email was found.
 */
export async function latestMailFor(
	request: APIRequestContext,
	recipient: string
): Promise<{ subject: string; body: string } | null> {
	try {
		const response = await request.get(
			`${AUTH_SERVER_URL}/admin/mail-inspect?to=${encodeURIComponent(recipient)}`,
			{ timeout: 2_000 }
		);
		if (!response.ok()) return null;
		const data = (await response.json()) as { subject: string; body: string } | null;
		return data;
	} catch {
		return null;
	}
}

/**
 * Read the latest magic-link token for `recipient` from the admin inspector.
 */
export async function latestMagicTokenFor(
	request: APIRequestContext,
	recipient: string
): Promise<string | null> {
	try {
		const response = await request.get(
			`${AUTH_SERVER_URL}/admin/magic-inspect?to=${encodeURIComponent(recipient)}`,
			{ timeout: 2_000 }
		);
		if (!response.ok()) return null;
		const data = (await response.json()) as { token: string } | null;
		return data?.token ?? null;
	} catch {
		return null;
	}
}
