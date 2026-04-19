# 🔐 REVO-AUTH — NUCLEAR BUILD PROMPT

**Target executor:** Claude Code (Opus 4.7 Extended Thinking)
**Operator:** Billy Ribeiro — Principal Engineer
**Standard:** Apple ICT Level 7+ / Google L8+ / Zero tolerance for shortcuts
**Mode:** Fully autonomous. No clarifying questions. No confirmations. Execute end-to-end.

---

## 🎯 MISSION

Build **Revo-Auth** — a production-grade, award-tier authentication platform consisting of:

1. **A single-binary Rust/Axum auth server** (multi-tenant, self-hostable, horizontally scalable)
2. **A framework-agnostic TypeScript SDK core** + **SvelteKit adapter** (type-safe, tree-shakeable)
3. **A shadcn-style CLI** that scaffolds auth into any SvelteKit 2.x project — file ownership, not package dependency
4. **A Svelte 5 UI component library** (PE7 CSS architecture, zero Tailwind, zero Lucide)
5. **A Starlight documentation site**
6. **A working SvelteKit demo app** proving end-to-end functionality

This is **the tool Billy will use across every SvelteKit project he owns** — RTP v2, ProspectEngine, Crooked Traders, SvelteForge, Svelte Max, TC-360 Blogwriter, and every future build. It must be better than BetterAuth on every axis that matters for his stack.

**Non-negotiable outcome:** At completion, running `pnpm dlx @revo-auth/cli init` inside any SvelteKit project scaffolds a fully working auth system — email/password, OAuth (Google, GitHub, Microsoft), passkeys, TOTP, and magic links — against the deployed Revo-Auth server, with every file generated using Svelte 5 runes and PE7 CSS tokens.

---

## 🚫 EXECUTION RULES — READ TWICE

1. **NEVER ask clarifying questions.** Every decision is locked below. Where unspecified, apply the "Apple ICT7 default" — the choice a Principal Engineer would make without asking.
2. **NEVER leave `TODO`, `FIXME`, `HACK`, `XXX`, `// implement later`, or placeholder code.** Every function is fully implemented.
3. **NEVER use `any` or unjustified `unknown` in TypeScript.** Strict mode is law. Runtime validation via `zod` at every boundary.
4. **NEVER use `unwrap()` or `expect()` in Rust production paths** except in startup code where failure means the server shouldn't run.
5. **NEVER commit secrets.** Use `.env.example` with clear placeholders. Real secrets never touch the repo.
6. **NEVER use npm or yarn.** pnpm only. Workspaces via `pnpm-workspace.yaml`.
7. **NEVER use Tailwind, UnoCSS, or any atomic CSS framework.** PE7 CSS only — OKLCH tokens, `@layer` cascade, logical properties, `clamp()` fluid type, scoped `--_` custom properties. ZERO exceptions.
8. **NEVER use Lucide or heroicons.** Iconify only, with `@iconify-json/ph` (Phosphor) or `@iconify-json/carbon` (Carbon). Pick one per component and stay consistent.
9. **NEVER use Svelte 4 patterns.** Runes-only. `{@attach}` not `use:`, `{#snippet}/{@render}` not slots, `Spring`/`Tween` classes not stores. No `export let` — use `$props()`.
10. **NEVER use `bcrypt`, `scrypt`, or `pbkdf2` for new password hashes.** Argon2id only. Parameters locked in spec.
11. **ALWAYS run the full quality gate before marking any phase complete.** Gates defined in § Quality Gates.
12. **ALWAYS favor native platform primitives over abstractions** — native CSS nesting, native Container Queries, native View Transitions, native `dialog` + `popover`.
13. **ALWAYS write code for 10-year longevity.** If a dependency feels faddish, use the platform instead.

---

## 🏗 ARCHITECTURE

```
┌───────────────────────────────────────────────────────────────┐
│  auth.revo-auth.dev  (Rust/Axum, single binary, multi-tenant) │
│  ┌──────────────────────────────────────────────────────┐     │
│  │ Postgres  — users, accounts, sessions, apps, orgs    │     │
│  │ Redis     — session cache, rate limits, revocation   │     │
│  │ Fly.io    — edge deployment, auto-scale              │     │
│  └──────────────────────────────────────────────────────┘     │
└─────────────────────┬──────────────────┬──────────────────────┘
                      │                  │
         ┌────────────┘                  └────────────┐
         │                                            │
  ┌──────▼──────────┐                         ┌───────▼────────┐
  │  SvelteKit app  │                         │  SvelteKit app │
  │  (RTP v2)       │                         │  (ProspectEng) │
  │                 │                         │                │
  │  Scaffolded by  │                         │ Scaffolded by  │
  │  @revo-auth/cli │                         │ @revo-auth/cli │
  │                 │                         │                │
  │  Uses:          │                         │  Uses:         │
  │  @revo-auth/    │                         │  @revo-auth/   │
  │    sveltekit    │                         │    sveltekit   │
  └─────────────────┘                         └────────────────┘
```

**Layered customization model:**

- **Layer 1** — Server tenant config (per-app, stored in DB, managed via admin API)
- **Layer 2** — `auth.config.ts` in each SvelteKit project (fully editable, owned by user)
- **Layer 3** — Scaffolded code in `src/lib/auth/*` (fully editable, owned by user, diff-merged on update)

Server enforces security. Client configures experience. Worst-case misconfig on the client narrows the feature set; it cannot weaken security.

---

## 📦 MONOREPO LAYOUT (EXACT — CREATE THIS)

