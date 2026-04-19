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
