use crate::gateway_prelude::*;
use crate::GatewayApiState;
use axum::extract::Query;
use sdkwork_api_app_billing::{
    resolve_payable_account_for_gateway_request_context, summarize_account_balance,
    AccountBalanceSnapshot,
};
use sdkwork_api_app_commerce::{
    load_portal_commerce_catalog, preview_portal_commerce_quote, CommerceError, PortalApiProduct,
    PortalCommerceQuote, PortalCommerceQuoteRequest, PortalProductOffer,
};
use sdkwork_api_app_identity::gateway_auth_subject_from_request_context;
use sdkwork_api_app_marketing::{
    confirm_coupon_for_subject, load_coupon_redemption_context_owned_by_subject,
    load_coupon_reservation_context_owned_by_subject, marketing_subject_scope_token,
    reserve_coupon_for_subject, resolve_idempotency_key as resolve_shared_idempotency_key,
    rollback_coupon_for_subject, validate_coupon_for_subject, ConfirmCouponInput,
    CouponValidationDecision, MarketingOperationError, MarketingRedemptionOwnershipView,
    MarketingReservationOwnershipView, MarketingSubjectSet, ReserveCouponInput,
    RollbackCouponInput, ValidatedCouponResult,
};
use sdkwork_api_app_rate_limit::{
    check_coupon_rate_limit, coupon_actor_bucket, CouponRateLimitAction,
};
use sdkwork_api_domain_billing::{AccountBenefitLotRecord, AccountRecord};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponRedemptionRecord, CouponRollbackRecord,
    CouponRollbackType, CouponTemplateRecord, MarketingCampaignRecord, MarketingSubjectScope,
};
use sdkwork_api_storage_core::AccountKernelStore;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[path = "gateway_market_support.rs"]
mod support;

use support::*;

