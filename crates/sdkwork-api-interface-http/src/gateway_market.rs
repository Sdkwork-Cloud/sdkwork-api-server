use sdkwork_api_app_billing::{
    resolve_payable_account_for_gateway_request_context, summarize_account_balance,
    AccountBalanceSnapshot,
};
use sdkwork_api_app_commerce::{
    load_portal_commerce_catalog, preview_portal_commerce_quote,
    reclaim_expired_coupon_reservations_for_code_if_needed, CommerceError, PortalApiProduct,
    PortalCommerceQuote, PortalCommerceQuoteRequest, PortalProductOffer,
};
use sdkwork_api_app_identity::gateway_auth_subject_from_request_context;
use sdkwork_api_app_marketing::{
    confirm_coupon_redemption, reserve_coupon_redemption, rollback_coupon_redemption,
    validate_coupon_stack, CouponValidationDecision, MarketingServiceError,
};
use sdkwork_api_app_rate_limit::{
    check_coupon_rate_limit, coupon_actor_bucket, CouponRateLimitAction,
};
use sdkwork_api_domain_billing::{AccountBenefitLotRecord, AccountRecord};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponRedemptionRecord, CouponRollbackRecord, CouponRollbackType,
    CouponTemplateRecord, MarketingCampaignRecord, MarketingSubjectScope,
};
use sdkwork_api_storage_core::{
    AccountKernelStore, AtomicCouponConfirmationCommand, AtomicCouponReservationCommand,
    AtomicCouponRollbackCommand,
};

