use super::*;
use sdkwork_api_app_catalog::normalize_commercial_pricing_plan_code;

pub(crate) async fn synchronize_canonical_pricing_lifecycle_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<PricingLifecycleSynchronizationReport>, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let report =
        synchronize_due_pricing_plan_lifecycle_with_report(commercial_billing.as_ref(), now_ms)
            .await
            .map_err(commercial_billing_error_response)?;
    Ok(Json(report))
}

pub(crate) async fn list_canonical_pricing_plans_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PricingPlanRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    plans.sort_by_key(|plan| plan.pricing_plan_id);
    Ok(Json(plans))
}

fn build_canonical_pricing_plan_record(
    pricing_plan_id: u64,
    request: &CreateCommercialPricingPlanRequest,
    created_at_ms: u64,
    updated_at_ms: u64,
) -> Result<PricingPlanRecord, (StatusCode, Json<ErrorResponse>)> {
    let plan_code = request.plan_code.trim();
    let plan_code = normalize_commercial_pricing_plan_code(plan_code)
        .map_err(|error| error_response(StatusCode::BAD_REQUEST, error.to_string()))?
        .unwrap_or_else(|| plan_code.to_owned());
    let display_name = request.display_name.trim();
    let status = request.status.trim();

    if plan_code.is_empty()
        || display_name.is_empty()
        || status.is_empty()
        || request.plan_version == 0
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "pricing plan requires non-empty code, display name, status, and plan version",
        ));
    }

    if let Some(effective_to_ms) = request.effective_to_ms {
        if effective_to_ms < request.effective_from_ms {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "pricing plan effective_to_ms must be greater than or equal to effective_from_ms",
            ));
        }
    }

    Ok(PricingPlanRecord::new(
        pricing_plan_id,
        request.tenant_id,
        request.organization_id,
        plan_code,
        request.plan_version,
    )
    .with_display_name(display_name.to_owned())
    .with_currency_code(request.currency_code.trim())
    .with_credit_unit_code(request.credit_unit_code.trim())
    .with_status(status.to_owned())
    .with_effective_from_ms(request.effective_from_ms)
    .with_effective_to_ms(request.effective_to_ms)
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

fn resolve_cloned_pricing_plan_version(
    source_plan: &PricingPlanRecord,
    plans: &[PricingPlanRecord],
    requested_version: Option<u64>,
) -> Result<u64, (StatusCode, Json<ErrorResponse>)> {
    let plan_version = requested_version.unwrap_or_else(|| {
        plans
            .iter()
            .filter(|plan| {
                plan.tenant_id == source_plan.tenant_id
                    && plan.organization_id == source_plan.organization_id
                    && plan.plan_code == source_plan.plan_code
            })
            .map(|plan| plan.plan_version)
            .max()
            .unwrap_or(source_plan.plan_version)
            + 1
    });

    if plan_version == 0 {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "cloned pricing plan requires a positive plan version",
        ));
    }

    let version_exists = plans.iter().any(|plan| {
        plan.pricing_plan_id != source_plan.pricing_plan_id
            && plan.tenant_id == source_plan.tenant_id
            && plan.organization_id == source_plan.organization_id
            && plan.plan_code == source_plan.plan_code
            && plan.plan_version == plan_version
    });
    if version_exists {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "pricing plan {} version {} already exists",
                source_plan.plan_code, plan_version
            ),
        ));
    }

    Ok(plan_version)
}

fn resolve_cloned_pricing_plan_display_name(
    source_plan: &PricingPlanRecord,
    requested_display_name: Option<String>,
    plan_version: u64,
) -> String {
    normalize_optional_admin_text(requested_display_name).unwrap_or_else(|| {
        let base_name = if source_plan.display_name.trim().is_empty() {
            source_plan.plan_code.as_str()
        } else {
            source_plan.display_name.as_str()
        };
        format!("{base_name} v{plan_version}")
    })
}

