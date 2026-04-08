use super::*;

async fn load_portal_async_job_or_404(
    state: &PortalApiState,
    claims: &AuthenticatedPortalClaims,
    job_id: &str,
) -> Result<AsyncJobRecord, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subject =
        gateway_auth_subject_from_request_context(&portal_workspace_request_context(&workspace));
    let job = find_async_job(state.store.as_ref(), job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if job.tenant_id != subject.tenant_id
        || job.organization_id != subject.organization_id
        || job.user_id != subject.user_id
    {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(job)
}

pub(crate) async fn list_async_jobs_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AsyncJobRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subject =
        gateway_auth_subject_from_request_context(&portal_workspace_request_context(&workspace));
    let mut jobs = list_async_jobs(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|job| {
            job.tenant_id == subject.tenant_id
                && job.organization_id == subject.organization_id
                && job.user_id == subject.user_id
        })
        .collect::<Vec<_>>();
    jobs.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| left.job_id.cmp(&right.job_id))
    });
    Ok(Json(jobs))
}

pub(crate) async fn list_async_job_attempts_handler(
    claims: AuthenticatedPortalClaims,
    Path(job_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AsyncJobAttemptRecord>>, StatusCode> {
    let _job = load_portal_async_job_or_404(&state, &claims, &job_id).await?;
    let mut attempts = list_async_job_attempts(state.store.as_ref(), &job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    attempts.sort_by_key(|attempt| attempt.attempt_id);
    Ok(Json(attempts))
}

pub(crate) async fn list_async_job_assets_handler(
    claims: AuthenticatedPortalClaims,
    Path(job_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AsyncJobAssetRecord>>, StatusCode> {
    let _job = load_portal_async_job_or_404(&state, &claims, &job_id).await?;
    let mut assets = list_async_job_assets(state.store.as_ref(), &job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    assets.sort_by(|left, right| left.asset_id.cmp(&right.asset_id));
    Ok(Json(assets))
}

