use super::*;

use sdkwork_api_domain_marketing::{
    CouponBenefitSpec, CouponRestrictionSpec, MarketingBenefitKind,
};

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
        "protocol_kind",
        "protocol_kind TEXT NOT NULL DEFAULT ''",
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
    ensure_sqlite_column(pool, "ai_model", "context_window", "context_window INTEGER").await?;
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
        "price_source_kind",
        "price_source_kind TEXT NOT NULL DEFAULT 'reference'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "billing_notes",
        "billing_notes TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_model_price",
        "pricing_tiers_json",
        "pricing_tiers_json TEXT NOT NULL DEFAULT '[]'",
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
    migrate_sqlite_legacy_table_with_common_columns(pool, "identity_users", "ai_portal_users")
        .await?;
    migrate_sqlite_legacy_table_with_common_columns(pool, "admin_users", "ai_admin_users").await?;
    migrate_sqlite_legacy_table_with_common_columns(pool, "tenant_records", "ai_tenants").await?;
    migrate_sqlite_legacy_table_with_common_columns(pool, "tenant_projects", "ai_projects").await?;
    migrate_sqlite_coupon_campaigns(pool).await?;
    migrate_sqlite_catalog_channels(pool).await?;
    migrate_sqlite_catalog_proxy_providers(pool).await?;
    migrate_sqlite_catalog_provider_channel_bindings(pool).await?;
    migrate_sqlite_credential_records(pool).await?;
    migrate_sqlite_catalog_models(pool).await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "routing_policies",
        "ai_routing_policies",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "routing_policy_providers",
        "ai_routing_policy_providers",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "project_routing_preferences",
        "ai_project_routing_preferences",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "routing_decision_logs",
        "ai_routing_decision_logs",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "routing_provider_health",
        "ai_provider_health_records",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(pool, "usage_records", "ai_usage_records")
        .await?;
    migrate_sqlite_legacy_table_with_common_columns(pool, "billing_events", "ai_billing_events")
        .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "billing_ledger_entries",
        "ai_billing_ledger_entries",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "billing_quota_policies",
        "ai_billing_quota_policies",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "identity_gateway_api_keys",
        "ai_app_api_keys",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "extension_installations",
        "ai_extension_installations",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "extension_instances",
        "ai_extension_instances",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "service_runtime_nodes",
        "ai_service_runtime_nodes",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "extension_runtime_rollouts",
        "ai_extension_runtime_rollouts",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "extension_runtime_rollout_participants",
        "ai_extension_runtime_rollout_participants",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "standalone_config_rollouts",
        "ai_standalone_config_rollouts",
    )
    .await?;
    migrate_sqlite_legacy_table_with_common_columns(
        pool,
        "standalone_config_rollout_participants",
        "ai_standalone_config_rollout_participants",
    )
    .await?;
    create_sqlite_compatibility_views(pool).await?;

    Ok(())
}

async fn migrate_sqlite_catalog_channels(pool: &SqlitePool) -> Result<()> {
    if sqlite_object_type(pool, "catalog_channels")
        .await?
        .as_deref()
        != Some("table")
    {
        return Ok(());
    }

    sqlx::query(
        "INSERT OR IGNORE INTO ai_channel (
            channel_id,
            channel_name,
            channel_description,
            sort_order,
            is_builtin,
            is_active,
            created_at_ms,
            updated_at_ms
        )
        SELECT
            id,
            name,
            '',
            0,
            0,
            1,
            0,
            0
        FROM catalog_channels",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE catalog_channels")
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate_sqlite_catalog_proxy_providers(pool: &SqlitePool) -> Result<()> {
    if sqlite_object_type(pool, "catalog_proxy_providers")
        .await?
        .as_deref()
        != Some("table")
    {
        return Ok(());
    }

    let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT id, channel_id, extension_id, adapter_kind, base_url, display_name
         FROM catalog_proxy_providers",
    )
    .fetch_all(pool)
    .await?;

    for (provider_id, channel_id, extension_id, adapter_kind, base_url, display_name) in rows {
        sqlx::query(
            "INSERT OR IGNORE INTO ai_proxy_provider (
                proxy_provider_id,
                primary_channel_id,
                extension_id,
                adapter_kind,
                protocol_kind,
                base_url,
                display_name,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(provider_id)
        .bind(channel_id)
        .bind(normalize_provider_extension_id(extension_id, &adapter_kind))
        .bind(adapter_kind.clone())
        .bind(normalize_provider_protocol_kind("", &adapter_kind))
        .bind(base_url)
        .bind(display_name)
        .bind(1_i64)
        .bind(0_i64)
        .bind(0_i64)
        .execute(pool)
        .await?;
    }

    sqlx::query("DROP TABLE catalog_proxy_providers")
        .execute(pool)
        .await?;
    Ok(())
}

