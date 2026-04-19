import { createClient, type ClientAPI } from '@revo-auth/sdk-core';
import { publicAuthConfig } from '../../../auth.config.js';

let singleton: ClientAPI | null = null;

/**
 * Browser-side auth client. The SDK handles CSRF cookie injection and the
 * user agent forwards session cookies automatically.
 */
export function authClient(): ClientAPI {
	if (singleton !== null) return singleton;
	singleton = createClient(publicAuthConfig);
	return singleton;
}
