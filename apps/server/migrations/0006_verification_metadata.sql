alter table verification_tokens
  add column if not exists metadata jsonb not null default '{}'::jsonb;
