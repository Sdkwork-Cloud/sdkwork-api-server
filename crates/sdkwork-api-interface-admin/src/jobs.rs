use super::*;

async fn load_admin_async_job_or_404(
    state: &AdminApiState,
    job_id: &str,
) -> Result<AsyncJobRecord, StatusCode> {
    find_async_job(state.store.as_ref(), job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

pub(crate) async fn list_async_jobs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobRecord>>, StatusCode> {
    list_async_jobs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_async_job_attempts_handler(
    _claims: AuthenticatedAdminClaims,
    Path(job_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobAttemptRecord>>, StatusCode> {
    let _job = load_admin_async_job_or_404(&state, &job_id).await?;
    list_async_job_attempts(state.store.as_ref(), &job_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_async_job_assets_handler(
    _claims: AuthenticatedAdminClaims,
    Path(job_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobAssetRecord>>, StatusCode> {
    let _job = load_admin_async_job_or_404(&state, &job_id).await?;
    list_async_job_assets(state.store.as_ref(), &job_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_async_job_callbacks_handler(
    _claims: AuthenticatedAdminClaims,
    Path(job_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobCallbackRecord>>, StatusCode> {
    let _job = load_admin_async_job_or_404(&state, &job_id).await?;
    list_async_job_callbacks(state.store.as_ref(), &job_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
