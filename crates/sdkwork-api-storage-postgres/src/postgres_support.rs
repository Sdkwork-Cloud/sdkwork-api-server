use super::*;

pub(crate) fn provider_channel_bindings(provider: &ProxyProvider) -> Vec<ProviderChannelBinding> {
    if provider.channel_bindings.is_empty() {
        vec![ProviderChannelBinding::primary(
            provider.id.clone(),
            provider.channel_id.clone(),
        )]
    } else {
        provider.channel_bindings.clone()
    }
}

pub(crate) async fn load_routing_policy_provider_ids(
    pool: &PgPool,
    policy_id: &str,
) -> Result<Vec<String>> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT provider_id
         FROM ai_routing_policy_providers
         WHERE policy_id = $1
         ORDER BY position, provider_id",
    )
    .bind(policy_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(provider_id,)| provider_id).collect())
}

pub(crate) async fn load_provider_channel_bindings(
    pool: &PgPool,
    provider_id: &str,
    channel_id: &str,
) -> Result<Vec<ProviderChannelBinding>> {
    let rows = sqlx::query_as::<_, (String, bool)>(
        "SELECT channel_id, is_primary
         FROM ai_proxy_provider_channel
         WHERE proxy_provider_id = $1
         ORDER BY is_primary DESC, channel_id",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(vec![ProviderChannelBinding::primary(
            provider_id.to_owned(),
            channel_id.to_owned(),
        )]);
    }

    Ok(rows
        .into_iter()
        .map(|(binding_channel_id, is_primary)| ProviderChannelBinding {
            provider_id: provider_id.to_owned(),
            channel_id: binding_channel_id,
            is_primary,
        })
        .collect())
}

