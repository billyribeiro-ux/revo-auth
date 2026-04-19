# Revo-Auth

Production-oriented authentication platform: Rust (Axum) auth server, TypeScript SDKs, SvelteKit adapter, CLI, and UI components — as specified in [`revo-auth-nuclear-prompt.md`](./revo-auth-nuclear-prompt.md).

## Current status

**Implemented (Rust server, `apps/server`):**

- Postgres schema and SQLx migrations (`0001`–`0005`) aligned with the nuclear prompt
- Multi-tenant resolution via `X-Revo-App-Id` + `X-Revo-App-Public-Key` or `Origin` + app id
- Email/password: signup, signin, signout, session read/refresh, list/revoke sessions
- Argon2id (m=65536, t=3, p=1), opaque session tokens (SHA-256 in DB), `__Host-revoauth.session` cookie support
- Admin API: list/create/patch apps (master key: `X-Revo-Master-Key`)
- Password reset request/confirm with enumeration-safe timing and verification tokens
- Redis-backed session revocation markers, readiness (`/ready`) checking DB + Redis
- Structured logging (tracing); email hooks with a log transport (SMTP/Resend wiring is the next step)

**Not yet implemented in this repo (per full spec):**

- Remaining `/v1/*` routes (OAuth, WebAuthn, TOTP, magic links, orgs, account linking), CSRF + tower-governor rate limits as specified, full OTEL pipeline, integration/E2E suites, `packages/*`, Starlight docs, `examples/sveltekit-demo`, CLI templates, CI/release automation to the letter of the prompt

See the nuclear prompt for the complete 21-phase plan and quality gates.

## Run the server locally

1. Start Postgres and Redis.
2. Copy `apps/server/.env.example` to `apps/server/.env` and set real values (JWT PEMs required for any JWT features you add).
3. From repo root:

```bash
cd apps/server
export $(grep -v '^#' .env | xargs)
cargo run --release
```

- Health: `GET http://localhost:8787/health`
- Readiness: `GET http://localhost:8787/ready`

Create an app with the admin API (`X-Revo-Master-Key`), then call `/v1/*` with tenant headers as described in the prompt.

## License

MIT — see [LICENSE](./LICENSE).
