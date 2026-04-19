import type { ClientAPI } from './client.js';
import type { OAuthProvider, Result } from './types.js';

/**
 * Begin an OAuth flow. Returns the authorize URL; the caller is responsible
 * for redirecting the browser (e.g. `window.location.assign(url)`).
 */
export async function beginOAuth(
	client: ClientAPI,
	provider: OAuthProvider,
	redirect?: string,
): Promise<Result<{ url: string }>> {
	return client.oauthAuthorizeUrl(provider, redirect);
}
