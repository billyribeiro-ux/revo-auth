# sveltekit-demo

End-to-end proof of the Revo-Auth stack. Exercises signup, signin, magic
links, passkeys, TOTP, OAuth linking, session management, and email
verification via `@revo-auth/sdk-sveltekit` and `@revo-auth/ui-sveltekit`.

## Prerequisites

- Node 22 (see `.nvmrc` at repo root)
- pnpm 10
- Docker (to run the Revo-Auth server locally for E2E)

## Build order (monorepo)

Because this app depends on workspace packages via `workspace:*`, build them
first the first time you clone:

```sh
pnpm install --no-frozen-lockfile
pnpm -F @revo-auth/sdk-core build
pnpm -F @revo-auth/ui-sveltekit build
pnpm -F @revo-auth/sdk-sveltekit build
pnpm -F sveltekit-demo dev
```

## Running E2E tests

```sh
# In one terminal, boot the auth server:
docker run --rm -p 8787:8787 ghcr.io/revo-auth/server:latest

# In another terminal:
pnpm -F sveltekit-demo e2e
```

Tests automatically `test.skip` if the auth server is unreachable, so they
never fail hard in that environment.

## Layout

- `src/hooks.server.ts` — wires `handleAuth` from the SvelteKit adapter.
- `auth.config.ts` — user-owned config; CLI only merges new keys.
- `src/lib/auth/{client,server}.ts` — factory helpers for browser / SSR clients.
- `src/routes/auth/callback/+server.ts` — OAuth return URL proxy.
- `src/routes/magic/+server.ts` — magic-link verify proxy.
- `tests/e2e/*.spec.ts` — Playwright scenarios.
