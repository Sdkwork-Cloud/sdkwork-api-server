use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateApiKeyRequest {
    environment: String,
    label: String,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    api_key_group_id: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateApiKeyStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateApiKeyGroupRequest {
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateApiKeyGroupRequest {
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}


pub(crate) async fn list_api_keys_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

pub(crate) async fn list_api_key_groups_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<ApiKeyGroupRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_key_groups(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

pub(crate) async fn create_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyGroupRequest>,
) -> Result<(StatusCode, Json<ApiKeyGroupRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key_group(
        state.store.as_ref(),
        &claims.claims().sub,
        PortalApiKeyGroupInput {
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    .map(|group| (StatusCode::CREATED, Json(group)))
    .map_err(portal_error_response)
}

pub(crate) async fn update_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyGroupRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_portal_api_key_group(
        state.store.as_ref(),
        &claims.claims().sub,
        &group_id,
        PortalApiKeyGroupInput {
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(group) => Ok(Json(group)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
    }
}

pub(crate) async fn update_api_key_group_status_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyStatusRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_portal_api_key_group_active(
        state.store.as_ref(),
        &claims.claims().sub,
        &group_id,
        request.active,
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(group) => Ok(Json(group)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
    }
}

pub(crate) async fn delete_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deleted =
        delete_portal_api_key_group(state.store.as_ref(), &claims.claims().sub, &group_id)
            .await
            .map_err(portal_error_response)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        )))
    }
}

pub(crate) async fn create_api_key_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key_with_metadata(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.environment,
        &request.label,
        request.expires_at_ms,
        request.api_key.as_deref(),
        request.notes.as_deref(),
        request.api_key_group_id.as_deref(),
    )
    .await
    .map(|created| (StatusCode::CREATED, Json(created)))
    .map_err(portal_error_response)
}

pub(crate) async fn update_api_key_status_handler(
    claims: AuthenticatedPortalClaims,
    Path(hashed_key): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyStatusRequest>,
) -> Result<Json<GatewayApiKeyRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_portal_api_key_active(
        state.store.as_ref(),
        &claims.claims().sub,
        &hashed_key,
        request.active,
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(record) => Ok(Json(record)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key not found".to_owned(),
        ))),
    }
}

pub(crate) async fn delete_api_key_handler(
    claims: AuthenticatedPortalClaims,
    Path(hashed_key): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deleted = delete_portal_api_key(state.store.as_ref(), &claims.claims().sub, &hashed_key)
        .await
        .map_err(portal_error_response)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(portal_error_response(PortalIdentityError::NotFound(
            "api key not found".to_owned(),
        )))
    }
}

