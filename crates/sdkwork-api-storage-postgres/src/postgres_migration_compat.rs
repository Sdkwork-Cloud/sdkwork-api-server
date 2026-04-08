use super::*;

const LEGACY_RENAMED_TABLE_MAPPINGS: [(&str, &str); 24] = [
    ("identity_users", "ai_portal_users"),
    ("admin_users", "ai_admin_users"),
    ("tenant_records", "ai_tenants"),
    ("tenant_projects", "ai_projects"),
    ("coupon_campaigns", "ai_coupon_campaigns"),
    ("routing_policies", "ai_routing_policies"),
    ("routing_policy_providers", "ai_routing_policy_providers"),
    (
        "project_routing_preferences",
        "ai_project_routing_preferences",
    ),
    ("routing_decision_logs", "ai_routing_decision_logs"),
    ("routing_provider_health", "ai_provider_health_records"),
    ("usage_records", "ai_usage_records"),
    ("billing_events", "ai_billing_events"),
    ("billing_ledger_entries", "ai_billing_ledger_entries"),
    ("billing_quota_policies", "ai_billing_quota_policies"),
    ("commerce_orders", "ai_commerce_orders"),
    ("commerce_payment_events", "ai_commerce_payment_events"),
    ("project_memberships", "ai_project_memberships"),
    ("extension_installations", "ai_extension_installations"),
    ("extension_instances", "ai_extension_instances"),
    ("service_runtime_nodes", "ai_service_runtime_nodes"),
    (
        "extension_runtime_rollouts",
        "ai_extension_runtime_rollouts",
    ),
    (
        "extension_runtime_rollout_participants",
        "ai_extension_runtime_rollout_participants",
    ),
    (
        "standalone_config_rollouts",
        "ai_standalone_config_rollouts",
    ),
    (
        "standalone_config_rollout_participants",
        "ai_standalone_config_rollout_participants",
    ),
];

pub(crate) async fn apply_postgres_legacy_table_compatibility(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    for (legacy_table_name, canonical_table_name) in LEGACY_RENAMED_TABLE_MAPPINGS {
        migrate_postgres_legacy_table_with_common_columns(
            &pool,
            legacy_table_name,
            canonical_table_name,
        )
        .await?;
    }

    if postgres_relation_kind(&pool, "catalog_channels")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_channel (
                channel_id,
                channel_name,
                channel_description,
                sort_order,
                is_builtin,
                is_active,
                created_at_ms,
                updated_at_ms
            )
            SELECT id, name, '', 0, FALSE, TRUE, 0, 0
            FROM catalog_channels
            ON CONFLICT (channel_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE catalog_channels")
            .execute(&pool)
            .await?;
    }
    Ok(())
}

pub(crate) async fn migrate_postgres_legacy_catalog_records(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    if postgres_relation_kind(&pool, "catalog_proxy_providers")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_proxy_provider (
                proxy_provider_id,
                primary_channel_id,
                extension_id,
                adapter_kind,
                base_url,
                display_name,
                is_active,
                created_at_ms,
                updated_at_ms
            )
            SELECT id, channel_id, extension_id, adapter_kind, base_url, display_name, TRUE, 0, 0
            FROM catalog_proxy_providers
            ON CONFLICT (proxy_provider_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO ai_proxy_provider_channel (
                proxy_provider_id,
                channel_id,
                is_primary,
                created_at_ms,
                updated_at_ms
            )
            SELECT id, channel_id, TRUE, 0, 0
            FROM catalog_proxy_providers
            ON CONFLICT (proxy_provider_id, channel_id) DO UPDATE SET
                is_primary = EXCLUDED.is_primary,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .execute(&pool)
        .await?;
    }

    if postgres_relation_kind(&pool, "catalog_provider_channel_bindings")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_proxy_provider_channel (
                proxy_provider_id,
                channel_id,
                is_primary,
                created_at_ms,
                updated_at_ms
            )
            SELECT provider_id, channel_id, is_primary, 0, 0
            FROM catalog_provider_channel_bindings
            ON CONFLICT (proxy_provider_id, channel_id) DO UPDATE SET
                is_primary = EXCLUDED.is_primary,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE catalog_provider_channel_bindings")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "catalog_proxy_providers")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query("DROP TABLE catalog_proxy_providers")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "credential_records")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_router_credential_records (
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
            FROM credential_records
            ON CONFLICT (tenant_id, proxy_provider_id, key_reference) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE credential_records")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "catalog_models")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_model (
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
                models.external_name,
                models.external_name,
                models.capabilities,
                models.streaming,
                models.context_window,
                '',
                0,
                0
            FROM catalog_models models
            INNER JOIN ai_proxy_provider providers
                ON providers.proxy_provider_id = models.provider_id
            ON CONFLICT (channel_id, model_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO ai_model_price (
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
                is_active,
                created_at_ms,
                updated_at_ms
            )
            SELECT
                providers.primary_channel_id,
                models.external_name,
                models.provider_id,
                'USD',
                'per_1m_tokens',
                0,
                0,
                0,
                0,
                0,
                TRUE,
                0,
                0
            FROM catalog_models models
            INNER JOIN ai_proxy_provider providers
                ON providers.proxy_provider_id = models.provider_id
            ON CONFLICT (channel_id, model_id, proxy_provider_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE catalog_models")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "identity_gateway_api_keys")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_app_api_keys (
                hashed_key,
                raw_key,
                tenant_id,
                project_id,
                environment,
                api_key_group_id,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            )
            SELECT
                hashed_key,
                NULL,
                tenant_id,
                project_id,
                environment,
                NULL,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            FROM identity_gateway_api_keys
            ON CONFLICT (hashed_key) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE identity_gateway_api_keys")
            .execute(&pool)
            .await?;
    }
    Ok(())
}

pub(crate) async fn recreate_postgres_legacy_compatibility_views(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    for (legacy_table_name, canonical_table_name) in LEGACY_RENAMED_TABLE_MAPPINGS {
        let select_sql = format!("SELECT * FROM {canonical_table_name}");
        recreate_postgres_compatibility_view(&pool, legacy_table_name, &select_sql).await?;
    }

    recreate_postgres_compatibility_view(
        &pool,
        "catalog_channels",
        "SELECT channel_id AS id, channel_name AS name FROM ai_channel",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
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
    recreate_postgres_compatibility_view(
        &pool,
        "catalog_provider_channel_bindings",
        "SELECT
            proxy_provider_id AS provider_id,
            channel_id,
            is_primary
         FROM ai_proxy_provider_channel",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
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
            secret_key_version
         FROM ai_router_credential_records",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
        "catalog_models",
        "SELECT
            models.model_id AS external_name,
            prices.proxy_provider_id AS provider_id,
            models.capabilities_json AS capabilities,
            models.streaming_enabled AS streaming,
            models.context_window
         FROM ai_model models
         INNER JOIN ai_model_price prices
             ON prices.channel_id = models.channel_id
            AND prices.model_id = models.model_id
         INNER JOIN ai_proxy_provider providers
             ON providers.proxy_provider_id = prices.proxy_provider_id
         WHERE models.channel_id = providers.primary_channel_id",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
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
    Ok(())
}