#[derive(Debug, Serialize, ToSchema)]
struct GatewayApiErrorBody {
    message: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayApiErrorResponse {
    error: GatewayApiErrorBody,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayMarketProductsResponse {
    items: Vec<PortalApiProduct>,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayMarketOffersResponse {
    items: Vec<PortalProductOffer>,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum GatewayCouponEffectKind {
    CheckoutDiscount,
    AccountEntitlement,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponApplicabilitySummary {
    target_kinds: Vec<String>,
    all_target_kinds_eligible: bool,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponEffectSummary {
    effect_kind: GatewayCouponEffectKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    discount_percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    discount_amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grant_units: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
struct GatewayCouponValidationRequest {
    coupon_code: String,
    subject_scope: MarketingSubjectScope,
    target_kind: String,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponValidationDecisionResponse {
    eligible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rejection_reason: Option<String>,
    reservable_budget_minor: u64,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponValidationResponse {
    decision: GatewayCouponValidationDecisionResponse,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    applicability: GatewayCouponApplicabilitySummary,
    effect: GatewayCouponEffectSummary,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize, ToSchema)]
struct GatewayCouponReservationRequest {
    coupon_code: String,
    subject_scope: MarketingSubjectScope,
    target_kind: String,
    reserve_amount_minor: u64,
    ttl_ms: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponReservationResponse {
    reservation: sdkwork_api_domain_marketing::CouponReservationRecord,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    applicability: GatewayCouponApplicabilitySummary,
    effect: GatewayCouponEffectSummary,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize, ToSchema)]
struct GatewayCouponRedemptionConfirmRequest {
    coupon_reservation_id: String,
    subsidy_amount_minor: u64,
    #[serde(default)]
    order_id: Option<String>,
    #[serde(default)]
    payment_event_id: Option<String>,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponRedemptionConfirmResponse {
    reservation: sdkwork_api_domain_marketing::CouponReservationRecord,
    redemption: CouponRedemptionRecord,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    applicability: GatewayCouponApplicabilitySummary,
    effect: GatewayCouponEffectSummary,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize, ToSchema)]
struct GatewayCouponRedemptionRollbackRequest {
    coupon_redemption_id: String,
    rollback_type: CouponRollbackType,
    restored_budget_minor: u64,
    restored_inventory_count: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCouponRedemptionRollbackResponse {
    redemption: CouponRedemptionRecord,
    rollback: CouponRollbackRecord,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    applicability: GatewayCouponApplicabilitySummary,
    effect: GatewayCouponEffectSummary,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCommercialAccountResponse {
    account: AccountRecord,
    balance: AccountBalanceSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCommercialBenefitLotItem {
    lot_id: u64,
    benefit_type: sdkwork_api_domain_billing::AccountBenefitType,
    source_type: sdkwork_api_domain_billing::AccountBenefitSourceType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_id: Option<u64>,
    status: sdkwork_api_domain_billing::AccountBenefitLotStatus,
    original_quantity: f64,
    remaining_quantity: f64,
    held_quantity: f64,
    issued_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope_order_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GatewayCommercialBenefitLotsQuery {
    #[serde(default)]
    after_lot_id: Option<u64>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCommercialBenefitLotPage {
    limit: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    after_lot_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next_after_lot_id: Option<u64>,
    has_more: bool,
    returned_count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
struct GatewayCommercialBenefitLotsResponse {
    account: AccountRecord,
    balance: AccountBalanceSnapshot,
    page: GatewayCommercialBenefitLotPage,
    benefit_lots: Vec<GatewayCommercialBenefitLotItem>,
}

#[derive(Debug, Clone)]
struct GatewayMarketingCouponContext {
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

fn gateway_error_response(status: StatusCode, message: impl Into<String>) -> Response {
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

fn clamp_gateway_benefit_lot_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(limit) if limit > 0 => limit.min(200),
        _ => 100,
    }
}

fn gateway_not_implemented_response(message: impl Into<String>) -> Response {
    gateway_error_response(StatusCode::NOT_IMPLEMENTED, message)
}

fn gateway_internal_error_response(message: impl Into<String>) -> Response {
    gateway_error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

fn gateway_commerce_error_response(error: CommerceError) -> Response {
    let status = match error {
        CommerceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CommerceError::NotFound(_) => StatusCode::NOT_FOUND,
        CommerceError::Conflict(_) => StatusCode::CONFLICT,
        CommerceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    gateway_error_response(status, error.to_string())
}

fn marketing_atomic_status(error: anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("changed concurrently")
        || message.contains("already exists with different state")
        || message.contains(" is missing")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn gateway_current_time_millis() -> Result<u64, Response> {
    current_billing_timestamp_ms()
        .map_err(|_| gateway_internal_error_response("failed to read current timestamp"))
}

fn gateway_account_kernel_store(
    state: &GatewayApiState,
) -> Result<&dyn AccountKernelStore, Response> {
    state.store.account_kernel_store().ok_or_else(|| {
        gateway_not_implemented_response(
            "commercial account routes are unavailable for the current storage runtime",
        )
    })
}

async fn load_gateway_account_context(
    state: &GatewayApiState,
    request: &AuthenticatedGatewayRequest,
) -> Result<(AccountRecord, AccountBalanceSnapshot), Response> {
    let account_store = gateway_account_kernel_store(state)?;
    let account = resolve_payable_account_for_gateway_request_context(account_store, request.context())
        .await
        .map_err(|_| gateway_internal_error_response("failed to resolve commercial account"))?
        .ok_or_else(|| {
            gateway_error_response(StatusCode::NOT_FOUND, "commercial account is not provisioned")
        })?;
    let balance = summarize_account_balance(
        account_store,
        account.account_id,
        gateway_current_time_millis()?,
    )
    .await
    .map_err(|_| gateway_internal_error_response("failed to summarize commercial account balance"))?;
    Ok((account, balance))
}

fn gateway_coupon_validation_decision_response(
    decision: CouponValidationDecision,
) -> GatewayCouponValidationDecisionResponse {
    GatewayCouponValidationDecisionResponse {
        eligible: decision.eligible,
        rejection_reason: decision.rejection_reason,
        reservable_budget_minor: decision.reservable_budget_minor,
    }
}

fn gateway_coupon_applicability_summary(
    template: &CouponTemplateRecord,
) -> GatewayCouponApplicabilitySummary {
    GatewayCouponApplicabilitySummary {
        target_kinds: template.restriction.eligible_target_kinds.clone(),
        all_target_kinds_eligible: template.restriction.eligible_target_kinds.is_empty(),
    }
}

fn gateway_coupon_effect_summary(template: &CouponTemplateRecord) -> GatewayCouponEffectSummary {
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

fn gateway_target_kind_allowed(template: &CouponTemplateRecord, target_kind: &str) -> bool {
    template.restriction.eligible_target_kinds.is_empty()
        || template
            .restriction
            .eligible_target_kinds
            .iter()
            .any(|eligible| eligible == target_kind)
}

async fn load_marketing_coupon_context_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> Result<Option<GatewayMarketingCouponContext>, Response> {
    let normalized = normalize_coupon_code(code);
    reclaim_expired_coupon_reservations_for_code_if_needed(store, &normalized, now_ms)
        .await
        .map_err(|_| gateway_internal_error_response("failed to reclaim expired coupon reservations"))?;

    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon code"))?
    {
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, code_record, now_ms).await?
        {
            return Ok(Some(context));
        }
    }

    Ok(None)
}

async fn load_marketing_coupon_context_from_code_record(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    now_ms: u64,
) -> Result<Option<GatewayMarketingCouponContext>, Response> {
    let Some(template) = store
        .find_coupon_template_record(&code.coupon_template_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon template"))?
    else {
        return Ok(None);
    };

    let campaigns = store
        .list_marketing_campaign_records_for_template(&template.coupon_template_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load marketing campaigns"))?;
    let Some(campaign) = select_effective_marketing_campaign(campaigns, now_ms) else {
        return Ok(None);
    };

    let budgets = store
        .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load campaign budgets"))?;
    let Some(budget) = select_campaign_budget_record(budgets) else {
        return Ok(None);
    };

    Ok(Some(GatewayMarketingCouponContext {
        template,
        campaign,
        budget,
        code,
    }))
}

async fn load_marketing_coupon_context_for_code_id(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    now_ms: u64,
) -> Result<GatewayMarketingCouponContext, Response> {
    let Some(code) = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon code"))?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon code not found",
        ));
    };

    load_marketing_coupon_context_from_code_record(store, code, now_ms)
        .await?
        .ok_or_else(|| gateway_internal_error_response("coupon context is unavailable"))
}

fn select_effective_marketing_campaign(
    campaigns: Vec<MarketingCampaignRecord>,
    now_ms: u64,
) -> Option<MarketingCampaignRecord> {
    campaigns
        .into_iter()
        .filter(|campaign| campaign.is_effective_at(now_ms))
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
        })
}

fn select_campaign_budget_record(budgets: Vec<CampaignBudgetRecord>) -> Option<CampaignBudgetRecord> {
    budgets.into_iter().max_by(|left, right| {
        left.updated_at_ms
            .cmp(&right.updated_at_ms)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    })
}

fn normalize_coupon_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn normalize_idempotency_key(value: Option<&str>) -> Result<Option<String>, Response> {
    let Some(value) = value.map(str::trim) else {
        return Ok(None);
    };
    if value.is_empty() || value.len() > 128 || value.chars().any(|ch| ch.is_control()) {
        return Err(gateway_error_response(
            StatusCode::BAD_REQUEST,
            "invalid idempotency key",
        ));
    }
    Ok(Some(value.to_owned()))
}

fn resolve_idempotency_key(
    headers: &HeaderMap,
    body_value: Option<&str>,
) -> Result<Option<String>, Response> {
    let body_value = normalize_idempotency_key(body_value)?;
    let header_value = match headers.get("idempotency-key") {
        Some(value) => normalize_idempotency_key(Some(
            value
                .to_str()
                .map_err(|_| gateway_error_response(StatusCode::BAD_REQUEST, "invalid idempotency key header"))?,
        ))?,
        None => None,
    };
    match (body_value, header_value) {
        (Some(body_value), Some(header_value)) if body_value != header_value => Err(
            gateway_error_response(
                StatusCode::BAD_REQUEST,
                "conflicting idempotency keys between header and body",
            ),
        ),
        (Some(body_value), Some(_)) | (Some(body_value), None) => Ok(Some(body_value)),
        (None, Some(header_value)) => Ok(Some(header_value)),
        (None, None) => Ok(None),
    }
}

fn marketing_subject_scope_token(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

fn slugify_id_fragment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn derive_coupon_reservation_id(
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    target_kind: &str,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_reservation_{}_{}_{}_{}",
        marketing_subject_scope_token(subject_scope),
        slugify_id_fragment(subject_id),
        slugify_id_fragment(target_kind),
        slugify_id_fragment(idempotency_key),
    )
}

fn derive_coupon_redemption_id(
    reservation: &sdkwork_api_domain_marketing::CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_redemption_{}_{}_{}",
        marketing_subject_scope_token(reservation.subject_scope),
        slugify_id_fragment(&reservation.subject_id),
        slugify_id_fragment(idempotency_key),
    )
}

fn derive_coupon_rollback_id(
    reservation: &sdkwork_api_domain_marketing::CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_rollback_{}_{}_{}",
        marketing_subject_scope_token(reservation.subject_scope),
        slugify_id_fragment(&reservation.subject_id),
        slugify_id_fragment(idempotency_key),
    )
}

async fn gateway_marketing_subject_id(
    state: &GatewayApiState,
    request: &AuthenticatedGatewayRequest,
    scope: MarketingSubjectScope,
) -> Result<String, Response> {
    match scope {
        MarketingSubjectScope::User => Ok(
            gateway_auth_subject_from_request_context(request.context())
                .user_id
                .to_string(),
        ),
        MarketingSubjectScope::Project => Ok(request.project_id().to_owned()),
        MarketingSubjectScope::Workspace => {
            Ok(format!("{}:{}", request.tenant_id(), request.project_id()))
        }
        MarketingSubjectScope::Account => {
            let (account, _) = load_gateway_account_context(state, request).await?;
            Ok(account.account_id.to_string())
        }
    }
}

async fn enforce_gateway_coupon_rate_limit(
    store: &dyn AdminStore,
    request: &AuthenticatedGatewayRequest,
    action: CouponRateLimitAction,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    coupon_code: &str,
) -> Result<(), Response> {
    let actor_bucket = coupon_actor_bucket(marketing_subject_scope_token(subject_scope), subject_id);
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

fn coupon_code_is_exclusive(template: &CouponTemplateRecord) -> bool {
    !matches!(
        template.distribution_kind,
        CouponDistributionKind::SharedCode
    )
}

fn code_after_reservation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    reserved_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        reserved_code.clone()
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_confirmation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        original_code
            .clone()
            .with_status(CouponCodeStatus::Redeemed)
            .with_updated_at_ms(now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn restore_coupon_code_availability(
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    let next_status = if original_code.expires_at_ms.is_some_and(|value| now_ms > value) {
        CouponCodeStatus::Expired
    } else {
        CouponCodeStatus::Available
    };
    original_code
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms)
}

fn code_after_rollback(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        restore_coupon_code_availability(original_code, now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn reserve_campaign_budget(
    budget: &CampaignBudgetRecord,
    reserved_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_add(reserved_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            budget.consumed_budget_minor,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn confirm_campaign_budget(
    budget: &CampaignBudgetRecord,
    consumed_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_sub(consumed_amount_minor);
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_add(consumed_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn rollback_campaign_budget(
    budget: &CampaignBudgetRecord,
    restored_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_sub(restored_amount_minor);
    budget
        .clone()
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            budget.reserved_budget_minor,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn campaign_budget_status_after_mutation(
    total_budget_minor: u64,
    reserved_budget_minor: u64,
    consumed_budget_minor: u64,
    prior_status: CampaignBudgetStatus,
) -> CampaignBudgetStatus {
    if matches!(
        prior_status,
        CampaignBudgetStatus::Closed | CampaignBudgetStatus::Draft
    ) {
        return prior_status;
    }

    let available_budget_minor = total_budget_minor
        .saturating_sub(reserved_budget_minor)
        .saturating_sub(consumed_budget_minor);
    if available_budget_minor == 0 {
        CampaignBudgetStatus::Exhausted
    } else {
        CampaignBudgetStatus::Active
    }
}

fn parse_scope_order_id(scope_json: Option<&str>) -> Option<String> {
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

fn gateway_benefit_lot_item(lot: AccountBenefitLotRecord) -> GatewayCommercialBenefitLotItem {
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

async fn list_market_products_handler(
    _request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Result<Json<GatewayMarketProductsResponse>, Response> {
    let catalog = load_portal_commerce_catalog(state.store.as_ref())
        .await
        .map_err(gateway_commerce_error_response)?;
    Ok(Json(GatewayMarketProductsResponse {
        items: catalog.products,
    }))
}

async fn list_market_offers_handler(
    _request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Result<Json<GatewayMarketOffersResponse>, Response> {
    let catalog = load_portal_commerce_catalog(state.store.as_ref())
        .await
        .map_err(gateway_commerce_error_response)?;
    Ok(Json(GatewayMarketOffersResponse { items: catalog.offers }))
}

async fn create_market_quote_handler(
    _request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<Json<PortalCommerceQuote>, Response> {
    preview_portal_commerce_quote(state.store.as_ref(), &request)
        .await
        .map(Json)
        .map_err(gateway_commerce_error_response)
}

async fn gateway_coupon_reservation_owned_by_subject(
    store: &dyn AdminStore,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    reservation_id: &str,
) -> Result<sdkwork_api_domain_marketing::CouponReservationRecord, Response> {
    let Some(reservation) = store
        .find_coupon_reservation_record(reservation_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon reservation"))?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon reservation not found",
        ));
    };

    if reservation.subject_scope != subject_scope || reservation.subject_id != subject_id {
        return Err(gateway_error_response(
            StatusCode::FORBIDDEN,
            "coupon reservation is not owned by the current subject",
        ));
    }

    Ok(reservation)
}

async fn gateway_coupon_redemption_owned_by_subject(
    store: &dyn AdminStore,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    redemption_id: &str,
) -> Result<CouponRedemptionRecord, Response> {
    let Some(redemption) = store
        .find_coupon_redemption_record(redemption_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon redemption"))?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon redemption not found",
        ));
    };

    let reservation = gateway_coupon_reservation_owned_by_subject(
        store,
        subject_scope,
        subject_id,
        &redemption.coupon_reservation_id,
    )
    .await?;
    if reservation.coupon_reservation_id != redemption.coupon_reservation_id {
        return Err(gateway_error_response(
            StatusCode::FORBIDDEN,
            "coupon redemption is not owned by the current subject",
        ));
    }

    Ok(redemption)
}

async fn validate_coupon_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Json(payload): Json<GatewayCouponValidationRequest>,
) -> Result<Json<GatewayCouponValidationResponse>, Response> {
    let subject_id = gateway_marketing_subject_id(&state, &request, payload.subject_scope).await?;
    let target_kind = payload.target_kind.trim();
    if target_kind.is_empty() {
        return Err(gateway_error_response(
            StatusCode::BAD_REQUEST,
            "target_kind is required",
        ));
    }
    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Validate,
        payload.subject_scope,
        &subject_id,
        &payload.coupon_code,
    )
    .await?;

    let now_ms = gateway_current_time_millis()?;
    let Some(context) =
        load_marketing_coupon_context_by_value(state.store.as_ref(), &payload.coupon_code, now_ms)
            .await?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon code not found",
        ));
    };

    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        payload.order_amount_minor,
        payload.reserve_amount_minor,
    );
    let decision = if decision.eligible && !gateway_target_kind_allowed(&context.template, target_kind)
    {
        CouponValidationDecision::rejected("target_kind_not_eligible")
    } else {
        decision
    };

    Ok(Json(GatewayCouponValidationResponse {
        decision: gateway_coupon_validation_decision_response(decision),
        applicability: gateway_coupon_applicability_summary(&context.template),
        effect: gateway_coupon_effect_summary(&context.template),
        template: context.template,
        campaign: context.campaign,
        budget: context.budget,
        code: context.code,
    }))
}

async fn reserve_coupon_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Json(payload): Json<GatewayCouponReservationRequest>,
) -> Result<(StatusCode, Json<GatewayCouponReservationResponse>), Response> {
    let subject_id = gateway_marketing_subject_id(&state, &request, payload.subject_scope).await?;
    let target_kind = payload.target_kind.trim();
    if target_kind.is_empty() {
        return Err(gateway_error_response(
            StatusCode::BAD_REQUEST,
            "target_kind is required",
        ));
    }

    let idempotency_key = resolve_idempotency_key(&headers, payload.idempotency_key.as_deref())?;
    let now_ms = gateway_current_time_millis()?;
    let coupon_reservation_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_reservation_id(payload.subject_scope, &subject_id, target_kind, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_reservation_{}_{}",
                normalize_coupon_code(&payload.coupon_code).to_ascii_lowercase(),
                now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_reservation) = state
            .store
            .find_coupon_reservation_record(&coupon_reservation_id)
            .await
            .map_err(|_| gateway_internal_error_response("failed to load coupon reservation"))?
        {
            let Some(existing_code) = state
                .store
                .find_coupon_code_record(&existing_reservation.coupon_code_id)
                .await
                .map_err(|_| gateway_internal_error_response("failed to load coupon code"))?
            else {
                return Err(gateway_internal_error_response("coupon code is unavailable"));
            };
            let existing_ttl_ms = existing_reservation
                .expires_at_ms
                .saturating_sub(existing_reservation.created_at_ms);
            if existing_reservation.subject_scope != payload.subject_scope
                || existing_reservation.subject_id != subject_id
                || normalize_coupon_code(&existing_code.code_value)
                    != normalize_coupon_code(&payload.coupon_code)
                || existing_reservation.budget_reserved_minor != payload.reserve_amount_minor
                || existing_ttl_ms != payload.ttl_ms
            {
                return Err(gateway_error_response(
                    StatusCode::CONFLICT,
                    "idempotent reservation replay does not match the original request",
                ));
            }

            let context = load_marketing_coupon_context_from_code_record(
                state.store.as_ref(),
                existing_code,
                now_ms,
            )
            .await?
            .ok_or_else(|| gateway_internal_error_response("coupon context is unavailable"))?;

            return Ok((
                StatusCode::OK,
                Json(GatewayCouponReservationResponse {
                    reservation: existing_reservation,
                    applicability: gateway_coupon_applicability_summary(&context.template),
                    effect: gateway_coupon_effect_summary(&context.template),
                    template: context.template,
                    campaign: context.campaign,
                    budget: context.budget,
                    code: context.code,
                }),
            ));
        }
    }

    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Reserve,
        payload.subject_scope,
        &subject_id,
        &payload.coupon_code,
    )
    .await?;

    let Some(context) =
        load_marketing_coupon_context_by_value(state.store.as_ref(), &payload.coupon_code, now_ms)
            .await?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon code not found",
        ));
    };
    if !gateway_target_kind_allowed(&context.template, target_kind) {
        return Err(gateway_error_response(
            StatusCode::BAD_REQUEST,
            "target_kind_not_eligible",
        ));
    }

    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        payload.reserve_amount_minor,
        payload.reserve_amount_minor,
    );
    if !decision.eligible {
        return Err(gateway_error_response(
            StatusCode::CONFLICT,
            decision
                .rejection_reason
                .unwrap_or_else(|| "coupon reservation rejected".to_owned()),
        ));
    }

    let (reserved_code, reservation) = reserve_coupon_redemption(
        &context.code,
        coupon_reservation_id,
        payload.subject_scope,
        subject_id,
        payload.reserve_amount_minor,
        now_ms,
        payload.ttl_ms,
    )
    .map_err(|error: MarketingServiceError| {
        gateway_error_response(StatusCode::BAD_REQUEST, error.to_string())
    })?;

    let atomic_result = state
        .store
        .reserve_coupon_redemption_atomic(&AtomicCouponReservationCommand {
            template_to_persist: None,
            campaign_to_persist: None,
            expected_budget: context.budget.clone(),
            next_budget: reserve_campaign_budget(&context.budget, payload.reserve_amount_minor, now_ms),
            expected_code: context.code.clone(),
            next_code: code_after_reservation(&context.template, &context.code, &reserved_code, now_ms),
            reservation,
        })
        .await
        .map_err(|error| {
            gateway_error_response(marketing_atomic_status(error), "failed to reserve coupon")
        })?;

    Ok((
        StatusCode::CREATED,
        Json(GatewayCouponReservationResponse {
            reservation: atomic_result.reservation,
            applicability: gateway_coupon_applicability_summary(&context.template),
            effect: gateway_coupon_effect_summary(&context.template),
            template: context.template,
            campaign: context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        }),
    ))
}

async fn confirm_coupon_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Json(payload): Json<GatewayCouponRedemptionConfirmRequest>,
) -> Result<Json<GatewayCouponRedemptionConfirmResponse>, Response> {
    let reservation = state
        .store
        .find_coupon_reservation_record(&payload.coupon_reservation_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon reservation"))?
        .ok_or_else(|| {
            gateway_error_response(StatusCode::NOT_FOUND, "coupon reservation not found")
        })?;
    let subject_id =
        gateway_marketing_subject_id(&state, &request, reservation.subject_scope).await?;
    let reservation = gateway_coupon_reservation_owned_by_subject(
        state.store.as_ref(),
        reservation.subject_scope,
        &subject_id,
        &payload.coupon_reservation_id,
    )
    .await?;

    if payload.subsidy_amount_minor > reservation.budget_reserved_minor {
        return Err(gateway_error_response(
            StatusCode::BAD_REQUEST,
            "subsidy amount exceeds reserved coupon budget",
        ));
    }

    let idempotency_key = resolve_idempotency_key(&headers, payload.idempotency_key.as_deref())?;
    let now_ms = gateway_current_time_millis()?;
    let coupon_redemption_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_redemption_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_redemption_{}_{}",
                reservation.coupon_reservation_id, now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_redemption) = state
            .store
            .find_coupon_redemption_record(&coupon_redemption_id)
            .await
            .map_err(|_| gateway_internal_error_response("failed to load coupon redemption"))?
        {
            if existing_redemption.coupon_reservation_id != reservation.coupon_reservation_id
                || existing_redemption.subsidy_amount_minor != payload.subsidy_amount_minor
                || existing_redemption.order_id != payload.order_id
                || existing_redemption.payment_event_id != payload.payment_event_id
            {
                return Err(gateway_error_response(
                    StatusCode::CONFLICT,
                    "idempotent redemption replay does not match the original request",
                ));
            }

            let current_reservation = gateway_coupon_reservation_owned_by_subject(
                state.store.as_ref(),
                reservation.subject_scope,
                &subject_id,
                &existing_redemption.coupon_reservation_id,
            )
            .await?;
            let context = load_marketing_coupon_context_for_code_id(
                state.store.as_ref(),
                &existing_redemption.coupon_code_id,
                now_ms,
            )
            .await?;

            return Ok(Json(GatewayCouponRedemptionConfirmResponse {
                reservation: current_reservation,
                redemption: existing_redemption,
                applicability: gateway_coupon_applicability_summary(&context.template),
                effect: gateway_coupon_effect_summary(&context.template),
                template: context.template,
                campaign: context.campaign,
                budget: context.budget,
                code: context.code,
            }));
        }
    }

    let Some(code) = state
        .store
        .find_coupon_code_record(&reservation.coupon_code_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon code"))?
    else {
        return Err(gateway_error_response(StatusCode::NOT_FOUND, "coupon code not found"));
    };
    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Confirm,
        reservation.subject_scope,
        &reservation.subject_id,
        &code.code_value,
    )
    .await?;

    let Some(context) =
        load_marketing_coupon_context_from_code_record(state.store.as_ref(), code, now_ms).await?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon context is unavailable",
        ));
    };

    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        coupon_redemption_id,
        context.code.coupon_code_id.clone(),
        context.template.coupon_template_id.clone(),
        payload.subsidy_amount_minor,
        payload.order_id.clone(),
        payload.payment_event_id.clone(),
        now_ms,
    )
    .map_err(|error: MarketingServiceError| {
        gateway_error_response(StatusCode::CONFLICT, error.to_string())
    })?;

    let atomic_result = state
        .store
        .confirm_coupon_redemption_atomic(&AtomicCouponConfirmationCommand {
            expected_budget: context.budget.clone(),
            next_budget: confirm_campaign_budget(&context.budget, payload.subsidy_amount_minor, now_ms),
            expected_code: context.code.clone(),
            next_code: code_after_confirmation(&context.template, &context.code, now_ms),
            expected_reservation: reservation,
            next_reservation: confirmed_reservation,
            redemption,
        })
        .await
        .map_err(|error| {
            gateway_error_response(
                marketing_atomic_status(error),
                "failed to confirm coupon redemption",
            )
        })?;

    Ok(Json(GatewayCouponRedemptionConfirmResponse {
        reservation: atomic_result.reservation,
        redemption: atomic_result.redemption,
        applicability: gateway_coupon_applicability_summary(&context.template),
        effect: gateway_coupon_effect_summary(&context.template),
        template: context.template,
        campaign: context.campaign,
        budget: atomic_result.budget,
        code: atomic_result.code,
    }))
}

async fn rollback_coupon_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Json(payload): Json<GatewayCouponRedemptionRollbackRequest>,
) -> Result<Json<GatewayCouponRedemptionRollbackResponse>, Response> {
    let redemption = state
        .store
        .find_coupon_redemption_record(&payload.coupon_redemption_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon redemption"))?
        .ok_or_else(|| gateway_error_response(StatusCode::NOT_FOUND, "coupon redemption not found"))?;
    let reservation = state
        .store
        .find_coupon_reservation_record(&redemption.coupon_reservation_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon reservation"))?
        .ok_or_else(|| gateway_error_response(StatusCode::NOT_FOUND, "coupon reservation not found"))?;
    let subject_id =
        gateway_marketing_subject_id(&state, &request, reservation.subject_scope).await?;
    let redemption = gateway_coupon_redemption_owned_by_subject(
        state.store.as_ref(),
        reservation.subject_scope,
        &subject_id,
        &payload.coupon_redemption_id,
    )
    .await?;
    let reservation = gateway_coupon_reservation_owned_by_subject(
        state.store.as_ref(),
        reservation.subject_scope,
        &subject_id,
        &redemption.coupon_reservation_id,
    )
    .await?;

    if payload.restored_budget_minor > redemption.subsidy_amount_minor {
        return Err(gateway_error_response(
            StatusCode::BAD_REQUEST,
            "restored budget exceeds redeemed coupon subsidy",
        ));
    }

    let idempotency_key = resolve_idempotency_key(&headers, payload.idempotency_key.as_deref())?;
    let now_ms = gateway_current_time_millis()?;
    let coupon_rollback_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_rollback_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_rollback_{}_{}",
                redemption.coupon_redemption_id, now_ms
            )
        });
    if idempotency_key.is_some() {
        let existing_rollback = state
            .store
            .list_coupon_rollback_records()
            .await
            .map_err(|_| gateway_internal_error_response("failed to list coupon rollbacks"))?
            .into_iter()
            .find(|rollback| rollback.coupon_rollback_id == coupon_rollback_id);
        if let Some(existing_rollback) = existing_rollback {
            if existing_rollback.coupon_redemption_id != redemption.coupon_redemption_id
                || existing_rollback.rollback_type != payload.rollback_type
                || existing_rollback.restored_budget_minor != payload.restored_budget_minor
                || existing_rollback.restored_inventory_count != payload.restored_inventory_count
            {
                return Err(gateway_error_response(
                    StatusCode::CONFLICT,
                    "idempotent rollback replay does not match the original request",
                ));
            }

            let current_redemption = gateway_coupon_redemption_owned_by_subject(
                state.store.as_ref(),
                reservation.subject_scope,
                &subject_id,
                &existing_rollback.coupon_redemption_id,
            )
            .await?;
            let context = load_marketing_coupon_context_for_code_id(
                state.store.as_ref(),
                &current_redemption.coupon_code_id,
                now_ms,
            )
            .await?;

            return Ok(Json(GatewayCouponRedemptionRollbackResponse {
                redemption: current_redemption,
                rollback: existing_rollback,
                applicability: gateway_coupon_applicability_summary(&context.template),
                effect: gateway_coupon_effect_summary(&context.template),
                template: context.template,
                campaign: context.campaign,
                budget: context.budget,
                code: context.code,
            }));
        }
    }

