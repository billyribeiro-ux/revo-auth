---
title: SSR vs CSR
description: Where each half of your SvelteKit app runs, what it can access, and how Revo-Auth hydrates both.
---

SvelteKit renders on the server for the first load and on the client for subsequent navigations. Revo-Auth is designed to make that split invisible for auth concerns — but the split is real, and knowing where your code is running saves you from a class of subtle bugs.

## The two worlds

| Concern | SSR (`+page.server.ts`, `+layout.server.ts`, hooks) | CSR (`.svelte`, `+page.ts`, `+layout.ts`) |
|---|---|---|
| Has `locals` | Yes | No |
| Can read cookies directly | Yes (`event.cookies`) | No (use `document.cookie` — but don't, `__Host-` cookies are HttpOnly) |
| Can call Revo-Auth with app secret | Yes | No — the secret never leaves the server |
| Runs on every request | Yes | Only on client-side navigation |
| Runs at build time (prerender) | Yes | Yes |

The app secret is the bright line. Anything that uses it must run in a `.server.ts` file. The SDK enforces this with module-level imports: `@revo-auth/sdk-sveltekit/server` errors at build time if imported from a `.svelte` or non-`.server.ts` file.

## The hydration handoff

1. Browser requests `/dashboard`. SvelteKit runs hooks, `handleAuth` populates `locals`, `+layout.server.ts` returns `{ user }`, `+page.server.ts` returns page data.
2. The server renders HTML and serializes the loaded data into a `<script>` tag.
3. The browser receives HTML, parses it, and runs hydration. The client `+layout.ts` (if any) runs with the serialized `data`, then components mount.
4. `setSessionContext` reads `data.user` and seeds the runes store. `useSession()` in components sees the same user the server saw.

No second round-trip. The session hydrates from the same SSR pass that rendered the page.

## Client-side navigation

On `<a>` clicks (intercepted by SvelteKit's client router) or `goto()` calls:

1. Client runs load functions — the server ones via a `fetch` to the same page's `__data.json` endpoint, client ones locally.
2. Returned data flows into the layout/page, and the session store updates if `data.user` changed.

The `__data.json` fetch carries the same cookie the original request did, so `handleAuth` runs on the server and the session is re-verified for every navigation. This is the correct trade-off: slightly more network, impossible to miss a revocation.

## Disabling SSR

Some pages don't want SSR (e.g. a passkey registration page that's useless until WebAuthn APIs are available). Disable it with:

```ts
// src/routes/settings/passkey/+page.ts
export const ssr = false;
```

Revo-Auth still works — `+layout.server.ts` ran during the navigation and seeded `data.user`. The page just doesn't render its own HTML until the client mounts.

## Prerendering

Pages with `export const prerender = true` run their server loads at build time. Revo-Auth's `handleAuth` short-circuits during prerender (there's no cookie, no session), so `locals.user === null`. Don't prerender pages that gate on `requireAuth` — the build will error. Public marketing pages are fine.
