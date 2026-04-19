---
title: Database and Redis
description: Recommended managed services (Neon + Upstash) and the self-hosted alternatives.
---

Revo-Auth needs Postgres for durable state and Redis for session revocation. Both have a canonical managed recommendation and a perfectly good self-hosted path.

## Postgres

### Recommended: Neon

[Neon](https://neon.tech/) is the default recommendation. Serverless Postgres with branching, pay-per-request pricing, and a generous free tier. Revo-Auth has been tested against Neon on every release.

- Create a project in the region closest to your Revo-Auth server.
- Use the **pooled** connection string (ends in `?sslmode=require&pgbouncer=true`). Revo-Auth opens enough connections that the pooler earns its keep.
- Set `shared_preload_libraries` doesn't apply — Neon handles extensions via the dashboard. Revo-Auth uses only `pgcrypto` (for `gen_random_bytes`), which is pre-enabled.

Minimum plan: the Free tier works for development and small production deploys up to a few thousand MAU. Scale up when you outgrow it.

### Alternative: Supabase

Supabase Postgres works identically. Pick the pooled URL (transaction mode), same `sslmode=require` story. Revo-Auth doesn't use Supabase's auth/storage features — only the raw Postgres.

### Self-hosted

Any Postgres 14+ works. We test against 14, 15, 16, and 17. Requirements:

- `pgcrypto` extension installed (`CREATE EXTENSION IF NOT EXISTS pgcrypto;` — the first migration does this for you if your role has permission).
- UTF-8 encoding.
- Timezone `UTC` (you can set this per-role: `ALTER ROLE revo SET timezone TO 'UTC';`).

Performance notes:

- Revo-Auth's hot tables (`sessions`, `audit_log`) have appropriate indexes — no manual tuning needed up to millions of rows.
- The audit log is append-only; consider partitioning by month if you expect > 100M events over the retention window.
- Connection pool sizing: we default to 25 connections. Tune via `REVO_AUTH_DB_POOL_MAX` if you have many replicas.

### Migrations

`revo-auth-server migrate` runs all pending migrations. The server boots with auto-migrate enabled by default; disable with `REVO_AUTH_AUTO_MIGRATE=false` if you prefer to run migrations out-of-band (recommended for production).

## Redis

### Recommended: Upstash

[Upstash](https://upstash.com/) Redis has a global edge network, pay-per-request pricing, and TLS on every connection. The free tier covers typical small-app traffic.

- Create a database in the same region as Revo-Auth.
- Enable TLS — Revo-Auth refuses `redis://` URLs in production (requires `rediss://`).
- Use the "Regional" database type unless you have a specific reason to go global.

### Alternative: managed Redis

ElastiCache, MemoryDB, Redis Cloud, Railway — all work. Any Redis 6+ with TLS support.

### Self-hosted

Redis 6+ on the same network as Revo-Auth. The workload is simple — session hashes in a set with TTLs. A 256MB instance comfortably handles a million active sessions.

Requirements:

- TLS enabled (self-signed certs are fine; disable verification via `REVO_AUTH_REDIS_TLS_INSECURE=true` only in controlled environments).
- `maxmemory-policy` set to `noeviction`. Revocation entries getting evicted would let revoked sessions sail through. This is non-negotiable.

### Sizing

Memory usage is roughly `number_of_active_revocations × 100 bytes`. Revocations TTL out automatically, so the working set equals "users who signed out in the last 14 days plus admins you've kicked." For most deployments, a tiny instance is plenty.

## Backups

Back up Postgres. Don't bother backing up Redis — rebuilding the revocation set from Postgres on disaster recovery takes seconds and you lose no real data (the worst case is that recently revoked sessions become valid again, which is a security problem worth pager-duty action, not a silent failure).

For Neon, backups are continuous (point-in-time restore). For self-hosted, use `pg_basebackup` and WAL archiving to object storage — or let your hosting provider do it.

## Connection troubleshooting

`revo-auth doctor` from the CLI checks connectivity end to end. If your Revo-Auth server is up but reporting `503` to clients, check:

1. `DATABASE_URL` reachable from the server's network. Neon's pooler hostnames are different from direct-connect.
2. `REDIS_URL` reachable and using `rediss://`. Upstash issues a non-TLS URL by default — make sure you copied the TLS one.
3. Postgres connection count. If Revo-Auth is erroring with "too many connections," tune `REVO_AUTH_DB_POOL_MAX` down or upgrade the Postgres plan.
