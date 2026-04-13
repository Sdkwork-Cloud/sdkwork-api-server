use super::*;

pub(crate) async fn list_operator_users_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AdminUserProfile>>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityRead)
        .map_err(|_| admin_forbidden_response())?;
    list_admin_user_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_error_response)
}

pub(crate) async fn upsert_operator_user_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertOperatorUserRequest>,
) -> Result<(StatusCode, Json<AdminUserProfile>), (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    let user = upsert_admin_user(
        state.store.as_ref(),
        request.id.as_deref(),
        &request.email,
        &request.display_name,
        request.password.as_deref(),
        request.role,
        request.active,
    )
    .await
    .map_err(admin_error_response)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "admin_user.create",
        "admin_user",
        user.id.clone(),
        audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to record admin audit event"))?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub(crate) async fn update_operator_user_status_handler(
    claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    let user = set_admin_user_active(state.store.as_ref(), &user_id, request.active)
        .await
        .map_err(admin_error_response)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "admin_user.status.update",
        "admin_user",
        user_id,
        audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to record admin audit event"))?;
    Ok(Json(user))
}

pub(crate) async fn reset_operator_user_password_handler(
    claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResetUserPasswordRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    let user = reset_admin_user_password(state.store.as_ref(), &user_id, &request.new_password)
        .await
        .map_err(admin_error_response)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "admin_user.password.reset",
        "admin_user",
        user_id,
        audit::APPROVAL_SCOPE_IDENTITY_CONTROL,
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to record admin audit event"))?;
    Ok(Json(user))
}

pub(crate) async fn delete_operator_user_handler(
    claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityWrite)
        .map_err(|_| admin_forbidden_response())?;
    if claims.claims().sub == user_id {
        return Err(admin_error_response(AdminIdentityError::Protected(
            "current admin session cannot be deleted".to_owned(),
        )));
    }

    match delete_admin_user(state.store.as_ref(), &user_id).await {
        Ok(true) => {
            audit::record_admin_audit_event(
                &state,
                &claims,
                "admin_user.delete",
                "admin_user",
                user_id,
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
            "admin user not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

pub(crate) async fn list_portal_users_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PortalUserProfile>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_user_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

pub(crate) async fn upsert_portal_user_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPortalUserRequest>,
) -> Result<(StatusCode, Json<PortalUserProfile>), (StatusCode, Json<ErrorResponse>)> {
    upsert_portal_user(
        state.store.as_ref(),
        sdkwork_api_app_identity::UpsertPortalUserInput {
            user_id: request.id.as_deref(),
            email: &request.email,
            display_name: &request.display_name,
            password: request.password.as_deref(),
            workspace_tenant_id: &request.workspace_tenant_id,
            workspace_project_id: &request.workspace_project_id,
            active: request.active,
        },
    )
    .await
    .map(|user| (StatusCode::CREATED, Json(user)))
    .map_err(portal_admin_error_response)
}

pub(crate) async fn update_portal_user_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    set_portal_user_active(state.store.as_ref(), &user_id, request.active)
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

pub(crate) async fn reset_portal_user_password_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResetUserPasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    reset_portal_user_password(state.store.as_ref(), &user_id, &request.new_password)
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

pub(crate) async fn delete_portal_user_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_portal_user(state.store.as_ref(), &user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(portal_admin_error_response(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ))),
        Err(error) => Err(portal_admin_error_response(error)),
    }
}
