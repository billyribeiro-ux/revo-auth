import { env } from '$env/dynamic/private';
import type { RevoAuthConfig } from '@revo-auth/sdk-core';

/**
 * Revo-Auth configuration shared between server hook and client factories.
 * User-owned: the CLI will only merge new keys with defaults; values stay put.
 */
export const authConfig = {
	serverUrl: env.REVO_AUTH_SERVER_URL ?? 'http://localhost:8787',
	appId: env.REVO_AUTH_APP_ID ?? 'demo-app',
	publicKey: env.REVO_AUTH_PUBLIC_KEY ?? 'pk_demo_replace_me'
} satisfies RevoAuthConfig;

/** Public config (safe to ship to the browser). */
export const publicAuthConfig = {
	serverUrl: authConfig.serverUrl,
	appId: authConfig.appId,
	publicKey: authConfig.publicKey
} satisfies RevoAuthConfig;
