import type { RevoAuthError } from './errors.js';

export interface RevoAuthUser {
	id: string;
	email: string | null;
	emailVerified: boolean;
	name: string | null;
	image: string | null;
	customFields: Record<string, unknown>;
	createdAt: string;
}

export interface RevoAuthSession {
	id: string;
	userId: string;
	expiresAt: string;
	user: RevoAuthUser;
}

export interface RevoAuthLogger {
	debug(msg: string, meta?: Record<string, unknown>): void;
	info(msg: string, meta?: Record<string, unknown>): void;
	warn(msg: string, meta?: Record<string, unknown>): void;
	error(msg: string, meta?: Record<string, unknown>): void;
}

export interface RevoAuthConfig {
	serverUrl: string;
	appId: string;
	publicKey: string;
	fetchFn?: typeof fetch;
	logger?: RevoAuthLogger;
	/**
	 * When set, the SDK will attach this value as the session cookie on outgoing
	 * requests (used by the SvelteKit SSR adapter). In the browser, cookies are
	 * handled by the user agent automatically.
	 */
	cookie?: string;
	/**
	 * Optional CSRF token override (used server-side when the cookie jar is not
	 * accessible via `document.cookie`).
	 */
	csrfToken?: string;
}

export type Result<T, E = RevoAuthError> =
	| { ok: true; data: T }
	| { ok: false; error: E };

export interface OkResponse {
	ok: true;
}

export interface ListResponse<T> {
	items: T[];
}

export type OAuthProvider =
	| 'google'
	| 'github'
	| 'microsoft'
	| 'apple'
	| 'discord'
	| (string & {});

export interface SignupInput {
	email: string;
	password: string;
	name?: string;
	customFields?: Record<string, unknown>;
}

export interface SigninInput {
	email: string;
	password: string;
	totpCode?: string;
}

export interface PasswordResetRequestInput {
	email: string;
}

export interface PasswordResetConfirmInput {
	token: string;
	password: string;
}

export interface EmailVerifyConfirmInput {
	token: string;
}

export interface MagicRequestInput {
	email: string;
	redirect?: string;
}

export interface TotpVerifyInput {
	code: string;
}

export interface TotpConfirmInput {
	code: string;
}

export interface CreateOrgInput {
	name: string;
	slug?: string;
}

export interface InviteMemberInput {
	email: string;
	role: 'admin' | 'member' | 'viewer';
}

export interface AcceptInviteInput {
	token: string;
}

export interface Passkey {
	id: string;
	createdAt: string;
	lastUsedAt: string | null;
	name: string | null;
}

export interface Organization {
	id: string;
	name: string;
	slug: string;
	role: 'owner' | 'admin' | 'member' | 'viewer';
}

export interface TotpSetupResult {
	secret: string;
	otpauthUrl: string;
	qrCodeDataUrl: string;
}
