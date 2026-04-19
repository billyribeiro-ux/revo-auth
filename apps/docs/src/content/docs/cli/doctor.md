---
title: revo-auth doctor
description: Diagnose misconfiguration before it shows up as a mysterious 401 in production.
---

`doctor` runs a battery of checks against your project and the Revo-Auth server it's pointed at. It's the first command to run when something's wrong and the first command to run before shipping.

## Usage

```sh
pnpm dlx @revo-auth/cli doctor
```

The output is a checklist. Each item is `PASS`, `WARN`, or `FAIL`. Failures exit with a non-zero code so you can gate CI on it.

## What it checks

### Environment

- `REVO_AUTH_URL` is set and resolves.
- `REVO_AUTH_APP_ID` is set and matches `revo-auth.json`.
- `REVO_AUTH_APP_SECRET` is set and authenticates against the server.
- Public env vars (`PUBLIC_REVO_AUTH_URL`, `PUBLIC_REVO_AUTH_APP_ID`) are set and match their private counterparts.

### Connectivity

- The server URL returns `200` from `/health`.
- Server version is compatible with the installed SDK (fetches `/v1/compatibility`).
- The app's `allowed_origins` list includes at least one origin.

### Project

- `src/hooks.server.ts` imports `handleAuth`.
- `src/app.d.ts` declares `App.Locals` with `user` and `session`.
- `@revo-auth/sdk-sveltekit` is in `dependencies` (not `devDependencies`).
- Node version is 22 or newer.

### Common footguns

- No file under `src/` imports `@revo-auth/sdk-sveltekit/server` from a non-`.server.ts` path.
- `tsconfig.json` has `"strict": true`.
- `.env` is gitignored.
- No leaked `REVO_AUTH_APP_SECRET` in committed files (greps your git history shallowly).

## Example failing output

```
revo-auth doctor
  [PASS] Environment variables
  [FAIL] Connectivity
    - Origin http://localhost:5173 is not in allowed_origins
      Fix: pnpm dlx @revo-auth/cli add origin http://localhost:5173
  [WARN] Project
    - Node version is 20.x; Revo-Auth requires 22+
  [PASS] Common footguns

1 failure, 1 warning.
```

Each FAIL includes a "Fix:" line with the exact command to run.

## In CI

```yaml
- run: pnpm dlx @revo-auth/cli doctor --ci
```

`--ci` format is machine-readable (line-delimited JSON) and skips interactive fixes. Pair with `continue-on-error: false` to fail the job on any `FAIL`.

## Flags

- `--ci` — JSONL output, no color.
- `--fix` — apply suggested fixes automatically where safe (e.g. add a missing origin). Won't touch anything that requires a secret.
- `--skip <check>` — disable a specific check by name.