async fn migrate_sqlite_catalog_provider_channel_bindings(pool: &SqlitePool) -> Result<()> {
    if sqlite_object_type(pool, "catalog_provider_channel_bindings")
        .await?
        .as_deref()
        != Some("table")
    {
        return Ok(());
    }

    sqlx::query(
        "INSERT OR IGNORE INTO ai_proxy_provider_channel (
            proxy_provider_id,
            channel_id,
            is_primary,
            created_at_ms,
            updated_at_ms
        )
        SELECT
            provider_id,
            channel_id,
            is_primary,
            0,
            0
        FROM catalog_provider_channel_bindings",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE catalog_provider_channel_bindings")
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate_sqlite_credential_records(pool: &SqlitePool) -> Result<()> {
    if sqlite_object_type(pool, "credential_records")
        .await?
        .as_deref()
        != Some("table")
    {
        return Ok(());
    }

    sqlx::query(
        "INSERT OR IGNORE INTO ai_router_credential_records (
            tenant_id,
            proxy_provider_id,
            key_reference,
            secret_backend,
            secret_local_file,
            secret_keyring_service,
            secret_master_key_id,
            secret_ciphertext,
            secret_key_version,
            created_at_ms,
            updated_at_ms
        )
        SELECT
            tenant_id,
            provider_id,
            key_reference,
            secret_backend,
            secret_local_file,
            secret_keyring_service,
            secret_master_key_id,
            secret_ciphertext,
            secret_key_version,
            0,
            0
        FROM credential_records",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE credential_records")
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate_sqlite_catalog_models(pool: &SqlitePool) -> Result<()> {
    if sqlite_object_type(pool, "catalog_models").await?.as_deref() != Some("table") {
        return Ok(());
    }

    sqlx::query(
        "INSERT OR IGNORE INTO ai_model (
            channel_id,
            model_id,
            model_display_name,
            capabilities_json,
            streaming_enabled,
            context_window,
            description,
            created_at_ms,
            updated_at_ms
        )
        SELECT
            providers.primary_channel_id,
            legacy.external_name,
            legacy.external_name,
            legacy.capabilities,
            legacy.streaming,
            legacy.context_window,
            '',
            0,
            0
        FROM catalog_models legacy
        INNER JOIN ai_proxy_provider providers
            ON providers.proxy_provider_id = legacy.provider_id",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO ai_proxy_provider_model (
            proxy_provider_id,
            channel_id,
            model_id,
            provider_model_id,
            provider_model_family,
            capabilities_json,
            streaming_enabled,
            context_window,
            max_output_tokens,
            supports_prompt_caching,
            supports_reasoning_usage,
            supports_tool_usage_metrics,
            is_default_route,
            is_active,
            created_at_ms,
            updated_at_ms
        )
        SELECT
            legacy.provider_id,
            providers.primary_channel_id,
            legacy.external_name,
            legacy.external_name,
            NULL,
            legacy.capabilities,
            legacy.streaming,
            legacy.context_window,
            NULL,
            0,
            0,
            0,
            1,
            1,
            0,
            0
        FROM catalog_models legacy
        INNER JOIN ai_proxy_provider providers
            ON providers.proxy_provider_id = legacy.provider_id",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO ai_model_price (
            channel_id,
            model_id,
            proxy_provider_id,
            currency_code,
            price_unit,
            input_price,
            output_price,
            cache_read_price,
            cache_write_price,
            request_price,
            price_source_kind,
            billing_notes,
            pricing_tiers_json,
            is_active,
            created_at_ms,
            updated_at_ms
        )
        SELECT
            providers.primary_channel_id,
            legacy.external_name,
            legacy.provider_id,
            'USD',
            'per_1m_tokens',
            0,
            0,
            0,
            0,
            0,
            'reference',
            NULL,
            '[]',
            1,
            0,
            0
        FROM catalog_models legacy
        INNER JOIN ai_proxy_provider providers
            ON providers.proxy_provider_id = legacy.provider_id",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE catalog_models")
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate_sqlite_coupon_campaigns(pool: &SqlitePool) -> Result<()> {
    if sqlite_object_type(pool, "coupon_campaigns")
        .await?
        .as_deref()
        != Some("table")
    {
        return Ok(());
    }

    let columns = sqlite_table_columns(pool, "coupon_campaigns").await?;
    if sqlite_coupon_campaign_projection_columns(&columns) {
        migrate_sqlite_coupon_campaign_projection_table(pool).await?;
        return Ok(());
    }

    if sqlite_legacy_coupon_campaign_columns(&columns) {
        migrate_sqlite_legacy_coupon_campaign_rows(pool).await?;
    }

    Ok(())
}

