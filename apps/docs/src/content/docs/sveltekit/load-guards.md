---
title: Load guards
description: requireAuth, requirePermission, and requireStepUp — failing closed in load functions and form actions.
---

Load guards are the opinionated way to protect routes in Revo-Auth. Each one takes the `RequestEvent` and either returns normally (letting the load continue) or throws a redirect/error. You never write the "if not authed then redirect" branch by hand.

## `requireAuth`

Use on any page that needs a signed-in user.

```ts
// src/routes/dashboard/+page.server.ts
import { requireAuth } from "@revo-auth/sdk-sveltekit";

export const load = async (event) => {
  const user = await requireAuth(event);
  return { user };
};
```

Under the hood: if `locals.user` is null, `requireAuth` throws a SvelteKit `redirect(302, "/login?returnTo=...")`. The `returnTo` round-trips through the login page so the user lands where they were trying to go.

## `requirePermission`

Gate by RBAC permission:

```ts
import { requirePermission } from "@revo-auth/sdk-sveltekit";

export const load = async (event) => {
  await requirePermission(event, "billing:read");
  const invoices = await fetchInvoices(event.locals.user!.id);
  return { invoices };
};
```

If the user isn't signed in, it redirects to login (same as `requireAuth`). If they're signed in but missing the permission, it throws `error(403, "forbidden")`. SvelteKit renders `+error.svelte` with a `403` — your UI can show a friendly "you don't have access" page.

## `requireStepUp`

For sensitive actions. Reject sessions that haven't produced a fresh second factor within the window:

```ts
import { requireStepUp } from "@revo-auth/sdk-sveltekit";

export const load = async (event) => {
  await requireStepUp(event, { maxAge: 5 * 60 });
  // session.step_up_at is within 5 minutes — safe to render
};
```

Stale step-up state triggers a redirect to `/auth/step-up?returnTo=...`. The built-in step-up page accepts TOTP, passkey, or recovery code and returns the user to the original URL.

## In form actions

Guards work identically in actions:

```ts
export const actions = {
  delete: async (event) => {
    await requireStepUp(event, { maxAge: 60 });
    await deleteAccount(event.locals.user!.id);
    throw redirect(303, "/goodbye");
  },
};
```

The throw-redirect-from-load pattern composes naturally — SvelteKit catches the thrown response and handles it.

## Composing guards

Guards are plain `async` functions. Call them in sequence to enforce multiple conditions:

```ts
await requireAuth(event);
await requirePermission(event, "admin:write");
await requireStepUp(event, { maxAge: 60 });
```

Order matters only for the error you surface first. `requirePermission` implicitly calls `requireAuth`, so you can skip the first line when you're also checking a permission — we recommend leaving it in for readability.

## What guards do *not* do

They don't check resource ownership. "Is this user allowed to call this action on *this* invoice?" is a question about your domain data, not Revo-Auth's authentication. Write that check next to the query.