```
revo-auth/
├── .github/
│   └── workflows/
│       ├── ci.yml                       # Rust + TS matrix
│       ├── release.yml                  # Changesets release
│       └── security.yml                 # cargo-audit + pnpm audit
├── apps/
│   ├── server/                          # The Rust/Axum auth server
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── config.rs                # Loaded from env via figment
│   │   │   ├── error.rs                 # thiserror + IntoResponse
│   │   │   ├── state.rs                 # AppState (pool, redis, crypto)
│   │   │   ├── db/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── apps.rs              # Multi-tenant app records
│   │   │   │   ├── users.rs
│   │   │   │   ├── accounts.rs          # OAuth provider accounts
│   │   │   │   ├── sessions.rs
│   │   │   │   ├── verification.rs
│   │   │   │   ├── passkeys.rs
│   │   │   │   ├── totp.rs
│   │   │   │   ├── magic_links.rs
│   │   │   │   ├── orgs.rs
│   │   │   │   ├── memberships.rs
│   │   │   │   └── audit.rs
│   │   │   ├── crypto/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── password.rs          # Argon2id
│   │   │   │   ├── tokens.rs            # Secure random via ring
│   │   │   │   ├── jwt.rs               # ES256 for signed tokens
│   │   │   │   └── hkdf.rs              # Key derivation
│   │   │   ├── providers/
│   │   │   │   ├── mod.rs               # trait OAuthProvider
│   │   │   │   ├── google.rs            # enabled
│   │   │   │   ├── github.rs            # enabled
│   │   │   │   ├── microsoft.rs         # enabled
│   │   │   │   ├── discord.rs           # enabled
│   │   │   │   └── apple.rs             # scaffolded, enabled=false
│   │   │   ├── routes/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── health.rs
│   │   │   │   ├── v1/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── signup.rs
│   │   │   │   │   ├── signin.rs
│   │   │   │   │   ├── signout.rs
│   │   │   │   │   ├── session.rs
│   │   │   │   │   ├── oauth.rs         # /oauth/:provider + callback
│   │   │   │   │   ├── passkey.rs       # register + authenticate
│   │   │   │   │   ├── totp.rs          # setup + verify
│   │   │   │   │   ├── magic.rs         # request + verify
│   │   │   │   │   ├── email.rs         # verify + resend
│   │   │   │   │   ├── password.rs      # reset flow
│   │   │   │   │   ├── account.rs       # link/unlink providers
│   │   │   │   │   └── org.rs           # organizations + invites
│   │   │   │   └── admin/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── apps.rs          # CRUD tenants
│   │   │   │       └── users.rs         # impersonate, list, ban
│   │   │   ├── middleware/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── auth.rs              # Bearer + cookie extraction
│   │   │   │   ├── tenant.rs            # app_id resolution + origin check
│   │   │   │   ├── rate_limit.rs        # tower-governor integration
│   │   │   │   ├── csrf.rs              # double-submit cookie
│   │   │   │   └── request_id.rs
│   │   │   ├── email/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── transport.rs         # SMTP + Resend + SES adapters
│   │   │   │   └── templates/           # MJML-compiled HTML + text
│   │   │   │       ├── verify.rs
│   │   │   │       ├── reset.rs
│   │   │   │       ├── magic.rs
│   │   │   │       └── invite.rs
│   │   │   ├── webauthn.rs              # webauthn-rs integration
│   │   │   ├── telemetry.rs             # tracing + OTEL
│   │   │   └── lib.rs
│   │   ├── migrations/
│   │   │   ├── 0001_init.sql
│   │   │   ├── 0002_passkeys.sql
│   │   │   ├── 0003_orgs.sql
│   │   │   ├── 0004_audit.sql
│   │   │   └── 0005_magic_links.sql
│   │   ├── tests/
│   │   │   ├── common/mod.rs            # testcontainers harness
│   │   │   ├── signup_signin.rs
│   │   │   ├── oauth_google.rs          # wiremock'd
│   │   │   ├── passkey.rs
│   │   │   ├── totp.rs
│   │   │   ├── magic_link.rs
│   │   │   ├── rate_limit.rs
│   │   │   └── tenant_isolation.rs
│   │   ├── Cargo.toml
│   │   ├── Dockerfile                   # distroless, multi-stage
│   │   ├── fly.toml
│   │   └── README.md
│   └── docs/                            # Starlight site
│       ├── src/content/docs/
│       │   ├── index.md
│       │   ├── quickstart.md
│       │   ├── concepts/
│       │   ├── sveltekit/
│       │   ├── cli/
│       │   ├── providers/
│       │   └── deployment/
│       ├── astro.config.mjs
│       └── package.json
├── packages/
│   ├── sdk-core/                        # Framework-agnostic
│   │   ├── src/
│   │   │   ├── client.ts                # fetch wrapper, typed
│   │   │   ├── errors.ts                # RevoAuthError hierarchy
│   │   │   ├── schemas.ts               # zod schemas
│   │   │   ├── types.ts                 # Session, User, App
│   │   │   ├── oauth.ts
│   │   │   ├── passkey.ts               # wraps @simplewebauthn/browser
│   │   │   ├── totp.ts
│   │   │   ├── magic.ts
│   │   │   └── index.ts
│   │   ├── tests/
│   │   ├── tsup.config.ts
│   │   └── package.json
│   ├── sdk-sveltekit/                   # SvelteKit adapter
│   │   ├── src/
│   │   │   ├── hooks.ts                 # handleAuth composable
│   │   │   ├── load.ts                  # requireAuth helper
│   │   │   ├── server.ts                # server-side client
│   │   │   ├── client.svelte.ts         # $state reactive session
│   │   │   ├── guards.ts                # route protection
│   │   │   └── index.ts
│   │   ├── tests/
│   │   └── package.json
│   ├── cli/                             # @revo-auth/cli
│   │   ├── src/
│   │   │   ├── bin.ts                   # shebang entry
│   │   │   ├── commands/
│   │   │   │   ├── init.ts
│   │   │   │   ├── add.ts
│   │   │   │   ├── update.ts
│   │   │   │   ├── ui.ts
│   │   │   │   ├── dev.ts               # spin up local auth server
│   │   │   │   ├── doctor.ts            # diagnose config
│   │   │   │   └── logout.ts
│   │   │   ├── detect/
│   │   │   │   ├── framework.ts         # SvelteKit version, TS, etc.
│   │   │   │   ├── pm.ts                # force pnpm
│   │   │   │   └── paths.ts             # src/, lib/, routes/
│   │   │   ├── ast/
│   │   │   │   ├── merge-hooks.ts       # magicast merge into hooks.server.ts
│   │   │   │   ├── merge-env.ts         # .env.example merge
│   │   │   │   └── merge-config.ts      # auth.config.ts merge
│   │   │   ├── templates/
│   │   │   │   ├── manifest.ts          # hash tracker
│   │   │   │   ├── render.ts            # handlebars rendering
│   │   │   │   └── files/               # .hbs template files
│   │   │   │       ├── sveltekit/
│   │   │   │       │   ├── auth.config.ts.hbs
│   │   │   │       │   ├── lib/
│   │   │   │       │   │   ├── client.ts.hbs
│   │   │   │       │   │   ├── server.ts.hbs
│   │   │   │       │   │   ├── middleware.ts.hbs
│   │   │   │       │   │   └── types.ts.hbs
│   │   │   │       │   ├── hooks.server.ts.hbs
│   │   │   │       │   └── routes/
│   │   │   │       │       ├── login.svelte.hbs
│   │   │   │       │       ├── signup.svelte.hbs
│   │   │   │       │       ├── callback.server.ts.hbs
│   │   │   │       │       ├── verify-email.svelte.hbs
│   │   │   │       │       └── reset-password.svelte.hbs
│   │   │   │       └── providers/
│   │   │   │           ├── google.ts.hbs
│   │   │   │           ├── github.ts.hbs
│   │   │   │           ├── microsoft.ts.hbs
│   │   │   │           └── apple.ts.hbs  (enabled:false)
│   │   │   ├── presets/
│   │   │   │   ├── trading-platform.ts  # RTP-style
│   │   │   │   ├── b2b-saas.ts          # ProspectEngine-style
│   │   │   │   ├── privacy-first.ts     # Crooked Traders-style
│   │   │   │   └── minimal.ts
│   │   │   ├── prompts.ts               # @clack/prompts wrappers
│   │   │   └── diff.ts                  # 3-way merge for update
│   │   ├── tests/
│   │   ├── tsup.config.ts
│   │   └── package.json
│   └── ui-sveltekit/                    # Pre-built Svelte 5 components
│       ├── src/
│       │   ├── components/
│       │   │   ├── LoginForm.svelte
│       │   │   ├── SignupForm.svelte
│       │   │   ├── OAuthButton.svelte
│       │   │   ├── PasskeyButton.svelte
│       │   │   ├── MagicLinkForm.svelte
│       │   │   ├── MfaSetup.svelte
│       │   │   ├── MfaChallenge.svelte
│       │   │   ├── PasswordField.svelte
│       │   │   ├── PasswordStrengthMeter.svelte
│       │   │   ├── SessionList.svelte
│       │   │   ├── AccountLinking.svelte
│       │   │   └── VerifyEmailBanner.svelte
│       │   ├── styles/
│       │   │   ├── tokens.css           # PE7 OKLCH tokens
│       │   │   ├── reset.css
│       │   │   ├── base.css
│       │   │   └── components.css
│       │   ├── lib/
│       │   │   ├── form.svelte.ts       # reactive form state
│       │   │   └── validation.ts        # zod-powered
│       │   └── index.ts
│       ├── tests/
│       └── package.json
├── examples/
│   └── sveltekit-demo/                  # End-to-end proof
│       ├── src/
│       ├── static/
│       ├── svelte.config.js
│       ├── vite.config.ts
│       └── package.json
├── .changeset/
├── .editorconfig
├── .gitignore
├── .npmrc                               # shamefully-hoist=false, strict
├── .nvmrc
├── biome.json                           # Biome for JS/TS lint + format
├── pnpm-workspace.yaml
├── pnpm-lock.yaml
├── package.json                         # root scripts via turbo
├── turbo.json
├── Cargo.toml                           # workspace
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── LICENSE                              # MIT
├── README.md                            # polished, with diagrams
└── CONTRIBUTING.md
```

