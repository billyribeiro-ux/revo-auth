---
title: revo-auth init
description: Scaffold Revo-Auth into a SvelteKit app in one command.
---

`init` is the first command you run in any new project. It detects your SvelteKit version and adapter, installs the SDK and UI packages, writes hooks and a login page, and seeds your `.env.example` with the keys Revo-Auth needs.

## Usage

```sh
pnpm dlx @revo-auth/cli init
```

Run it from the root of a SvelteKit project (the one containing `svelte.config.js`). The CLI refuses to run if it can't detect SvelteKit, and it refuses to overwrite existing Revo-Auth files unless you pass `--force`.

## What it prompts for

1. **App name.** Shown in the dashboard. Defaults to your `package.json` `name`.
2. **Server URL.** The Revo-Auth instance to point at. Defaults to the hosted dev instance `https://auth.revo-auth.dev` — fine for prototyping, swap for your own for production.
3. **Providers.** Multi-select. Options: Google, GitHub, Microsoft, Discord, Apple, Magic Link, Passkey, TOTP. Picking providers only seeds the env file and login UI — you still configure the provider side per the [provider guides](/providers/).

## What it writes

- `src/hooks.server.ts` — `handleAuth` wired to env vars.
- `src/lib/auth.ts` — exports a typed `sdk` plus `requireAuth`/`requirePermission`/`requireStepUp`.
- `src/app.d.ts` — adds `App.Locals` typing for `user` and `session`.
- `src/routes/login/+page.svelte` — a login page using `@revo-auth/ui-sveltekit`.
- `src/routes/auth/callback/+server.ts` — OAuth callback receiver (only if an OAuth provider was picked).
- `src/routes/+layout.server.ts` — returns `{ user }` for the client session store.
- `src/routes/+layout.svelte` — calls `setSessionContext`.
- `.env.example` — appended with `REVO_AUTH_*` keys.
- `revo-auth.json` — CLI config for subsequent commands.

## What it installs

`init` runs your package manager to install:

- `@revo-auth/sdk-sveltekit`
- `@revo-auth/ui-sveltekit`

Both go into `dependencies`, not `devDependencies` — they ship to production.

## After running

1. Copy `.env.example` to `.env` and fill the secret values.
2. Run `pnpm dlx @revo-auth/cli add origin http://localhost:5173`.
3. `pnpm dev` and open `/login`.

That path is covered in depth in the [quickstart](/quickstart/).

## Flags

- `--force` — overwrite existing Revo-Auth files. Use with a clean git working tree.
- `--adapter <name>` — force a specific SvelteKit adapter. Rarely needed; detection is reliable.
- `--no-install` — write files but skip `pnpm install`. Useful in monorepos where you batch installs.
- `--yes` — accept all defaults, no prompts. For CI scaffolding.
