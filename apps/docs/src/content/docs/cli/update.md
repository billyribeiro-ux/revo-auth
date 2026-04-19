---
title: revo-auth update
description: Keep the SDK, UI, and CLI versions aligned with your Revo-Auth server.
---

`update` bumps `@revo-auth/*` packages in your project to versions compatible with the server you're pointing at. It's the safer alternative to `pnpm update` — the CLI knows the compatibility matrix, `pnpm update` doesn't.

## Usage

```sh
pnpm dlx @revo-auth/cli update
```

The command:

1. Fetches the server's `/v1/compatibility` endpoint for the range of SDK/CLI versions it supports.
2. Compares against your `package.json`.
3. Proposes a diff.
4. Applies the diff (on confirmation) and runs `pnpm install`.

If the server is ahead of your SDK and a breaking version is required, you'll see a migration note with links to the relevant changesets. Accept to proceed; `update` won't mutate your code to match the new API — that remains your job.

## Updating UI components

UI components installed via `revo-auth ui add` live inside your source tree. They don't get updated automatically. Re-run `revo-auth ui add <Component>` to pull the latest version; the CLI will show a diff and ask before overwriting your local copy.

## Flags

- `--dry-run` — show the proposed diff and exit without writing or installing.
- `--yes` — apply without confirmation. For CI workflows.
- `--channel <name>` — pull from `next` or `beta` channels instead of `latest`. Prereleases only; default is `latest`.

## When to run it

- After upgrading your self-hosted Revo-Auth server.
- Before cutting a release of your app, to pick up security patches.
- When the dashboard shows a "new SDK version available" banner.

Running `update` when you're on the right versions is a no-op — the command exits cleanly with "everything is up to date."
