use super::*;

pub(crate) async fn apply_postgres_catalog_gateway_schema(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_channel (
            channel_id TEXT PRIMARY KEY NOT NULL,
            channel_name TEXT NOT NULL,
            channel_description TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
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
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_primary_channel
         ON ai_proxy_provider (primary_channel_id, is_active, proxy_provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider_channel (
            proxy_provider_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            is_primary BOOLEAN NOT NULL DEFAULT FALSE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (proxy_provider_id, channel_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_channel_channel_provider
         ON ai_proxy_provider_channel (channel_id, proxy_provider_id, is_primary)",
    )
    .execute(&pool)
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
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (tenant_id, proxy_provider_id, key_reference)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_router_credential_records_tenant_updated
         ON ai_router_credential_records (tenant_id, updated_at_ms DESC, proxy_provider_id, key_reference)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_router_credential_records_provider_updated
         ON ai_router_credential_records (proxy_provider_id, updated_at_ms DESC, tenant_id, key_reference)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_official_provider_configs (
            provider_id TEXT PRIMARY KEY NOT NULL,
            key_reference TEXT NOT NULL DEFAULT '',
            base_url TEXT NOT NULL DEFAULT '',
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_official_provider_configs_enabled
         ON ai_official_provider_configs (enabled, provider_id)",
    )
    .execute(&pool)
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
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            routing_tags_json TEXT NOT NULL DEFAULT '[]',
            health_score_hint DOUBLE PRECISION,
            latency_ms_hint BIGINT,
            cost_hint DOUBLE PRECISION,
            success_rate_hint DOUBLE PRECISION,
            throughput_hint DOUBLE PRECISION,
            max_concurrency BIGINT,
            daily_budget DOUBLE PRECISION,
            notes TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_account_provider_active
         ON ai_provider_account (provider_id, enabled, priority DESC, provider_account_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_account_instance
         ON ai_provider_account (execution_instance_id, provider_account_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_model (
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            model_display_name TEXT NOT NULL,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            streaming_enabled BOOLEAN NOT NULL DEFAULT FALSE,
            context_window BIGINT,
            description TEXT NOT NULL DEFAULT '',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (channel_id, model_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_model_streaming
         ON ai_model (model_id, streaming_enabled, channel_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider_model (
            proxy_provider_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            provider_model_id TEXT NOT NULL DEFAULT '',
            provider_model_family TEXT,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            streaming_enabled BOOLEAN NOT NULL DEFAULT FALSE,
            context_window BIGINT,
            max_output_tokens BIGINT,
            supports_prompt_caching BOOLEAN NOT NULL DEFAULT FALSE,
            supports_reasoning_usage BOOLEAN NOT NULL DEFAULT FALSE,
            supports_tool_usage_metrics BOOLEAN NOT NULL DEFAULT FALSE,
            is_default_route BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (proxy_provider_id, channel_id, model_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_model_channel_active
         ON ai_proxy_provider_model (channel_id, model_id, is_active, proxy_provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_model_provider_active
         ON ai_proxy_provider_model (proxy_provider_id, is_active, channel_id, model_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_model_price (
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            proxy_provider_id TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            price_unit TEXT NOT NULL DEFAULT 'per_1m_tokens',
            input_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            output_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            cache_read_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            cache_write_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            request_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            price_source_kind TEXT NOT NULL DEFAULT 'reference',
            billing_notes TEXT,
            pricing_tiers_json TEXT NOT NULL DEFAULT '[]',
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (channel_id, model_id, proxy_provider_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_provider_active
         ON ai_model_price (proxy_provider_id, is_active, channel_id, model_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_channel_active
         ON ai_model_price (channel_id, model_id, is_active, proxy_provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_model_active
         ON ai_model_price (model_id, is_active, channel_id, proxy_provider_id)",
    )
    .execute(&pool)
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
            active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_app_api_key_groups_workspace_slug
         ON ai_app_api_key_groups (tenant_id, project_id, environment, slug)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_key_groups_workspace_active
         ON ai_app_api_key_groups (tenant_id, project_id, environment, active, created_at_ms DESC)",
    )
    .execute(&pool)
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
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            last_used_at_ms BIGINT,
            expires_at_ms BIGINT,
            active BOOLEAN NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS channel_description TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS sort_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS is_builtin BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS extension_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS protocol_kind TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_channel ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_channel ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_local_file TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_keyring_service TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_master_key_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_ciphertext TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_key_version INTEGER",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_official_provider_configs ADD COLUMN IF NOT EXISTS key_reference TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_official_provider_configs ADD COLUMN IF NOT EXISTS base_url TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_official_provider_configs ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_official_provider_configs ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_official_provider_configs ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS capabilities_json TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS streaming_enabled BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS context_window BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS provider_model_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS provider_model_family TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS capabilities_json TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS streaming_enabled BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS context_window BIGINT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS max_output_tokens BIGINT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS supports_prompt_caching BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS supports_reasoning_usage BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS supports_tool_usage_metrics BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS is_default_route BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_model ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'USD'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS price_unit TEXT NOT NULL DEFAULT 'per_1m_tokens'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS input_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS output_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS cache_read_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS cache_write_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS request_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS price_source_kind TEXT NOT NULL DEFAULT 'reference'",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS billing_notes TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS pricing_tiers_json TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS raw_key TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS api_key_group_id TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS description TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS color TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS default_capability_scope TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS default_routing_profile_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS default_accounting_mode TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS notes TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS last_used_at_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS expires_at_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_keys_project_active
         ON ai_app_api_keys (project_id, active, created_at_ms DESC, hashed_key)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_keys_tenant_environment
         ON ai_app_api_keys (tenant_id, environment, active, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_proxy_providers",
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS extension_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_proxy_providers",
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_proxy_providers",
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_local_file TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_keyring_service TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_master_key_id TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_ciphertext TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_key_version INTEGER",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_models",
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS capabilities TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_models",
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS streaming BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_models",
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS context_window BIGINT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS notes TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS last_used_at_ms BIGINT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS expires_at_ms BIGINT",
    )
    .await?;
    Ok(())
}