pub(crate) async fn load_provider_channel_bindings_for_providers(
    pool: &PgPool,
    providers: &[(String, String)],
) -> Result<HashMap<String, Vec<ProviderChannelBinding>>> {
    let mut bindings_by_provider = providers
        .iter()
        .map(|(provider_id, _)| (provider_id.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();

    if providers.is_empty() {
        return Ok(bindings_by_provider);
    }

    let mut query = String::from(
        "SELECT proxy_provider_id, channel_id, is_primary
         FROM ai_proxy_provider_channel
         WHERE proxy_provider_id IN (",
    );
    for (index, _) in providers.iter().enumerate() {
        if index > 0 {
            query.push_str(", ");
        }
        query.push('$');
        query.push_str(&(index + 1).to_string());
    }
    query.push_str(") ORDER BY proxy_provider_id, is_primary DESC, channel_id");

    let mut statement = sqlx::query_as::<_, (String, String, bool)>(&query);
    for (provider_id, _) in providers {
        statement = statement.bind(provider_id);
    }
    let rows = statement.fetch_all(pool).await?;

    for (provider_id, channel_id, is_primary) in rows {
        bindings_by_provider
            .entry(provider_id.clone())
            .or_default()
            .push(ProviderChannelBinding {
                provider_id,
                channel_id,
                is_primary,
            });
    }

    for (provider_id, channel_id) in providers {
        let bindings = bindings_by_provider.entry(provider_id.clone()).or_default();
        if bindings.is_empty() {
            bindings.push(ProviderChannelBinding::primary(
                provider_id.clone(),
                channel_id.clone(),
            ));
        }
    }

    Ok(bindings_by_provider)
}

pub(crate) fn encode_model_capabilities(capabilities: &[ModelCapability]) -> Result<String> {
    Ok(serde_json::to_string(capabilities)?)
}

pub(crate) fn decode_model_capabilities(capabilities: &str) -> Result<Vec<ModelCapability>> {
    Ok(serde_json::from_str(capabilities)?)
}

pub(crate) fn encode_extension_config(config: &Value) -> Result<String> {
    Ok(serde_json::to_string(config)?)
}

pub(crate) fn decode_extension_config(config_json: &str) -> Result<Value> {
    Ok(serde_json::from_str(config_json)?)
}

pub(crate) fn encode_routing_assessments(
    assessments: &[RoutingCandidateAssessment],
) -> Result<String> {
    Ok(serde_json::to_string(assessments)?)
}

pub(crate) fn decode_routing_assessments(
    assessments_json: &str,
) -> Result<Vec<RoutingCandidateAssessment>> {
    Ok(serde_json::from_str(assessments_json)?)
}

pub(crate) fn encode_string_list(values: &[String]) -> Result<String> {
    Ok(serde_json::to_string(values)?)
}

pub(crate) fn decode_string_list(values_json: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(values_json)?)
}

pub(crate) fn decode_billing_event_row(row: &PgRow) -> Result<BillingEventRecord> {
    Ok(BillingEventRecord {
        event_id: row.try_get("event_id")?,
        tenant_id: row.try_get("tenant_id")?,
        project_id: row.try_get("project_id")?,
        api_key_group_id: row.try_get("api_key_group_id")?,
        capability: row.try_get("capability")?,
        route_key: row.try_get("route_key")?,
        usage_model: row.try_get("usage_model")?,
        provider_id: row.try_get("provider_id")?,
        accounting_mode: BillingAccountingMode::from_str(
            &row.try_get::<String, _>("accounting_mode")?,
        )
        .unwrap_or(BillingAccountingMode::PlatformCredit),
        operation_kind: row.try_get("operation_kind")?,
        modality: row.try_get("modality")?,
        api_key_hash: row.try_get("api_key_hash")?,
        channel_id: row.try_get("channel_id")?,
        reference_id: row.try_get("reference_id")?,
        latency_ms: row
            .try_get::<Option<i64>, _>("latency_ms")?
            .map(u64::try_from)
            .transpose()?,
        units: u64::try_from(row.try_get::<i64, _>("units")?)?,
        request_count: u64::try_from(row.try_get::<i64, _>("request_count")?)?,
        input_tokens: u64::try_from(row.try_get::<i64, _>("input_tokens")?)?,
        output_tokens: u64::try_from(row.try_get::<i64, _>("output_tokens")?)?,
        total_tokens: u64::try_from(row.try_get::<i64, _>("total_tokens")?)?,
        cache_read_tokens: u64::try_from(row.try_get::<i64, _>("cache_read_tokens")?)?,
        cache_write_tokens: u64::try_from(row.try_get::<i64, _>("cache_write_tokens")?)?,
        image_count: u64::try_from(row.try_get::<i64, _>("image_count")?)?,
        audio_seconds: row.try_get("audio_seconds")?,
        video_seconds: row.try_get("video_seconds")?,
        music_seconds: row.try_get("music_seconds")?,
        upstream_cost: row.try_get("upstream_cost")?,
        customer_charge: row.try_get("customer_charge")?,
        applied_routing_profile_id: row.try_get("applied_routing_profile_id")?,
        compiled_routing_snapshot_id: row.try_get("compiled_routing_snapshot_id")?,
        fallback_reason: row.try_get("fallback_reason")?,
        created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    })
}

pub(crate) fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

pub(crate) fn normalize_coupon_code_value(code_value: &str) -> String {
    code_value.trim().to_ascii_uppercase()
}

pub(crate) fn coupon_template_status_as_str(status: CouponTemplateStatus) -> &'static str {
    match status {
        CouponTemplateStatus::Draft => "draft",
        CouponTemplateStatus::Active => "active",
        CouponTemplateStatus::Archived => "archived",
    }
}

pub(crate) fn coupon_distribution_kind_as_str(kind: CouponDistributionKind) -> &'static str {
    match kind {
        CouponDistributionKind::SharedCode => "shared_code",
        CouponDistributionKind::UniqueCode => "unique_code",
        CouponDistributionKind::AutoClaim => "auto_claim",
    }
}

pub(crate) fn marketing_campaign_status_as_str(status: MarketingCampaignStatus) -> &'static str {
    match status {
        MarketingCampaignStatus::Draft => "draft",
        MarketingCampaignStatus::Scheduled => "scheduled",
        MarketingCampaignStatus::Active => "active",
        MarketingCampaignStatus::Paused => "paused",
        MarketingCampaignStatus::Ended => "ended",
        MarketingCampaignStatus::Archived => "archived",
    }
}

pub(crate) fn campaign_budget_status_as_str(status: CampaignBudgetStatus) -> &'static str {
    match status {
        CampaignBudgetStatus::Draft => "draft",
        CampaignBudgetStatus::Active => "active",
        CampaignBudgetStatus::Exhausted => "exhausted",
        CampaignBudgetStatus::Closed => "closed",
    }
}

pub(crate) fn coupon_code_status_as_str(status: CouponCodeStatus) -> &'static str {
    match status {
        CouponCodeStatus::Available => "available",
        CouponCodeStatus::Reserved => "reserved",
        CouponCodeStatus::Redeemed => "redeemed",
        CouponCodeStatus::Expired => "expired",
        CouponCodeStatus::Disabled => "disabled",
    }
}

