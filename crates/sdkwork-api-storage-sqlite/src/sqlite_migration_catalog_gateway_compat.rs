use super::*;

pub(crate) async fn apply_sqlite_catalog_gateway_compatibility(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_keys_tenant_environment
         ON ai_app_api_keys (tenant_id, environment, active, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_channel",
        "channel_description",
        "channel_description TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_channel",
        "sort_order",
        "sort_order INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_channel",
        "is_builtin",
        "is_builtin INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_channel",
        "is_active",
        "is_active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_channel",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_channel",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider",
        "extension_id",
        "extension_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider",
        "adapter_kind",
        "adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider",
        "base_url",
        "base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider",
        "is_active",
        "is_active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider_channel",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_proxy_provider_channel",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "secret_backend",
        "secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "secret_local_file",
        "secret_local_file TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "secret_keyring_service",
        "secret_keyring_service TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "secret_master_key_id",
        "secret_master_key_id TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "secret_ciphertext",
        "secret_ciphertext TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "secret_key_version",
        "secret_key_version INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_router_credential_records",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model",
        "capabilities_json",
        "capabilities_json TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model",
        "streaming_enabled",
        "streaming_enabled INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model",
        "context_window",
        "context_window INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model",
        "description",
        "description TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "currency_code",
        "currency_code TEXT NOT NULL DEFAULT 'USD'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "price_unit",
        "price_unit TEXT NOT NULL DEFAULT 'per_1m_tokens'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "input_price",
        "input_price REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "output_price",
        "output_price REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "cache_read_price",
        "cache_read_price REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "cache_write_price",
        "cache_write_price REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "request_price",
        "request_price REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "is_active",
        "is_active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_app_api_keys", "raw_key", "raw_key TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_keys",
        "label",
        "label TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_app_api_keys", "notes", "notes TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "description",
        "description TEXT",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_app_api_key_groups", "color", "color TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "default_capability_scope",
        "default_capability_scope TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "default_routing_profile_id",
        "default_routing_profile_id TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "default_accounting_mode",
        "default_accounting_mode TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "active",
        "active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_key_groups",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_keys",
        "api_key_group_id",
        "api_key_group_id TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_keys",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_keys",
        "last_used_at_ms",
        "last_used_at_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_app_api_keys",
        "expires_at_ms",
        "expires_at_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "catalog_proxy_providers",
        "extension_id",
        "extension_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "catalog_proxy_providers",
        "adapter_kind",
        "adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "catalog_proxy_providers",
        "base_url",
        "base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "credential_records",
        "secret_backend",
        "secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "credential_records",
        "secret_local_file",
        "secret_local_file TEXT",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "credential_records",
        "secret_keyring_service",
        "secret_keyring_service TEXT",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "credential_records",
        "secret_master_key_id",
        "secret_master_key_id TEXT",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "credential_records",
        "secret_ciphertext",
        "secret_ciphertext TEXT",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "credential_records",
        "secret_key_version",
        "secret_key_version INTEGER",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "catalog_models",
        "capabilities",
        "capabilities TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "catalog_models",
        "streaming",
        "streaming INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "catalog_models",
        "context_window",
        "context_window INTEGER",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "identity_gateway_api_keys",
        "label",
        "label TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(pool, "identity_gateway_api_keys", "notes", "notes TEXT")
        .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "identity_gateway_api_keys",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "identity_gateway_api_keys",
        "last_used_at_ms",
        "last_used_at_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "identity_gateway_api_keys",
        "expires_at_ms",
        "expires_at_ms INTEGER",
    )
    .await?;

    Ok(())
}
