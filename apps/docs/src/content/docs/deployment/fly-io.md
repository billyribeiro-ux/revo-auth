---
title: Deploy to Fly.io
description: Ship Revo-Auth to Fly with Neon Postgres, Upstash Redis, volumes, and multi-region health checks.
---

Fly.io is the recommended target for self-hosting Revo-Auth. The Rust binary is small, boot time is low, and Fly's global network lets you put the auth server close to your users.

## 1. Install flyctl and log in

```sh
curl -L https://fly.io/install.sh | sh
fly auth login
```

If you're in a monorepo, `cd` into `apps/server/` — that's where the Dockerfile and `fly.toml` live.

## 2. Create the app

```sh
fly launch --no-deploy --copy-config
```

Pick a unique app name (e.g. `revo-auth-prod`) and a primary region close to your Postgres. Accept the Dockerfile detection. `--no-deploy` prevents Fly from shipping before you've configured secrets.

## 3. Provision Postgres on Neon

Revo-Auth's official recommendation is [Neon](https://neon.tech/) for Postgres. Serverless, branching, and fast to spin up.

- Create a Neon project in the same region as your Fly app.
- Create a branch (or use `main`).
- Copy the pooled connection string — the one ending in `?sslmode=require&pgbouncer=true`.

```sh
fly secrets set DATABASE_URL="postgres://user:pass@ep-...-pooler.aws.neon.tech/revo_auth?sslmode=require"
```

On first boot Revo-Auth runs migrations automatically. If you want to run them manually first (safer for production), `revo-auth-server migrate` inside a `fly ssh console`.

## 4. Provision Redis on Upstash

[Upstash](https://upstash.com/) Redis is the recommended managed Redis. Global replication, pay-per-request, and free for low traffic.

- Create a Redis database in the same region as Fly.
- Enable **TLS** (mandatory — reject the non-TLS option).
- Copy the **Redis URL** — it starts with `rediss://`.

```sh
fly secrets set REDIS_URL="rediss://default:<token>@apn1-xxx.upstash.io:6379"
```

## 5. Set the rest of the secrets

```sh
fly secrets set \
  REVO_AUTH_COOKIE_DOMAIN=".example.com" \
  REVO_AUTH_SIGNING_KEY="$(openssl rand -hex 32)" \
  REVO_AUTH_ADMIN_EMAIL="you@example.com"
```

See [secrets](/deployment/secrets/) for the full list.

## 6. Volumes

The Revo-Auth server is stateless — Postgres and Redis own the state. You do **not** need a Fly volume unless you've enabled the local audit-log buffer (for SIEM sinks that might be temporarily unreachable), in which case:

```sh
fly volumes create revo_auth_data --region <your-region> --size 1
```

And in `fly.toml`:

```toml
[mounts]
source = "revo_auth_data"
destination = "/var/lib/revo-auth"
```

## 7. Deploy

```sh
fly deploy
```

Fly builds the Docker image, pushes to the Fly registry, and rolls out with zero downtime (health checks gate the switchover).

## 8. Health checks and scaling

`fly.toml` ships with:

```toml
[checks.health]
  grace_period = "10s"
  interval = "15s"
  method = "GET"
  path = "/health"
  protocol = "http"
  timeout = "2s"
```

The `/health` endpoint returns `200` when Postgres and Redis are both reachable. Autoscaling is configured via `[[services.concurrency]]`; sensible defaults are in the shipped `fly.toml`.

## 9. Multi-region

Run Revo-Auth close to your users, but keep Postgres in one region (Neon's primary). Add regions with:

```sh
fly scale count 3 --region ord,ams,syd
```

Fly routes requests to the nearest instance. Redis from Upstash is global-replicated by default — no action needed.

## 10. Verify

```sh
curl https://<app-name>.fly.dev/health
```

Should return `{"status":"ok"}`. Then point your SvelteKit app's `REVO_AUTH_URL` at the new server, `revo-auth add origin https://yourapp.example.com`, and run `revo-auth doctor` locally to confirm connectivity.

## Custom domain

```sh
fly certs add auth.example.com
```

Add the CNAME Fly prints. Certificates provision via Let's Encrypt in a minute or two. Update `REVO_AUTH_COOKIE_DOMAIN` to match (and re-deploy to pick up the new value).
