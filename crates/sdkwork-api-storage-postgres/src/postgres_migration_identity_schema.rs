use super::*;

pub(crate) async fn apply_postgres_identity_schema(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_portal_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
            workspace_tenant_id TEXT NOT NULL DEFAULT '',
            workspace_project_id TEXT NOT NULL DEFAULT '',
            active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS display_name TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS password_salt TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS workspace_tenant_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS workspace_project_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_portal_users_email ON ai_portal_users (email)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_admin_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT 'super_admin',
            active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS display_name TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS password_salt TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'super_admin'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_admin_users_email ON ai_admin_users (email)",
    )
    .execute(&pool)
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
            recorded_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_admin_audit_events_recorded
         ON ai_admin_audit_events (recorded_at_ms DESC, event_id DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_tenants (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_projects (
            id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_user (
            user_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            external_user_ref TEXT,
            username TEXT,
            display_name TEXT,
            email TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_user_scope
         ON ai_user (tenant_id, organization_id, user_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_user_email
         ON ai_user (tenant_id, organization_id, email)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_api_key (
            api_key_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            user_id BIGINT NOT NULL,
            key_prefix TEXT NOT NULL DEFAULT '',
            key_hash TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'active',
            expires_at_ms BIGINT,
            last_used_at_ms BIGINT,
            rotated_from_api_key_id BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_api_key_hash
         ON ai_api_key (key_hash)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_api_key_user_status
         ON ai_api_key (tenant_id, organization_id, user_id, status)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_identity_binding (
            identity_binding_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            user_id BIGINT NOT NULL,
            binding_type TEXT NOT NULL,
            issuer TEXT,
            subject TEXT,
            platform TEXT,
            owner TEXT,
            external_ref TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_identity_binding_lookup
         ON ai_identity_binding (tenant_id, organization_id, binding_type, issuer, subject, status)",
    )
    .execute(&pool)
    .await?;
    Ok(())
}
