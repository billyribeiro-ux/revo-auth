import { createAuthClient } from '@revo-auth/sdk-sveltekit';
import type { RequestEvent } from '@sveltejs/kit';
import { authConfig } from '../../../auth.config.js';

/**
 * Build a server-side auth client bound to the current request's cookies so
 * SSR calls propagate the session and CSRF tokens to the Revo-Auth server.
 */
export function serverAuthClient(event: RequestEvent) {
	return createAuthClient({
		...authConfig,
		fetchFn: event.fetch,
		cookie: event.request.headers.get('cookie') ?? undefined
	});
}