---

## 🔩 LOCKED-IN STACK

### Rust (server)

| Dependency | Version pin | Purpose |
|---|---|---|
| `tokio` | `1` (latest) | Async runtime, full features |
| `axum` | `0.8` (latest stable) | HTTP framework |
| `tower` | latest | Middleware |
| `tower-http` | latest | CORS, trace, compression, request-id |
| `tower-governor` | latest | Rate limiting |
| `hyper` | latest | HTTP (axum transitively) |
| `sqlx` | latest | Async Postgres, compile-time query check, macros |
| `argon2` | latest | Argon2id password hashing |
| `ring` or `aws-lc-rs` | latest | Core crypto primitives |
| `rand` | latest | Token randomness (OsRng) |
| `jsonwebtoken` or `josekit` | latest | JWT signing (ES256) |
| `oauth2` | latest | OAuth 2.0 client with PKCE |
| `openidconnect` | latest | OIDC provider flows |
| `webauthn-rs` | latest stable | Passkeys / WebAuthn |
| `totp-rs` | latest | TOTP generation + verification |
| `fred` | latest | Redis client, clustered-ready |
| `lettre` | latest | SMTP |
| `reqwest` | latest | HTTP client for Resend/SES APIs |
| `serde` + `serde_json` | latest | Serialization |
| `garde` | latest | Input validation (preferred over validator) |
| `thiserror` | latest | Library errors |
| `anyhow` | latest | Binary errors (main.rs only) |
| `tracing` + `tracing-subscriber` | latest | Structured logging |
| `opentelemetry` + `opentelemetry-otlp` | latest | Distributed tracing |
| `uuid` | latest, v7 feature | Time-ordered IDs |
| `chrono` | latest | Timestamps (or `time`, pick one) |
| `figment` | latest | Layered config (env + file) |
| `testcontainers` (dev) | latest | Real Postgres/Redis in tests |
| `wiremock` (dev) | latest | HTTP mocking for OAuth providers |
| `cargo-nextest` | via CI | Test runner |
| `cargo-audit` | via CI | Advisory check |
| `cargo-deny` | via CI | License + supply-chain check |

### TypeScript (SDKs, CLI, UI)

| Dependency | Version | Purpose |
|---|---|---|
| `typescript` | `^5.7` | Strict mode, verbatimModuleSyntax |
| `pnpm` | `^10` | Package manager (enforced via `packageManager` field) |
| `turbo` | latest | Monorepo task runner |
| `tsup` | latest | Library builds (ESM + types, no CJS) |
| `biome` | latest | Lint + format (replaces ESLint + Prettier) |
| `vitest` | latest | Unit + integration tests |
| `@playwright/test` | latest | E2E tests |
| `zod` | latest | Runtime validation at boundaries |
| `@simplewebauthn/browser` | latest | Passkey client helpers |
| `@clack/prompts` | latest | CLI prompts |
| `citty` | latest | CLI command router |
| `magicast` | latest | AST merging for TS files |
| `ts-morph` | latest | Fallback for complex AST work |
| `handlebars` | latest | Template rendering |
| `execa` | latest | Run subprocess (pnpm install, etc.) |
| `picocolors` | latest | Terminal colors |
| `consola` | latest | CLI logger |
| `ofetch` | latest | Fetch wrapper for SDK |
| `defu` | latest | Config merging |
| `pathe` | latest | Cross-platform paths |
| `@iconify-json/ph` | latest | Phosphor icons |
| `unplugin-icons` | latest | Build-time icon resolution in Svelte |

