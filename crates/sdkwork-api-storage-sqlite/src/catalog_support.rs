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

pub(crate) async fn load_provider_channel_bindings(
    pool: &SqlitePool,
    provider_id: &str,
    channel_id: &str,
) -> Result<Vec<ProviderChannelBinding>> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT channel_id, is_primary
         FROM ai_proxy_provider_channel
         WHERE proxy_provider_id = ?
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
            is_primary: is_primary != 0,
        })
        .collect())
}

pub(crate) async fn load_provider_channel_bindings_for_providers(
    pool: &SqlitePool,
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
        query.push('?');
    }
    query.push_str(") ORDER BY proxy_provider_id, is_primary DESC, channel_id");

    let mut statement = sqlx::query_as::<_, (String, String, i64)>(&query);
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
                is_primary: is_primary != 0,
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

pub(crate) fn encode_model_price_tiers(pricing_tiers: &[ModelPriceTier]) -> Result<String> {
    Ok(serde_json::to_string(pricing_tiers)?)
}

pub(crate) fn decode_model_price_tiers(pricing_tiers: &str) -> Result<Vec<ModelPriceTier>> {
    Ok(serde_json::from_str(pricing_tiers)?)
}

pub(crate) fn encode_catalog_string_list(values: &[String]) -> Result<String> {
    Ok(serde_json::to_string(values)?)
}

pub(crate) fn decode_catalog_string_list(values_json: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(values_json)?)
}

pub(crate) type CredentialRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub(crate) type ChannelModelRow = (String, String, String, String, i64, Option<i64>, String);

pub(crate) type ProviderModelRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    i64,
    Option<i64>,
    Option<i64>,
    i64,
    i64,
    i64,
    i64,
    i64,
);

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
    String,
    Option<String>,
    String,
    i64,
);

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
        .with_streaming(streaming_enabled != 0)
        .with_description_option((!description.is_empty()).then_some(description));
    for capability in decode_model_capabilities(&capabilities_json)? {
        record = record.with_capability(capability);
    }
    Ok(record)
}

pub(crate) fn decode_provider_account_sqlite_row(row: SqliteRow) -> Result<ProviderAccountRecord> {
    Ok(ProviderAccountRecord::new(
        row.try_get::<String, _>("provider_account_id")?,
        row.try_get::<String, _>("provider_id")?,
        row.try_get::<String, _>("display_name")?,
        row.try_get::<String, _>("account_kind")?,
        row.try_get::<String, _>("execution_instance_id")?,
    )
    .with_owner_scope(row.try_get::<String, _>("owner_scope")?)
    .with_owner_tenant_id_option(row.try_get::<Option<String>, _>("owner_tenant_id")?)
    .with_base_url_override_option(row.try_get::<Option<String>, _>("base_url_override")?)
    .with_region_option(row.try_get::<Option<String>, _>("region")?)
    .with_priority(i32::try_from(row.try_get::<i64, _>("priority")?)?)
    .with_weight(u32::try_from(row.try_get::<i64, _>("weight")?)?)
    .with_enabled(row.try_get::<i64, _>("enabled")? != 0)
    .with_routing_tags(decode_catalog_string_list(
        &row.try_get::<String, _>("routing_tags_json")?,
    )?)
    .with_health_score_hint_option(row.try_get::<Option<f64>, _>("health_score_hint")?)
    .with_latency_ms_hint_option(
        row.try_get::<Option<i64>, _>("latency_ms_hint")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_cost_hint_option(row.try_get::<Option<f64>, _>("cost_hint")?)
    .with_success_rate_hint_option(row.try_get::<Option<f64>, _>("success_rate_hint")?)
    .with_throughput_hint_option(row.try_get::<Option<f64>, _>("throughput_hint")?)
    .with_max_concurrency_option(
        row.try_get::<Option<i64>, _>("max_concurrency")?
            .map(u32::try_from)
            .transpose()?,
    )
    .with_daily_budget_option(row.try_get::<Option<f64>, _>("daily_budget")?)
    .with_notes_option(row.try_get::<Option<String>, _>("notes")?))
}

pub(crate) fn decode_provider_model_row(row: ProviderModelRow) -> Result<ProviderModelRecord> {
    let (
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
    ) = row;

    let mut record = ProviderModelRecord::new(proxy_provider_id, channel_id, model_id)
        .with_provider_model_id(provider_model_id)
        .with_provider_model_family_option(provider_model_family)
        .with_streaming(streaming_enabled != 0)
        .with_context_window_option(context_window.map(u64::try_from).transpose()?)
        .with_max_output_tokens_option(max_output_tokens.map(u64::try_from).transpose()?)
        .with_supports_prompt_caching(supports_prompt_caching != 0)
        .with_supports_reasoning_usage(supports_reasoning_usage != 0)
        .with_supports_tool_usage_metrics(supports_tool_usage_metrics != 0)
        .with_default_route(is_default_route != 0)
        .with_active(is_active != 0);
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
        price_source_kind,
        billing_notes,
        pricing_tiers_json,
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
        .with_price_source_kind(price_source_kind)
        .with_billing_notes_option(billing_notes)
        .with_pricing_tiers(decode_model_price_tiers(&pricing_tiers_json).unwrap_or_default())
        .with_active(is_active != 0)
}
