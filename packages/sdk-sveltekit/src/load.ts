import type { RevoAuthSession } from '@revo-auth/sdk-core';
import { type RequestEvent, redirect } from '@sveltejs/kit';
import { getCachedSession } from './hooks.js';

export interface RequireAuthOptions {
	/** Path to redirect to when there is no session. Defaults to `/login`. */
	loginPath?: string;
	/** Optional query param name for return URL. Defaults to `redirect`. */
	redirectParam?: string;
}

/**
 * Throws a SvelteKit `redirect(302)` to the login path if the user has no
 * active session. Otherwise returns the session.
 */
export function requireAuth(
	event: RequestEvent,
	options: RequireAuthOptions = {},
): RevoAuthSession {
	const session = getCachedSession(event);
	if (session !== null) {
		return session;
	}
	const loginPath = options.loginPath ?? '/login';
	const redirectParam = options.redirectParam ?? 'redirect';
	const target = `${loginPath}?${redirectParam}=${encodeURIComponent(event.url.pathname + event.url.search)}`;
	throw redirect(302, target);
}

export function optionalAuth(event: RequestEvent): RevoAuthSession | null {
	return getCachedSession(event);
}