### Svelte

| Dependency | Version | Notes |
|---|---|---|
| `svelte` | `^5` latest | Runes-only |
| `@sveltejs/kit` | `^2.49` latest | SSR, endpoints |
| `vite` | latest | Build tool |
| `@sveltejs/adapter-node` (demo) | latest | For the example app |

### Docs

| Dependency | Version | Purpose |
|---|---|---|
| `astro` + `@astrojs/starlight` | latest | Docs site |
| `shiki` | latest | Code highlighting |
| `pagefind` | latest | Static search |

---

## 🗄 DATABASE SCHEMA (Postgres)

Write these as SQL migrations under `apps/server/migrations/`. Use SQLx. Use `uuidv7_sub_ms()` via pgcrypto + extension, OR generate UUIDv7 in Rust and insert — pick the second, more portable.

```sql
-- 0001_init.sql
create extension if not exists citext;
create extension if not exists pgcrypto;

create table apps (
  id              uuid primary key,
  slug            text unique not null,
  name            text not null,
  origins         text[] not null default '{}',
  public_key      text not null,
  secret_key_hash text not null,          -- argon2id of secret, shown once
  settings        jsonb not null default '{}'::jsonb,
  created_at      timestamptz not null default now(),
  updated_at      timestamptz not null default now()
);

create table users (
  id                uuid primary key,
  app_id            uuid not null references apps(id) on delete cascade,
  email             citext,
  email_verified_at timestamptz,
  password_hash     text,                  -- nullable; null if OAuth-only
  name              text,
  image_url         text,
  custom_fields     jsonb not null default '{}'::jsonb,
  banned_at         timestamptz,
  created_at        timestamptz not null default now(),
  updated_at        timestamptz not null default now(),
  unique (app_id, email)
);
create index on users (app_id);

create table accounts (
  id                uuid primary key,
  user_id           uuid not null references users(id) on delete cascade,
  provider          text not null,         -- 'google' | 'github' | ...
  provider_account  text not null,         -- provider's user id
  access_token_enc  bytea,
  refresh_token_enc bytea,
  expires_at        timestamptz,
  scope             text,
  id_token          text,
  created_at        timestamptz not null default now(),
  unique (provider, provider_account)
);
create index on accounts (user_id);

create table sessions (
  id              uuid primary key,
  user_id         uuid not null references users(id) on delete cascade,
  token_hash      bytea not null unique,   -- sha256 of opaque token
  user_agent      text,
  ip              inet,
  expires_at      timestamptz not null,
  created_at      timestamptz not null default now(),
  last_used_at    timestamptz not null default now(),
  revoked_at      timestamptz
);
create index on sessions (user_id);
create index on sessions (expires_at);

create table verification_tokens (
  id          uuid primary key,
  user_id     uuid not null references users(id) on delete cascade,
  kind        text not null,              -- 'email_verify' | 'password_reset'
  token_hash  bytea not null unique,
  expires_at  timestamptz not null,
  used_at     timestamptz,
  created_at  timestamptz not null default now()
);

create table audit_log (
  id          bigserial primary key,
  app_id      uuid references apps(id) on delete set null,
  user_id     uuid references users(id) on delete set null,
  event       text not null,
  meta        jsonb not null default '{}'::jsonb,
  ip          inet,
  user_agent  text,
  created_at  timestamptz not null default now()
);
create index on audit_log (user_id, created_at desc);
create index on audit_log (app_id, created_at desc);

-- 0002_passkeys.sql
create table passkeys (
  id               uuid primary key,
  user_id          uuid not null references users(id) on delete cascade,
  credential_id    bytea not null unique,
  public_key       bytea not null,
  sign_count       bigint not null default 0,
  transports       text[] not null default '{}',
  backup_eligible  boolean not null default false,
  backup_state     boolean not null default false,
  friendly_name    text,
  last_used_at     timestamptz,
  created_at       timestamptz not null default now()
);

-- 0003_orgs.sql
create table organizations (
  id          uuid primary key,
  app_id      uuid not null references apps(id) on delete cascade,
  slug        text not null,
  name        text not null,
  created_at  timestamptz not null default now(),
  unique (app_id, slug)
);

create table memberships (
  id          uuid primary key,
  org_id      uuid not null references organizations(id) on delete cascade,
  user_id     uuid not null references users(id) on delete cascade,
  role        text not null,              -- 'owner' | 'admin' | 'member'
  created_at  timestamptz not null default now(),
  unique (org_id, user_id)
);

-- 0004_audit.sql (index tuning)
-- 0005_magic_links.sql
create table magic_links (
  id          uuid primary key,
  user_id     uuid references users(id) on delete cascade,
  email       citext not null,
  app_id      uuid not null references apps(id) on delete cascade,
  token_hash  bytea not null unique,
  expires_at  timestamptz not null,
  used_at     timestamptz,
  created_at  timestamptz not null default now()
);

create table totp_secrets (
  user_id      uuid primary key references users(id) on delete cascade,
  secret_enc   bytea not null,
  confirmed_at timestamptz,
  recovery_codes_hash bytea[] not null default '{}',
  created_at   timestamptz not null default now()
);
```

Encrypt provider tokens and TOTP secrets at rest with a server-held key derived from `REVO_AUTH_ENCRYPTION_KEY` via HKDF. Never store plaintext access tokens.

---

## 🌐 API SURFACE (v1)

All routes are tenant-scoped via `X-Revo-App-Id` header + `X-Revo-App-Public-Key` header OR origin-matched from `apps.origins`. Failing either → 403.

