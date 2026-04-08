use super::*;

pub(crate) async fn login_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session = login_admin_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map_err(admin_error_response)?;
    let token = session.token.clone();
    let claims = verify_jwt(&token, &state.jwt_signing_secret)
        .map_err(|error| admin_error_response(AdminIdentityError::Storage(error)))?;
    Ok(Json(LoginResponse {
        token,
        claims,
        user: session.user,
    }))
}

pub(crate) async fn me_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<AdminUserProfile>, StatusCode> {
    load_admin_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

pub(crate) async fn change_password_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_admin_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(admin_error_response)
}