    let Some(code) = state
        .store
        .find_coupon_code_record(&redemption.coupon_code_id)
        .await
        .map_err(|_| gateway_internal_error_response("failed to load coupon code"))?
    else {
        return Err(gateway_error_response(StatusCode::NOT_FOUND, "coupon code not found"));
    };
    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Rollback,
        reservation.subject_scope,
        &reservation.subject_id,
        &code.code_value,
    )
    .await?;

    let Some(context) =
        load_marketing_coupon_context_from_code_record(state.store.as_ref(), code, now_ms).await?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon context is unavailable",
        ));
    };

    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        coupon_rollback_id,
        payload.rollback_type,
        payload.restored_budget_minor,
        payload.restored_inventory_count,
        now_ms,
    )
    .map_err(|error: MarketingServiceError| {
        gateway_error_response(StatusCode::CONFLICT, error.to_string())
    })?;

    let atomic_result = state
        .store
        .rollback_coupon_redemption_atomic(&AtomicCouponRollbackCommand {
            expected_budget: context.budget.clone(),
            next_budget: rollback_campaign_budget(&context.budget, payload.restored_budget_minor, now_ms),
            expected_code: context.code.clone(),
            next_code: code_after_rollback(&context.template, &context.code, now_ms),
            expected_redemption: redemption,
            next_redemption: rolled_back_redemption,
            rollback,
        })
        .await
        .map_err(|error| {
            gateway_error_response(
                marketing_atomic_status(error),
                "failed to rollback coupon redemption",
            )
        })?;

    Ok(Json(GatewayCouponRedemptionRollbackResponse {
        redemption: atomic_result.redemption,
        rollback: atomic_result.rollback,
        applicability: gateway_coupon_applicability_summary(&context.template),
        effect: gateway_coupon_effect_summary(&context.template),
        template: context.template,
        campaign: context.campaign,
        budget: atomic_result.budget,
        code: atomic_result.code,
    }))
}