| Method | Path | Auth | Purpose |
|---|---|---|---|
| POST | `/v1/signup` | app | Create user with email + password |
| POST | `/v1/signin` | app | Email + password → session |
| POST | `/v1/signout` | session | Revoke current session |
| GET | `/v1/session` | session | Get current session + user |
| POST | `/v1/session/refresh` | session | Rotate token |
| GET | `/v1/sessions` | session | List all user sessions |
| DELETE | `/v1/sessions/:id` | session | Revoke specific session |
| POST | `/v1/password/reset/request` | app | Email password reset link |
| POST | `/v1/password/reset/confirm` | app | Consume reset token, set new pwd |
| POST | `/v1/email/verify/request` | session | Resend verification |
| POST | `/v1/email/verify/confirm` | app | Consume verify token |
| GET | `/v1/oauth/:provider/authorize` | app | Begin OAuth flow (redirect) |
| GET | `/v1/oauth/:provider/callback` | app | Complete OAuth flow |
| POST | `/v1/passkey/register/begin` | session | WebAuthn registration challenge |
| POST | `/v1/passkey/register/finish` | session | Verify registration |
| POST | `/v1/passkey/authenticate/begin` | app | WebAuthn assertion challenge |
| POST | `/v1/passkey/authenticate/finish` | app | Verify assertion → session |
| GET | `/v1/passkey` | session | List user passkeys |
| DELETE | `/v1/passkey/:id` | session | Revoke passkey |
| POST | `/v1/totp/setup` | session | Generate TOTP secret + QR |
| POST | `/v1/totp/confirm` | session | Confirm first TOTP code |
| POST | `/v1/totp/verify` | app | Verify TOTP during MFA |
| POST | `/v1/totp/disable` | session | Disable TOTP |
| POST | `/v1/magic/request` | app | Send magic link |
| GET | `/v1/magic/verify` | app | Consume magic link → session |
| POST | `/v1/account/link/:provider` | session | Begin account link |
| DELETE | `/v1/account/:provider` | session | Unlink provider |
| GET | `/v1/orgs` | session | List user's orgs |
| POST | `/v1/orgs` | session | Create org |
| POST | `/v1/orgs/:id/invite` | session+role(admin) | Invite by email |
| POST | `/v1/orgs/:id/accept` | session | Accept invite |
| GET | `/admin/apps` | master | List tenants (master key only) |
| POST | `/admin/apps` | master | Create tenant |
| PATCH | `/admin/apps/:id` | master | Update tenant |
| POST | `/admin/impersonate/:user_id` | master | Start impersonation session |
| GET | `/health` | public | Liveness |
| GET | `/ready` | public | Readiness (DB + Redis ping) |

**Error envelope (always):**

```json
{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Email or password is incorrect.",
    "request_id": "01J5Z7K7RQ0A3N1WZY..."
  }
}
```

Use a Rust enum with `thiserror` for error codes. Map to HTTP status + envelope via `IntoResponse`.

---

## 🛡 SECURITY REQUIREMENTS (LOCKED)

1. **Argon2id** params: `m=65536` (64 MiB), `t=3`, `p=1`, 16-byte salt, 32-byte tag. Rehash on login if params were lower.
2. **Session tokens**: 32 random bytes from `OsRng`, encoded base64url (43 chars). Store SHA-256 in DB, send opaque token to client.
3. **Session cookies**: `__Host-revoauth.session`, `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`. No `Domain` attribute.
4. **CSRF**: Double-submit cookie pattern for browser requests. Skip for Bearer-token requests.
5. **Rate limits** (tower-governor):
   - `/v1/signin`: 5 req/min per (IP, email) combo
   - `/v1/signup`: 3 req/hour per IP
   - `/v1/password/reset/request`: 3 req/hour per (IP, email)
   - `/v1/magic/request`: 5 req/hour per (IP, email)
   - `/v1/oauth/*/callback`: 10 req/min per IP
6. **Account enumeration**: Signin, password reset request, and magic link request MUST return identical responses whether or not the email exists. Timing must be constant-ish (add a random sleep 50–150ms on miss).
7. **Timing-safe comparisons** for token verification. Use `subtle::ConstantTimeEq` or `ring::constant_time`.
8. **Password policy defaults**: min 12 chars, must not be in HIBP top-1k common passwords list (embed the list). Optional breach check via HIBP k-anonymity API when `password_policy.breach_check = true`.
9. **Session rotation**: Issue new token on email verification, password change, and after MFA step-up.
10. **Session revocation**: Add revoked session IDs to Redis set; middleware checks Redis first, DB second.
11. **JWT (where used — short-lived tokens only)**: ES256, `iss`=auth server URL, `aud`=app id, `exp` ≤ 15min.
12. **Provider token encryption**: AES-256-GCM with per-record nonce. Key from HKDF of `REVO_AUTH_ENCRYPTION_KEY`.
13. **SQL**: Only parameterized queries via SQLx macros. No `format!()` into queries. Ever.
14. **Email output**: HTML emails go through an explicit escape helper. Never concatenate user input into HTML.
15. **OAuth state**: State param is HMAC-signed, contains tenant + nonce + redirect. Verify on callback.
16. **OAuth PKCE**: Required for all providers that support it. S256 method.
17. **WebAuthn**: `userVerification: "preferred"`, RP ID = tenant's primary origin hostname.
18. **Audit log**: Record signup, signin, signout, failed-signin, password-change, MFA-enable, MFA-disable, passkey-add, passkey-remove, admin actions, impersonation start/stop. 90-day retention default.
19. **Secrets**: Never log. Redact via `tracing` filters. `#[derive(Debug)]` on structs with secrets must be manually implemented to redact.
20. **CORS**: Allow only origins from `apps.origins` for the resolved tenant. Wildcard origin is an error.

---

## 🧩 SDK CORE — TYPE CONTRACT

```typescript
// packages/sdk-core/src/types.ts
export interface RevoAuthUser {
  id: string;
  email: string | null;
  emailVerified: boolean;
  name: string | null;
  image: string | null;
  customFields: Record<string, unknown>;
  createdAt: string;
}

export interface RevoAuthSession {
  id: string;
  userId: string;
  expiresAt: string;
  user: RevoAuthUser;
}

export interface RevoAuthConfig {
  serverUrl: string;
  appId: string;
  publicKey: string;
  fetchFn?: typeof fetch;
}

// Typed result pattern — never throw across SDK boundary
export type Result<T, E = RevoAuthError> =
  | { ok: true; data: T }
  | { ok: false; error: E };
```

Every SDK method returns `Promise<Result<T>>`. No throwing. Consumers can destructure or use a `.unwrap()` helper.

---

## 🧱 SVELTEKIT ADAPTER CONTRACT

```typescript
// packages/sdk-sveltekit/src/index.ts
export function handleAuth(options: AuthHookOptions): Handle;
export function requireAuth(event: RequestEvent): RevoAuthSession;   // throws redirect
export function optionalAuth(event: RequestEvent): RevoAuthSession | null;
export function createAuthClient(config: RevoAuthConfig): ClientAPI;

// packages/sdk-sveltekit/src/client.svelte.ts
export function createSessionStore(): {
  readonly current: RevoAuthSession | null;
  readonly loading: boolean;
  refresh(): Promise<void>;
};
// Exposes $state-backed reactive getters — NOT a writable store.
```

