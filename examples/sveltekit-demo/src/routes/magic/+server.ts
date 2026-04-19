import type { RequestHandler } from '@sveltejs/kit';
import { authConfig } from '../../../auth.config.js';

/**
 * Magic-link verify handler. Proxies `/magic/verify?token=…` to the server's
 * `/v1/magic/verify` endpoint, propagating Set-Cookie headers and redirecting
 * to the destination the server suggests.
 */
export const GET: RequestHandler = async ({ url, fetch, request }) => {
	const target = new URL('/v1/magic/verify', authConfig.serverUrl);
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

	const destination = serverResponse.headers.get('location') ?? '/dashboard';

	const headers = new Headers({ location: destination });
	const setCookies = serverResponse.headers.getSetCookie?.() ?? [];
	for (const cookie of setCookies) headers.append('set-cookie', cookie);

	return new Response(null, { status: 303, headers });
};