pub(crate) fn marketing_subject_scope_as_str(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

pub(crate) fn coupon_reservation_status_as_str(status: CouponReservationStatus) -> &'static str {
    match status {
        CouponReservationStatus::Reserved => "reserved",
        CouponReservationStatus::Released => "released",
        CouponReservationStatus::Confirmed => "confirmed",
        CouponReservationStatus::Expired => "expired",
    }
}

pub(crate) fn coupon_redemption_status_as_str(status: CouponRedemptionStatus) -> &'static str {
    match status {
        CouponRedemptionStatus::Pending => "pending",
        CouponRedemptionStatus::Redeemed => "redeemed",
        CouponRedemptionStatus::PartiallyRolledBack => "partially_rolled_back",
        CouponRedemptionStatus::RolledBack => "rolled_back",
        CouponRedemptionStatus::Failed => "failed",
    }
}

pub(crate) fn coupon_rollback_type_as_str(rollback_type: CouponRollbackType) -> &'static str {
    match rollback_type {
        CouponRollbackType::Cancel => "cancel",
        CouponRollbackType::Refund => "refund",
        CouponRollbackType::PartialRefund => "partial_refund",
        CouponRollbackType::Manual => "manual",
    }
}

pub(crate) fn coupon_rollback_status_as_str(status: CouponRollbackStatus) -> &'static str {
    match status {
        CouponRollbackStatus::Pending => "pending",
        CouponRollbackStatus::Completed => "completed",
        CouponRollbackStatus::Failed => "failed",
    }
}

pub(crate) fn marketing_outbox_event_status_as_str(
    status: MarketingOutboxEventStatus,
) -> &'static str {
    match status {
        MarketingOutboxEventStatus::Pending => "pending",
        MarketingOutboxEventStatus::Delivered => "delivered",
        MarketingOutboxEventStatus::Failed => "failed",
    }
}

pub(crate) type PortalUserRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    bool,
    i64,
);

pub(crate) type AdminUserRow = (String, String, String, String, String, bool, i64);

pub(crate) type CouponRow = (
    String,
    String,
    String,
    String,
    i64,
    bool,
    String,
    String,
    i64,
);

pub(crate) type CredentialRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub(crate) type ChannelModelRow = (String, String, String, String, bool, Option<i64>, String);

pub(crate) type ModelPriceRow = (
    String,
    String,
    String,
    String,
    String,
    f64,
    f64,
    f64,
    f64,
    f64,
    bool,
);

