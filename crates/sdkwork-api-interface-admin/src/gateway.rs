use super::*;

pub(crate) async fn list_api_keys_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::SecretRead)?;
    list_gateway_api_keys(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_api_key_groups_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ApiKeyGroupRecord>>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityRead)?;
    list_api_key_groups(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_api_key_group_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyGroupRequest>,
) -> Result<(StatusCode, Json<ApiKeyGroupRecord>), (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    let group = create_api_key_group(
        state.store.as_ref(),
        ApiKeyGroupInput {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
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
    .map_err(admin_error_response)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "api_key_group.create",
        "api_key_group",
        group.group_id.clone(),
        audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to record admin audit event"))?;
    Ok((StatusCode::CREATED, Json(group)))
}

pub(crate) async fn update_api_key_group_handler(
    claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateApiKeyGroupRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    match update_api_key_group(
        state.store.as_ref(),
        &group_id,
        ApiKeyGroupInput {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
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
    {
        Ok(Some(group)) => {
            audit::record_admin_audit_event(
                &state,
                &claims,
                "api_key_group.update",
                "api_key_group",
                group.group_id.clone(),
                audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
            )
            .await
            .map_err(|_| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record admin audit event",
                )
            })?;
            Ok(Json(group))
        }
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

pub(crate) async fn update_api_key_group_status_handler(
    claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    match set_api_key_group_active(state.store.as_ref(), &group_id, request.active).await {
        Ok(Some(group)) => {
            audit::record_admin_audit_event(
                &state,
                &claims,
                "api_key_group.status.update",
                "api_key_group",
                group.group_id.clone(),
                audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
            )
            .await
            .map_err(|_| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record admin audit event",
                )
            })?;
            Ok(Json(group))
        }
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

pub(crate) async fn delete_api_key_group_handler(
    claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    match delete_api_key_group(state.store.as_ref(), &group_id).await {
        Ok(true) => {
            audit::record_admin_audit_event(
                &state,
                &claims,
                "api_key_group.delete",
                "api_key_group",
                group_id,
                audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
            )
            .await
            .map_err(|_| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record admin audit event",
                )
            })?;
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

pub(crate) async fn create_api_key_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::SecretWrite)
        .map_err(|_| admin_forbidden_response())?;
    let metadata_label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{} gateway key", request.environment.trim()));
    let created = sdkwork_api_app_identity::persist_gateway_api_key_with_metadata(
        state.store.as_ref(),
        sdkwork_api_app_identity::PersistGatewayApiKeyInput {
            tenant_id: &request.tenant_id,
            project_id: &request.project_id,
            environment: &request.environment,
            label: &metadata_label,
            expires_at_ms: request.expires_at_ms,
            plaintext_key: request.plaintext_key.as_deref(),
            notes: request.notes.as_deref(),
            api_key_group_id: request.api_key_group_id.as_deref(),
        },
    )
    .await
    .map_err(gateway_api_key_create_error_response)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "gateway_api_key.create",
        "gateway_api_key",
        created.hashed.clone(),
        audit::APPROVAL_SCOPE_SECRET_CONTROL,
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to record admin audit event"))?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub(crate) async fn update_api_key_handler(
    claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<Json<GatewayApiKeyRecord>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::SecretWrite)
        .map_err(|_| admin_forbidden_response())?;
    match update_gateway_api_key_metadata(
        state.store.as_ref(),
        sdkwork_api_app_identity::UpdateGatewayApiKeyMetadataInput {
            hashed_key: &hashed_key,
            tenant_id: &request.tenant_id,
            project_id: &request.project_id,
            environment: &request.environment,
            label: &request.label,
            expires_at_ms: request.expires_at_ms,
            notes: request.notes.as_deref(),
            api_key_group_id: request.api_key_group_id.as_deref(),
        },
    )
    .await
    {
        Ok(Some(record)) => {
            audit::record_admin_audit_event(
                &state,
                &claims,
                "gateway_api_key.update",
                "gateway_api_key",
                record.hashed_key.clone(),
                audit::APPROVAL_SCOPE_SECRET_CONTROL,
            )
            .await
            .map_err(|_| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record admin audit event",
                )
            })?;
            Ok(Json(record))
        }
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "gateway api key not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

pub(crate) async fn update_api_key_status_handler(
    claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<GatewayApiKeyRecord>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::SecretWrite)?;
    match set_gateway_api_key_active(state.store.as_ref(), &hashed_key, request.active)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(record) => {
            audit::record_admin_audit_event(
                &state,
                &claims,
                "gateway_api_key.status.update",
                "gateway_api_key",
                record.hashed_key.clone(),
                audit::APPROVAL_SCOPE_SECRET_CONTROL,
            )
            .await?;
            Ok(Json(record))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub(crate) async fn delete_api_key_handler(
    claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::SecretWrite)?;
    let deleted = delete_gateway_api_key(state.store.as_ref(), &hashed_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        audit::record_admin_audit_event(
            &state,
            &claims,
            "gateway_api_key.delete",
            "gateway_api_key",
            hashed_key,
            audit::APPROVAL_SCOPE_SECRET_CONTROL,
        )
        .await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_rate_limit_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RateLimitPolicy>>, StatusCode> {
    list_rate_limit_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_rate_limit_window_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RateLimitWindowSnapshot>>, StatusCode> {
    state
        .store
        .list_rate_limit_window_snapshots()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_rate_limit_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRateLimitPolicyRequest>,
) -> Result<(StatusCode, Json<RateLimitPolicy>), StatusCode> {
    let policy = create_rate_limit_policy(
        &request.policy_id,
        &request.project_id,
        request.requests_per_window,
        request.window_seconds,
        request.burst_requests,
        request.enabled,
        request.route_key.as_deref(),
        request.api_key_hash.as_deref(),
        request.model_name.as_deref(),
        request.notes.as_deref(),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_rate_limit_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

fn gateway_api_key_create_error_response(
    error: anyhow::Error,
) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = if looks_like_gateway_api_key_input_error(&message) {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    let body = ErrorResponse {
        error: ErrorBody { message },
    };
    (status, Json(body))
}

pub(crate) fn looks_like_gateway_api_key_input_error(message: &str) -> bool {
    matches!(
        message,
        "tenant_id is required"
            | "project_id is required"
            | "environment is required"
            | "label is required"
            | "expires_at_ms must be in the future"
            | "api key is required when custom key mode is selected"
            | "api_key is required when custom key mode is selected"
            | "api key group not found"
            | "api key group tenant does not match"
            | "api key group project does not match"
            | "api key group environment does not match"
            | "api key group is inactive"
    )
}
