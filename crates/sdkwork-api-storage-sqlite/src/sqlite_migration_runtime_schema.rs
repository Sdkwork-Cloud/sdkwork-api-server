use super::*;

pub(crate) async fn apply_sqlite_runtime_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_installations (
            installation_id TEXT PRIMARY KEY NOT NULL,
            extension_id TEXT NOT NULL,
            runtime TEXT NOT NULL,
            enabled INTEGER NOT NULL,
            entrypoint TEXT,
            config_json TEXT NOT NULL DEFAULT '{}'
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_instances (
            instance_id TEXT PRIMARY KEY NOT NULL,
            installation_id TEXT NOT NULL,
            extension_id TEXT NOT NULL,
            enabled INTEGER NOT NULL,
            base_url TEXT,
            credential_ref TEXT,
            config_json TEXT NOT NULL DEFAULT '{}'
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_service_runtime_nodes (
            node_id TEXT PRIMARY KEY NOT NULL,
            service_kind TEXT NOT NULL,
            started_at_ms INTEGER NOT NULL,
            last_seen_at_ms INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_service_runtime_nodes_last_seen
         ON ai_service_runtime_nodes (last_seen_at_ms DESC, node_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_runtime_rollouts (
            rollout_id TEXT PRIMARY KEY NOT NULL,
            scope TEXT NOT NULL,
            requested_extension_id TEXT,
            requested_instance_id TEXT,
            resolved_extension_id TEXT,
            created_by TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            deadline_at_ms INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_extension_runtime_rollouts_created_at
         ON ai_extension_runtime_rollouts (created_at_ms DESC, rollout_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_runtime_rollout_participants (
            rollout_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            service_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT,
            updated_at_ms INTEGER NOT NULL,
            PRIMARY KEY (rollout_id, node_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_extension_runtime_rollout_participants_node_status
         ON ai_extension_runtime_rollout_participants (node_id, status, updated_at_ms, rollout_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_extension_runtime_rollout_participants_rollout
         ON ai_extension_runtime_rollout_participants (rollout_id, node_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_standalone_config_rollouts (
            rollout_id TEXT PRIMARY KEY NOT NULL,
            requested_service_kind TEXT,
            created_by TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            deadline_at_ms INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_standalone_config_rollouts_created_at
         ON ai_standalone_config_rollouts (created_at_ms DESC, rollout_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_standalone_config_rollout_participants (
            rollout_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            service_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT,
            updated_at_ms INTEGER NOT NULL,
            PRIMARY KEY (rollout_id, node_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_standalone_config_rollout_participants_node_status
         ON ai_standalone_config_rollout_participants (node_id, status, updated_at_ms, rollout_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_standalone_config_rollout_participants_rollout
         ON ai_standalone_config_rollout_participants (rollout_id, node_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
