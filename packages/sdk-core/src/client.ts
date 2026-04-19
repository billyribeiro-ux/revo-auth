import { defu } from 'defu';
import { type $Fetch, type FetchOptions, FetchError, createFetch, ofetch } from 'ofetch';
import type { ZodSchema } from 'zod';
import { z } from 'zod';
import {
	NetworkError,
	RateLimitedError,
	RevoAuthError,
	UnknownError,
	ValidationError,
	errorFromEnvelope,
} from './errors.js';
import {
	configSchema,
	errorEnvelopeSchema,
	magicRequestResponseSchema,
	oauthAuthorizeResponseSchema,
	okResponseSchema,
	organizationListResponseSchema,
	organizationSchema,
	passkeyAuthenticateBeginResponseSchema,
	passkeyAuthenticateFinishResponseSchema,
	passkeyListResponseSchema,
	passkeyRegisterBeginResponseSchema,
	passkeyRegisterFinishResponseSchema,
	sessionNullableResponseSchema,
	sessionResponseSchema,
	sessionsListResponseSchema,
	totpSetupResponseSchema,
} from './schemas.js';
import type {
	AcceptInviteInput,
	CreateOrgInput,
	EmailVerifyConfirmInput,
	InviteMemberInput,
	ListResponse,
	MagicRequestInput,
	OAuthProvider,
	OkResponse,
	Organization,
	Passkey,
	PasswordResetConfirmInput,
	PasswordResetRequestInput,
	Result,
	RevoAuthConfig,
	RevoAuthLogger,
	RevoAuthSession,
	SigninInput,
	SignupInput,
	TotpConfirmInput,
	TotpSetupResult,
	TotpVerifyInput,
} from './types.js';

const noopLogger: RevoAuthLogger = {
	debug: () => undefined,
	info: () => undefined,
	warn: () => undefined,
	error: () => undefined,
};

const MUTATING_METHODS = new Set(['POST', 'PUT', 'PATCH', 'DELETE']);
const CSRF_COOKIE_NAME = '__Host-revoauth.csrf';

function readCsrfFromDocument(): string | undefined {
	if (typeof document === 'undefined') return undefined;
	const raw = document.cookie;
	if (!raw) return undefined;
	const parts = raw.split(';');
	for (const part of parts) {
		const trimmed = part.trim();
		if (trimmed.startsWith(`${CSRF_COOKIE_NAME}=`)) {
			return decodeURIComponent(trimmed.slice(CSRF_COOKIE_NAME.length + 1));
		}
	}
	return undefined;
}

export interface ClientAPI {
	signup(input: SignupInput): Promise<Result<RevoAuthSession>>;
	signin(input: SigninInput): Promise<Result<RevoAuthSession>>;
	signout(): Promise<Result<OkResponse>>;
	session(): Promise<Result<RevoAuthSession | null>>;
	sessionRefresh(): Promise<Result<RevoAuthSession>>;
	listSessions(): Promise<Result<ListResponse<RevoAuthSession>>>;
	revokeSession(id: string): Promise<Result<OkResponse>>;
	passwordResetRequest(input: PasswordResetRequestInput): Promise<Result<OkResponse>>;
	passwordResetConfirm(input: PasswordResetConfirmInput): Promise<Result<OkResponse>>;
	emailVerifyRequest(): Promise<Result<OkResponse>>;
	emailVerifyConfirm(input: EmailVerifyConfirmInput): Promise<Result<OkResponse>>;
	oauthAuthorizeUrl(provider: OAuthProvider, redirect?: string): Promise<Result<{ url: string }>>;
	passkeyRegisterBegin(): Promise<Result<{ publicKey: Record<string, unknown> }>>;
	passkeyRegisterFinish(response: Record<string, unknown>): Promise<Result<{ passkey: Passkey }>>;
	passkeyAuthenticateBegin(
		email?: string,
	): Promise<Result<{ publicKey: Record<string, unknown> }>>;
	passkeyAuthenticateFinish(response: Record<string, unknown>): Promise<Result<RevoAuthSession>>;
	listPasskeys(): Promise<Result<ListResponse<Passkey>>>;
	revokePasskey(id: string): Promise<Result<OkResponse>>;
	totpSetup(): Promise<Result<TotpSetupResult>>;
	totpConfirm(input: TotpConfirmInput): Promise<Result<OkResponse>>;
	totpVerify(input: TotpVerifyInput): Promise<Result<OkResponse>>;
	totpDisable(): Promise<Result<OkResponse>>;
	magicRequest(input: MagicRequestInput): Promise<Result<OkResponse>>;
	magicVerifyUrl(token: string): string;
	linkAccount(provider: OAuthProvider, redirect?: string): Promise<Result<{ url: string }>>;
	unlinkAccount(provider: OAuthProvider): Promise<Result<OkResponse>>;
	listOrgs(): Promise<Result<ListResponse<Organization>>>;
	createOrg(input: CreateOrgInput): Promise<Result<Organization>>;
	inviteMember(orgId: string, input: InviteMemberInput): Promise<Result<OkResponse>>;
	acceptInvite(orgId: string, input: AcceptInviteInput): Promise<Result<OkResponse>>;
	readonly config: Readonly<RevoAuthConfig>;
}

