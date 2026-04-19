---
title: CLI overview
description: The @revo-auth/cli handles scaffolding, package installation, local dev, and health checks.
---

`@revo-auth/cli` is how you touch Revo-Auth from the terminal. Scaffolding a new integration, adding a provider, pulling UI components into your project, running a local auth server during development — it's one binary and a handful of verbs.

## Running the CLI

You don't install the CLI globally. Run it with `pnpm dlx`:

```sh
pnpm dlx @revo-auth/cli <command>
```

That always pulls the latest compatible version for your SvelteKit major. If you prefer a pinned version, add `@revo-auth/cli` to your app's `devDependencies` and call it via `pnpm exec revo-auth`.

## Command reference

- [`init`](/cli/init/) — one-time scaffold for a new SvelteKit app. Writes hooks, env, and a login page.
- [`add`](/cli/add/) — add a provider, origin, or feature to an existing install.
- [`update`](/cli/update/) — bump SDK, UI, and CLI versions in lockstep with your server.
- [`ui`](/cli/ui/) — copy UI primitives from `@revo-auth/ui-sveltekit` into your project for customization.
- [`dev`](/cli/dev/) — run a local Revo-Auth server against SQLite + an in-process Redis stand-in.
- [`doctor`](/cli/doctor/) — diagnostic checks: env, server reachability, cookie domains, origin whitelist.
- [`logout`](/cli/logout/) — clear CLI-side credentials (the dashboard API token, not user sessions).

## Auth for the CLI itself

Some commands (e.g. adding an OAuth provider through the dashboard API) need a CLI-side token. The first time you run such a command, the CLI opens a browser window and walks you through OAuth against your dashboard. The resulting token lives in `~/.config/revo-auth/credentials.json` (or the OS keychain on macOS), scoped to the apps you manage.

Use [`logout`](/cli/logout/) to wipe it.

## Configuration file

The CLI looks for `revo-auth.json` in your project root. It's written by `init` and read by every other command. A typical file:

```json
{
  "schemaVersion": 1,
  "serverUrl": "https://auth.revo-auth.dev",
  "appId": "app_01HX0YOURAPPID",
  "adapter": "sveltekit",
  "providers": ["google", "github"],
  "uiComponents": []
}
```

Commit it. It carries no secrets — the app secret lives in `.env`.
