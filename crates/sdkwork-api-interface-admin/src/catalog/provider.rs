use super::*;
use axum::extract::Query;

#[derive(Debug, Clone)]
struct NormalizedProviderCreateRequest {
    adapter_kind: String,
    protocol_kind: Option<String>,
    extension_id: Option<String>,
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
    claims: AuthenticatedAdminClaims,
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
    .map_err(|error| super::catalog_write_error_status(&error))?;
    if let Some(supported_models) = request.supported_models.as_ref() {
        super::provider_model::sync_provider_models_from_request(
            state.store.as_ref(),
            &provider.id,
            supported_models,
        )
        .await
        .map_err(|error| super::catalog_write_error_status(&error))?;
    }
    invalidate_catalog_cache_after_mutation().await;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "provider.create",
        "provider",
        provider.id.clone(),
        audit::APPROVAL_SCOPE_CATALOG_CONTROL,
    )
    .await?;
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
    claims: AuthenticatedAdminClaims,
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
        audit::record_admin_audit_event(
            &state,
            &claims,
            "provider.delete",
            "provider",
            provider_id,
            audit::APPROVAL_SCOPE_CATALOG_CONTROL,
        )
        .await?;
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
