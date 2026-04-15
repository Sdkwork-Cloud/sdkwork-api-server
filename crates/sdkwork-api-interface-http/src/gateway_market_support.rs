use super::*;

pub(super) fn gateway_error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(GatewayApiErrorResponse {
            error: GatewayApiErrorBody {
                message: message.into(),
            },
        }),
    )
        .into_response()
}

pub(super) fn clamp_gateway_benefit_lot_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(limit) if limit > 0 => limit.min(200),
        _ => 100,
    }
}

pub(super) fn gateway_not_implemented_response(message: impl Into<String>) -> Response {
    gateway_error_response(StatusCode::NOT_IMPLEMENTED, message)
}

pub(super) fn gateway_internal_error_response(message: impl Into<String>) -> Response {
    gateway_error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

pub(super) fn gateway_commerce_error_response(error: CommerceError) -> Response {
    let status = match error {
        CommerceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CommerceError::NotFound(_) => StatusCode::NOT_FOUND,
        CommerceError::Conflict(_) => StatusCode::CONFLICT,
        CommerceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    gateway_error_response(status, error.to_string())
}

pub(super) fn gateway_marketing_operation_response(error: MarketingOperationError) -> Response {
    match error {
        MarketingOperationError::InvalidInput(message) => {
            gateway_error_response(StatusCode::BAD_REQUEST, message)
        }
        MarketingOperationError::NotFound(message) => {
            gateway_error_response(StatusCode::NOT_FOUND, message)
        }
        MarketingOperationError::Conflict(message) => {
            gateway_error_response(StatusCode::CONFLICT, message)
        }
        MarketingOperationError::Forbidden(message) => {
            gateway_error_response(StatusCode::FORBIDDEN, message)
        }
        MarketingOperationError::Storage(error) => {
            gateway_error_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
    }
}

pub(super) fn gateway_current_time_millis() -> Result<u64, Response> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .map_err(|_| gateway_internal_error_response("failed to read current timestamp"))
}

pub(super) fn gateway_account_kernel_store(
    state: &GatewayApiState,
) -> Result<&dyn AccountKernelStore, Response> {
    state.store.account_kernel_store().ok_or_else(|| {
        gateway_not_implemented_response(
            "commercial account routes are unavailable for the current storage runtime",
        )
    })
}

pub(super) async fn load_gateway_account_context(
    state: &GatewayApiState,
    request: &AuthenticatedGatewayRequest,
) -> Result<(AccountRecord, AccountBalanceSnapshot), Response> {
    let account = resolve_gateway_account_record(state, request).await?;
    let account_store = gateway_account_kernel_store(state)?;
    let balance = summarize_account_balance(
        account_store,
        account.account_id,
        gateway_current_time_millis()?,
    )
    .await
    .map_err(|_| {
        gateway_internal_error_response("failed to summarize commercial account balance")
    })?;
    Ok((account, balance))
}

pub(super) async fn resolve_gateway_account_record(
    state: &GatewayApiState,
    request: &AuthenticatedGatewayRequest,
) -> Result<AccountRecord, Response> {
    let account_store = gateway_account_kernel_store(state)?;
    resolve_payable_account_for_gateway_request_context(account_store, request.context())
        .await
        .map_err(|_| gateway_internal_error_response("failed to resolve commercial account"))?
        .ok_or_else(|| {
            gateway_error_response(
                StatusCode::NOT_FOUND,
                "commercial account is not provisioned",
            )
        })
}

pub(super) fn gateway_coupon_validation_decision_response(
    decision: CouponValidationDecision,
) -> GatewayCouponValidationDecisionResponse {
    GatewayCouponValidationDecisionResponse {
        eligible: decision.eligible,
        rejection_reason: decision.rejection_reason,
        reservable_budget_minor: decision.reservable_budget_minor,
    }
}

pub(super) fn gateway_coupon_applicability_summary(
    template: &CouponTemplateRecord,
) -> GatewayCouponApplicabilitySummary {
    GatewayCouponApplicabilitySummary {
        target_kinds: template.restriction.eligible_target_kinds.clone(),
        all_target_kinds_eligible: template.restriction.eligible_target_kinds.is_empty(),
    }
}

pub(super) fn gateway_coupon_effect_summary(
    template: &CouponTemplateRecord,
) -> GatewayCouponEffectSummary {
    GatewayCouponEffectSummary {
        effect_kind: if template.benefit.grant_units.is_some() {
            GatewayCouponEffectKind::AccountEntitlement
        } else {
            GatewayCouponEffectKind::CheckoutDiscount
        },
        discount_percent: template.benefit.discount_percent,
        discount_amount_minor: template.benefit.discount_amount_minor,
        grant_units: template.benefit.grant_units,
    }
}

pub(super) fn resolve_idempotency_key(
    headers: &HeaderMap,
    body_value: Option<&str>,
) -> Result<Option<String>, Response> {
    let header_value = headers
        .get("idempotency-key")
        .map(|value| {
            value.to_str().map_err(|_| {
                gateway_error_response(StatusCode::BAD_REQUEST, "invalid idempotency key header")
            })
        })
        .transpose()?;
    resolve_shared_idempotency_key(header_value, body_value).map_err(|error| match error {
        sdkwork_api_app_marketing::MarketingIdempotencyError::InvalidKey => {
            gateway_error_response(StatusCode::BAD_REQUEST, "invalid idempotency key")
        }
        sdkwork_api_app_marketing::MarketingIdempotencyError::ConflictingKeys => {
            gateway_error_response(
                StatusCode::BAD_REQUEST,
                "conflicting idempotency keys between header and body",
            )
        }
    })
}

pub(super) async fn gateway_marketing_subject_id(
    state: &GatewayApiState,
    request: &AuthenticatedGatewayRequest,
    scope: MarketingSubjectScope,
) -> Result<String, Response> {
    match scope {
        MarketingSubjectScope::User => {
            Ok(gateway_auth_subject_from_request_context(request.context())
                .user_id
                .to_string())
        }
        MarketingSubjectScope::Project => Ok(request.project_id().to_owned()),
        MarketingSubjectScope::Workspace => {
            Ok(format!("{}:{}", request.tenant_id(), request.project_id()))
        }
        MarketingSubjectScope::Account => {
            let account = resolve_gateway_account_record(state, request).await?;
            Ok(account.account_id.to_string())
        }
    }
}

pub(super) async fn enforce_gateway_coupon_rate_limit(
    store: &dyn AdminStore,
    request: &AuthenticatedGatewayRequest,
    action: CouponRateLimitAction,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    coupon_code: &str,
) -> Result<(), Response> {
    let actor_bucket =
        coupon_actor_bucket(marketing_subject_scope_token(subject_scope), subject_id);
    let evaluation = check_coupon_rate_limit(
        store,
        request.project_id(),
        action,
        Some(actor_bucket.as_str()),
        Some(coupon_code),
        1,
    )
    .await
    .map_err(|_| gateway_internal_error_response("failed to evaluate coupon rate limit"))?;
    if evaluation.allowed {
        Ok(())
    } else {
        Err(gateway_error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "coupon rate limit exceeded",
        ))
    }
}

pub(super) fn parse_scope_order_id(scope_json: Option<&str>) -> Option<String> {
    let scope_json = scope_json?.trim();
    if scope_json.is_empty() {
        return None;
    }
    let parsed: serde_json::Value = serde_json::from_str(scope_json).ok()?;
    let order_id = parsed.get("order_id")?.as_str()?.trim();
    if order_id.is_empty() {
        None
    } else {
        Some(order_id.to_owned())
    }
}

pub(super) fn gateway_benefit_lot_item(
    lot: AccountBenefitLotRecord,
) -> GatewayCommercialBenefitLotItem {
    let scope_order_id = parse_scope_order_id(lot.scope_json.as_deref());
    GatewayCommercialBenefitLotItem {
        lot_id: lot.lot_id,
        benefit_type: lot.benefit_type,
        source_type: lot.source_type,
        source_id: lot.source_id,
        status: lot.status,
        original_quantity: lot.original_quantity,
        remaining_quantity: lot.remaining_quantity,
        held_quantity: lot.held_quantity,
        issued_at_ms: lot.issued_at_ms,
        expires_at_ms: lot.expires_at_ms,
        scope_json: lot.scope_json,
        scope_order_id,
    }
}
