import {
	startAuthentication,
	startRegistration,
} from '@simplewebauthn/browser';
import type { ClientAPI } from './client.js';
import { UnknownError } from './errors.js';
import type { Passkey, Result, RevoAuthSession } from './types.js';

export interface RegisterPasskeyOptions {
	/** Optional human-readable name for the passkey. */
	name?: string;
}

export async function registerPasskey(
	client: ClientAPI,
	_opts: RegisterPasskeyOptions = {},
): Promise<Result<{ passkey: Passkey }>> {
	const begin = await client.passkeyRegisterBegin();
	if (!begin.ok) return begin;

	try {
		// The server's `publicKey` is the PublicKeyCredentialCreationOptionsJSON.
		const attestation = await startRegistration(
			begin.data.publicKey as unknown as Parameters<typeof startRegistration>[0],
		);
		return client.passkeyRegisterFinish(attestation as unknown as Record<string, unknown>);
	} catch (err) {
		return {
			ok: false,
			error: new UnknownError(
				err instanceof Error ? err.message : 'Passkey registration failed',
				err instanceof Error ? { cause: err } : undefined,
			),
		};
	}
}

export async function authenticateWithPasskey(
	client: ClientAPI,
	email?: string,
): Promise<Result<RevoAuthSession>> {
	const begin = await client.passkeyAuthenticateBegin(email);
	if (!begin.ok) return begin;

	try {
		const assertion = await startAuthentication(
			begin.data.publicKey as unknown as Parameters<typeof startAuthentication>[0],
		);
		return client.passkeyAuthenticateFinish(assertion as unknown as Record<string, unknown>);
	} catch (err) {
		return {
			ok: false,
			error: new UnknownError(
				err instanceof Error ? err.message : 'Passkey authentication failed',
				err instanceof Error ? { cause: err } : undefined,
			),
		};
	}
}