pub(crate) fn apply_stateful_market_and_commercial_routes(
    router: Router<GatewayApiState>,
) -> Router<GatewayApiState> {
    router
        .route("/market/products", get(list_market_products_handler))
        .route("/market/offers", get(list_market_offers_handler))
        .route("/market/quotes", post(create_market_quote_handler))
        .route("/marketing/coupons/validate", post(validate_coupon_handler))
        .route("/marketing/coupons/reserve", post(reserve_coupon_handler))
        .route("/marketing/coupons/confirm", post(confirm_coupon_handler))
        .route("/marketing/coupons/rollback", post(rollback_coupon_handler))
        .route("/commercial/account", get(get_commercial_account_handler))
        .route(
            "/commercial/account/benefit-lots",
            get(list_commercial_account_benefit_lots_handler),
        )
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GatewayApiErrorBody {
    message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GatewayApiErrorResponse {
    error: GatewayApiErrorBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GatewayMarketProductsResponse {
    items: Vec<PortalApiProduct>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GatewayMarketOffersResponse {
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
pub(crate) struct GatewayCouponValidationRequest {
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
pub(crate) struct GatewayCouponValidationResponse {
    decision: GatewayCouponValidationDecisionResponse,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    applicability: GatewayCouponApplicabilitySummary,
    effect: GatewayCouponEffectSummary,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct GatewayCouponReservationRequest {
    coupon_code: String,
    subject_scope: MarketingSubjectScope,
    target_kind: String,
    #[serde(default)]
    order_amount_minor: u64,
    reserve_amount_minor: u64,
    ttl_ms: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GatewayCouponReservationResponse {
    reservation: sdkwork_api_domain_marketing::CouponReservationRecord,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    applicability: GatewayCouponApplicabilitySummary,
    effect: GatewayCouponEffectSummary,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct GatewayCouponRedemptionConfirmRequest {
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
pub(crate) struct GatewayCouponRedemptionConfirmResponse {
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
pub(crate) struct GatewayCouponRedemptionRollbackRequest {
    coupon_redemption_id: String,
    rollback_type: CouponRollbackType,
    restored_budget_minor: u64,
    restored_inventory_count: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GatewayCouponRedemptionRollbackResponse {
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
pub(crate) struct GatewayCommercialAccountResponse {
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
pub(crate) struct GatewayCommercialBenefitLotsResponse {
    account: AccountRecord,
    balance: AccountBalanceSnapshot,
    page: GatewayCommercialBenefitLotPage,
    benefit_lots: Vec<GatewayCommercialBenefitLotItem>,
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
    Ok(Json(GatewayMarketOffersResponse {
        items: catalog.offers,
    }))
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

async fn gateway_coupon_reservation_context_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    reservation_id: &str,
) -> Result<MarketingReservationOwnershipView, Response> {
    let Some(reservation_view) =
        load_coupon_reservation_context_owned_by_subject(store, subjects, reservation_id)
            .await
            .map_err(|_| gateway_internal_error_response("failed to load coupon reservation"))?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon reservation not found",
        ));
    };

    Ok(reservation_view)
}

async fn gateway_coupon_redemption_context_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    redemption_id: &str,
) -> Result<MarketingRedemptionOwnershipView, Response> {
    let Some(redemption_view) =
        load_coupon_redemption_context_owned_by_subject(store, subjects, redemption_id)
            .await
            .map_err(|_| gateway_internal_error_response("failed to load coupon redemption"))?
    else {
        return Err(gateway_error_response(
            StatusCode::NOT_FOUND,
            "coupon redemption not found",
        ));
    };

    Ok(redemption_view)
}

async fn gateway_marketing_subjects(
    state: &GatewayApiState,
    request: &AuthenticatedGatewayRequest,
) -> Result<MarketingSubjectSet, Response> {
    let user_id = Some(
        gateway_auth_subject_from_request_context(request.context())
            .user_id
            .to_string(),
    );
    let project_id = Some(request.project_id().to_owned());
    let workspace_id = Some(format!("{}:{}", request.tenant_id(), request.project_id()));
    let account_id = match resolve_gateway_account_record(state, request).await {
        Ok(account) => Some(account.account_id.to_string()),
        Err(error) if error.status() == StatusCode::NOT_FOUND => None,
        Err(error) => return Err(error),
    };
    Ok(MarketingSubjectSet::new(
        user_id,
        project_id,
        workspace_id,
        account_id,
    ))
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
    let ValidatedCouponResult { context, decision } = validate_coupon_for_subject(
        state.store.as_ref(),
        &payload.coupon_code,
        payload.subject_scope,
        &subject_id,
        target_kind,
        payload.order_amount_minor,
        payload.reserve_amount_minor,
        now_ms,
    )
    .await
    .map_err(gateway_marketing_operation_response)?;

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
    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Reserve,
        payload.subject_scope,
        &subject_id,
        &payload.coupon_code,
    )
    .await?;
    let now_ms = gateway_current_time_millis()?;
    let result = reserve_coupon_for_subject(
        state.store.as_ref(),
        ReserveCouponInput {
            coupon_code: &payload.coupon_code,
            subject_scope: payload.subject_scope,
            subject_id: &subject_id,
            target_kind,
            order_amount_minor: payload.order_amount_minor,
            reserve_amount_minor: payload.reserve_amount_minor,
            ttl_ms: payload.ttl_ms,
            idempotency_key: idempotency_key.as_deref(),
            now_ms,
        },
    )
    .await
    .map_err(gateway_marketing_operation_response)?;

    Ok((
        if result.created {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(GatewayCouponReservationResponse {
            reservation: result.reservation,
            applicability: gateway_coupon_applicability_summary(&result.context.template),
            effect: gateway_coupon_effect_summary(&result.context.template),
            template: result.context.template,
            campaign: result.context.campaign,
            budget: result.context.budget,
            code: result.context.code,
        }),
    ))
}

async fn confirm_coupon_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Json(payload): Json<GatewayCouponRedemptionConfirmRequest>,
) -> Result<Json<GatewayCouponRedemptionConfirmResponse>, Response> {
    let subjects = gateway_marketing_subjects(&state, &request).await?;
    let reservation_view = gateway_coupon_reservation_context_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &payload.coupon_reservation_id,
    )
    .await?;
    let reservation_code_value = reservation_view.code.code_value.clone();
    let reservation = reservation_view.reservation;
    let subject_id = subjects
        .subject_id_for_scope(reservation.subject_scope)
        .ok_or_else(|| gateway_error_response(StatusCode::NOT_FOUND, "coupon subject not found"))?;

    let idempotency_key = resolve_idempotency_key(&headers, payload.idempotency_key.as_deref())?;
    let now_ms = gateway_current_time_millis()?;
    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Confirm,
        reservation.subject_scope,
        &reservation.subject_id,
        &reservation_code_value,
    )
    .await?;
    let result = confirm_coupon_for_subject(
        state.store.as_ref(),
        ConfirmCouponInput {
            coupon_reservation_id: &payload.coupon_reservation_id,
            subject_scope: reservation.subject_scope,
            subject_id: &subject_id,
            subsidy_amount_minor: payload.subsidy_amount_minor,
            order_id: payload.order_id.clone(),
            payment_event_id: payload.payment_event_id.clone(),
            idempotency_key: idempotency_key.as_deref(),
            now_ms,
        },
    )
    .await
    .map_err(gateway_marketing_operation_response)?;

    Ok(Json(GatewayCouponRedemptionConfirmResponse {
        reservation: result.reservation,
        redemption: result.redemption,
        applicability: gateway_coupon_applicability_summary(&result.context.template),
        effect: gateway_coupon_effect_summary(&result.context.template),
        template: result.context.template,
        campaign: result.context.campaign,
        budget: result.context.budget,
        code: result.context.code,
    }))
}

async fn rollback_coupon_handler(
    request: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Json(payload): Json<GatewayCouponRedemptionRollbackRequest>,
) -> Result<Json<GatewayCouponRedemptionRollbackResponse>, Response> {
    let subjects = gateway_marketing_subjects(&state, &request).await?;
    let redemption_view = gateway_coupon_redemption_context_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &payload.coupon_redemption_id,
    )
    .await?;
    let redemption_code_value = redemption_view.code.code_value.clone();
    let reservation = redemption_view.reservation;
    let subject_id = subjects
        .subject_id_for_scope(reservation.subject_scope)
        .ok_or_else(|| gateway_error_response(StatusCode::NOT_FOUND, "coupon subject not found"))?;

    let idempotency_key = resolve_idempotency_key(&headers, payload.idempotency_key.as_deref())?;
    let now_ms = gateway_current_time_millis()?;
    enforce_gateway_coupon_rate_limit(
        state.store.as_ref(),
        &request,
        CouponRateLimitAction::Rollback,
        reservation.subject_scope,
        &reservation.subject_id,
        &redemption_code_value,
    )
    .await?;
    let result = rollback_coupon_for_subject(
        state.store.as_ref(),
        RollbackCouponInput {
            coupon_redemption_id: &payload.coupon_redemption_id,
            subject_scope: reservation.subject_scope,
            subject_id: &subject_id,
            rollback_type: payload.rollback_type,
            restored_budget_minor: payload.restored_budget_minor,
            restored_inventory_count: payload.restored_inventory_count,
            idempotency_key: idempotency_key.as_deref(),
            now_ms,
        },
    )
    .await
    .map_err(gateway_marketing_operation_response)?;

    Ok(Json(GatewayCouponRedemptionRollbackResponse {
        redemption: result.redemption,
        rollback: result.rollback,
        applicability: gateway_coupon_applicability_summary(&result.context.template),
        effect: gateway_coupon_effect_summary(&result.context.template),
        template: result.context.template,
        campaign: result.context.campaign,
        budget: result.context.budget,
        code: result.context.code,
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
