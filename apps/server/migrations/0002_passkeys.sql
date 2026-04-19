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