---

## 🖥 CLI — COMMAND SPEC

```
revo-auth init [--preset <name>] [--server <url>] [--yes]
revo-auth add <feature> [--force]
    Features: email | google | github | microsoft | apple | discord |
              passkeys | totp | magic-link | organizations | audit-log |
              custom-fields | anonymous-mode
revo-auth update [--interactive|--auto]
revo-auth ui <component...>
    Components: login-form | signup-form | oauth-button | passkey-button |
                magic-link-form | mfa-setup | mfa-challenge |
                session-list | account-linking | password-field |
                password-strength-meter | verify-email-banner
revo-auth dev [--port 8787]
revo-auth doctor
revo-auth logout
```

**`init` flow (all auto when `--yes` or `--preset`):**
1. Detect SvelteKit version — fail if < 2.0.
2. Detect TypeScript — required. If not present, run `pnpm add -D typescript` and init `tsconfig.json`.
3. Force pnpm. If `package-lock.json` or `yarn.lock` present, error with migration instructions.
4. Prompt (skipped with `--yes`): server URL, preset, methods, session strategy.
5. Register app with server via `POST /admin/apps` (if master key present) OR write `.env` with placeholders for manual registration.
6. Scaffold files via template renderer with manifest tracking.
7. AST-merge `src/hooks.server.ts` using magicast. Wrap existing `handle` with `sequence(revoAuth, existing)`.
8. Add `.env` entries — `REVO_AUTH_SERVER_URL`, `REVO_AUTH_APP_ID`, `REVO_AUTH_PUBLIC_KEY`, `REVO_AUTH_SECRET_KEY`.
9. Write `.gitignore` entries if missing.
10. Run `pnpm install` for new deps.
11. Write `.revo-auth/manifest.json` with file hashes + version.
12. Print next-steps banner with provider setup links.

**`update` flow:**
1. Read manifest. For each file, compare current hash vs stored hash.
2. Files unchanged from template → overwrite silently with new template.
3. Files modified → 3-way merge (original template, user-modified, new template). On conflict: write `.orig` sidecar and show diff, prompt.
4. `auth.config.ts` is ALWAYS treated as user-owned — only add new keys with sensible defaults via magicast; never overwrite values.
5. Update manifest.

---

## 🎨 UI COMPONENT STANDARDS (PE7 CSS + Svelte 5)

Every component:
- Runes-only: `$state`, `$derived`, `$effect`, `$props()`
- Uses `{@attach}` for DOM hooks
- Uses `{#snippet}` + `{@render}` for composition
- Props typed with TS interfaces, no `any`
- Imports icons from `phosphor-svelte` via unplugin-icons pattern OR inline SVG — no raster images
- Scoped styles only; no global selectors
- Uses scoped `--_` custom properties for state-driven values
- All colors via OKLCH tokens from `tokens.css`
- All spacing via `clamp()` fluid rhythm tokens
- All breakpoints via the PE7 9-tier system (xs 320, sm 480, md 768, lg 1024, xl 1280, xl2 1536, xl3 1920, xl4 2560, xl5 3840)
- Logical properties exclusively (`margin-inline`, `padding-block-start`, `inline-size`, etc.)
- Zero `!important`
- Zero inline styles except for dynamic single-property cases via style props
- Container queries for component-level responsive behavior
- Respects `prefers-reduced-motion`
- Respects `prefers-color-scheme` via OKLCH token variants
- Keyboard accessible: all interactive elements have visible `:focus-visible` state using `outline`, not `box-shadow` hacks
- ARIA roles/labels where semantic HTML isn't sufficient
- Form fields announce errors via `aria-describedby` + `aria-invalid`
- Password fields have show/hide toggle (Phosphor eye/eye-slash)
- Loading states communicated via `aria-busy` + visible spinner

Example token file:

```css
/* packages/ui-sveltekit/src/styles/tokens.css */
@layer tokens {
  :root {
    /* Semantic surface */
    --surface-base:    oklch(99% 0.002 255);
    --surface-raised:  oklch(100% 0 0);
    --surface-sunken:  oklch(97% 0.003 255);

    /* Semantic text */
    --text-primary:    oklch(22% 0.02 255);
    --text-secondary:  oklch(45% 0.015 255);
    --text-muted:      oklch(62% 0.012 255);

    /* Semantic accent */
    --accent-base:     oklch(62% 0.19 259);
    --accent-hover:    oklch(56% 0.21 259);
    --accent-text:     oklch(100% 0 0);

    /* Semantic status */
    --status-success:  oklch(65% 0.17 155);
    --status-warning:  oklch(78% 0.17 75);
    --status-danger:   oklch(62% 0.22 27);

    /* Fluid rhythm */
    --space-3xs: clamp(0.25rem, 0.2rem + 0.2vw, 0.375rem);
    --space-2xs: clamp(0.5rem, 0.4rem + 0.4vw, 0.75rem);
    --space-xs:  clamp(0.75rem, 0.6rem + 0.6vw, 1rem);
    --space-sm:  clamp(1rem, 0.8rem + 0.8vw, 1.5rem);
    --space-md:  clamp(1.5rem, 1.2rem + 1.2vw, 2.25rem);
    --space-lg:  clamp(2rem, 1.6rem + 1.6vw, 3rem);
    --space-xl:  clamp(3rem, 2.4rem + 2.4vw, 4.5rem);

    /* Fluid type */
    --type-sm:   clamp(0.875rem, 0.83rem + 0.22vw, 0.95rem);
    --type-base: clamp(1rem, 0.95rem + 0.25vw, 1.125rem);
    --type-lg:   clamp(1.25rem, 1.15rem + 0.5vw, 1.5rem);
    --type-xl:   clamp(1.5rem, 1.35rem + 0.75vw, 2rem);

    /* Motion */
    --motion-fast: 140ms;
    --motion-base: 220ms;
    --motion-slow: 420ms;
    --motion-ease: cubic-bezier(0.2, 0.7, 0.2, 1);

    /* Radii */
    --radius-sm: 6px;
    --radius-md: 10px;
    --radius-lg: 14px;
    --radius-full: 9999px;

    /* Elevation */
    --elev-1: 0 1px 2px oklch(20% 0.02 255 / 0.06);
    --elev-2: 0 4px 10px oklch(20% 0.02 255 / 0.08);
    --elev-3: 0 12px 28px oklch(20% 0.02 255 / 0.12);

    color-scheme: light dark;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --surface-base:   oklch(16% 0.015 255);
      --surface-raised: oklch(20% 0.017 255);
      --surface-sunken: oklch(12% 0.012 255);
      --text-primary:   oklch(96% 0.01 255);
      --text-secondary: oklch(80% 0.015 255);
      --text-muted:     oklch(65% 0.015 255);
    }
  }
}
```

