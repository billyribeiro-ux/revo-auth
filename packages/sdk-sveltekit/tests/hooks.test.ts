import { describe, expect, it, vi } from 'vitest';
import type { RevoAuthSession, RevoAuthUser } from '@revo-auth/sdk-core';
import { handleAuth, sessionCacheKey } from '../src/hooks.js';

function makeUser(): RevoAuthUser {
	return {
		id: 'usr_1',
		email: 'user@example.com',
		emailVerified: true,
		name: null,
		image: null,
		customFields: {},
		createdAt: new Date().toISOString(),
	};
}

function makeSession(): RevoAuthSession {
	return {
		id: 'sess_1',
		userId: 'usr_1',
		expiresAt: new Date(Date.now() + 3600_000).toISOString(),
		user: makeUser(),
	};
}

function makeEvent(opts: {
	cookies?: Record<string, string>;
	fetchBody?: unknown;
	fetchStatus?: number;
}): {
	event: Parameters<ReturnType<typeof handleAuth>>[0]['event'];
	deleted: string[];
} {
	const cookieMap = new Map(Object.entries(opts.cookies ?? {}));
	const deleted: string[] = [];
	const fetchFn = vi.fn(async () => {
		return new Response(JSON.stringify(opts.fetchBody ?? { session: null }), {
			status: opts.fetchStatus ?? 200,
			headers: { 'content-type': 'application/json' },
		});
	});

	const event = {
		cookies: {
			get: (name: string) => cookieMap.get(name),
			getAll: () =>
				Array.from(cookieMap.entries()).map(([name, value]) => ({ name, value })),
			set: (name: string, value: string) => cookieMap.set(name, value),
			delete: (name: string) => {
				deleted.push(name);
				cookieMap.delete(name);
			},
			serialize: () => '',
		},
		request: new Request('https://app.example.com/', {
			headers: {
				cookie: Array.from(cookieMap.entries())
					.map(([k, v]) => `${k}=${v}`)
					.join('; '),
			},
		}),
		url: new URL('https://app.example.com/'),
		fetch: fetchFn,
		locals: {},
		params: {},
		platform: undefined,
		route: { id: null },
		setHeaders: () => undefined,
		isDataRequest: false,
		isSubRequest: false,
		isRemoteRequest: false,
	} as unknown as Parameters<ReturnType<typeof handleAuth>>[0]['event'];

	return { event, deleted };
}

describe('handleAuth', () => {
	it('populates event.locals.session as null when no cookie is present', async () => {
		const hook = handleAuth({
			config: {
				serverUrl: 'https://auth.example.com',
				appId: 'app_test',
				publicKey: 'pk_test',
			},
		});

		const { event } = makeEvent({});
		const resolve = vi.fn(async () => new Response('ok'));
		await hook({ event, resolve });

		const locals = event.locals as unknown as { session?: unknown; [k: symbol]: unknown };
		expect(locals.session).toBeNull();
		expect(locals[sessionCacheKey]).toBeNull();
	});

	it('populates event.locals.session on successful fetch', async () => {
		const session = makeSession();
		const hook = handleAuth({
			config: {
				serverUrl: 'https://auth.example.com',
				appId: 'app_test',
				publicKey: 'pk_test',
			},
		});

		const { event } = makeEvent({
			cookies: { '__Host-revoauth.session': 'token' },
			fetchBody: { session },
		});
		const resolve = vi.fn(async () => new Response('ok'));
		await hook({ event, resolve });

		const locals = event.locals as unknown as { session?: RevoAuthSession | null };
		expect(locals.session).not.toBeNull();
		expect(locals.session?.id).toBe(session.id);
	});

	it('clears cookies on failure', async () => {
		const hook = handleAuth({
			config: {
				serverUrl: 'https://auth.example.com',
				appId: 'app_test',
				publicKey: 'pk_test',
			},
		});

		const { event, deleted } = makeEvent({
			cookies: { '__Host-revoauth.session': 'token' },
			fetchStatus: 401,
			fetchBody: {
				error: {
					code: 'SESSION_EXPIRED',
					message: 'expired',
				},
			},
		});
		const resolve = vi.fn(async () => new Response('ok'));
		await hook({ event, resolve });

		expect(deleted).toContain('__Host-revoauth.session');
		expect(deleted).toContain('__Host-revoauth.csrf');
	});
});
