create index if not exists audit_log_event_created_idx on audit_log (event, created_at desc);
