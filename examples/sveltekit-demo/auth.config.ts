import { env } from '$env/dynamic/public';
import type { RevoAuthConfig } from '@revo-auth/sdk-core';

/**
 * Public Revo-Auth config (safe to ship to the browser). Uses `PUBLIC_`
 * prefixed env vars per SvelteKit's public-env contract.
 *
 * User-owned file: the CLI will only merge new keys with defaults; values
 * stay put on subsequent `revo-auth update` runs.
 */
export const publicAuthConfig = {
	serverUrl: env.PUBLIC_REVO_AUTH_SERVER_URL ?? 'http://localhost:8787',
	appId: env.PUBLIC_REVO_AUTH_APP_ID ?? 'demo-app',
	publicKey: env.PUBLIC_REVO_AUTH_PUBLIC_KEY ?? 'pk_demo_replace_me'
} satisfies RevoAuthConfig;

/** Alias for server-side code that only needs the public subset. */
export const authConfig = publicAuthConfig;