Login form component baseline:

```svelte
<!-- packages/ui-sveltekit/src/components/LoginForm.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { createForm } from '../lib/form.svelte';
  import { loginSchema } from '../lib/validation';
  import PasswordField from './PasswordField.svelte';
  import OAuthButton from './OAuthButton.svelte';

  interface Props {
    providers?: readonly ('google' | 'github' | 'microsoft')[];
    onSubmit: (values: { email: string; password: string }) => Promise<void>;
    header?: Snippet;
    footer?: Snippet;
  }
  let { providers = [], onSubmit, header, footer }: Props = $props();

  const form = createForm({
    schema: loginSchema,
    initial: { email: '', password: '' },
    onSubmit,
  });
</script>

<form {@attach form.attach} novalidate>
  {#if header}{@render header()}{/if}

  {#if providers.length > 0}
    <div class="oauth-grid">
      {#each providers as provider (provider)}
        <OAuthButton {provider} />
      {/each}
    </div>
    <div class="divider" role="separator">or</div>
  {/if}

  <label class="field">
    <span>Email</span>
    <input
      type="email"
      autocomplete="email"
      required
      bind:value={form.values.email}
      aria-invalid={form.errors.email ? 'true' : undefined}
      aria-describedby={form.errors.email ? 'email-err' : undefined}
    />
    {#if form.errors.email}
      <em id="email-err">{form.errors.email}</em>
    {/if}
  </label>

  <PasswordField
    bind:value={form.values.password}
    error={form.errors.password}
    autocomplete="current-password"
  />

  <button type="submit" disabled={form.submitting} aria-busy={form.submitting}>
    {form.submitting ? 'Signing in…' : 'Sign in'}
  </button>

  {#if footer}{@render footer()}{/if}
</form>

<style>
  form {
    display: grid;
    gap: var(--space-sm);
    container-type: inline-size;
  }
  .oauth-grid {
    display: grid;
    gap: var(--space-2xs);
    grid-template-columns: 1fr;
  }
  @container (inline-size > 380px) {
    .oauth-grid { grid-template-columns: repeat(2, 1fr); }
  }
  .field {
    display: grid;
    gap: var(--space-3xs);
    --_field-border: oklch(85% 0.01 255);
    --_field-border-focus: var(--accent-base);

    & input {
      padding-block: var(--space-2xs);
      padding-inline: var(--space-xs);
      border: 1px solid var(--_field-border);
      border-radius: var(--radius-md);
      font-size: var(--type-base);
      background: var(--surface-raised);
      color: var(--text-primary);
      transition: border-color var(--motion-fast) var(--motion-ease);

      &:focus-visible {
        outline: 2px solid var(--_field-border-focus);
        outline-offset: 2px;
        border-color: var(--_field-border-focus);
      }
    }
    & em {
      color: var(--status-danger);
      font-size: var(--type-sm);
      font-style: normal;
    }
  }
  .divider {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: var(--space-xs);
    color: var(--text-muted);
    font-size: var(--type-sm);
    &::before, &::after {
      content: '';
      block-size: 1px;
      background: oklch(90% 0.005 255);
    }
  }
  button[type='submit'] {
    padding-block: var(--space-xs);
    padding-inline: var(--space-md);
    background: var(--accent-base);
    color: var(--accent-text);
    border: 0;
    border-radius: var(--radius-md);
    font-size: var(--type-base);
    font-weight: 600;
    cursor: pointer;
    transition: background var(--motion-fast) var(--motion-ease);

    &:hover:not(:disabled) { background: var(--accent-hover); }
    &:focus-visible { outline: 2px solid var(--accent-base); outline-offset: 2px; }
    &:disabled { opacity: 0.6; cursor: not-allowed; }
  }
</style>
```

This is the quality bar. Every component matches or exceeds it.

---

## 🧪 QUALITY GATES (MUST ALL PASS)

Phase is not complete until:

**Rust:**
- `cargo fmt --check` — zero diff
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — zero warnings
- `cargo test --workspace` — all pass
- `cargo nextest run --workspace` — all pass
- `cargo audit` — zero advisories
- `cargo deny check` — pass

**TypeScript:**
- `pnpm -r typecheck` — zero errors, strict mode on
- `pnpm -r lint` (Biome) — zero errors
- `pnpm -r test` (Vitest) — all pass
- `pnpm build` — all packages build clean
- `pnpm audit --prod` — zero high/critical

**E2E:**
- `pnpm e2e` (Playwright, against `examples/sveltekit-demo`) — all pass
- Scenarios: signup-signin-signout, email verification, password reset, Google OAuth (mocked), passkey register + authenticate, TOTP enable + challenge, magic link flow.

**Docs:**
- Starlight builds without warnings
- All API routes documented
- Every CLI command documented with example
- Every UI component has a usage example

**Repo hygiene:**
- Zero `TODO`, `FIXME`, `XXX`, `HACK` strings (grep-verified)
- Zero `console.log` in shipped code
- Zero `dbg!` in shipped Rust
- Zero `any` in TS (verify via `tsc --noEmit`)
- All packages have `README.md`
- Root `README.md` has quickstart + architecture diagram + contributing link
- `CHANGELOG.md` seeded via changesets

---

## 🚀 EXECUTION PLAN (PHASES)

Execute phases in strict order. Do not proceed to next phase until all gates pass.

**Phase 0 — Scaffold**
- Create monorepo, workspace files, Biome/clippy configs, CI workflow skeletons
- `pnpm install` succeeds, `cargo check` succeeds

**Phase 1 — Rust core**
- Config, state, error, DB pool, Redis pool, tracing
- Health + readiness routes
- First integration test: server starts, `/health` returns 200

**Phase 2 — Schema + migrations**
- All migrations from spec applied cleanly via sqlx-cli
- DB query functions with compile-time checks

**Phase 3 — Email/password auth**
- Signup, signin, signout, session endpoints
- Argon2id hashing, session token generation, cookie handling
- CSRF middleware
- Tests: happy path + enumeration-safe + rate limit

