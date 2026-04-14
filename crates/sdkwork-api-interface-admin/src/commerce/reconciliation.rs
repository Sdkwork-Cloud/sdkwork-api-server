use super::*;

pub(crate) async fn list_commerce_webhook_inbox_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceWebhookInboxRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_webhook_inbox(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_webhook_delivery_attempts_handler(
    _claims: AuthenticatedAdminClaims,
    Path(webhook_inbox_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceWebhookDeliveryAttemptRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_webhook_delivery_attempts(state.store.as_ref(), &webhook_inbox_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_reconciliation_runs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceReconciliationRunRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_reconciliation_runs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn create_commerce_reconciliation_run_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<AdminCommerceReconciliationRunCreateRequest>,
) -> Result<Json<CommerceReconciliationRunRecord>, (StatusCode, Json<ErrorResponse>)> {
    let run = create_admin_commerce_reconciliation_run(
        state.store.as_ref(),
        &state.secret_manager,
        &request,
    )
    .await
    .map_err(admin_commerce_error_response)?;
    crate::audit::record_admin_audit_event(
        &state,
        &claims,
        "commerce_reconciliation_run.create",
        "commerce_reconciliation_run",
        run.reconciliation_run_id.clone(),
        crate::audit::APPROVAL_SCOPE_COMMERCE_CONTROL,
    )
    .await
    .map_err(|_| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record admin audit event",
        )
    })?;
    Ok(Json(run))
}

pub(crate) async fn list_commerce_reconciliation_items_handler(
    _claims: AuthenticatedAdminClaims,
    Path(reconciliation_run_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceReconciliationItemRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_reconciliation_items(state.store.as_ref(), &reconciliation_run_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}