fn sqlite_coupon_campaign_projection_columns(columns: &[String]) -> bool {
    sqlite_columns_include(
        columns,
        &[
            "id",
            "coupon_template_id",
            "status",
            "created_at_ms",
            "updated_at_ms",
            "record_json",
        ],
    )
}

fn sqlite_legacy_coupon_campaign_columns(columns: &[String]) -> bool {
    sqlite_columns_include(columns, &["id", "code", "discount_label", "active"])
}

fn sqlite_columns_include(columns: &[String], required: &[&str]) -> bool {
    required
        .iter()
        .all(|name| columns.iter().any(|column| column == name))
}

async fn migrate_sqlite_coupon_campaign_projection_table(pool: &SqlitePool) -> Result<()> {
    ensure_sqlite_column_if_table_exists(
        pool,
        "coupon_campaigns",
        "start_at_ms",
        "start_at_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "coupon_campaigns",
        "end_at_ms",
        "end_at_ms INTEGER",
    )
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO ai_marketing_campaign (
            marketing_campaign_id,
            coupon_template_id,
            status,
            start_at_ms,
            end_at_ms,
            created_at_ms,
            updated_at_ms,
            record_json
        )
        SELECT
            id,
            coupon_template_id,
            status,
            start_at_ms,
            end_at_ms,
            created_at_ms,
            updated_at_ms,
            record_json
        FROM coupon_campaigns",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE coupon_campaigns")
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate_sqlite_legacy_coupon_campaign_rows(pool: &SqlitePool) -> Result<()> {
    ensure_sqlite_column_if_table_exists(
        pool,
        "coupon_campaigns",
        "audience",
        "audience TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "coupon_campaigns",
        "remaining",
        "remaining INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "coupon_campaigns",
        "note",
        "note TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column_if_table_exists(pool, "coupon_campaigns", "expires_on", "expires_on TEXT")
        .await?;
    ensure_sqlite_column_if_table_exists(
        pool,
        "coupon_campaigns",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            i64,
            i64,
            String,
            Option<String>,
            i64,
        ),
    >(
        "SELECT
            id,
            code,
            discount_label,
            audience,
            remaining,
            active,
            note,
            expires_on,
            created_at_ms
         FROM coupon_campaigns",
    )
    .fetch_all(pool)
    .await?;

    for (id, code, discount_label, audience, _remaining, active, note, expires_on, created_at_ms) in
        rows
    {
        let created_at_ms = u64::try_from(created_at_ms.max(0))?;
        let updated_at_ms = created_at_ms;
        let expires_at_ms = sqlite_legacy_coupon_expiry_ms(pool, expires_on.as_deref()).await?;

        let template = CouponTemplateRecord::new(
            legacy_coupon_template_id(&id),
            legacy_coupon_template_key(&id),
            MarketingBenefitKind::GrantUnits,
        )
        .with_display_name(legacy_coupon_display_name(&discount_label, &note))
        .with_status(legacy_coupon_template_status(active))
        .with_distribution_kind(CouponDistributionKind::SharedCode)
        .with_benefit(parse_legacy_coupon_benefit(&discount_label))
        .with_restriction(CouponRestrictionSpec::new(MarketingSubjectScope::Project))
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(updated_at_ms);
        insert_sqlite_legacy_coupon_template(pool, &template).await?;

        let campaign =
            MarketingCampaignRecord::new(id.clone(), template.coupon_template_id.clone())
                .with_display_name(legacy_coupon_display_name(&note, &audience))
                .with_status(legacy_marketing_campaign_status(active))
                .with_start_at_ms(Some(created_at_ms))
                .with_end_at_ms(expires_at_ms)
                .with_created_at_ms(created_at_ms)
                .with_updated_at_ms(updated_at_ms);
        insert_sqlite_legacy_marketing_campaign(pool, &campaign).await?;

        let coupon_code = CouponCodeRecord::new(
            legacy_coupon_code_id(&id),
            template.coupon_template_id.clone(),
            code,
        )
        .with_status(legacy_coupon_code_status(active))
        .with_expires_at_ms(expires_at_ms)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(updated_at_ms);
        insert_sqlite_legacy_coupon_code(pool, &coupon_code).await?;
    }

    sqlx::query("DROP TABLE coupon_campaigns")
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_sqlite_legacy_coupon_template(
    pool: &SqlitePool,
    record: &CouponTemplateRecord,
) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO ai_marketing_coupon_template (
            coupon_template_id,
            template_key,
            status,
            distribution_kind,
            created_at_ms,
            updated_at_ms,
            record_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&record.coupon_template_id)
    .bind(&record.template_key)
    .bind(coupon_template_status_as_str(record.status))
    .bind(coupon_distribution_kind_as_str(record.distribution_kind))
    .bind(i64::try_from(record.created_at_ms)?)
    .bind(i64::try_from(record.updated_at_ms)?)
    .bind(serde_json::to_string(record)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_sqlite_legacy_marketing_campaign(
    pool: &SqlitePool,
    record: &MarketingCampaignRecord,
) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO ai_marketing_campaign (
            marketing_campaign_id,
            coupon_template_id,
            status,
            start_at_ms,
            end_at_ms,
            created_at_ms,
            updated_at_ms,
            record_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&record.marketing_campaign_id)
    .bind(&record.coupon_template_id)
    .bind(marketing_campaign_status_as_str(record.status))
    .bind(record.start_at_ms.map(i64::try_from).transpose()?)
    .bind(record.end_at_ms.map(i64::try_from).transpose()?)
    .bind(i64::try_from(record.created_at_ms)?)
    .bind(i64::try_from(record.updated_at_ms)?)
    .bind(serde_json::to_string(record)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_sqlite_legacy_coupon_code(
    pool: &SqlitePool,
    record: &CouponCodeRecord,
) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO ai_marketing_coupon_code (
            coupon_code_id,
            coupon_template_id,
            code_value,
            normalized_code_value,
            status,
            claimed_subject_scope,
            claimed_subject_id,
            expires_at_ms,
            created_at_ms,
            updated_at_ms,
            record_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&record.coupon_code_id)
    .bind(&record.coupon_template_id)
    .bind(&record.code_value)
    .bind(normalize_coupon_code_value(&record.code_value))
    .bind(coupon_code_status_as_str(record.status))
    .bind(
        record
            .claimed_subject_scope
            .map(marketing_subject_scope_as_str),
    )
    .bind(&record.claimed_subject_id)
    .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
    .bind(i64::try_from(record.created_at_ms)?)
    .bind(i64::try_from(record.updated_at_ms)?)
    .bind(serde_json::to_string(record)?)
    .execute(pool)
    .await?;
    Ok(())
}

fn legacy_coupon_template_id(legacy_id: &str) -> String {
    format!("legacy_tpl_{legacy_id}")
}

fn legacy_coupon_code_id(legacy_id: &str) -> String {
    format!("legacy_code_{legacy_id}")
}

fn legacy_coupon_template_key(legacy_id: &str) -> String {
    let normalized = legacy_id
        .trim()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    if normalized.is_empty() {
        "legacy-coupon".to_owned()
    } else {
        format!("legacy-{normalized}")
    }
}

fn legacy_coupon_display_name(primary: &str, fallback: &str) -> String {
    let primary = primary.trim();
    if !primary.is_empty() {
        return primary.to_owned();
    }

    let fallback = fallback.trim();
    if !fallback.is_empty() {
        return fallback.to_owned();
    }

    "Legacy coupon campaign".to_owned()
}

fn legacy_coupon_template_status(active: i64) -> CouponTemplateStatus {
    if active != 0 {
        CouponTemplateStatus::Active
    } else {
        CouponTemplateStatus::Archived
    }
}

fn legacy_marketing_campaign_status(active: i64) -> MarketingCampaignStatus {
    if active != 0 {
        MarketingCampaignStatus::Active
    } else {
        MarketingCampaignStatus::Archived
    }
}

fn legacy_coupon_code_status(active: i64) -> CouponCodeStatus {
    if active != 0 {
        CouponCodeStatus::Available
    } else {
        CouponCodeStatus::Disabled
    }
}

fn parse_legacy_coupon_benefit(discount_label: &str) -> CouponBenefitSpec {
    if let Some(percent) = parse_legacy_coupon_percent(discount_label) {
        return CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
            .with_discount_percent(Some(percent))
            .with_currency_code(Some("USD".to_owned()));
    }

    if let Some(amount_minor) = parse_legacy_coupon_fixed_amount_minor(discount_label) {
        return CouponBenefitSpec::new(MarketingBenefitKind::FixedAmountOff)
            .with_discount_amount_minor(Some(amount_minor))
            .with_currency_code(Some("USD".to_owned()))
            .with_max_discount_minor(Some(amount_minor));
    }

    if let Some(units) = parse_legacy_coupon_bonus_units(discount_label) {
        return CouponBenefitSpec::new(MarketingBenefitKind::GrantUnits)
            .with_grant_units(Some(units));
    }

    CouponBenefitSpec::new(MarketingBenefitKind::GrantUnits)
}

fn parse_legacy_coupon_percent(discount_label: &str) -> Option<u8> {
    let trimmed = discount_label.trim();
    let index = trimmed.find('%')?;
    let digits = trimmed[..index].trim();
    let value = digits.parse::<u8>().ok()?;
    Some(value.min(100))
}

fn parse_legacy_coupon_fixed_amount_minor(discount_label: &str) -> Option<u64> {
    let trimmed = discount_label.trim();
    let remainder = trimmed.strip_prefix('$')?;
    let value = remainder
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    if value.is_empty() {
        return None;
    }
    let mut parts = value.splitn(2, '.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let fractional = parts.next().unwrap_or("");
    let cents = match fractional.len() {
        0 => 0,
        1 => fractional.parse::<u64>().ok()?.saturating_mul(10),
        _ => fractional[..2].parse::<u64>().ok()?,
    };
    Some(major.saturating_mul(100).saturating_add(cents))
}

fn parse_legacy_coupon_bonus_units(discount_label: &str) -> Option<u64> {
    let trimmed = discount_label.trim();
    let digits = trimmed
        .trim_start_matches('+')
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

async fn sqlite_legacy_coupon_expiry_ms(
    pool: &SqlitePool,
    expires_on: Option<&str>,
) -> Result<Option<u64>> {
    let Some(expires_on) = expires_on.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let value = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT CAST(strftime('%s', ? || ' 23:59:59') AS INTEGER) * 1000",
    )
    .bind(expires_on)
    .fetch_one(pool)
    .await?;
    value
        .map(|millis| u64::try_from(millis).map_err(anyhow::Error::from))
        .transpose()
}

async fn create_sqlite_compatibility_views(pool: &SqlitePool) -> Result<()> {
    recreate_sqlite_compatibility_view(
        pool,
        "identity_users",
        "SELECT
            id,
            email,
            display_name,
            password_salt,
            password_hash,
            workspace_tenant_id,
            workspace_project_id,
            active,
            created_at_ms
         FROM ai_portal_users",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "admin_users",
        "SELECT
            id,
            email,
            display_name,
            password_salt,
            password_hash,
            role,
            active,
            created_at_ms
         FROM ai_admin_users",
    )
    .await?;
    recreate_sqlite_compatibility_view(pool, "tenant_records", "SELECT id, name FROM ai_tenants")
        .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "tenant_projects",
        "SELECT id, tenant_id, name FROM ai_projects",
    )
    .await?;
    ensure_sqlite_compatibility_view(
        pool,
        "coupon_campaigns",
        "SELECT
            marketing_campaign_id AS id,
            coupon_template_id,
            status,
            start_at_ms,
            end_at_ms,
            created_at_ms,
            updated_at_ms,
            record_json
         FROM ai_marketing_campaign",
        false,
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "catalog_channels",
        "SELECT
            channel_id AS id,
            channel_name AS name,
            channel_description,
            sort_order,
            is_builtin,
            is_active,
            created_at_ms,
            updated_at_ms
         FROM ai_channel",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "catalog_proxy_providers",
        "SELECT
            proxy_provider_id AS id,
            primary_channel_id AS channel_id,
            extension_id,
            adapter_kind,
            base_url,
            display_name
         FROM ai_proxy_provider",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "catalog_provider_channel_bindings",
        "SELECT
            proxy_provider_id AS provider_id,
            channel_id,
            is_primary,
            created_at_ms,
            updated_at_ms
         FROM ai_proxy_provider_channel",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "credential_records",
        "SELECT
            tenant_id,
            proxy_provider_id AS provider_id,
            key_reference,
            secret_backend,
            secret_local_file,
            secret_keyring_service,
            secret_master_key_id,
            secret_ciphertext,
            secret_key_version,
            created_at_ms,
            updated_at_ms
         FROM ai_router_credential_records",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "catalog_models",
        "SELECT
            model_id AS external_name,
            proxy_provider_id AS provider_id,
            capabilities_json AS capabilities,
            streaming_enabled AS streaming,
            context_window
         FROM ai_proxy_provider_model",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "routing_policies",
        "SELECT
            policy_id,
            capability,
            model_pattern,
            enabled,
            priority,
            strategy,
            default_provider_id,
            execution_failover_enabled,
            upstream_retry_max_attempts,
            upstream_retry_base_delay_ms,
            upstream_retry_max_delay_ms,
            max_cost,
            max_latency_ms,
            require_healthy
         FROM ai_routing_policies",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "routing_policy_providers",
        "SELECT policy_id, provider_id, position FROM ai_routing_policy_providers",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "project_routing_preferences",
        "SELECT
            project_id,
            preset_id,
            strategy,
            ordered_provider_ids_json,
            default_provider_id,
            max_cost,
            max_latency_ms,
            require_healthy,
            preferred_region,
            updated_at_ms
         FROM ai_project_routing_preferences",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "routing_decision_logs",
        "SELECT
            decision_id,
            decision_source,
            tenant_id,
            project_id,
            api_key_group_id,
            capability,
            route_key,
            selected_provider_id,
            matched_policy_id,
            applied_routing_profile_id,
            compiled_routing_snapshot_id,
            strategy,
            selection_seed,
            selection_reason,
            fallback_reason,
            requested_region,
            slo_applied,
            slo_degraded,
            created_at_ms,
            assessments_json
         FROM ai_routing_decision_logs",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "routing_provider_health",
        "SELECT
            provider_id,
            extension_id,
            runtime,
            observed_at_ms,
            instance_id,
            running,
            healthy,
            message
         FROM ai_provider_health_records",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "usage_records",
        "SELECT
            project_id,
            model,
            provider_id,
            units,
            amount,
            input_tokens,
            output_tokens,
            total_tokens,
            api_key_hash,
            channel_id,
            latency_ms,
            reference_amount,
            created_at_ms
         FROM ai_usage_records",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "billing_events",
        "SELECT
            event_id,
            tenant_id,
            project_id,
            api_key_group_id,
            capability,
            route_key,
            usage_model,
            provider_id,
            accounting_mode,
            operation_kind,
            modality,
            api_key_hash,
            channel_id,
            reference_id,
            latency_ms,
            units,
            request_count,
            input_tokens,
            output_tokens,
            total_tokens,
            cache_read_tokens,
            cache_write_tokens,
            image_count,
            audio_seconds,
            video_seconds,
            music_seconds,
            upstream_cost,
            customer_charge,
            applied_routing_profile_id,
            compiled_routing_snapshot_id,
            fallback_reason,
            created_at_ms
         FROM ai_billing_events",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "billing_ledger_entries",
        "SELECT project_id, units, amount, created_at_ms FROM ai_billing_ledger_entries",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "billing_quota_policies",
        "SELECT policy_id, project_id, max_units, enabled FROM ai_billing_quota_policies",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "identity_gateway_api_keys",
        "SELECT
            hashed_key,
            tenant_id,
            project_id,
            environment,
            label,
            notes,
            created_at_ms,
            last_used_at_ms,
            expires_at_ms,
            active
         FROM ai_app_api_keys",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "extension_installations",
        "SELECT
            installation_id,
            extension_id,
            runtime,
            enabled,
            entrypoint,
            config_json
         FROM ai_extension_installations",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "extension_instances",
        "SELECT
            instance_id,
            installation_id,
            extension_id,
            enabled,
            base_url,
            credential_ref,
            config_json
         FROM ai_extension_instances",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "service_runtime_nodes",
        "SELECT node_id, service_kind, started_at_ms, last_seen_at_ms FROM ai_service_runtime_nodes",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "extension_runtime_rollouts",
        "SELECT
            rollout_id,
            scope,
            requested_extension_id,
            requested_instance_id,
            resolved_extension_id,
            created_by,
            created_at_ms,
            deadline_at_ms
         FROM ai_extension_runtime_rollouts",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "extension_runtime_rollout_participants",
        "SELECT
            rollout_id,
            node_id,
            service_kind,
            status,
            message,
            updated_at_ms
         FROM ai_extension_runtime_rollout_participants",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "standalone_config_rollouts",
        "SELECT
            rollout_id,
            requested_service_kind,
            created_by,
            created_at_ms,
            deadline_at_ms
         FROM ai_standalone_config_rollouts",
    )
    .await?;
    recreate_sqlite_compatibility_view(
        pool,
        "standalone_config_rollout_participants",
        "SELECT
            rollout_id,
            node_id,
            service_kind,
            status,
            message,
            updated_at_ms
         FROM ai_standalone_config_rollout_participants",
    )
    .await?;

    Ok(())
}

async fn ensure_sqlite_compatibility_view(
    pool: &SqlitePool,
    legacy_name: &str,
    select_sql: &str,
    replace_legacy_table: bool,
) -> Result<()> {
    match sqlite_object_type(pool, legacy_name).await?.as_deref() {
        Some("table") if !replace_legacy_table => Ok(()),
        _ => recreate_sqlite_compatibility_view(pool, legacy_name, select_sql).await,
    }
}