pub(crate) fn decode_portal_user_row(
    row: Option<PortalUserRow>,
) -> Result<Option<PortalUserRecord>> {
    row.map(
        |(
            id,
            email,
            display_name,
            password_salt,
            password_hash,
            workspace_tenant_id,
            workspace_project_id,
            active,
            created_at_ms,
        )| {
            Ok(PortalUserRecord {
                id,
                email,
                display_name,
                password_salt,
                password_hash,
                workspace_tenant_id,
                workspace_project_id,
                active,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

pub(crate) fn decode_admin_user_row(row: Option<AdminUserRow>) -> Result<Option<AdminUserRecord>> {
    row.map(
        |(id, email, display_name, password_salt, password_hash, active, created_at_ms)| {
            Ok(AdminUserRecord {
                id,
                email,
                display_name,
                password_salt,
                password_hash,
                active,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

pub(crate) fn decode_coupon_row(row: Option<CouponRow>) -> Result<Option<CouponCampaign>> {
    row.map(
        |(
            id,
            code,
            discount_label,
            audience,
            remaining,
            active,
            note,
            expires_on,
            created_at_ms,
        )| {
            Ok(CouponCampaign {
                id,
                code,
                discount_label,
                audience,
                remaining: u64::try_from(remaining)?,
                active,
                note,
                expires_on,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

pub(crate) fn decode_credential_row(row: CredentialRow) -> UpstreamCredential {
    let (
        tenant_id,
        provider_id,
        key_reference,
        secret_backend,
        secret_local_file,
        secret_keyring_service,
        secret_master_key_id,
    ) = row;

    UpstreamCredential {
        tenant_id,
        provider_id,
        key_reference,
        secret_backend,
        secret_local_file,
        secret_keyring_service,
        secret_master_key_id,
    }
}

pub(crate) type RoutingDecisionLogRow = PgRow;

pub(crate) fn decode_routing_profile_row(row: PgRow) -> Result<RoutingProfileRecord> {
    Ok(RoutingProfileRecord::new(
        row.try_get::<String, _>("profile_id")?,
        row.try_get::<String, _>("tenant_id")?,
        row.try_get::<String, _>("project_id")?,
        row.try_get::<String, _>("name")?,
        row.try_get::<String, _>("slug")?,
    )
    .with_description_option(row.try_get::<Option<String>, _>("description")?)
    .with_active(row.try_get::<bool, _>("active")?)
    .with_strategy(
        RoutingStrategy::from_str(&row.try_get::<String, _>("strategy")?)
            .unwrap_or(RoutingStrategy::DeterministicPriority),
    )
    .with_ordered_provider_ids(decode_string_list(
        &row.try_get::<String, _>("ordered_provider_ids_json")?,
    )?)
    .with_default_provider_id_option(row.try_get::<Option<String>, _>("default_provider_id")?)
    .with_max_cost_option(row.try_get::<Option<f64>, _>("max_cost")?)
    .with_max_latency_ms_option(
        row.try_get::<Option<i64>, _>("max_latency_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_require_healthy(row.try_get::<bool, _>("require_healthy")?)
    .with_preferred_region_option(row.try_get::<Option<String>, _>("preferred_region")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_compiled_routing_snapshot_row(
    row: PgRow,
) -> Result<CompiledRoutingSnapshotRecord> {
    Ok(CompiledRoutingSnapshotRecord::new(
        row.try_get::<String, _>("snapshot_id")?,
        row.try_get::<String, _>("capability")?,
        row.try_get::<String, _>("route_key")?,
    )
    .with_tenant_id_option(row.try_get::<Option<String>, _>("tenant_id")?)
    .with_project_id_option(row.try_get::<Option<String>, _>("project_id")?)
    .with_api_key_group_id_option(row.try_get::<Option<String>, _>("api_key_group_id")?)
    .with_matched_policy_id_option(row.try_get::<Option<String>, _>("matched_policy_id")?)
    .with_project_routing_preferences_project_id_option(
        row.try_get::<Option<String>, _>("project_routing_preferences_project_id")?,
    )
    .with_applied_routing_profile_id_option(
        row.try_get::<Option<String>, _>("applied_routing_profile_id")?,
    )
    .with_strategy(row.try_get::<String, _>("strategy")?)
    .with_ordered_provider_ids(decode_string_list(
        &row.try_get::<String, _>("ordered_provider_ids_json")?,
    )?)
    .with_default_provider_id_option(row.try_get::<Option<String>, _>("default_provider_id")?)
    .with_max_cost_option(row.try_get::<Option<f64>, _>("max_cost")?)
    .with_max_latency_ms_option(
        row.try_get::<Option<i64>, _>("max_latency_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_require_healthy(row.try_get::<bool, _>("require_healthy")?)
    .with_preferred_region_option(row.try_get::<Option<String>, _>("preferred_region")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_routing_decision_log_row(
    row: RoutingDecisionLogRow,
) -> Result<RoutingDecisionLog> {
    Ok(RoutingDecisionLog::new(
        row.try_get::<String, _>("decision_id")?,
        RoutingDecisionSource::from_str(&row.try_get::<String, _>("decision_source")?)
            .unwrap_or(RoutingDecisionSource::Gateway),
        row.try_get::<String, _>("capability")?,
        row.try_get::<String, _>("route_key")?,
        row.try_get::<String, _>("selected_provider_id")?,
        row.try_get::<String, _>("strategy")?,
        u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    )
    .with_tenant_id_option(row.try_get::<Option<String>, _>("tenant_id")?)
    .with_project_id_option(row.try_get::<Option<String>, _>("project_id")?)
    .with_api_key_group_id_option(row.try_get::<Option<String>, _>("api_key_group_id")?)
    .with_matched_policy_id_option(row.try_get::<Option<String>, _>("matched_policy_id")?)
    .with_applied_routing_profile_id_option(
        row.try_get::<Option<String>, _>("applied_routing_profile_id")?,
    )
    .with_compiled_routing_snapshot_id_option(
        row.try_get::<Option<String>, _>("compiled_routing_snapshot_id")?,
    )
    .with_selection_seed_option(
        row.try_get::<Option<i64>, _>("selection_seed")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_selection_reason_option(row.try_get::<Option<String>, _>("selection_reason")?)
    .with_fallback_reason_option(row.try_get::<Option<String>, _>("fallback_reason")?)
    .with_requested_region_option(row.try_get::<Option<String>, _>("requested_region")?)
    .with_slo_state(
        row.try_get::<bool, _>("slo_applied")?,
        row.try_get::<bool, _>("slo_degraded")?,
    )
    .with_assessments(decode_routing_assessments(
        &row.try_get::<String, _>("assessments_json")?,
    )?))
}

pub(crate) fn decode_channel_model_row(row: ChannelModelRow) -> Result<ChannelModelRecord> {
    let (
        channel_id,
        model_id,
        model_display_name,
        capabilities_json,
        streaming_enabled,
        context_window,
        description,
    ) = row;

    let mut record = ChannelModelRecord::new(channel_id, model_id, model_display_name)
        .with_context_window_option(context_window.map(u64::try_from).transpose()?)
        .with_streaming(streaming_enabled)
        .with_description_option((!description.is_empty()).then_some(description));
    for capability in decode_model_capabilities(&capabilities_json)? {
        record = record.with_capability(capability);
    }
    Ok(record)
}

pub(crate) fn decode_model_price_row(row: ModelPriceRow) -> ModelPriceRecord {
    let (
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
    ) = row;

    ModelPriceRecord::new(channel_id, model_id, proxy_provider_id)
        .with_currency_code(currency_code)
        .with_price_unit(price_unit)
        .with_input_price(input_price)
        .with_output_price(output_price)
        .with_cache_read_price(cache_read_price)
        .with_cache_write_price(cache_write_price)
        .with_request_price(request_price)
        .with_active(is_active)
}

pub(crate) async fn postgres_relation_kind(
    pool: &PgPool,
    relation_name: &str,
) -> Result<Option<String>> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT c.relkind::text
         FROM pg_class c
         INNER JOIN pg_namespace n
             ON n.oid = c.relnamespace
         WHERE n.nspname = current_schema()
           AND c.relname = $1",
    )
    .bind(relation_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(kind,)| kind))
}

pub(crate) async fn postgres_table_columns(pool: &PgPool, table_name: &str) -> Result<Vec<String>> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT column_name
         FROM information_schema.columns
         WHERE table_schema = current_schema()
           AND table_name = $1
         ORDER BY ordinal_position",
    )
    .bind(table_name)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(column_name,)| column_name).collect())
}

pub(crate) async fn ensure_postgres_column_if_table_exists(
    pool: &PgPool,
    table_name: &str,
    alter_statement: &str,
) -> Result<()> {
    if postgres_relation_kind(pool, table_name).await?.as_deref() == Some("r") {
        sqlx::query(alter_statement).execute(pool).await?;
    }
    Ok(())
}

pub(crate) async fn migrate_postgres_legacy_table_with_common_columns(
    pool: &PgPool,
    legacy_table_name: &str,
    canonical_table_name: &str,
) -> Result<()> {
    if postgres_relation_kind(pool, legacy_table_name)
        .await?
        .as_deref()
        != Some("r")
    {
        return Ok(());
    }

    let legacy_columns = postgres_table_columns(pool, legacy_table_name).await?;
    let canonical_columns = postgres_table_columns(pool, canonical_table_name).await?;
    let common_columns: Vec<String> = canonical_columns
        .into_iter()
        .filter(|column_name| legacy_columns.contains(column_name))
        .collect();

    if !common_columns.is_empty() {
        let column_list = common_columns.join(", ");
        let insert = format!(
            "INSERT INTO {canonical_table_name} ({column_list})
             SELECT {column_list} FROM {legacy_table_name}
             ON CONFLICT DO NOTHING"
        );
        sqlx::query(&insert).execute(pool).await?;
    }

    let drop_table = format!("DROP TABLE {legacy_table_name}");
    sqlx::query(&drop_table).execute(pool).await?;
    Ok(())
}

pub(crate) async fn recreate_postgres_compatibility_view(
    pool: &PgPool,
    legacy_name: &str,
    select_sql: &str,
) -> Result<()> {
    match postgres_relation_kind(pool, legacy_name).await?.as_deref() {
        Some("r") => {
            let drop_table = format!("DROP TABLE {legacy_name}");
            sqlx::query(&drop_table).execute(pool).await?;
        }
        Some("v") => {
            let drop_view = format!("DROP VIEW {legacy_name}");
            sqlx::query(&drop_view).execute(pool).await?;
        }
        _ => {}
    }

    let create_view = format!("CREATE VIEW {legacy_name} AS {select_sql}");
    sqlx::query(&create_view).execute(pool).await?;
    Ok(())
}
