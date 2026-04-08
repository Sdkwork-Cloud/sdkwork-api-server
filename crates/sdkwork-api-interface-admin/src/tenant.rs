use super::*;

pub(crate) async fn list_tenants_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    list_tenants(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_tenant_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), StatusCode> {
    let tenant = persist_tenant(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tenant)))
}

pub(crate) async fn delete_tenant_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(tenant_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let tenant_exists = state
        .store
        .list_tenants()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .any(|tenant| tenant.id == tenant_id);
    if !tenant_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    delete_tenant_credentials_with_manager(state.store.as_ref(), &state.secret_manager, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = delete_workspace_tenant(state.store.as_ref(), &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_projects_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    list_projects(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_project_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    let project = persist_project(
        state.store.as_ref(),
        &request.tenant_id,
        &request.id,
        &request.name,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(project)))
}

pub(crate) async fn delete_project_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_tenant_project(state.store.as_ref(), &project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
