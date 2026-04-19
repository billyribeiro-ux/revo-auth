export { getCachedSession, handleAuth, sessionCacheKey } from './hooks.js';
export type { AuthHookOptions } from './hooks.js';
export { optionalAuth, requireAuth } from './load.js';
export type { RequireAuthOptions } from './load.js';
export { requireOrgRole } from './guards.js';
export type { OrgMembership, OrgRole } from './guards.js';
export { createServerClient } from './server.js';
export type { ServerClientOptions } from './server.js';

// Re-export the core client factory for convenience.
export { createClient as createAuthClient } from '@revo-auth/sdk-core';
export type {
	ClientAPI,
	RevoAuthConfig,
	RevoAuthSession,
	RevoAuthUser,
	Result,
} from '@revo-auth/sdk-core';
