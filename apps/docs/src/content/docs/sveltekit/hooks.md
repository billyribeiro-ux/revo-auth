---
title: Hooks
description: Install handleAuth in your server hooks to populate locals on every request.
---

`handleAuth` is the single hook that bridges SvelteKit and Revo-Auth. It reads the `__Host-revo_session` cookie, verifies it against the server, and attaches `locals.user` and `locals.session` for the rest of the request.

## Minimal setup

```ts
// src/hooks.server.ts
import { handleAuth } from "@revo-auth/sdk-sveltekit/server";

export const handle = handleAuth({
  url: process.env.REVO_AUTH_URL!,
  appId: process.env.REVO_AUTH_APP_ID!,
  appSecret: process.env.REVO_AUTH_APP_SECRET!,
});
```

That's it. On every request, `event.locals.user` is either a `User` or `null`, and `event.locals.session` is either a `Session` or `null`. They are populated together — if one is defined, both are. There is no "session without user" state to worry about.

## Composing with your own hooks

SvelteKit's `sequence` composes hooks in order:

```ts
import { sequence } from "@sveltejs/kit/hooks";
import { handleAuth } from "@revo-auth/sdk-sveltekit/server";
import { handleLogging } from "$lib/server/logging";

export const handle = sequence(
  handleAuth({ /* ... */ }),
  handleLogging,
);
```

Put `handleAuth` first so downstream hooks see `locals.user`.

## Caching

`handleAuth` caches the session → user resolution in-process for the lifetime of the request. Multiple load functions can read `locals.user` without causing extra server round-trips. Across requests, Revo-Auth validates every cookie — we do not trust an in-memory session cache between requests because sessions can be revoked at any time.

If you need to skip the server round-trip entirely (e.g. on a public marketing page), use the `skip` option:

```ts
handleAuth({
  /* ... */
  skip: ({ url }) => url.pathname.startsWith("/marketing/"),
});
```

Skipped requests get `locals.user = null` without any network call.

## Error handling

If Revo-Auth is unreachable, `handleAuth` throws a `503` by default — the request fails closed. Override with `onError` if you have a different policy (e.g. serve a stale cached session for a bounded window):

```ts
handleAuth({
  /* ... */
  onError: (err, event) => {
    console.error("auth unreachable", err);
    event.locals.user = null;
    event.locals.session = null;
  },
});
```

Be deliberate — returning `null` on errors is equivalent to "log everyone out when the auth server is down," which is often the wrong choice.

## Typing `locals`

Add to `src/app.d.ts`:

```ts
import type { User, Session } from "@revo-auth/sdk-sveltekit";

declare global {
  namespace App {
    interface Locals {
      user: User | null;
      session: Session | null;
    }
  }
}

export {};
```

The CLI's `init` command writes this automatically.
