import type { ClientAPI } from './client.js';
import type { OkResponse, Result, TotpSetupResult } from './types.js';

export async function setupTotp(client: ClientAPI): Promise<Result<TotpSetupResult>> {
	return client.totpSetup();
}

export async function confirmTotp(client: ClientAPI, code: string): Promise<Result<OkResponse>> {
	return client.totpConfirm({ code });
}

export async function verifyTotp(client: ClientAPI, code: string): Promise<Result<OkResponse>> {
	return client.totpVerify({ code });
}

export async function disableTotp(client: ClientAPI): Promise<Result<OkResponse>> {
	return client.totpDisable();
}
