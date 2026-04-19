# `@revo-auth/server`

The Rust (Axum) authentication server for Revo-Auth. Distroless image, multi-tenant,
Postgres + Redis backed, opaque sessions with ES256 JWTs for service tokens.

## Prerequisites

- Rust `1.83+` (pinned via root `rust-toolchain.toml`)
- Postgres `16+`
- Redis `7+`
- Docker (for the production image)

## Required environment

| Variable | Description |
| --- | --- |
| `REVO_AUTH_HOST` | Bind host. Defaults to `0.0.0.0` in the container. |
| `REVO_AUTH_PORT` | Bind port. Defaults to `8080` in the container, `8787` locally. |
| `REVO_AUTH_DATABASE_URL` | Postgres connection string. |
| `REVO_AUTH_REDIS_URL` | Redis connection string (e.g. `redis://127.0.0.1:6379/0`). |
| `REVO_AUTH_MASTER_KEY` | Admin bearer for the `/admin/*` routes (`X-Revo-Master-Key`). |
| `REVO_AUTH_ENCRYPTION_KEY` | >= 32 bytes used for per-tenant secret encryption (AES-GCM via HKDF). |
| `REVO_AUTH_JWT_ISSUER` | Canonical issuer URL. |
| `REVO_AUTH_JWT_ES256_PRIVATE_PEM` | ES256 private key (PEM). |
| `REVO_AUTH_JWT_ES256_PUBLIC_PEM` | ES256 public key (PEM). |
| `REVO_AUTH_COOKIE_SECURE` | `true` in production; toggles the `Secure` attribute on session cookies. |

Copy `.env.example` to `.env` to iterate locally; never commit populated secrets.

## Generate keys

ES256 (P-256) keypair for JWT signing:

```bash
openssl ecparam -name prime256v1 -genkey -noout -out es256-private.pem
openssl ec -in es256-private.pem -pubout -out es256-public.pem
```

A 32-byte encryption key (base64):

```bash
openssl rand -base64 32
```

An Ed25519 keypair (used by the CLI/SDK for signed bootstrap payloads):

```bash
openssl genpkey -algorithm ED25519 -out ed25519-private.pem
openssl pkey -in ed25519-private.pem -pubout -out ed25519-public.pem
```

Paste the PEMs verbatim into the matching environment variables (newlines preserved).

## Run locally

```bash
# from repo root
docker compose up -d postgres redis     # or bring up your own services
cd apps/server
cp .env.example .env                    # populate real values
export $(grep -v '^#' .env | xargs)
cargo run --release
```

Health: `GET http://localhost:8787/health` - Readiness: `GET http://localhost:8787/ready`.

## Tests

```bash
cargo nextest run                       # unit + integration
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Build the container

```bash
docker build -t revo-auth-server:dev -f apps/server/Dockerfile .
docker run --rm -p 8080:8080 --env-file apps/server/.env revo-auth-server:dev
```

The resulting image is based on `gcr.io/distroless/cc-debian12:nonroot` and targets
< 60MB (spec Phase 20 §1138).

## Deploy to Fly.io

```bash
fly launch --no-deploy --copy-config --dockerfile apps/server/Dockerfile
fly secrets set \
  REVO_AUTH_DATABASE_URL=... \
  REVO_AUTH_REDIS_URL=... \
  REVO_AUTH_MASTER_KEY=... \
  REVO_AUTH_ENCRYPTION_KEY=... \
  REVO_AUTH_JWT_ISSUER=https://auth.example.com \
  REVO_AUTH_JWT_ES256_PRIVATE_PEM="$(cat es256-private.pem)" \
  REVO_AUTH_JWT_ES256_PUBLIC_PEM="$(cat es256-public.pem)" \
  REVO_AUTH_COOKIE_SECURE=true
fly deploy
```

See `fly.toml` for the runtime configuration (auto-start machines, `/health` checks,
shared-cpu-1x/512MB by default).
