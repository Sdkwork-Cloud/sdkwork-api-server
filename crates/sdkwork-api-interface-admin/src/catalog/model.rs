use super::*;

pub(crate) async fn list_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelCatalogEntry>>, StatusCode> {
    list_model_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<ModelCatalogEntry>), StatusCode> {
    let model = persist_model_with_metadata(
        state.store.as_ref(),
        &request.external_name,
        &request.provider_id,
        &request.capabilities,
        request.streaming,
        request.context_window,
    )
    .await
    .map_err(|error| super::catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(model)))
}

pub(crate) async fn delete_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((external_name, provider_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_model_variant(state.store.as_ref(), &external_name, &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_model_prices_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelPriceRecord>>, StatusCode> {
    list_model_prices(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_model_price_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelPriceRequest>,
) -> Result<(StatusCode, Json<ModelPriceRecord>), StatusCode> {
    let record = persist_model_price_with_rates_and_metadata(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.proxy_provider_id,
        &request.currency_code,
        &request.price_unit,
        request.input_price,
        request.output_price,
        request.cache_read_price,
        request.cache_write_price,
        request.request_price,
        &request.price_source_kind,
        request.billing_notes.as_deref(),
        request.pricing_tiers,
        request.is_active,
    )
    .await
    .map_err(|error| super::catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "model_price.create",
        "model_price",
        format!(
            "{}:{}:{}",
            record.channel_id, record.model_id, record.proxy_provider_id
        ),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_model_price_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id, proxy_provider_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_model_price(
        state.store.as_ref(),
        &channel_id,
        &model_id,
        &proxy_provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        audit::record_admin_audit_event(
            &state,
            &claims,
            "model_price.delete",
            "model_price",
            format!("{channel_id}:{model_id}:{proxy_provider_id}"),
            audit::APPROVAL_SCOPE_FINANCE_CONTROL,
        )
        .await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
