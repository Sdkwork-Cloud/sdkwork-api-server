use super::*;
use axum::extract::Query;

#[derive(Debug, Clone)]
struct NormalizedProviderCreateRequest {
    adapter_kind: String,
    protocol_kind: Option<String>,
    extension_id: Option<String>,
}

pub(crate) async fn list_channels_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Channel>>, StatusCode> {
    list_channels(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), StatusCode> {
    let channel = persist_channel(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|error| catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(channel)))
}

pub(crate) async fn delete_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(channel_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_channel(state.store.as_ref(), &channel_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_providers_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<ListProvidersQuery>,
) -> Result<Json<Vec<ProviderCatalogResponse>>, StatusCode> {
    let providers = list_providers(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let execution_views = inspect_provider_execution_views(state.store.as_ref(), &providers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let configured_provider_ids = match query.tenant_id.as_deref() {
        Some(tenant_id) => Some(
            sdkwork_api_app_credential::list_configured_provider_ids_for_tenant(
                state.store.as_ref(),
                tenant_id,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        ),
        None => None,
    };

    let mut response = Vec::with_capacity(providers.len());
    for provider in providers {
        let Some(execution) = execution_views.get(&provider.id).cloned() else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        let credential_readiness = configured_provider_ids.as_ref().map(|provider_ids| {
            sdkwork_api_app_credential::provider_credential_readiness_view(
                provider_ids.contains(&provider.id),
            )
        });
        response.push(ProviderCatalogResponse {
            integration: sdkwork_api_app_catalog::provider_integration_view(&provider),
            provider,
            execution,
            credential_readiness,
        });
    }

    Ok(Json(response))
}

pub(crate) async fn create_provider_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProviderCreateResponse>), StatusCode> {
    let normalized =
        normalize_provider_create_request(&request).map_err(|_| StatusCode::BAD_REQUEST)?;
    let primary_channel_id = request
        .channel_bindings
        .iter()
        .find(|binding| binding.is_primary)
        .map(|binding| binding.channel_id.as_str())
        .unwrap_or(&request.channel_id);
    let bindings = provider_bindings_from_request(&request);
    let provider = persist_provider_with_bindings_and_extension_id(
        state.store.as_ref(),
        PersistProviderWithBindingsRequest {
            id: &request.id,
            channel_id: primary_channel_id,
            adapter_kind: &normalized.adapter_kind,
            protocol_kind: normalized.protocol_kind.as_deref(),
            extension_id: normalized.extension_id.as_deref(),
            base_url: &request.base_url,
            display_name: &request.display_name,
            channel_bindings: &bindings,
        },
    )
    .await
    .map_err(|error| catalog_write_error_status(&error))?;
    if let Some(supported_models) = request.supported_models.as_ref() {
        sync_provider_models_from_request(state.store.as_ref(), &provider.id, supported_models)
            .await
            .map_err(|error| catalog_write_error_status(&error))?;
    }
    invalidate_catalog_cache_after_mutation().await;
    Ok((
        StatusCode::CREATED,
        Json(ProviderCreateResponse {
            integration: sdkwork_api_app_catalog::provider_integration_view(&provider),
            provider,
        }),
    ))
}

pub(crate) async fn list_tenant_provider_readiness_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<Vec<TenantProviderReadinessResponse>>, StatusCode> {
    let providers = list_providers(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let configured_provider_ids =
        sdkwork_api_app_credential::list_configured_provider_ids_for_tenant(
            state.store.as_ref(),
            &tenant_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        providers
            .into_iter()
            .map(|provider| TenantProviderReadinessResponse {
                id: provider.id.clone(),
                display_name: provider.display_name.clone(),
                protocol_kind: provider.protocol_kind().to_owned(),
                integration: sdkwork_api_app_catalog::provider_integration_view(&provider),
                credential_readiness:
                    sdkwork_api_app_credential::provider_credential_readiness_view(
                        configured_provider_ids.contains(&provider.id),
                    ),
            })
            .collect(),
    ))
}

pub(crate) async fn delete_provider_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let provider_exists = state
        .store
        .find_provider(&provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    if !provider_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    delete_provider_credentials_with_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = delete_catalog_provider(state.store.as_ref(), &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_credentials_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UpstreamCredential>>, StatusCode> {
    list_credentials(state.store.as_ref())
        .await
        .map(|credentials| {
            credentials
                .into_iter()
                .filter(|credential| {
                    credential.tenant_id
                        != sdkwork_api_app_credential::OFFICIAL_PROVIDER_PLATFORM_TENANT_ID
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_credential_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<UpstreamCredential>), StatusCode> {
    let credential = persist_credential_with_secret_and_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &request.tenant_id,
        &request.provider_id,
        &request.key_reference,
        &request.secret_value,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(credential)))
}

pub(crate) async fn list_official_provider_configs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<OfficialProviderConfigResponse>>, StatusCode> {
    let configs = list_official_provider_configs(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut response = Vec::with_capacity(configs.len());
    for config in configs {
        response.push(
            official_provider_config_response(state.store.as_ref(), &config)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        );
    }
    Ok(Json(response))
}

pub(crate) async fn upsert_official_provider_config_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertOfficialProviderConfigRequest>,
) -> Result<(StatusCode, Json<OfficialProviderConfigResponse>), StatusCode> {
    let config = if let Some(api_key) = request.api_key.as_deref() {
        persist_official_provider_config_with_secret_and_manager(
            state.store.as_ref(),
            &state.secret_manager,
            &request.provider_id,
            &request.base_url,
            request.enabled,
            api_key,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        state
            .store
            .upsert_official_provider_config(&OfficialProviderConfig::new(
                &request.provider_id,
                sdkwork_api_app_credential::OFFICIAL_PROVIDER_KEY_REFERENCE,
                &request.base_url,
                request.enabled,
            ))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok((
        StatusCode::CREATED,
        Json(
            official_provider_config_response(state.store.as_ref(), &config)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        ),
    ))
}

pub(crate) async fn delete_credential_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((tenant_id, provider_id, key_reference)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_credential_with_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &tenant_id,
        &provider_id,
        &key_reference,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_channel_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ChannelModelRecord>>, StatusCode> {
    list_channel_models(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_channel_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelModelRequest>,
) -> Result<(StatusCode, Json<ChannelModelRecord>), StatusCode> {
    let record = persist_channel_model_with_metadata(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.model_display_name,
        &request.capabilities,
        request.streaming,
        request.context_window,
        request.description.as_deref(),
    )
    .await
    .map_err(|error| catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_channel_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_channel_model(state.store.as_ref(), &channel_id, &model_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_provider_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderModelRecord>>, StatusCode> {
    list_provider_models(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_provider_accounts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderAccountRecord>>, StatusCode> {
    list_catalog_provider_accounts(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_provider_account_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderAccountRequest>,
) -> Result<(StatusCode, Json<ProviderAccountRecord>), StatusCode> {
    let record = persist_provider_account(
        state.store.as_ref(),
        &request.provider_account_id,
        &request.provider_id,
        &request.display_name,
        &request.account_kind,
        &request.owner_scope,
        request.owner_tenant_id.as_deref(),
        &request.execution_instance_id,
        request.base_url_override.as_deref(),
        request.region.as_deref(),
        request.priority,
        request.weight,
        request.enabled,
        &request.routing_tags,
        request.health_score_hint,
        request.latency_ms_hint,
        request.cost_hint,
        request.success_rate_hint,
        request.throughput_hint,
        request.max_concurrency,
        request.daily_budget,
        request.notes.as_deref(),
    )
    .await
    .map_err(|error| catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_provider_account_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(provider_account_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_provider_account(state.store.as_ref(), &provider_account_id)
        .await
        .map_err(|error| catalog_write_error_status(&error))?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn create_provider_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderModelRequestWithProvider>,
) -> Result<(StatusCode, Json<ProviderModelRecord>), StatusCode> {
    let record = persist_provider_model_with_metadata(
        state.store.as_ref(),
        &request.proxy_provider_id,
        &request.channel_id,
        &request.model_id,
        request.provider_model_id.as_deref(),
        request.provider_model_family.as_deref(),
        (!request.capabilities.is_empty()).then_some(request.capabilities.as_slice()),
        request.streaming,
        request.context_window,
        request.max_output_tokens,
        request.supports_prompt_caching,
        request.supports_reasoning_usage,
        request.supports_tool_usage_metrics,
        request.is_default_route,
        request.is_active,
    )
    .await
    .map_err(|error| catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_provider_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((proxy_provider_id, channel_id, model_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_provider_model(
        state.store.as_ref(),
        &proxy_provider_id,
        &channel_id,
        &model_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelCatalogEntry>>, StatusCode> {
    list_model_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<ModelCatalogEntry>), StatusCode> {
    let model = persist_model_with_metadata(
        state.store.as_ref(),
        &request.external_name,
        &request.provider_id,
        &request.capabilities,
        request.streaming,
        request.context_window,
    )
    .await
    .map_err(|error| catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(model)))
}

pub(crate) async fn delete_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((external_name, provider_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_model_variant(state.store.as_ref(), &external_name, &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_model_prices_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelPriceRecord>>, StatusCode> {
    list_model_prices(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_model_price_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelPriceRequest>,
) -> Result<(StatusCode, Json<ModelPriceRecord>), StatusCode> {
    let record = persist_model_price_with_rates_and_metadata(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.proxy_provider_id,
        &request.currency_code,
        &request.price_unit,
        request.input_price,
        request.output_price,
        request.cache_read_price,
        request.cache_write_price,
        request.request_price,
        &request.price_source_kind,
        request.billing_notes.as_deref(),
        request.pricing_tiers,
        request.is_active,
    )
    .await
    .map_err(|error| catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_model_price_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id, proxy_provider_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_model_price(
        state.store.as_ref(),
        &channel_id,
        &model_id,
        &proxy_provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn provider_bindings_from_request(request: &CreateProviderRequest) -> Vec<ProviderChannelBinding> {
    let mut bindings = if request.channel_bindings.is_empty() {
        vec![ProviderChannelBinding::primary(
            &request.id,
            &request.channel_id,
        )]
    } else {
        request
            .channel_bindings
            .iter()
            .map(|binding| {
                let base = ProviderChannelBinding::new(&request.id, &binding.channel_id);
                if binding.is_primary {
                    ProviderChannelBinding::primary(&request.id, &binding.channel_id)
                } else {
                    base
                }
            })
            .collect::<Vec<_>>()
    };

    if !bindings
        .iter()
        .any(|binding| binding.channel_id == request.channel_id)
    {
        bindings.push(ProviderChannelBinding::primary(
            &request.id,
            &request.channel_id,
        ));
    }

    bindings
}

fn normalize_provider_create_request(
    request: &CreateProviderRequest,
) -> anyhow::Result<NormalizedProviderCreateRequest> {
    let normalized = normalize_provider_integration(
        request.adapter_kind.as_deref(),
        request.protocol_kind.as_deref(),
        request.extension_id.as_deref(),
        request.default_plugin_family.as_deref(),
    )?;
    Ok(NormalizedProviderCreateRequest {
        adapter_kind: normalized.adapter_kind,
        protocol_kind: normalized.protocol_kind,
        extension_id: normalized.extension_id,
    })
}

fn catalog_write_error_status(error: &anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("required")
        || message.contains("not registered")
        || message.contains("not bound")
        || message.contains("execution_instance_id must reference")
        || message.contains("provider-account can be saved")
        || message.contains("must exist before pricing can be saved")
        || message.contains("unsupported default_plugin_family")
        || message.contains("cannot override")
        || message.contains("must match")
    {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub(crate) struct CreateProviderModelRequestWithProvider {
    pub(crate) proxy_provider_id: String,
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    pub(crate) provider_model_id: Option<String>,
    pub(crate) provider_model_family: Option<String>,
    pub(crate) capabilities: Vec<ModelCapability>,
    pub(crate) streaming: Option<bool>,
    pub(crate) context_window: Option<u64>,
    pub(crate) max_output_tokens: Option<u64>,
    pub(crate) supports_prompt_caching: bool,
    pub(crate) supports_reasoning_usage: bool,
    pub(crate) supports_tool_usage_metrics: bool,
    pub(crate) is_default_route: bool,
    pub(crate) is_active: bool,
}

impl From<(String, CreateProviderModelRequest)> for CreateProviderModelRequestWithProvider {
    fn from((proxy_provider_id, request): (String, CreateProviderModelRequest)) -> Self {
        Self {
            proxy_provider_id,
            channel_id: request.channel_id,
            model_id: request.model_id,
            provider_model_id: request.provider_model_id,
            provider_model_family: request.provider_model_family,
            capabilities: request.capabilities,
            streaming: request.streaming,
            context_window: request.context_window,
            max_output_tokens: request.max_output_tokens,
            supports_prompt_caching: request.supports_prompt_caching,
            supports_reasoning_usage: request.supports_reasoning_usage,
            supports_tool_usage_metrics: request.supports_tool_usage_metrics,
            is_default_route: request.is_default_route,
            is_active: request.is_active,
        }
    }
}

async fn sync_provider_models_from_request(
    store: &dyn AdminStore,
    proxy_provider_id: &str,
    supported_models: &[CreateProviderModelRequest],
) -> anyhow::Result<()> {
    let mut requested_keys = std::collections::HashSet::new();
    for record in supported_models {
        requested_keys.insert(format!("{}::{}", record.channel_id, record.model_id));
        persist_provider_model_with_metadata(
            store,
            proxy_provider_id,
            &record.channel_id,
            &record.model_id,
            record.provider_model_id.as_deref(),
            record.provider_model_family.as_deref(),
            (!record.capabilities.is_empty()).then_some(record.capabilities.as_slice()),
            record.streaming,
            record.context_window,
            record.max_output_tokens,
            record.supports_prompt_caching,
            record.supports_reasoning_usage,
            record.supports_tool_usage_metrics,
            record.is_default_route,
            record.is_active,
        )
        .await?;
    }
    for existing in store
        .list_provider_models_for_provider(proxy_provider_id)
        .await?
    {
        if !requested_keys.contains(&format!("{}::{}", existing.channel_id, existing.model_id)) {
            store
                .delete_provider_model(
                    &existing.proxy_provider_id,
                    &existing.channel_id,
                    &existing.model_id,
                )
                .await?;
        }
    }
    Ok(())
}

async fn official_provider_config_response(
    store: &dyn AdminStore,
    config: &OfficialProviderConfig,
) -> Result<OfficialProviderConfigResponse, anyhow::Error> {
    Ok(OfficialProviderConfigResponse {
        provider_id: config.provider_id.clone(),
        base_url: config.base_url.clone(),
        enabled: config.enabled,
        secret_configured: official_provider_secret_configured(store, &config.provider_id).await?,
    })
}
