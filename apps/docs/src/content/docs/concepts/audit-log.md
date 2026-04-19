---
title: Audit log
description: What gets logged, where it lives, and how to stream it to your SIEM.
---

Every meaningful action in Revo-Auth writes a row to the audit log. "Meaningful" is a long list — authentication events, permission grants, MFA enrollment, admin actions, failed login attempts, session revocation — and the log is append-only, tenant-scoped, and queryable.

## Shape

The `audit_log` table is minimal on purpose:

| Column | Type | Notes |
|---|---|---|
| `id` | UUIDv7 | Chronologically sortable. |
| `app_id` | UUID | Scopes every query. |
| `actor_id` | UUID, nullable | The user who caused the event. Null for system-initiated events. |
| `actor_type` | Enum | `user`, `admin`, `system`, `service`. |
| `action` | Text | Dotted identifier, e.g. `session.created`, `passkey.registered`, `role.granted`. |
| `target_type` | Text, nullable | The kind of thing acted upon (`user`, `session`, `role`). |
| `target_id` | UUID, nullable | The specific target. |
| `metadata` | JSONB | Action-specific details (IP, user-agent, old/new values). |
| `created_at` | Timestamptz | Server wall clock, UTC. |

## What Revo-Auth logs automatically

- `auth.login.succeeded` / `auth.login.failed` with the reason.
- `session.created` / `session.revoked` / `session.expired`.
- `passkey.registered` / `passkey.deleted`.
- `totp.enrolled` / `totp.disabled` / `totp.code.used`.
- `recovery_code.generated` / `recovery_code.consumed`.
- `user.created` / `user.updated` / `user.deleted`.
- `role.granted` / `role.revoked`.
- `origin.added` / `origin.removed`.
- `admin.session_revoked` / `admin.mfa_reset` / `admin.role_override`.

The list is closed — there is no "custom event" escape hatch for application-level logs. Revo-Auth audits *Revo-Auth*; your app should own its own domain audit log.

## Querying

The dashboard ships a filterable view. Programmatically, query via the server's `/v1/audit` endpoint, filtered by actor, action, target, or time range. Responses are paginated by cursor (never by offset — offset pagination lies on a table that grows during iteration).

## Streaming out

Compliance regimes want the log *somewhere else*. Configure a sink per app:

- **S3 JSONL** — one object per hour, gzipped. Good enough for SOC 2.
- **Webhook** — POST each row as it lands, with HMAC signatures. Retry with exponential backoff; dead-letter after 24h.
- **Syslog** — for shipping to Datadog, Splunk, or self-hosted ELK.

A sink failure does not block the write. Revo-Auth buffers up to 7 days locally and backfills when the sink recovers.

## Retention

Default retention is 365 days. Shorten or lengthen per-app; there is no way to configure "forever" — unbounded retention is an availability risk the product refuses to enable by default.
