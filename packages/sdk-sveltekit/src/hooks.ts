import type { RevoAuthConfig, RevoAuthSession } from '@revo-auth/sdk-core';
import type { Handle, RequestEvent } from '@sveltejs/kit';
import { createServerClient } from './server.js';

const SESSION_COOKIES = ['__Host-revoauth.session', '__Host-revoauth.csrf'];

export const sessionCacheKey = Symbol.for('@revo-auth/sdk-sveltekit/session');

interface LocalsWithCache {
	[sessionCacheKey]?: RevoAuthSession | null;
	session?: RevoAuthSession | null;
}

export interface AuthHookOptions {
	config: Omit<RevoAuthConfig, 'fetchFn' | 'cookie' | 'csrfToken'>;
	/**
	 * When true, the hook will clear session/csrf cookies on `SESSION_EXPIRED`
	 * or any auth error that invalidates the session. Defaults to true.
	 */
	clearOnFailure?: boolean;
}

export function handleAuth(options: AuthHookOptions): Handle {
	const clearOnFailure = options.clearOnFailure ?? true;

	return async ({ event, resolve }) => {
		const locals = event.locals as unknown as LocalsWithCache;

		if (sessionCacheKey in locals) {
			locals.session = locals[sessionCacheKey] ?? null;
			return resolve(event);
		}

		const hasSessionCookie = event.cookies.get('__Host-revoauth.session') !== undefined;

		let session: RevoAuthSession | null = null;
		if (hasSessionCookie) {
			const client = createServerClient(event, { config: options.config });
			const result = await client.session();
			if (result.ok) {
				session = result.data;
			} else if (clearOnFailure) {
				for (const name of SESSION_COOKIES) {
					event.cookies.delete(name, { path: '/' });
				}
			}
		}

		locals[sessionCacheKey] = session;
		locals.session = session;

		return resolve(event);
	};
}

/** Retrieve the cached session off `event.locals`. */
export function getCachedSession(event: RequestEvent): RevoAuthSession | null {
	const locals = event.locals as unknown as LocalsWithCache;
	if (sessionCacheKey in locals) {
		return locals[sessionCacheKey] ?? null;
	}
	return locals.session ?? null;
}
