---
title: Multi-tenancy
description: The apps table, the X-Revo-App-Id header, and how Revo-Auth validates origins per tenant.
---

Every installation of Revo-Auth can host many logically isolated tenants. A "tenant" here is what we call an **app** — a single frontend deployment with its own users, its own OAuth credentials, its own session cookie domain, and its own allowed origins.

## The `apps` table

The `apps` table is the spine of the multi-tenant model. Each row holds:

| Column | Purpose |
|---|---|
| `id` | UUIDv7, prefixed as `app_...` when surfaced to clients. |
| `name` | Human-readable label shown in the dashboard. |
| `secret_hash` | SHA-256 of the app secret used for server-to-server calls. |
| `allowed_origins` | JSON array of exact origins permitted to send browser requests. |
| `session_ttl_seconds` | Per-app session ceiling (defaults to 14 days). |
| `features` | Bit-flags: passkeys, TOTP, magic links, social providers. |
| `created_at` | Audit timestamp. |

Users, sessions, OAuth links, passkeys, TOTP factors, roles, and audit rows all carry an `app_id` foreign key. Deleting a row from `apps` cascades through every tenant-scoped table in a single transaction, which makes offboarding clean and auditable.

There is no cross-app user sharing. A human with two accounts on two apps has two `users` rows. That is deliberate — it keeps consent, audit, and revocation boundaries crisp.

## The `X-Revo-App-Id` header

Every call from a client SDK — browser or server — carries `X-Revo-App-Id: app_...`. The server uses that header to:

1. Look up the app row.
2. Scope all subsequent queries to `WHERE app_id = $1`.
3. Enforce per-app feature flags (e.g. reject a passkey registration when the app has passkeys disabled).

The header is trusted for *routing* but not for *authentication*. Authentication is the session cookie plus, on server-to-server calls, the app secret. An attacker who swaps the header to another app ID gets either zero rows (public endpoints) or `401` (authenticated endpoints), because the session cookie is bound to the original app.

## Origin validation

Browsers send a `Origin` header on cross-site requests. On every non-`GET` request, Revo-Auth compares the `Origin` value to the app's `allowed_origins` list using exact-string equality — no wildcards, no subdomain matching, no protocol coercion. If the origin is not in the list, the request is rejected with `403` before any handler runs.

You manage the list from the CLI:

```sh
pnpm dlx @revo-auth/cli add origin https://app.example.com
pnpm dlx @revo-auth/cli add origin http://localhost:5173
```

Local development deliberately requires you to whitelist `http://localhost:5173` (or whatever port you use) — the server will not "just allow localhost." That symmetry between dev and prod prevents a class of "worked on my machine" vulnerabilities.

## Practical shape

A typical deployment has one Revo-Auth server, one Postgres, one Redis, and 1-to-N apps. A SaaS with a marketing site and a product dashboard would register two apps: one for `marketing.example.com`, one for `app.example.com`. They share zero state, which is exactly what you want.
