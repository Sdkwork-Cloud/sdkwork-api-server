use super::*;

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
        active_sibling_plan_ids.contains(&plan.pricing_plan_id)
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
        let published_rate = super::build_pricing_rate_with_status(rate, "active", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&published_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in rates.iter().filter(|rate| {
        active_sibling_plan_ids.contains(&rate.pricing_plan_id)
    }) {
        let archived_rate = super::build_pricing_rate_with_status(rate, "archived", now_ms);
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
        let scheduled_rate = super::build_pricing_rate_with_status(rate, "planned", now_ms);
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
        let retired_rate = super::build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&retired_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(retired_plan)))
}
