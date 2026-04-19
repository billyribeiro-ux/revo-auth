---
title: Sessions
description: How Revo-Auth issues, stores, and revokes sessions — cookies, hashing, expiry, and the role of Redis.
---

A Revo-Auth session is a server-issued, opaque token that represents a logged-in user on a specific app. Sessions are the single source of truth for "is this request authenticated?" — there are no JWTs, no refresh-token dances, and no token introspection endpoints to get wrong.

## The cookie

The session cookie is named `__Host-revo_session`. The `__Host-` prefix is a browser-enforced contract: the cookie must be `Secure`, `Path=/`, and must have no `Domain` attribute. Browsers refuse to set `__Host-` cookies that violate any of those rules, which makes it impossible for a subdomain or an attacker setting a cookie from a sibling site to shadow the session cookie.

Other attributes Revo-Auth always sets:

- `HttpOnly` — JavaScript cannot read the cookie, blunting most XSS exfiltration.
- `SameSite=Lax` — CSRF-safe for top-level navigations, still allows OAuth redirects to complete.
- `Secure` — required by `__Host-` and enforced even in local dev when talking to a server on HTTPS.

## Token hashing

The raw session token lives in the cookie and only in the cookie. What the server stores in Postgres is `sha256(token)`. When a request arrives, the server hashes the presented token and looks up the row. This means:

- A read-only database leak does not grant the attacker active sessions.
- The token is never logged, even accidentally, because no log line ever sees the post-hash form and the pre-hash form is never written to disk.

SHA-256 is the right primitive here — the input is high-entropy (32 bytes from a CSPRNG), so a password-style KDF adds cost without security.

## Expiry and rolling

Sessions default to **14 days** of absolute expiry. On every authenticated request, Revo-Auth slides the expiry forward, so active users stay signed in and idle sessions lapse. You can shorten this per-app in the dashboard; you cannot disable the ceiling.

Sessions carry an optional `step_up_at` timestamp that MFA flows bump. Sensitive routes ask `requireStepUp(5 * 60)` and the server rejects anything older than the window.

## Redis revocation

When a user signs out, changes a password, or is kicked by an admin, the session's hash is written to a Redis set with a TTL matching the session's remaining life. Every authenticated request checks Redis first — `O(1)` — before touching Postgres. If Redis is down, Revo-Auth fails closed: requests return `503` rather than accepting potentially revoked sessions.

Redis is not a cache here. It is the revocation list. Losing it means you lose the ability to kill sessions early, which is a security-critical property.

## What the SDK exposes

`locals.session` gives you `{ id, userId, appId, createdAt, expiresAt, stepUpAt }`. `locals.user` gives you the denormalized user record. Both are `null` when the request is anonymous — there are no "half-authenticated" states.
