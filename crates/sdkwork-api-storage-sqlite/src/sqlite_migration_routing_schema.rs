use super::*;

pub(crate) async fn apply_sqlite_routing_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            capability TEXT NOT NULL,
            model_pattern TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            priority INTEGER NOT NULL DEFAULT 0,
            strategy TEXT NOT NULL DEFAULT 'deterministic_priority',
            default_provider_id TEXT,
            execution_failover_enabled INTEGER NOT NULL DEFAULT 1,
            upstream_retry_max_attempts INTEGER,
            upstream_retry_base_delay_ms INTEGER,
            upstream_retry_max_delay_ms INTEGER
        )",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "strategy",
        "strategy TEXT NOT NULL DEFAULT 'deterministic_priority'",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_routing_policies", "max_cost", "max_cost REAL").await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "max_latency_ms",
        "max_latency_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "require_healthy",
        "require_healthy INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "execution_failover_enabled",
        "execution_failover_enabled INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "upstream_retry_max_attempts",
        "upstream_retry_max_attempts INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "upstream_retry_base_delay_ms",
        "upstream_retry_base_delay_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_policies",
        "upstream_retry_max_delay_ms",
        "upstream_retry_max_delay_ms INTEGER",
    )
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_policy_providers (
            policy_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            PRIMARY KEY (policy_id, provider_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_policy_providers_policy_position
         ON ai_routing_policy_providers (policy_id, position, provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_policy_providers_provider_position
         ON ai_routing_policy_providers (provider_id, position, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_policies_capability_priority
         ON ai_routing_policies (capability, enabled, priority DESC, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_profiles (
            profile_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            slug TEXT NOT NULL,
            description TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            strategy TEXT NOT NULL DEFAULT 'deterministic_priority',
            ordered_provider_ids_json TEXT NOT NULL DEFAULT '[]',
            default_provider_id TEXT,
            max_cost REAL,
            max_latency_ms INTEGER,
            require_healthy INTEGER NOT NULL DEFAULT 0,
            preferred_region TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_routing_profiles_workspace_slug
         ON ai_routing_profiles (tenant_id, project_id, slug)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_profiles_workspace_active
         ON ai_routing_profiles (tenant_id, project_id, active, updated_at_ms DESC, profile_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_compiled_routing_snapshots (
            snapshot_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT,
            project_id TEXT,
            api_key_group_id TEXT,
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            matched_policy_id TEXT,
            project_routing_preferences_project_id TEXT,
            applied_routing_profile_id TEXT,
            strategy TEXT NOT NULL DEFAULT '',
            ordered_provider_ids_json TEXT NOT NULL DEFAULT '[]',
            default_provider_id TEXT,
            max_cost REAL,
            max_latency_ms INTEGER,
            require_healthy INTEGER NOT NULL DEFAULT 0,
            preferred_region TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_compiled_routing_snapshots_scope_updated_at
         ON ai_compiled_routing_snapshots (tenant_id, project_id, api_key_group_id, updated_at_ms DESC, snapshot_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_project_routing_preferences (
            project_id TEXT PRIMARY KEY NOT NULL,
            preset_id TEXT NOT NULL DEFAULT '',
            strategy TEXT NOT NULL DEFAULT 'deterministic_priority',
            ordered_provider_ids_json TEXT NOT NULL DEFAULT '[]',
            default_provider_id TEXT,
            max_cost REAL,
            max_latency_ms INTEGER,
            require_healthy INTEGER NOT NULL DEFAULT 0,
            preferred_region TEXT,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_decision_logs (
            decision_id TEXT PRIMARY KEY NOT NULL,
            decision_source TEXT NOT NULL,
            tenant_id TEXT,
            project_id TEXT,
            api_key_group_id TEXT,
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            selected_provider_id TEXT NOT NULL,
            matched_policy_id TEXT,
            applied_routing_profile_id TEXT,
            compiled_routing_snapshot_id TEXT,
            strategy TEXT NOT NULL,
            selection_seed INTEGER,
            selection_reason TEXT,
            fallback_reason TEXT,
            requested_region TEXT,
            slo_applied INTEGER NOT NULL DEFAULT 0,
            slo_degraded INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL,
            assessments_json TEXT NOT NULL DEFAULT '[]'
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_decision_logs_project_created_at
         ON ai_routing_decision_logs (project_id, created_at_ms DESC, decision_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_decision_logs_provider_created_at
         ON ai_routing_decision_logs (selected_provider_id, created_at_ms DESC, decision_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_decision_logs_capability_created_at
         ON ai_routing_decision_logs (capability, created_at_ms DESC, decision_id)",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_decision_logs",
        "requested_region",
        "requested_region TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_decision_logs",
        "api_key_group_id",
        "api_key_group_id TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_decision_logs",
        "applied_routing_profile_id",
        "applied_routing_profile_id TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_decision_logs",
        "compiled_routing_snapshot_id",
        "compiled_routing_snapshot_id TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_routing_decision_logs",
        "fallback_reason",
        "fallback_reason TEXT",
    )
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_provider_health_records (
            provider_id TEXT NOT NULL,
            extension_id TEXT NOT NULL,
            runtime TEXT NOT NULL,
            observed_at_ms INTEGER NOT NULL,
            instance_id TEXT,
            running INTEGER NOT NULL DEFAULT 0,
            healthy INTEGER NOT NULL DEFAULT 0,
            message TEXT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_health_records_provider_observed_at
         ON ai_provider_health_records (provider_id, observed_at_ms DESC, runtime)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_health_records_extension_runtime_observed_at
         ON ai_provider_health_records (extension_id, runtime, observed_at_ms DESC, provider_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
