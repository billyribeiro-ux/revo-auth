# Contributing to Revo-Auth

Thanks for helping improve Revo-Auth. This document covers the workflow we expect for
bugs, features, and docs changes. The full architectural contract lives in
[`revo-auth-nuclear-prompt.md`](./revo-auth-nuclear-prompt.md) - please skim it before
opening a substantive PR.

## Ground rules

- Be respectful; follow the Contributor Covenant when interacting on issues and PRs.
- No secrets in commits. `.env*`, private keys, and tokens are git-ignored.
- Security issues go to `security@revo-auth.dev`, not public issues. See
  [`SECURITY.md`](./SECURITY.md).

## Local setup

Prerequisites:

- Node `22+` and `pnpm@10`
- Rust `1.83+` (pinned via `rust-toolchain.toml`)
- Docker (for Postgres + Redis and the distroless image build)

Bootstrap the workspace:

```bash
pnpm install
cargo build --workspace
```

Bring up the services the auth server expects:

```bash
docker compose up -d postgres redis
cp apps/server/.env.example apps/server/.env
```

Run the server:

```bash
cd apps/server && cargo run --release
```

## Tests

- TypeScript: `pnpm test` (vitest) and `pnpm -F sveltekit-demo exec playwright test`.
- Rust: `cargo nextest run` (preferred) or `cargo test`.
- Lint: `pnpm lint` (Biome) and `cargo clippy --workspace --all-targets -- -D warnings`.
- Format: `pnpm dlx @biomejs/biome format --write .` and `cargo fmt --all`.

All of these run in CI on both `main` and every PR - merges are blocked until they pass.

## Commit style

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(sdk-sveltekit): add passkey login action
fix(server): reject reused refresh tokens
docs(readme): clarify quickstart
```

Scopes mirror the workspace: `server`, `sdk-core`, `sdk-sveltekit`, `ui-sveltekit`,
`cli`, `docs`, `examples`, `ci`.

## Changesets

Every user-visible change to a published package requires a changeset:

```bash
pnpm changeset
```

Pick the affected packages, choose `patch`/`minor`/`major`, and write a changelog line
in the imperative mood. The `release` workflow turns merged changesets into tagged
releases.

## Pull request checklist

Before requesting review, confirm:

- [ ] Tests added or updated (unit + integration where relevant).
- [ ] `pnpm test` and `cargo nextest run` pass locally.
- [ ] `pnpm lint` and `cargo clippy -- -D warnings` are clean.
- [ ] Docs updated (`apps/docs`, package READMEs, or root README).
- [ ] Changeset added for any package change that ships to npm.
- [ ] No stray `TODO`, `FIXME`, `XXX`, or `dbg!` markers.
- [ ] No new runtime dependency without a license check (`cargo deny check licenses`).

## Code style

- TypeScript / Svelte: [Biome](https://biomejs.dev) for format + lint. Svelte 5 runes
  only - no legacy stores for new code.
- Rust: `rustfmt` (config in `rustfmt.toml`) and `clippy` with `-D warnings`. Prefer
  `thiserror` for library errors and `anyhow` only at the binary edge.
- SQL: migrations live in `apps/server/migrations` and are append-only.

## Release flow

1. Merge PRs with changesets.
2. The `release` workflow opens/maintains a "Version Packages" PR.
3. Merging the version PR publishes to npm and pushes the distroless server image to
   `ghcr.io/<owner>/revo-auth-server`.

Thanks again for contributing.
