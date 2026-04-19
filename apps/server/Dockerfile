# syntax=docker/dockerfile:1.7
# Multi-stage, reproducible, distroless build for the Revo-Auth server.
# Final image target: < 60MB, non-root, no shell.

ARG RUST_VERSION=1.83

# ---------- Stage 1: plan dependency graph with cargo-chef ----------
FROM rust:${RUST_VERSION}-bookworm AS planner
WORKDIR /app
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------- Stage 2: build and strip the release binary ----------
FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /app
RUN cargo install cargo-chef --locked
COPY --from=planner /app/recipe.json recipe.json
# Cook only the server crate's deps to maximize layer cache reuse.
RUN cargo chef cook --release --recipe-path recipe.json -p revo-auth-server
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked -p revo-auth-server --bin revo-auth-server \
    && strip target/release/revo-auth-server

# ---------- Stage 3: minimal, non-root runtime ----------
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=builder /app/target/release/revo-auth-server /app/revo-auth-server
COPY --from=builder /app/apps/server/migrations /app/migrations
EXPOSE 8080
USER nonroot
ENV REVO_AUTH_HOST=0.0.0.0 \
    REVO_AUTH_PORT=8080 \
    RUST_LOG=info
ENTRYPOINT ["/app/revo-auth-server"]
