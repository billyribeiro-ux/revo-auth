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
  role        text not null,
  created_at  timestamptz not null default now(),
  unique (org_id, user_id)
);

create table org_invites (
  id          uuid primary key,
  org_id      uuid not null references organizations(id) on delete cascade,
  email       citext not null,
  token_hash  bytea not null unique,
  role        text not null,
  expires_at  timestamptz not null,
  used_at     timestamptz,
  created_at  timestamptz not null default now()
);
create index on org_invites (org_id);
