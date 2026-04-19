---
title: Quickstart
description: Scaffold Revo-Auth into a SvelteKit app, set your environment variables, and sign in — all in under five minutes.
---

You will need Node 22, pnpm, and a running Revo-Auth server (either self-hosted or the hosted dev instance at `https://auth.revo-auth.dev`). Have your app's URL ready.

## 1. Scaffold with the CLI

From the root of your SvelteKit app, run:

```sh
pnpm dlx @revo-auth/cli init
```

The CLI will:

- Detect your SvelteKit version and adapter.
- Install `@revo-auth/sdk-sveltekit` and `@revo-auth/ui-sveltekit`.
- Write `src/hooks.server.ts` wired to `handleAuth`.
- Add `src/lib/auth.ts` exporting a typed client.
- Append the required env keys to `.env.example`.
- Create `src/routes/login/+page.svelte` using the UI primitives.

Answer the three prompts: your app name, your Revo-Auth server URL, and which providers you want enabled.

## 2. Set environment variables

Open `.env` and fill in the values the CLI wrote as placeholders:

```sh
REVO_AUTH_URL="https://auth.revo-auth.dev"
REVO_AUTH_APP_ID="app_01HX0YOURAPPID"
REVO_AUTH_APP_SECRET="sk_live_..."      # server-side only, never expose
PUBLIC_REVO_AUTH_URL="https://auth.revo-auth.dev"
PUBLIC_REVO_AUTH_APP_ID="app_01HX0YOURAPPID"
```

`REVO_AUTH_APP_ID` maps your app to a row in the server's `apps` table. The secret authenticates server-to-server calls. The `PUBLIC_` copies are safe in the browser — they carry no authority beyond identifying your tenant.

Register your dev origin with the server so browser requests survive origin validation:

```sh
pnpm dlx @revo-auth/cli add origin http://localhost:5173
```

## 3. Log in

Start the dev server:

```sh
pnpm dev
```

Open `http://localhost:5173/login`, click the provider button you enabled, complete the OAuth dance, and you will land back on `/` with a `__Host-revo_session` cookie set. The session is already revocable from the Revo-Auth dashboard.

To verify server-side, add this to any `+page.server.ts`:

```ts
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ locals }) => {
  return { user: locals.user };
};
```

`locals.user` is populated by the hook when the cookie is valid and `null` otherwise. That is the end of the quickstart — you have auth.

## Where next

- [Protect routes](/sveltekit/load-guards/) with `requireAuth`.
- [Add passkeys](/concepts/passkeys/) so users can drop passwords entirely.
- [Deploy](/deployment/fly-io/) your own server instead of using the hosted dev instance.
