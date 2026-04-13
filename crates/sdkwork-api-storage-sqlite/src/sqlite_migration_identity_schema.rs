use super::*;

pub(crate) async fn apply_sqlite_identity_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_portal_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
            workspace_tenant_id TEXT NOT NULL DEFAULT '',
            workspace_project_id TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "display_name",
        "display_name TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "password_salt",
        "password_salt TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "password_hash",
        "password_hash TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "workspace_tenant_id",
        "workspace_tenant_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "workspace_project_id",
        "workspace_project_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "active",
        "active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_portal_users",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_portal_users_email ON ai_portal_users (email)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_admin_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT 'super_admin',
            active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_admin_users",
        "display_name",
        "display_name TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_admin_users",
        "password_salt",
        "password_salt TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_admin_users",
        "password_hash",
        "password_hash TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_admin_users",
        "role",
        "role TEXT NOT NULL DEFAULT 'super_admin'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_admin_users",
        "active",
        "active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_admin_users",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_admin_users_email ON ai_admin_users (email)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_admin_audit_events (
            event_id TEXT PRIMARY KEY NOT NULL,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            approval_scope TEXT NOT NULL,
            actor_user_id TEXT NOT NULL,
            actor_email TEXT NOT NULL,
            actor_role TEXT NOT NULL,
            recorded_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_admin_audit_events_recorded
         ON ai_admin_audit_events (recorded_at_ms DESC, event_id DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_tenants (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_projects (
            id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_user (
            user_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            external_user_ref TEXT,
            username TEXT,
            display_name TEXT,
            email TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_user_scope
         ON ai_user (tenant_id, organization_id, user_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_user_email
         ON ai_user (tenant_id, organization_id, email)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_api_key (
            api_key_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER NOT NULL,
            key_prefix TEXT NOT NULL DEFAULT '',
            key_hash TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'active',
            expires_at_ms INTEGER,
            last_used_at_ms INTEGER,
            rotated_from_api_key_id INTEGER,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_api_key_hash
         ON ai_api_key (key_hash)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_api_key_user_status
         ON ai_api_key (tenant_id, organization_id, user_id, status)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_identity_binding (
            identity_binding_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER NOT NULL,
            binding_type TEXT NOT NULL,
            issuer TEXT,
            subject TEXT,
            platform TEXT,
            owner TEXT,
            external_ref TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_identity_binding_lookup
         ON ai_identity_binding (tenant_id, organization_id, binding_type, issuer, subject, status)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