**Phase 4 — Multi-tenancy**
- Apps table, app resolution middleware, origin validation
- Admin API for tenant CRUD
- Tests: tenant isolation (users in app A cannot authenticate in app B)

**Phase 5 — OAuth framework**
- `trait OAuthProvider`, state signing, PKCE
- Google, GitHub, Microsoft, Discord implementations
- Apple implementation wired but `enabled=false` in default config
- Tests: wiremock'd provider flows

**Phase 6 — Email verification + password reset**
- Verification token table, email templates, send flow
- Resend integration (primary), SMTP fallback
- Tests

**Phase 7 — Passkeys (WebAuthn)**
- webauthn-rs integration, registration + authentication ceremonies
- Tests via webauthn-authenticator-rs simulated authenticator

**Phase 8 — TOTP**
- Secret generation, QR URI, verification, recovery codes
- Tests

**Phase 9 — Magic links**
- Token generation, email send, verification → session
- Tests

**Phase 10 — Organizations + RBAC**
- Org CRUD, membership roles, invite flow
- Role check middleware
- Tests

**Phase 11 — Audit log + observability**
- Audit events at all security-relevant endpoints
- OpenTelemetry exporter configured
- Structured tracing across all handlers

**Phase 12 — SDK core (TypeScript)**
- Typed fetch client, error types, zod schemas at boundary
- Full provider coverage
- Vitest coverage

**Phase 13 — SvelteKit adapter**
- `handleAuth` hook, `requireAuth` / `optionalAuth` helpers
- Reactive session store (`.svelte.ts` file with $state)
- Tests

**Phase 14 — CLI skeleton**
- citty command router, clack prompts, framework detection, pnpm enforcement
- `init` command end-to-end, manifest tracking

**Phase 15 — CLI scaffolding templates**
- All SvelteKit templates with handlebars rendering
- Provider templates
- `hooks.server.ts` magicast merger
- Tests: init into a fresh SvelteKit project succeeds, app boots, auth works

**Phase 16 — CLI add/update**
- `add <feature>` commands
- `update` 3-way merge
- Presets
- Tests

**Phase 17 — UI component library**
- All components listed above, each meeting the PE7 standard
- Vitest snapshot + interaction tests via Testing Library
- Accessibility audit (axe-core)

**Phase 18 — Example SvelteKit app**
- Minimal but complete app under `examples/sveltekit-demo/`
- Uses the CLI to scaffold, uses UI components, demonstrates every feature
- Playwright E2E suite proves end-to-end

**Phase 19 — Docs**
- Starlight site with quickstart, concepts, guides, API reference, deployment
- Architecture diagrams (mermaid)
- Provider setup guides

**Phase 20 — Release infrastructure**
- Changesets configured
- GitHub Actions CI: Rust matrix (stable, beta), TS matrix (Node 22, 24)
- Release workflow publishes npm packages + Docker image to GHCR
- Dockerfile distroless, multi-stage, <60MB final image
- `fly.toml` with health checks, autoscaling, secrets guide

**Phase 21 — Polish**
- Root README with architecture diagram, quickstart, feature matrix vs BetterAuth
- CONTRIBUTING.md
- LICENSE (MIT)
- SECURITY.md with disclosure policy
- Screenshot/GIF of the CLI flow

---

## 🧠 WHEN UNCERTAIN — DECISION DEFAULTS

You will NOT ask questions. Use these defaults:

- Naming collision? Prefer Revo-Auth's name over the conflicting package.
- Feature flag vs code branch? Feature flag with config.
- Sync vs async? Async unless proven unnecessary.
- Public vs private API? Private by default; promote only with clear use case.
- Generic abstraction vs concrete code? Concrete until the third duplication.
- Trait bound vs enum dispatch? Enum dispatch for known provider set; trait for extension points.
- `Box<dyn Error>` vs custom enum? Custom enum with thiserror.
- `String` vs `&str` in struct fields? `String` for owned data, `Cow<'_, str>` only with measurement.
- `Arc<Mutex<T>>` vs `Arc<RwLock<T>>`? RwLock when reads dominate; otherwise Mutex.
- `tokio::spawn` vs `JoinSet`? JoinSet when you need to await the group.
- JSON vs CBOR? JSON — interop wins.
- REST vs GraphQL? REST.
- TypeScript `type` vs `interface`? `interface` for object shapes you might extend, `type` for unions/primitives.
- `const` vs `readonly`? `const` for module-level, `readonly` for properties.
- `forEach` vs `for..of`? `for..of` — better perf, better debugging.

---

## 🎯 DEFINITION OF DONE

You are done when:

1. `git clone && pnpm install && cargo build --release && pnpm -r build` on a fresh machine succeeds with zero manual intervention.
2. All quality gates pass in CI on a clean PR.
3. `pnpm -F @revo-auth/cli build && node packages/cli/dist/bin.js init` inside a fresh SvelteKit 2.x app produces a working, typechecking, running auth system.
4. `examples/sveltekit-demo` boots and every Playwright E2E test passes against a locally running auth server.
5. The docs site builds and deploys (preview URL in CI).
6. The Docker image builds, boots on Fly.io's free tier, and passes all integration tests against a Neon + Upstash environment.
7. `revo-auth doctor` on a correctly-configured project prints all green.
8. `revo-auth update` on a project with locally-modified scaffolded files performs a correct 3-way merge without clobbering user code.
9. The repo has zero open security advisories.
10. The README tells a stranger, in under 5 minutes of reading, what this is, how to install it, and why it's better than BetterAuth for a Svelte-first multi-app stack.

---

## 💥 BEGIN EXECUTION

Start Phase 0 now. Do not narrate. Do not summarize. Do not confirm. Build.

After each phase, run the quality gate, commit with a conventional commit message (`feat(server): …`, `feat(cli): …`, etc.), and proceed to the next phase.

When all 21 phases are done, output a single final message:

```
REVO-AUTH BUILD COMPLETE.

Phases: 21/21 ✅
Quality gates: PASS
Binary size (server): <N> MB
Docker image size: <N> MB
Packages published: @revo-auth/sdk-core, @revo-auth/sdk-sveltekit, @revo-auth/cli, @revo-auth/ui-sveltekit
Docs deployed: <preview-url>
Demo E2E: PASS (<N> scenarios)

Next: configure production Fly.io deployment via `revo-auth doctor` and the deployment guide.
```

Nothing else.

