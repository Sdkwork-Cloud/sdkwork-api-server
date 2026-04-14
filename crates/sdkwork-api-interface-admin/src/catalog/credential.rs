use super::*;

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
    claims: AuthenticatedAdminClaims,
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
    audit::record_admin_audit_event(
        &state,
        &claims,
        "credential.create",
        "credential",
        format!(
            "{}:{}:{}",
            credential.tenant_id, credential.provider_id, credential.key_reference
        ),
        audit::APPROVAL_SCOPE_SECRET_CONTROL,
    )
    .await?;
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
    claims: AuthenticatedAdminClaims,
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

    audit::record_admin_audit_event(
        &state,
        &claims,
        "official_provider_config.upsert",
        "official_provider_config",
        config.provider_id.clone(),
        audit::APPROVAL_SCOPE_SECRET_CONTROL,
    )
    .await?;

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
    claims: AuthenticatedAdminClaims,
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
        audit::record_admin_audit_event(
            &state,
            &claims,
            "credential.delete",
            "credential",
            format!("{tenant_id}:{provider_id}:{key_reference}"),
            audit::APPROVAL_SCOPE_SECRET_CONTROL,
        )
        .await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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