pub(crate) fn build_pricing_plan_with_status(
    plan: &PricingPlanRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingPlanRecord {
    PricingPlanRecord::new(
        plan.pricing_plan_id,
        plan.tenant_id,
        plan.organization_id,
        plan.plan_code.clone(),
        plan.plan_version,
    )
    .with_display_name(plan.display_name.clone())
    .with_currency_code(plan.currency_code.clone())
    .with_credit_unit_code(plan.credit_unit_code.clone())
    .with_status(status.to_owned())
    .with_effective_from_ms(plan.effective_from_ms)
    .with_effective_to_ms(plan.effective_to_ms)
    .with_created_at_ms(plan.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

pub(crate) fn build_pricing_rate_with_status(
    rate: &PricingRateRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingRateRecord {
    PricingRateRecord::new(
        rate.pricing_rate_id,
        rate.tenant_id,
        rate.organization_id,
        rate.pricing_plan_id,
        rate.metric_code.clone(),
    )
    .with_capability_code(rate.capability_code.clone())
    .with_model_code(rate.model_code.clone())
    .with_provider_code(rate.provider_code.clone())
    .with_charge_unit(rate.charge_unit.clone())
    .with_pricing_method(rate.pricing_method.clone())
    .with_quantity_step(rate.quantity_step)
    .with_unit_price(rate.unit_price)
    .with_display_price_unit(rate.display_price_unit.clone())
    .with_minimum_billable_quantity(rate.minimum_billable_quantity)
    .with_minimum_charge(rate.minimum_charge)
    .with_rounding_increment(rate.rounding_increment)
    .with_rounding_mode(rate.rounding_mode.clone())
    .with_included_quantity(rate.included_quantity)
    .with_priority(rate.priority)
    .with_notes(rate.notes.clone())
    .with_status(status.to_owned())
    .with_created_at_ms(rate.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

pub(crate) async fn create_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let pricing_plan = build_canonical_pricing_plan_record(
        next_admin_pricing_record_id(now_ms),
        &request,
        now_ms,
        now_ms,
    )?;

    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plan = commercial_billing
        .insert_pricing_plan_record(&pricing_plan)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::CREATED, Json(plan)))
}

pub(crate) async fn update_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let existing_plan = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    let pricing_plan = build_canonical_pricing_plan_record(
        pricing_plan_id,
        &request,
        existing_plan.created_at_ms,
        unix_timestamp_ms(),
    )?;
    let plan = commercial_billing
        .insert_pricing_plan_record(&pricing_plan)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::OK, Json(plan)))
}

pub(crate) async fn clone_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CloneCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let source_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    let cloned_plan_version =
        resolve_cloned_pricing_plan_version(&source_plan, &plans, request.plan_version)?;
    let cloned_status = {
        let status = request.status.trim();
        if status.is_empty() {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "cloned pricing plan requires a non-empty status",
            ));
        }
        status.to_owned()
    };
    let cloned_display_name = resolve_cloned_pricing_plan_display_name(
        &source_plan,
        request.display_name,
        cloned_plan_version,
    );
    let now_ms = unix_timestamp_ms();
    let cloned_plan = PricingPlanRecord::new(
        next_admin_pricing_record_id(now_ms),
        source_plan.tenant_id,
        source_plan.organization_id,
        source_plan.plan_code.clone(),
        cloned_plan_version,
    )
    .with_display_name(cloned_display_name)
    .with_currency_code(source_plan.currency_code.clone())
    .with_credit_unit_code(source_plan.credit_unit_code.clone())
    .with_status(cloned_status.clone())
    .with_effective_from_ms(source_plan.effective_from_ms)
    .with_effective_to_ms(source_plan.effective_to_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    let inserted_plan = commercial_billing
        .insert_pricing_plan_record(&cloned_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    let source_rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    for source_rate in source_rates
        .into_iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let cloned_rate = PricingRateRecord::new(
            next_admin_pricing_record_id(now_ms),
            source_rate.tenant_id,
            source_rate.organization_id,
            inserted_plan.pricing_plan_id,
            source_rate.metric_code.clone(),
        )
        .with_capability_code(source_rate.capability_code.clone())
        .with_model_code(source_rate.model_code.clone())
        .with_provider_code(source_rate.provider_code.clone())
        .with_charge_unit(source_rate.charge_unit.clone())
        .with_pricing_method(source_rate.pricing_method.clone())
        .with_quantity_step(source_rate.quantity_step)
        .with_unit_price(source_rate.unit_price)
        .with_display_price_unit(source_rate.display_price_unit.clone())
        .with_minimum_billable_quantity(source_rate.minimum_billable_quantity)
        .with_minimum_charge(source_rate.minimum_charge)
        .with_rounding_increment(source_rate.rounding_increment)
        .with_rounding_mode(source_rate.rounding_mode.clone())
        .with_included_quantity(source_rate.included_quantity)
        .with_priority(source_rate.priority)
        .with_notes(source_rate.notes.clone())
        .with_status(cloned_status.clone())
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms);
        commercial_billing
            .insert_pricing_rate_record(&cloned_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::CREATED, Json(inserted_plan)))
}

