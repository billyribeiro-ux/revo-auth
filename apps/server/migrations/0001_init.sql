create extension if not exists citext;
create extension if not exists pgcrypto;

create table apps (
  id              uuid primary key,
  slug            text unique not null,
  name            text not null,
  origins         text[] not null default '{}',
  public_key      text not null,
  secret_key_hash text not null,
  settings        jsonb not null default '{}'::jsonb,
  created_at      timestamptz not null default now(),
  updated_at      timestamptz not null default now()
);

create table users (
  id                uuid primary key,
  app_id            uuid not null references apps(id) on delete cascade,
  email             citext,
  email_verified_at timestamptz,
  password_hash     text,
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
  provider          text not null,
  provider_account  text not null,
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
  token_hash      bytea not null unique,
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
  kind        text not null,
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
