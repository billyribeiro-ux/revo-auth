---
title: revo-auth logout
description: Clear the CLI's stored dashboard credentials.
---

`logout` removes the dashboard API token the CLI uses for commands like `add provider` and `doctor`. It does not log any users out of your app — it's strictly about the CLI's own auth state.

## Usage

```sh
pnpm dlx @revo-auth/cli logout
```

The CLI deletes `~/.config/revo-auth/credentials.json` (or the matching keychain entry on macOS) and prints confirmation. Next time you run a command that needs dashboard access, you'll be prompted to re-authenticate.

## When to run it

- You're leaving a shared machine.
- You rotated your dashboard account's credentials.
- You switched from one Revo-Auth tenant to another and want a clean slate.
- The CLI is misbehaving and you suspect stale credentials.

## What it does not do

- It doesn't revoke the token server-side. That happens in the dashboard under Settings → API tokens. Prefer that flow if you suspect token compromise — local deletion alone leaves the token valid until its natural expiry.
- It doesn't clear sessions for your app's users. That's `sdk.session.revoke()` from within your app or the "Sign everyone out" button in the dashboard.

## Flags

- `--all` — remove credentials for every Revo-Auth server the CLI has ever authenticated with, not just the current `revo-auth.json` target.
