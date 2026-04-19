import type { RevoAuthSession } from '@revo-auth/sdk-core';
import { error, type RequestEvent } from '@sveltejs/kit';
import { requireAuth } from './load.js';

export type OrgRole = 'owner' | 'admin' | 'member' | 'viewer';

const ROLE_RANK: Record<OrgRole, number> = {
	owner: 4,
	admin: 3,
	member: 2,
	viewer: 1,
};

export interface OrgMembership {
	orgId: string;
	role: OrgRole;
}

interface SessionWithOrgs extends RevoAuthSession {
	orgs?: OrgMembership[];
}

/**
 * Ensure the authenticated user has at least the given role in the given org.
 * Throws `error(403)` when the user is signed in but lacks the role.
 * Throws `redirect(302, loginPath)` when there is no session.
 */
export function requireOrgRole(
	event: RequestEvent,
	orgId: string,
	role: OrgRole,
): RevoAuthSession {
	const session = requireAuth(event);
	const orgs = (session as SessionWithOrgs).orgs ?? [];
	const membership = orgs.find((o) => o.orgId === orgId);
	if (membership === undefined) {
		throw error(403, 'Not a member of this organization');
	}
	const required = ROLE_RANK[role];
	const actual = ROLE_RANK[membership.role];
	if (actual < required) {
		throw error(403, 'Insufficient organization role');
	}
	return session;
}
