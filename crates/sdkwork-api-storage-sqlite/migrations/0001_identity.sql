CREATE TABLE IF NOT EXISTS identity_users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    password_salt TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL DEFAULT '',
    workspace_tenant_id TEXT NOT NULL DEFAULT '',
    workspace_project_id TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1,
    created_at_ms INTEGER NOT NULL DEFAULT 0
);
