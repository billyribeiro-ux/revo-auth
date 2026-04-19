import { describe, expect, it, vi } from 'vitest';
import { createClient } from '../src/client.js';
import { InvalidCredentialsError } from '../src/errors.js';
import type { RevoAuthSession, RevoAuthUser } from '../src/types.js';

function makeUser(overrides: Partial<RevoAuthUser> = {}): RevoAuthUser {
	return {
		id: 'usr_1',
		email: 'user@example.com',
		emailVerified: false,
		name: null,
		image: null,
		customFields: {},
		createdAt: new Date().toISOString(),
		...overrides,
	};
}

function makeSession(overrides: Partial<RevoAuthSession> = {}): RevoAuthSession {
	return {
		id: 'sess_1',
		userId: 'usr_1',
		expiresAt: new Date(Date.now() + 3600_000).toISOString(),
		user: makeUser(),
		...overrides,
	};
}

interface MockCall {
	url: string;
	init: RequestInit;
}

function makeMockFetch(
	handler: (call: MockCall) => { status?: number; body: unknown },
): { fetchFn: typeof fetch; calls: MockCall[] } {
	const calls: MockCall[] = [];
	const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
		const url = typeof input === 'string' ? input : input instanceof URL ? input.toString() : input.url;
		const call: MockCall = { url, init: init ?? {} };
		calls.push(call);
		const { status = 200, body } = handler(call);
		return new Response(JSON.stringify(body), {
			status,
			headers: { 'content-type': 'application/json' },
		});
	}) as unknown as typeof fetch;
	return { fetchFn, calls };
}

describe('createClient', () => {
	it('signup returns a parsed session on 2xx', async () => {
		const session = makeSession();
		const { fetchFn, calls } = makeMockFetch(() => ({ body: { session } }));

		const client = createClient({
			serverUrl: 'https://auth.example.com',
			appId: 'app_test',
			publicKey: 'pk_test',
			fetchFn,
		});

		const result = await client.signup({ email: 'x@example.com', password: 'a-very-secure-password-123' });
		expect(result.ok).toBe(true);
		if (result.ok) {
			expect(result.data.id).toBe(session.id);
			expect(result.data.user.email).toBe('user@example.com');
		}
		expect(calls.length).toBe(1);
		const firstCall = calls[0];
		if (!firstCall) throw new Error('expected call');
		expect(firstCall.url).toContain('/v1/signup');
		const rawHeaders = firstCall.init.headers;
		const getHeader = (name: string): string | null => {
			if (rawHeaders instanceof Headers) return rawHeaders.get(name);
			if (Array.isArray(rawHeaders)) {
				const hit = rawHeaders.find(([k]) => k.toLowerCase() === name.toLowerCase());
				return hit ? (hit[1] ?? null) : null;
			}
			if (rawHeaders && typeof rawHeaders === 'object') {
				const obj = rawHeaders as Record<string, string>;
				const key = Object.keys(obj).find((k) => k.toLowerCase() === name.toLowerCase());
				return key !== undefined ? (obj[key] ?? null) : null;
			}
			return null;
		};
		expect(getHeader('X-Revo-App-Id')).toBe('app_test');
		expect(getHeader('X-Revo-App-Public-Key')).toBe('pk_test');
	});

	it('signin returns an InvalidCredentialsError for 401 envelope', async () => {
		const { fetchFn } = makeMockFetch(() => ({
			status: 401,
			body: {
				error: {
					code: 'INVALID_CREDENTIALS',
					message: 'Email or password is incorrect.',
					request_id: 'req_abc',
				},
			},
		}));

		const client = createClient({
			serverUrl: 'https://auth.example.com',
			appId: 'app_test',
			publicKey: 'pk_test',
			fetchFn,
		});

		const result = await client.signin({ email: 'x@example.com', password: 'bad' });
		expect(result.ok).toBe(false);
		if (!result.ok) {
			expect(result.error).toBeInstanceOf(InvalidCredentialsError);
			expect(result.error.code).toBe('INVALID_CREDENTIALS');
			expect(result.error.requestId).toBe('req_abc');
		}
	});

	it('rejects invalid config via zod at createClient time', () => {
		expect(() =>
			createClient({
				serverUrl: 'not-a-url',
				appId: '',
				publicKey: '',
			}),
		).toThrow();
	});
});
