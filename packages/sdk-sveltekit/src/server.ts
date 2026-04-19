import { type ClientAPI, type RevoAuthConfig, createClient } from '@revo-auth/sdk-core';
import type { RequestEvent } from '@sveltejs/kit';

const CSRF_COOKIE = '__Host-revoauth.csrf';

export interface ServerClientOptions {
	/**
	 * Passed through to `createClient`. `fetchFn` is forced to `event.fetch`
	 * for SSR-safe cookie forwarding.
	 */
	config: Omit<RevoAuthConfig, 'fetchFn' | 'cookie' | 'csrfToken'>;
}

/**
 * Create an SDK client bound to a SvelteKit `RequestEvent`. Uses
 * `event.fetch` so cookies flow through during SSR, and forwards the
 * client's cookie header + CSRF cookie to the auth server.
 */
export function createServerClient(event: RequestEvent, options: ServerClientOptions): ClientAPI {
	const cookieHeader = event.request.headers.get('cookie');
	const csrfToken = event.cookies.get(CSRF_COOKIE);

	const config: RevoAuthConfig = {
		...options.config,
		fetchFn: event.fetch,
	};
	if (cookieHeader !== null) {
		config.cookie = cookieHeader;
	}
	if (csrfToken !== undefined) {
		config.csrfToken = csrfToken;
	}

	return createClient(config);
}
