---
title: Secrets and environment
description: The complete list of environment variables Revo-Auth reads, grouped by concern.
---

Every Revo-Auth server reads its configuration from environment variables. This page is the exhaustive reference. For hosting-specific "how do I set these" commands, see the Fly and Docker guides.

## Required

| Variable | Purpose |
|---|---|
| `DATABASE_URL` | Postgres connection string. Must include `sslmode=require` in production. |
| `REDIS_URL` | Redis connection string. Must be `rediss://` (TLS) in production. |
| `REVO_AUTH_SIGNING_KEY` | 32-byte hex string. Used to sign internal tokens (password reset, email verification, step-up challenges). Generate with `openssl rand -hex 32`. Rotating invalidates outstanding reset links. |
| `REVO_AUTH_PUBLIC_URL` | The public base URL of the server, used in OAuth redirect URIs and emails. No trailing slash. |

## Cookies

| Variable | Default | Purpose |
|---|---|---|
| `REVO_AUTH_COOKIE_DOMAIN` | `""` (host-only) | Set to a parent domain like `.example.com` to share the cookie across subdomains. Leave empty to scope to the exact host. |
| `REVO_AUTH_SESSION_TTL_SECONDS` | `1209600` (14 days) | Absolute session expiry ceiling. Rolling extension happens inside this window. |

## OAuth providers

Set per provider, and only for providers you've enabled. See the individual [provider guides](/providers/) for portal-side steps.

- **Google:** `REVO_AUTH_GOOGLE_CLIENT_ID`, `REVO_AUTH_GOOGLE_CLIENT_SECRET`
- **GitHub:** `REVO_AUTH_GITHUB_CLIENT_ID`, `REVO_AUTH_GITHUB_CLIENT_SECRET`
- **Microsoft:** `REVO_AUTH_MICROSOFT_CLIENT_ID`, `REVO_AUTH_MICROSOFT_CLIENT_SECRET`, `REVO_AUTH_MICROSOFT_TENANT`
- **Discord:** `REVO_AUTH_DISCORD_CLIENT_ID`, `REVO_AUTH_DISCORD_CLIENT_SECRET`
- **Apple:** `REVO_AUTH_APPLE_CLIENT_ID`, `REVO_AUTH_APPLE_TEAM_ID`, `REVO_AUTH_APPLE_KEY_ID`, `REVO_AUTH_APPLE_PRIVATE_KEY`

## Email

Revo-Auth sends email for verification, magic links, and password reset. Pick exactly one transport.

| Variable | Purpose |
|---|---|
| `REVO_AUTH_SMTP_URL` | Full SMTP URL including auth: `smtps://user:pass@mail.example.com:465`. |
| `REVO_AUTH_POSTMARK_TOKEN` | Postmark server token. |
| `REVO_AUTH_RESEND_API_KEY` | Resend API key. |
| `REVO_AUTH_EMAIL_FROM` | Always required if any transport is set. `"Revo-Auth <auth@example.com>"`. |

## Administration

| Variable | Default | Purpose |
|---|---|---|
| `REVO_AUTH_ADMIN_EMAIL` | — | Email of the first admin user. Created automatically on first boot if no admins exist. |
| `REVO_AUTH_AUTO_MIGRATE` | `true` | Run migrations on boot. Set `false` in production if you run `revo-auth-server migrate` out-of-band. |
| `REVO_AUTH_LOG_LEVEL` | `info` | `trace`, `debug`, `info`, `warn`, `error`. |
| `REVO_AUTH_LOG_FORMAT` | `json` | `json` or `pretty`. Use `json` everywhere that isn't a local terminal. |

## Observability

| Variable | Purpose |
|---|---|
| `REVO_AUTH_METRICS_AUTH_HEADER` | If set, `/metrics` requires `Authorization: Bearer <this value>`. Leave unset and firewall the port to localhost if you prefer network-level gating. |
| `REVO_AUTH_TRACES_OTLP_ENDPOINT` | OpenTelemetry collector URL. When set, the server exports spans via OTLP/HTTP. |
| `REVO_AUTH_TRACES_SAMPLE_RATE` | `0.0` – `1.0`. Default `0.1` (10%). |

## Performance

| Variable | Default | Purpose |
|---|---|---|
| `REVO_AUTH_DB_POOL_MAX` | `25` | Postgres connection pool size per server instance. |
| `REVO_AUTH_HTTP_TIMEOUT_SECONDS` | `30` | Timeout for outbound calls (OAuth providers, webhooks). |
| `REVO_AUTH_REQUEST_BODY_LIMIT` | `1048576` (1MB) | Max incoming body size. WebAuthn attestations fit comfortably; raise only if you have a specific reason. |

## Rotation

Treat `REVO_AUTH_SIGNING_KEY` as rotatable. The server accepts two valid keys at once — set `REVO_AUTH_SIGNING_KEY_PREVIOUS` to the old value while you roll. After your max email-verification TTL (default 24 hours), remove the `_PREVIOUS` variable.

Provider secrets rotate the same way — set the new values, redeploy, old flows finish out, new flows use new secrets. No downtime.

## Validation

On boot the server validates every variable it reads. A missing `DATABASE_URL`, a non-TLS `REDIS_URL` in production, a signing key that isn't 32 bytes hex — all cause the server to refuse to start with a clear message. `revo-auth-server check-config` runs the validation without booting the rest of the app, which is useful in a CI pre-deploy step.