pub(crate) async fn publish_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(_request): Json<PublishCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let target_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    let rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !rates
        .iter()
        .any(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be published without rates"),
        ));
    }

    let now_ms = unix_timestamp_ms();
    if target_plan.effective_from_ms > now_ms {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be published before effective_from_ms"),
        ));
    }

    let active_sibling_plan_ids = plans
        .iter()
        .filter(|plan| {
            plan.pricing_plan_id != pricing_plan_id
                && plan.tenant_id == target_plan.tenant_id
                && plan.organization_id == target_plan.organization_id
                && plan.plan_code == target_plan.plan_code
                && plan.status == "active"
        })
        .map(|plan| plan.pricing_plan_id)
        .collect::<Vec<_>>();

    let published_plan = build_pricing_plan_with_status(&target_plan, "active", now_ms);
    let published_plan = commercial_billing
        .insert_pricing_plan_record(&published_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for archived_plan in plans.iter().filter(|plan| {
        active_sibling_plan_ids
            .iter()
            .any(|sibling_id| *sibling_id == plan.pricing_plan_id)
    }) {
        let archived_plan = build_pricing_plan_with_status(archived_plan, "archived", now_ms);
        commercial_billing
            .insert_pricing_plan_record(&archived_plan)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in rates
        .iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let published_rate = build_pricing_rate_with_status(rate, "active", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&published_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in rates.iter().filter(|rate| {
        active_sibling_plan_ids
            .iter()
            .any(|sibling_id| *sibling_id == rate.pricing_plan_id)
    }) {
        let archived_rate = build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&archived_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(published_plan)))
}

pub(crate) async fn schedule_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(_request): Json<ScheduleCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let target_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    if target_plan.status == "archived" {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be scheduled from archived status"),
        ));
    }

    let rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !rates
        .iter()
        .any(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be scheduled without rates"),
        ));
    }

    let now_ms = unix_timestamp_ms();
    if target_plan.effective_from_ms <= now_ms {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "pricing plan {pricing_plan_id} can only be scheduled for a future effective_from_ms"
            ),
        ));
    }

    let scheduled_plan = build_pricing_plan_with_status(&target_plan, "planned", now_ms);
    let scheduled_plan = commercial_billing
        .insert_pricing_plan_record(&scheduled_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for rate in rates
        .iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let scheduled_rate = build_pricing_rate_with_status(rate, "planned", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&scheduled_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(scheduled_plan)))
}

pub(crate) async fn retire_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(_request): Json<RetireCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let target_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;
    let now_ms = unix_timestamp_ms();
    let retired_plan = build_pricing_plan_with_status(&target_plan, "archived", now_ms);
    let retired_plan = commercial_billing
        .insert_pricing_plan_record(&retired_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    let rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    for rate in rates
        .iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let retired_rate = build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&retired_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(retired_plan)))
}

pub(crate) async fn list_canonical_pricing_rates_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PricingRateRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    rates.sort_by_key(|rate| rate.pricing_rate_id);
    Ok(Json(rates))
}

