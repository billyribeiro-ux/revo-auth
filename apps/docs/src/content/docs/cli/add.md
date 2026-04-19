---
title: revo-auth add
description: Add a provider, an allowed origin, or a feature to an existing Revo-Auth install.
---

`add` is the verb for extending a project that's already been `init`-ed. It handles three kinds of additions: OAuth providers, allowed origins, and optional features.

## Adding an OAuth provider

```sh
pnpm dlx @revo-auth/cli add provider google
```

This:

1. Hits the dashboard API and enables the provider for your app (you'll be asked for the provider's client ID and secret, or you can paste them later from the dashboard).
2. Adds the provider to the login page's button list.
3. Updates `revo-auth.json`.

Supported providers: `google`, `github`, `microsoft`, `discord`, `apple`. See [Providers](/providers/) for portal-side setup.

## Adding an origin

Browser requests are origin-validated. Before you can sign in from a new URL, register it:

```sh
pnpm dlx @revo-auth/cli add origin https://staging.example.com
```

This calls the server's admin API and appends to the app's `allowed_origins` array. Use `remove origin` to take it back off.

You'll add at least two during normal development:

```sh
pnpm dlx @revo-auth/cli add origin http://localhost:5173
pnpm dlx @revo-auth/cli add origin https://app.example.com
```

Exact string match — no wildcards. A port change is a new origin. HTTPS and HTTP are different origins.

## Adding a feature

Features are bit-flags on the app row:

```sh
pnpm dlx @revo-auth/cli add feature passkeys
pnpm dlx @revo-auth/cli add feature totp
pnpm dlx @revo-auth/cli add feature magic-links
```

Enabling a feature doesn't ship UI — it permits the server to accept requests for that feature. To get the UI, run `revo-auth ui add TotpEnroll` (or similar).

## Flags

- `--app <id>` — operate on a specific app ID when you have multiple. Defaults to the `appId` in `revo-auth.json`.
- `--yes` — skip confirmation prompts.
