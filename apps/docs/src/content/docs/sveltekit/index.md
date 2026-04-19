---
title: SvelteKit integration
description: How @revo-auth/sdk-sveltekit plugs into hooks, load functions, stores, and the SSR/CSR split.
---

`@revo-auth/sdk-sveltekit` is the primary way your application talks to Revo-Auth. It's a thin, typed layer over the transport SDK that understands SvelteKit's request lifecycle — hooks populate `locals`, load helpers gate pages and endpoints, and a runes-based store keeps the client in sync without prop-drilling.

## What the SDK gives you

- **`handleAuth`** — a `Handle` hook that reads the session cookie, verifies it against the server, and populates `event.locals.user` and `event.locals.session`.
- **`requireAuth`, `requirePermission`, `requireStepUp`** — load helpers that redirect or throw when the request doesn't meet the bar.
- **`createSessionStore`** — a runes-based store that exposes the current user reactively in client components.
- **`sdk`** — the typed transport client for the rare cases you need to call Revo-Auth directly (e.g. from a form action to register a passkey).

## Shape of a protected app

A typical app wires the SDK in three files:

1. `src/hooks.server.ts` — installs `handleAuth`.
2. `src/lib/auth.ts` — exports your configured `sdk` and re-exports the helpers.
3. `src/routes/+layout.server.ts` — returns `locals.user` so every page has it.

The CLI's `init` command writes all three for you. If you prefer to understand what's happening first:

- [Hooks](/sveltekit/hooks/) walks through `handleAuth` and how to compose it with your own.
- [Load guards](/sveltekit/load-guards/) covers `requireAuth`, `requirePermission`, and `requireStepUp`.
- [Session store](/sveltekit/session-store/) shows the runes-based client store.
- [SSR vs CSR](/sveltekit/ssr-vs-csr/) explains when each half runs and what each has access to.

## Versioning

The SDK follows SvelteKit's major versions. `@revo-auth/sdk-sveltekit@1.x` requires SvelteKit 2.x and Svelte 5 (runes). There is no Svelte 4 support — the store layer uses runes and the types assume `$state`/`$derived`.
