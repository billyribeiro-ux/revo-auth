import type { ClientAPI } from './client.js';
import type { OkResponse, Result } from './types.js';

export async function requestMagicLink(
	client: ClientAPI,
	email: string,
	redirect?: string,
): Promise<Result<OkResponse>> {
	const input: { email: string; redirect?: string } = { email };
	if (redirect !== undefined) input.redirect = redirect;
	return client.magicRequest(input);
}

export function magicVerifyUrl(client: ClientAPI, token: string): string {
	return client.magicVerifyUrl(token);
}