async fn get_commercial_account_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Result<Json<GatewayCommercialAccountResponse>, Response> {
    let (account, balance) = load_gateway_account_context(&state, &request).await?;
    Ok(Json(GatewayCommercialAccountResponse { account, balance }))
}

async fn list_commercial_account_benefit_lots_handler(
    request: AuthenticatedGatewayRequest,
    Query(query): Query<GatewayCommercialBenefitLotsQuery>,
    State(state): State<GatewayApiState>,
) -> Result<Json<GatewayCommercialBenefitLotsResponse>, Response> {
    let (account, balance) = load_gateway_account_context(&state, &request).await?;
    let account_store = gateway_account_kernel_store(&state)?;
    let limit = clamp_gateway_benefit_lot_limit(query.limit);
    let fetch_limit = limit.saturating_add(1);
    let mut benefit_lot_records = account_store
        .list_account_benefit_lots_for_account(account.account_id, query.after_lot_id, fetch_limit)
        .await
        .map_err(|_| gateway_internal_error_response("failed to list account benefit lots"))?;
    let has_more = benefit_lot_records.len() > limit;
    if has_more {
        benefit_lot_records.truncate(limit);
    }
    let next_after_lot_id = if has_more {
        benefit_lot_records.last().map(|lot| lot.lot_id)
    } else {
        None
    };
    let benefit_lots = benefit_lot_records
        .into_iter()
        .map(gateway_benefit_lot_item)
        .collect::<Vec<_>>();

    Ok(Json(GatewayCommercialBenefitLotsResponse {
        account,
        balance,
        page: GatewayCommercialBenefitLotPage {
            limit,
            after_lot_id: query.after_lot_id,
            next_after_lot_id,
            has_more,
            returned_count: benefit_lots.len(),
        },
        benefit_lots,
    }))
}
