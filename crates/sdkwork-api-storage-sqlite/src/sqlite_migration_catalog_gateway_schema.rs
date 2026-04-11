use super::*;

pub(crate) async fn apply_sqlite_catalog_gateway_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_channel (
            channel_id TEXT PRIMARY KEY NOT NULL,
            channel_name TEXT NOT NULL,
            channel_description TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider (
            proxy_provider_id TEXT PRIMARY KEY NOT NULL,
            primary_channel_id TEXT NOT NULL,
            extension_id TEXT NOT NULL DEFAULT '',
            adapter_kind TEXT NOT NULL DEFAULT 'openai',
            protocol_kind TEXT NOT NULL DEFAULT '',
            base_url TEXT NOT NULL DEFAULT 'http://localhost',
            display_name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_primary_channel
         ON ai_proxy_provider (primary_channel_id, is_active, proxy_provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider_channel (
            proxy_provider_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (proxy_provider_id, channel_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_channel_channel_provider
         ON ai_proxy_provider_channel (channel_id, proxy_provider_id, is_primary)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_router_credential_records (
            tenant_id TEXT NOT NULL,
            proxy_provider_id TEXT NOT NULL,
            key_reference TEXT NOT NULL,
            secret_backend TEXT NOT NULL DEFAULT 'database_encrypted',
            secret_local_file TEXT,
            secret_keyring_service TEXT,
            secret_master_key_id TEXT,
            secret_ciphertext TEXT,
            secret_key_version INTEGER,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (tenant_id, proxy_provider_id, key_reference)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_router_credential_records_tenant_updated
         ON ai_router_credential_records (tenant_id, updated_at_ms DESC, proxy_provider_id, key_reference)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_router_credential_records_provider_updated
         ON ai_router_credential_records (proxy_provider_id, updated_at_ms DESC, tenant_id, key_reference)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_official_provider_configs (
            provider_id TEXT PRIMARY KEY NOT NULL,
            key_reference TEXT NOT NULL DEFAULT '',
            base_url TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_official_provider_configs_enabled
         ON ai_official_provider_configs (enabled, provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_provider_account (
            provider_account_id TEXT PRIMARY KEY NOT NULL,
            provider_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            account_kind TEXT NOT NULL DEFAULT 'api_key',
            owner_scope TEXT NOT NULL DEFAULT 'platform',
            owner_tenant_id TEXT,
            execution_instance_id TEXT NOT NULL,
            base_url_override TEXT,
            region TEXT,
            priority INTEGER NOT NULL DEFAULT 0,
            weight INTEGER NOT NULL DEFAULT 1,
            enabled INTEGER NOT NULL DEFAULT 1,
            routing_tags_json TEXT NOT NULL DEFAULT '[]',
            health_score_hint REAL,
            latency_ms_hint INTEGER,
            cost_hint REAL,
            success_rate_hint REAL,
            throughput_hint REAL,
            max_concurrency INTEGER,
            daily_budget REAL,
            notes TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_account_provider_active
         ON ai_provider_account (provider_id, enabled, priority DESC, provider_account_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_account_instance
         ON ai_provider_account (execution_instance_id, provider_account_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_model (
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            model_display_name TEXT NOT NULL,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            streaming_enabled INTEGER NOT NULL DEFAULT 0,
            context_window INTEGER,
            description TEXT NOT NULL DEFAULT '',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (channel_id, model_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_model_streaming
         ON ai_model (model_id, streaming_enabled, channel_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider_model (
            proxy_provider_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            provider_model_id TEXT NOT NULL DEFAULT '',
            provider_model_family TEXT,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            streaming_enabled INTEGER NOT NULL DEFAULT 0,
            context_window INTEGER,
            max_output_tokens INTEGER,
            supports_prompt_caching INTEGER NOT NULL DEFAULT 0,
            supports_reasoning_usage INTEGER NOT NULL DEFAULT 0,
            supports_tool_usage_metrics INTEGER NOT NULL DEFAULT 0,
            is_default_route INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (proxy_provider_id, channel_id, model_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_model_channel_active
         ON ai_proxy_provider_model (channel_id, model_id, is_active, proxy_provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_model_provider_active
         ON ai_proxy_provider_model (proxy_provider_id, is_active, channel_id, model_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_model_price (
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            proxy_provider_id TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            price_unit TEXT NOT NULL DEFAULT 'per_1m_tokens',
            input_price REAL NOT NULL DEFAULT 0,
            output_price REAL NOT NULL DEFAULT 0,
            cache_read_price REAL NOT NULL DEFAULT 0,
            cache_write_price REAL NOT NULL DEFAULT 0,
            request_price REAL NOT NULL DEFAULT 0,
            price_source_kind TEXT NOT NULL DEFAULT 'reference',
            billing_notes TEXT,
            pricing_tiers_json TEXT NOT NULL DEFAULT '[]',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (channel_id, model_id, proxy_provider_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_provider_active
         ON ai_model_price (proxy_provider_id, is_active, channel_id, model_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_channel_active
         ON ai_model_price (channel_id, model_id, is_active, proxy_provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_model_active
         ON ai_model_price (model_id, is_active, channel_id, proxy_provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_app_api_key_groups (
            group_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            environment TEXT NOT NULL,
            name TEXT NOT NULL,
            slug TEXT NOT NULL,
            description TEXT,
            color TEXT,
            default_capability_scope TEXT,
            default_routing_profile_id TEXT,
            default_accounting_mode TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_app_api_key_groups_workspace_slug
         ON ai_app_api_key_groups (tenant_id, project_id, environment, slug)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_key_groups_workspace_active
         ON ai_app_api_key_groups (tenant_id, project_id, environment, active, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_app_api_keys (
            hashed_key TEXT PRIMARY KEY NOT NULL,
            raw_key TEXT,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            environment TEXT NOT NULL,
            api_key_group_id TEXT,
            label TEXT NOT NULL DEFAULT '',
            notes TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            last_used_at_ms INTEGER,
            expires_at_ms INTEGER,
            active INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_keys_project_active
         ON ai_app_api_keys (project_id, active, created_at_ms DESC, hashed_key)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
