---
title: revo-auth dev
description: Run a local Revo-Auth server in-process for frictionless offline development.
---

`dev` spins up a full Revo-Auth server on your machine, backed by SQLite and an in-process Redis stand-in, so you can develop against auth without running the hosted server, Docker, or any external services.

## Usage

```sh
pnpm dlx @revo-auth/cli dev
```

On first run, the CLI downloads the appropriate server binary for your platform (cached in `~/.cache/revo-auth/bin/`). Subsequent runs boot in under a second.

Output:

```
revo-auth dev
  server:    http://localhost:4000
  dashboard: http://localhost:4000/admin
  app:       app_dev_local (auto-created)
  db:        .revo-auth/dev.sqlite
```

Leave it running in a terminal tab. Your SvelteKit app's `REVO_AUTH_URL` should point at `http://localhost:4000`.

## What's different from production

- **SQLite instead of Postgres.** Migrations applied on boot. Schema is identical.
- **In-process revocation set.** No Redis binary. Functionally equivalent for a single-process dev server; explicitly not suitable for anything else.
- **Auto-created dev app.** No manual dashboard registration needed. The app ID and secret are printed once on first boot and persisted to `.revo-auth/dev.sqlite`.
- **Mailpit for email.** Magic links and verification emails go to a local Mailpit instance at `http://localhost:8025`. The CLI starts Mailpit automatically if it's installed; otherwise it prints the URLs to stdout.
- **OAuth providers are mocked.** The "Sign in with Google" button lets you pick a fake account from a list. Real provider keys still work if you set them in `.revo-auth/dev.env`.

## Persistence

State survives restarts — the SQLite file and the dev app's secret are checked into `.revo-auth/` (which the CLI adds to your `.gitignore` on first run). To start fresh:

```sh
pnpm dlx @revo-auth/cli dev --reset
```

That deletes `.revo-auth/dev.sqlite` and re-creates a new dev app on next boot.

## Flags

- `--port <n>` — default 4000.
- `--reset` — delete local state first.
- `--log-level <level>` — `trace`, `debug`, `info` (default), `warn`, `error`.
- `--no-mailpit` — skip starting Mailpit even if it's installed.

## When to use the hosted dev server instead

`https://auth.revo-auth.dev` is a shared sandbox that always runs the latest server. Use it when:

- You want to test with real OAuth providers without configuring them locally.
- You're collaborating with someone on a mobile device that can't reach your laptop.
- You're debugging an SDK version against the exact server build that's in production.

Otherwise prefer `revo-auth dev` — it's faster, offline, and doesn't make you share state with strangers.
