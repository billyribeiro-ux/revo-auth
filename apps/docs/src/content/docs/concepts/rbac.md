---
title: Role-based access control
description: Roles, permissions, and the helpers that enforce them across load functions and endpoints.
---

Revo-Auth ships a minimal, predictable RBAC model. Roles aggregate permissions; users hold zero or more roles within an app; the SDK exposes helpers that fail closed when a required permission is missing.

## The data model

Three tables, all scoped to `app_id`:

- `roles` — `{ id, app_id, name, description, permissions[] }`. Permissions are strings like `"billing:read"` or `"posts:delete"`. Naming is convention, not enforced schema — pick a style and stick to it.
- `user_roles` — `{ user_id, role_id }`. Many-to-many.
- `role_audit` — append-only log of grants and revocations, referenced by the audit log.

Two roles exist by default on every app: `owner` (all permissions, undeletable) and `member` (no permissions, assigned to every new user unless you override in the dashboard). You create additional roles in the dashboard or programmatically via the server API.

## Checking permissions

In a load function:

```ts
import { requirePermission } from "@revo-auth/sdk-sveltekit";

export const load = async (event) => {
  await requirePermission(event, "billing:read");
  // ... safe to render billing data
};
```

If `locals.user` is null, `requirePermission` redirects to the login page. If the user is authenticated but lacks the permission, it throws a `403`. You never get to write the unsafe happy-path branch by accident.

For conditional UI:

```ts
import { hasPermission } from "@revo-auth/sdk-sveltekit";

export const load = async (event) => {
  return {
    canInvite: await hasPermission(event, "team:invite"),
  };
};
```

`hasPermission` returns a boolean and never throws — useful for showing or hiding a button without gating the whole page.

## Wildcards and scopes

The check is exact-string. Revo-Auth does not do `"billing:*"` matching because wildcards are a well-known foot-gun — a junior dev adds `"billing:delete_all"` years later and the wildcard silently grants it.

If you need scoped permissions (e.g. "can edit posts in project X"), model the scope in your own application database. RBAC in Revo-Auth answers "is this user allowed to attempt this action?" — the resource-level check is yours.

## Auditing

Every `requirePermission` failure is logged to the audit table with the user ID, the attempted permission, and the route. `role_audit` captures grants and revocations. Both are queryable from the dashboard and exportable to S3 as JSONL.

## Performance

User → roles → permissions is denormalized into a single JSONB column on `users` that the server refreshes whenever a role changes. Permission checks are a single row fetch from Postgres (cached in the session's in-process memo for the lifetime of the request), so there is no per-check round-trip.