type FetchOpts = FetchOptions<'json'> & { method?: string };

function joinUrl(base: string, path: string): string {
	const b = base.endsWith('/') ? base.slice(0, -1) : base;
	const p = path.startsWith('/') ? path : `/${path}`;
	return `${b}${p}`;
}

export function createClient(rawConfig: RevoAuthConfig): ClientAPI {
	const parsed = configSchema.safeParse(rawConfig);
	if (!parsed.success) {
		throw new ValidationError(
			'Invalid RevoAuthConfig',
			parsed.error.issues.map((i) => ({ path: i.path, message: i.message })),
		);
	}
	const config: RevoAuthConfig = parsed.data as RevoAuthConfig;
	const logger: RevoAuthLogger = config.logger ?? noopLogger;

	const baseFetcher: $Fetch = config.fetchFn
		? createFetch({
				fetch: config.fetchFn,
				Headers: globalThis.Headers,
				AbortController: globalThis.AbortController,
			})
		: ofetch;
	const fetcher: $Fetch = baseFetcher.create({
		baseURL: config.serverUrl,
		retry: 0,
		// Allow error responses through so we can parse the envelope.
		ignoreResponseError: false,
	});

	async function request<T>(
		path: string,
		schema: ZodSchema<T>,
		opts: FetchOpts = {},
	): Promise<Result<T>> {
		const method = (opts.method ?? 'GET').toUpperCase();
		const headers: Record<string, string> = {
			'X-Revo-App-Id': config.appId,
			'X-Revo-App-Public-Key': config.publicKey,
			Accept: 'application/json',
			...(opts.headers as Record<string, string> | undefined),
		};
		if (config.cookie !== undefined) {
			headers['cookie'] = config.cookie;
		}
		if (MUTATING_METHODS.has(method)) {
			const csrf = config.csrfToken ?? readCsrfFromDocument();
			if (csrf !== undefined) {
				headers['X-CSRF-Token'] = csrf;
			}
		}

		const merged = defu({ headers }, opts) as FetchOpts;

		try {
			logger.debug('revo-auth: request', { method, path });
			const response = await fetcher(path, {
				...merged,
				method,
				credentials: merged.credentials ?? 'include',
			});
			const validated = schema.safeParse(response);
			if (!validated.success) {
				return {
					ok: false,
					error: new ValidationError(
						'Server response failed schema validation',
						validated.error.issues.map((i) => ({ path: i.path, message: i.message })),
					),
				};
			}
			return { ok: true, data: validated.data };
		} catch (err) {
			return { ok: false, error: normalizeError(err) };
		}
	}

	function normalizeError(err: unknown): RevoAuthError {
		if (err instanceof RevoAuthError) return err;
		if (err instanceof FetchError) {
			const status = err.response?.status ?? 0;
			const body: unknown = err.data;
			const envelope = errorEnvelopeSchema.safeParse(body);
			if (envelope.success) {
				const e = envelope.data.error;
				if (status === 429) {
					const retryHeader = err.response?.headers.get('retry-after');
					const retryAfterSeconds =
						retryHeader !== null && retryHeader !== undefined && retryHeader !== ''
							? Number.parseInt(retryHeader, 10)
							: undefined;
					const opts: { requestId?: string; retryAfterSeconds?: number } = {};
					if (e.request_id !== undefined) opts.requestId = e.request_id;
					if (retryAfterSeconds !== undefined && !Number.isNaN(retryAfterSeconds)) {
						opts.retryAfterSeconds = retryAfterSeconds;
					}
					return new RateLimitedError(e.message, opts);
				}
				return errorFromEnvelope(e.code, e.message, e.request_id);
			}
			if (status === 0) {
				return new NetworkError('Network request failed.', { cause: err });
			}
			return new UnknownError(err.message, { cause: err });
		}
		if (err instanceof TypeError) {
			return new NetworkError(err.message, { cause: err });
		}
		return new UnknownError(
			err instanceof Error ? err.message : 'Unknown error',
			err instanceof Error ? { cause: err } : undefined,
		);
	}

	async function unwrapSession(
		p: Promise<Result<{ session: RevoAuthSession }>>,
	): Promise<Result<RevoAuthSession>> {
		const r = await p;
		if (!r.ok) return r;
		return { ok: true, data: r.data.session };
	}

	async function unwrapOrg(
		p: Promise<Result<{ org: Organization }>>,
	): Promise<Result<Organization>> {
		const r = await p;
		if (!r.ok) return r;
		return { ok: true, data: r.data.org };
	}

	return {
		get config() {
			return config;
		},

		signup: (input) =>
			unwrapSession(
				request('/v1/signup', sessionResponseSchema, {
					method: 'POST',
					body: input,
				}),
			),

		signin: (input) =>
			unwrapSession(
				request('/v1/signin', sessionResponseSchema, {
					method: 'POST',
					body: input,
				}),
			),

		signout: () =>
			request('/v1/signout', okResponseSchema, { method: 'POST' }),

		session: async () => {
			const r = await request('/v1/session', sessionNullableResponseSchema, { method: 'GET' });
			if (!r.ok) return r;
			return { ok: true, data: r.data.session };
		},

		sessionRefresh: () =>
			unwrapSession(
				request('/v1/session/refresh', sessionResponseSchema, { method: 'POST' }),
			),

		listSessions: () =>
			request('/v1/sessions', sessionsListResponseSchema, { method: 'GET' }),

		revokeSession: (id) =>
			request(`/v1/sessions/${encodeURIComponent(id)}`, okResponseSchema, {
				method: 'DELETE',
			}),

		passwordResetRequest: (input) =>
			request('/v1/password/reset/request', okResponseSchema, {
				method: 'POST',
				body: input,
			}),

		passwordResetConfirm: (input) =>
			request('/v1/password/reset/confirm', okResponseSchema, {
				method: 'POST',
				body: input,
			}),

		emailVerifyRequest: () =>
			request('/v1/email/verify/request', okResponseSchema, { method: 'POST' }),

		emailVerifyConfirm: (input) =>
			request('/v1/email/verify/confirm', okResponseSchema, {
				method: 'POST',
				body: input,
			}),

		oauthAuthorizeUrl: async (provider, redirect) => {
			const query: Record<string, string> = {};
			if (redirect !== undefined) query.redirect = redirect;
			const r = await request(
				`/v1/oauth/${encodeURIComponent(provider)}/authorize`,
				oauthAuthorizeResponseSchema,
				{ method: 'GET', query },
			);
			if (!r.ok) return r;
			return { ok: true, data: { url: r.data.url } };
		},

		passkeyRegisterBegin: () =>
			request('/v1/passkey/register/begin', passkeyRegisterBeginResponseSchema, {
				method: 'POST',
			}),

		passkeyRegisterFinish: (response) =>
			request('/v1/passkey/register/finish', passkeyRegisterFinishResponseSchema, {
				method: 'POST',
				body: { response },
			}),

		passkeyAuthenticateBegin: (email) =>
			request('/v1/passkey/authenticate/begin', passkeyAuthenticateBeginResponseSchema, {
				method: 'POST',
				body: email !== undefined ? { email } : {},
			}),

		passkeyAuthenticateFinish: (response) =>
			unwrapSession(
				request('/v1/passkey/authenticate/finish', passkeyAuthenticateFinishResponseSchema, {
					method: 'POST',
					body: { response },
				}),
			),

		listPasskeys: () =>
			request('/v1/passkey', passkeyListResponseSchema, { method: 'GET' }),

		revokePasskey: (id) =>
			request(`/v1/passkey/${encodeURIComponent(id)}`, okResponseSchema, {
				method: 'DELETE',
			}),

		totpSetup: () =>
			request('/v1/totp/setup', totpSetupResponseSchema, { method: 'POST' }),

		totpConfirm: (input) =>
			request('/v1/totp/confirm', okResponseSchema, {
				method: 'POST',
				body: input,
			}),

		totpVerify: (input) =>
			request('/v1/totp/verify', okResponseSchema, {
				method: 'POST',
				body: input,
			}),

		totpDisable: () =>
			request('/v1/totp/disable', okResponseSchema, { method: 'POST' }),

		magicRequest: (input) =>
			request('/v1/magic/request', magicRequestResponseSchema, {
				method: 'POST',
				body: input,
			}),

		magicVerifyUrl: (token: string) =>
			`${joinUrl(config.serverUrl, '/v1/magic/verify')}?token=${encodeURIComponent(token)}`,

		linkAccount: async (provider, redirect) => {
			const query: Record<string, string> = {};
			if (redirect !== undefined) query.redirect = redirect;
			return request(
				`/v1/account/link/${encodeURIComponent(provider)}`,
				oauthAuthorizeResponseSchema,
				{ method: 'POST', query },
			);
		},

		unlinkAccount: (provider) =>
			request(`/v1/account/${encodeURIComponent(provider)}`, okResponseSchema, {
				method: 'DELETE',
			}),

		listOrgs: () =>
			request('/v1/orgs', organizationListResponseSchema, { method: 'GET' }),

		createOrg: (input) =>
			unwrapOrg(
				request('/v1/orgs', z.object({ org: organizationSchema }), {
					method: 'POST',
					body: input,
				}),
			),

		inviteMember: (orgId, input) =>
			request(`/v1/orgs/${encodeURIComponent(orgId)}/invite`, okResponseSchema, {
				method: 'POST',
				body: input,
			}),

		acceptInvite: (orgId, input) =>
			request(`/v1/orgs/${encodeURIComponent(orgId)}/accept`, okResponseSchema, {
				method: 'POST',
				body: input,
			}),
	};
}
