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
        .with_active(is_active != 0)
}
