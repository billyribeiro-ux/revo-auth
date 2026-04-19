---
title: Deploy with Docker
description: Run Revo-Auth from the official distroless image — Compose, Kubernetes, or any container runtime.
---

Revo-Auth ships a distroless, multi-stage Docker image on GitHub Container Registry. The final image is under 60MB, has no shell, and runs as a non-root user.

## Image location

```
ghcr.io/revo-auth/server:<version>
ghcr.io/revo-auth/server:latest
```

Tags follow semver. Pin the major version in production — `ghcr.io/revo-auth/server:1` tracks the latest `1.x` release.

## Minimal Compose setup

```yaml
# docker-compose.yml
services:
  revo-auth:
    image: ghcr.io/revo-auth/server:1
    restart: unless-stopped
    ports:
      - "4000:4000"
    environment:
      DATABASE_URL: postgres://revo:revo@postgres:5432/revo_auth
      REDIS_URL: redis://redis:6379
      REVO_AUTH_SIGNING_KEY: ${REVO_AUTH_SIGNING_KEY}
      REVO_AUTH_COOKIE_DOMAIN: .example.com
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    restart: unless-stopped
    environment:
      POSTGRES_USER: revo
      POSTGRES_PASSWORD: revo
      POSTGRES_DB: revo_auth
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U revo"]
      interval: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
```

Before `docker compose up`, generate a signing key:

```sh
export REVO_AUTH_SIGNING_KEY="$(openssl rand -hex 32)"
```

## Migrations

Migrations run automatically on boot. If you prefer to run them as a separate step (safer in production for zero-downtime deploys):

```sh
docker run --rm \
  -e DATABASE_URL="$DATABASE_URL" \
  ghcr.io/revo-auth/server:1 \
  migrate
```

## Kubernetes

A minimal Deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: revo-auth
spec:
  replicas: 2
  selector:
    matchLabels: { app: revo-auth }
  template:
    metadata:
      labels: { app: revo-auth }
    spec:
      containers:
        - name: server
          image: ghcr.io/revo-auth/server:1
          ports: [{ containerPort: 4000 }]
          envFrom:
            - secretRef: { name: revo-auth-secrets }
          readinessProbe:
            httpGet: { path: /health, port: 4000 }
            periodSeconds: 5
          livenessProbe:
            httpGet: { path: /health, port: 4000 }
            periodSeconds: 15
          resources:
            requests: { cpu: 100m, memory: 64Mi }
            limits:   { cpu: 500m, memory: 256Mi }
```

The server is stateless; replicas scale linearly. Resource requests are generous for headroom — real usage under a thousand requests per second fits comfortably in 64MB.

## Ports and protocol

- **4000** — HTTP. Terminate TLS at your load balancer (Fly, ALB, Caddy, Traefik). The server doesn't need to speak TLS directly; `__Host-` cookies require HTTPS *as seen by the browser*, which the load balancer provides.

## Graceful shutdown

The server handles `SIGTERM` cleanly: stops accepting new connections, drains in-flight requests, closes pool connections. Give it a 30-second grace period in your orchestrator.

## Observability

The image exposes Prometheus metrics at `/metrics` (not routable to the internet — firewall it or gate with a header). Logs go to stdout as JSON.
