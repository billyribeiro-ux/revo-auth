import type { RequestHandler } from '@sveltejs/kit';
import { authConfig } from '../../../../auth.config.js';

/**
 * OAuth return URL. We forward the querystring to the Revo-Auth server's
 * callback endpoint so it can exchange the code, set cookies on the response,
 * and hand control back to the demo app with Set-Cookie headers preserved.
 */
export const GET: RequestHandler = async ({ url, fetch, request }) => {
	const target = new URL('/v1/oauth/callback', authConfig.serverUrl);
	for (const [key, value] of url.searchParams) target.searchParams.append(key, value);

	const serverResponse = await fetch(target, {
		method: 'GET',
		headers: {
			'X-Revo-App-Id': authConfig.appId,
			'X-Revo-App-Public-Key': authConfig.publicKey,
			cookie: request.headers.get('cookie') ?? ''
		},
		redirect: 'manual'
	});

	const destination = url.searchParams.get('redirect') ?? '/dashboard';

	const headers = new Headers({ location: destination });
	// Propagate Set-Cookie entries (session + CSRF) so the browser is authed.
	const setCookies = serverResponse.headers.getSetCookie?.() ?? [];
	for (const cookie of setCookies) headers.append('set-cookie', cookie);

	return new Response(null, { status: 303, headers });
};
