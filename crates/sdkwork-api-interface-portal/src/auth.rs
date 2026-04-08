use super::*;

pub(crate) async fn register_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<PortalAuthSession>), (StatusCode, Json<ErrorResponse>)> {
    register_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &request.display_name,
        &state.jwt_signing_secret,
    )
    .await
    .map(|session| (StatusCode::CREATED, Json(session)))
    .map_err(portal_error_response)
}

pub(crate) async fn login_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<PortalAuthSession>, (StatusCode, Json<ErrorResponse>)> {
    login_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

pub(crate) async fn me_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalUserProfile>, StatusCode> {
    load_portal_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

pub(crate) async fn change_password_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_portal_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}