fn build_canonical_pricing_rate_record(
    pricing_rate_id: u64,
    request: &CreateCommercialPricingRateRequest,
    created_at_ms: u64,
    updated_at_ms: u64,
) -> Result<PricingRateRecord, (StatusCode, Json<ErrorResponse>)> {
    let metric_code = request.metric_code.trim();
    let charge_unit = request.charge_unit.trim();
    let pricing_method = request.pricing_method.trim();
    let display_price_unit = request.display_price_unit.trim();
    let rounding_mode = request.rounding_mode.trim();
    let status = request.status.trim();

    let invalid = metric_code.is_empty()
        || charge_unit.is_empty()
        || pricing_method.is_empty()
        || display_price_unit.is_empty()
        || rounding_mode.is_empty()
        || status.is_empty()
        || request.quantity_step <= 0.0
        || request.unit_price < 0.0
        || request.minimum_billable_quantity < 0.0
        || request.minimum_charge < 0.0
        || request.rounding_increment <= 0.0
        || request.included_quantity < 0.0;

    if invalid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "pricing rate requires metric, charge unit, pricing method, display unit, positive quantity and rounding step, and non-negative commercial amounts",
        ));
    }

    Ok(PricingRateRecord::new(
        pricing_rate_id,
        request.tenant_id,
        request.organization_id,
        request.pricing_plan_id,
        metric_code.to_owned(),
    )
    .with_capability_code(normalize_optional_admin_text(
        request.capability_code.clone(),
    ))
    .with_model_code(normalize_optional_admin_text(request.model_code.clone()))
    .with_provider_code(normalize_optional_admin_text(request.provider_code.clone()))
    .with_charge_unit(charge_unit.to_owned())
    .with_pricing_method(pricing_method.to_owned())
    .with_quantity_step(request.quantity_step)
    .with_unit_price(request.unit_price)
    .with_display_price_unit(display_price_unit.to_owned())
    .with_minimum_billable_quantity(request.minimum_billable_quantity)
    .with_minimum_charge(request.minimum_charge)
    .with_rounding_increment(request.rounding_increment)
    .with_rounding_mode(rounding_mode.to_owned())
    .with_included_quantity(request.included_quantity)
    .with_priority(request.priority)
    .with_notes(normalize_optional_admin_text(request.notes.clone()))
    .with_status(status.to_owned())
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

pub(crate) async fn create_canonical_pricing_rate_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingRateRequest>,
) -> Result<(StatusCode, Json<PricingRateRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let pricing_plan_exists = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .any(|plan| plan.pricing_plan_id == request.pricing_plan_id);
    if !pricing_plan_exists {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {} does not exist", request.pricing_plan_id),
        ));
    }

    let now_ms = unix_timestamp_ms();
    let pricing_rate = build_canonical_pricing_rate_record(
        next_admin_pricing_record_id(now_ms),
        &request,
        now_ms,
        now_ms,
    )?;
    let rate = commercial_billing
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::CREATED, Json(rate)))
}

pub(crate) async fn update_canonical_pricing_rate_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_rate_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingRateRequest>,
) -> Result<(StatusCode, Json<PricingRateRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !plans
        .iter()
        .any(|plan| plan.pricing_plan_id == request.pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {} does not exist", request.pricing_plan_id),
        ));
    }

    let existing_rate = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .find(|rate| rate.pricing_rate_id == pricing_rate_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing rate {pricing_rate_id} does not exist"),
            )
        })?;

    let pricing_rate = build_canonical_pricing_rate_record(
        pricing_rate_id,
        &request,
        existing_rate.created_at_ms,
        unix_timestamp_ms(),
    )?;
    let rate = commercial_billing
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::OK, Json(rate)))
}

pub(crate) async fn list_quota_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<QuotaPolicy>>, StatusCode> {
    list_quota_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_quota_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateQuotaPolicyRequest>,
) -> Result<(StatusCode, Json<QuotaPolicy>), StatusCode> {
    let policy = create_quota_policy(
        &request.policy_id,
        &request.project_id,
        request.max_units,
        request.enabled,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_quota_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}
