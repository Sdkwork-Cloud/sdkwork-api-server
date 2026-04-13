use async_trait::async_trait;
use sdkwork_api_app_marketing::{
    redeem_coupon_code, release_coupon_redemption_reservation, reserve_coupon_redemption,
    validate_coupon_for_quote, CouponQuoteValidation, RedeemCouponCodeInput,
    ReleaseCouponRedemptionReservationInput, ReserveCouponRedemptionInput,
    ValidateCouponForQuoteInput,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType,
    AccountLedgerAllocationRecord, AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord,
    AccountType, QuotaPolicy,
};
use sdkwork_api_domain_commerce::{CommerceOrderRecord, ProjectMembershipRecord};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_identity::{IdentityBindingRecord, IdentityUserRecord};
use sdkwork_api_domain_marketing::CouponRedemptionStatus;
use sdkwork_api_observability::{
    record_current_commercial_event, CommercialEventDimensions, CommercialEventKind,
};
use sdkwork_api_storage_core::{
    AccountKernelCommandBatch, AccountKernelStore, AdminStore, IdentityKernelStore,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub use sdkwork_api_domain_commerce::CommerceOrderRecord as PortalCommerceOrderRecord;
pub use sdkwork_api_domain_commerce::ProjectMembershipRecord as PortalProjectMembershipRecord;

type CommerceResult<T> = std::result::Result<T, CommerceError>;

const PORTAL_WORKSPACE_IDENTITY_BINDING_TYPE: &str = "portal_workspace_user";
const PORTAL_WORKSPACE_IDENTITY_BINDING_ISSUER: &str = "sdkwork-api-portal";

#[derive(Debug)]
pub enum CommerceError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Forbidden(String),
    Storage(anyhow::Error),
}

impl std::fmt::Display for CommerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Conflict(message) => write!(f, "{message}"),
            Self::Forbidden(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CommerceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for CommerceError {
    fn from(value: anyhow::Error) -> Self {
        Self::Storage(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalSubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price_label: String,
    pub cadence: String,
    pub included_units: u64,
    pub highlight: String,
    pub features: Vec<String>,
    pub cta: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalRechargePack {
    pub id: String,
    pub label: String,
    pub points: u64,
    pub price_label: String,
    pub note: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalRechargeOption {
    pub id: String,
    pub label: String,
    pub amount_cents: u64,
    pub amount_label: String,
    pub granted_units: u64,
    pub effective_ratio_label: String,
    pub note: String,
    pub recommended: bool,
    pub source: String,
}

#[derive(Debug, Clone, Copy)]
struct CanonicalPortalWorkspaceSubject {
    tenant_id: u64,
    organization_id: u64,
    user_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCustomRechargeRule {
    pub id: String,
    pub label: String,
    pub min_amount_cents: u64,
    pub max_amount_cents: u64,
    pub units_per_cent: u64,
    pub effective_ratio_label: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCustomRechargePolicy {
    pub enabled: bool,
    pub min_amount_cents: u64,
    pub max_amount_cents: u64,
    pub step_amount_cents: u64,
    pub suggested_amount_cents: u64,
    pub rules: Vec<PortalCustomRechargeRule>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommerceCoupon {
    pub id: String,
    pub code: String,
    pub discount_label: String,
    pub audience: String,
    pub remaining: u64,
    pub active: bool,
    pub note: String,
    pub expires_on: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default)]
    pub bonus_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommerceCatalog {
    pub plans: Vec<PortalSubscriptionPlan>,
    pub packs: Vec<PortalRechargePack>,
    pub recharge_options: Vec<PortalRechargeOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_recharge_policy: Option<PortalCustomRechargePolicy>,
    pub coupons: Vec<PortalCommerceCoupon>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommerceQuoteRequest {
    pub target_kind: String,
    pub target_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_amount_cents: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalAppliedCoupon {
    pub code: String,
    pub discount_label: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default)]
    pub bonus_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommerceQuote {
    pub target_kind: String,
    pub target_id: String,
    pub target_name: String,
    pub list_price_cents: u64,
    pub payable_price_cents: u64,
    pub list_price_label: String,
    pub payable_price_label: String,
    pub granted_units: u64,
    pub bonus_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_cents: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projected_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon: Option<PortalAppliedCoupon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_rule_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_ratio_label: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommerceCheckoutSessionMethod {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub action: String,
    pub availability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommerceCheckoutSession {
    pub order_id: String,
    pub order_status: String,
    pub session_status: String,
    pub provider: String,
    pub mode: String,
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout_url: Option<String>,
    pub payable_price_label: String,
    pub guidance: String,
    pub methods: Vec<PortalCommerceCheckoutSessionMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommercePaymentEventRequest {
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_order_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct CommerceCouponBenefit {
    discount_percent: Option<u8>,
    fixed_discount_cents: Option<u64>,
    bonus_units: u64,
}

#[derive(Debug, Clone)]
struct CommerceCouponDefinition {
    coupon: PortalCommerceCoupon,
    benefit: CommerceCouponBenefit,
}

#[derive(Debug, Clone, Copy)]
struct SubscriptionPlanSeed {
    id: &'static str,
    name: &'static str,
    price_cents: u64,
    cadence: &'static str,
    included_units: u64,
    highlight: &'static str,
    features: &'static [&'static str],
    cta: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct RechargePackSeed {
    id: &'static str,
    label: &'static str,
    points: u64,
    price_cents: u64,
    note: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct RechargeOptionSeed {
    id: &'static str,
    label: &'static str,
    amount_cents: u64,
    granted_units: u64,
    note: &'static str,
    recommended: bool,
}

#[derive(Debug, Clone, Copy)]
struct CustomRechargeRuleSeed {
    id: &'static str,
    label: &'static str,
    min_amount_cents: u64,
    max_amount_cents: u64,
    units_per_cent: u64,
    note: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct CouponSeed {
    id: &'static str,
    code: &'static str,
    discount_label: &'static str,
    audience: &'static str,
    remaining: u64,
    note: &'static str,
    expires_on: &'static str,
    discount_percent: Option<u8>,
    bonus_units: u64,
}

pub async fn load_portal_commerce_catalog(
    store: &dyn AdminStore,
) -> CommerceResult<PortalCommerceCatalog> {
    Ok(PortalCommerceCatalog {
        plans: subscription_plan_catalog(),
        packs: recharge_pack_catalog(),
        recharge_options: recharge_option_catalog(),
        custom_recharge_policy: Some(build_custom_recharge_policy()),
        coupons: load_coupon_catalog(store).await?,
    })
}

pub async fn preview_portal_commerce_quote(
    store: &dyn AdminStore,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<PortalCommerceQuote> {
    preview_portal_commerce_quote_with_subject_and_reservation(store, None, request, None).await
}

pub async fn preview_portal_commerce_quote_for_user(
    store: &dyn AdminStore,
    user_id: &str,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<PortalCommerceQuote> {
    preview_portal_commerce_quote_with_subject_and_reservation(store, Some(user_id), request, None)
        .await
}

async fn preview_portal_commerce_quote_with_subject_and_reservation(
    store: &dyn AdminStore,
    user_id: Option<&str>,
    request: &PortalCommerceQuoteRequest,
    reservation_idempotency_key: Option<&str>,
) -> CommerceResult<PortalCommerceQuote> {
    let target_kind = request.target_kind.trim();
    let target_id = request.target_id.trim();

    if target_kind.is_empty() {
        return Err(CommerceError::InvalidInput(
            "target_kind is required".to_owned(),
        ));
    }
    if target_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "target_id is required".to_owned(),
        ));
    }

    match target_kind {
        "subscription_plan" => {
            let plan = subscription_plan_seeds()
                .into_iter()
                .find(|candidate| candidate.id.eq_ignore_ascii_case(target_id))
                .ok_or_else(|| CommerceError::NotFound("subscription plan not found".to_owned()))?;
            let applied_coupon =
                load_optional_applied_coupon(store, user_id, request, reservation_idempotency_key)
                    .await?;
            Ok(build_priced_quote(
                "subscription_plan",
                plan.id,
                plan.name,
                plan.price_cents,
                plan.included_units,
                "workspace_seed",
                request.current_remaining_units,
                applied_coupon,
            ))
        }
        "recharge_pack" => {
            let pack = recharge_pack_seeds()
                .into_iter()
                .find(|candidate| candidate.id.eq_ignore_ascii_case(target_id))
                .ok_or_else(|| CommerceError::NotFound("recharge pack not found".to_owned()))?;
            let applied_coupon =
                load_optional_applied_coupon(store, user_id, request, reservation_idempotency_key)
                    .await?;
            Ok(build_priced_quote(
                "recharge_pack",
                pack.id,
                pack.label,
                pack.price_cents,
                pack.points,
                "workspace_seed",
                request.current_remaining_units,
                applied_coupon,
            ))
        }
        "custom_recharge" => {
            let custom_amount_cents =
                resolve_custom_recharge_amount_cents(target_id, request.custom_amount_cents)?;
            let applied_coupon =
                load_optional_applied_coupon(store, user_id, request, reservation_idempotency_key)
                    .await?;
            build_custom_recharge_quote(
                custom_amount_cents,
                request.current_remaining_units,
                applied_coupon,
            )
        }
        "coupon_redemption" => {
            let coupon = find_coupon_definition(store, target_id).await?;
            if coupon.benefit.bonus_units == 0 {
                return Err(CommerceError::InvalidInput(format!(
                    "coupon {} does not grant redeemable bonus units",
                    coupon.coupon.code
                )));
            }
            Ok(build_redemption_quote(
                coupon,
                request.current_remaining_units,
            ))
        }
        _ => Err(CommerceError::InvalidInput(format!(
            "unsupported target_kind: {target_kind}"
        ))),
    }
}

pub async fn submit_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let quote = preview_portal_commerce_quote_for_user(store, normalized_user_id, request).await?;
    let status = initial_order_status(&quote);

    let order_id = generate_entity_id("commerce_order")?;
    if should_fulfill_on_order_create(&quote) {
        fulfill_portal_commerce_quote(
            store,
            normalized_user_id,
            normalized_project_id,
            &order_id,
            &quote,
            None,
        )
        .await?;
    } else {
        reserve_marketing_coupon_if_needed(
            store,
            normalized_user_id,
            normalized_project_id,
            &order_id,
            quote.applied_coupon.as_ref(),
        )
        .await?;
    }

    let order = CommerceOrderRecord::new(
        order_id,
        normalized_project_id,
        normalized_user_id,
        quote.target_kind.clone(),
        quote.target_id.clone(),
        quote.target_name.clone(),
        quote.list_price_cents,
        quote.payable_price_cents,
        quote.list_price_label.clone(),
        quote.payable_price_label.clone(),
        quote.granted_units,
        quote.bonus_units,
        status,
        quote.source.clone(),
        current_time_ms()?,
    )
    .with_applied_coupon_code_option(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.code.clone()),
    );

    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub async fn settle_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    payment_order_id: Option<&str>,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "fulfilled" => {
            record_current_commercial_event(
                CommercialEventKind::SettlementReplay,
                CommercialEventDimensions::default()
                    .with_tenant(order.project_id.clone())
                    .with_result("replayed"),
            );
            backfill_marketing_redemption_payment_order_id_if_needed(
                store,
                &order,
                payment_order_id,
            )
            .await?;
            return Ok(order);
        }
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be settled from status {other}"
            )))
        }
    }

    let settlement_quote = load_order_settlement_quote(store, &order).await?;
    fulfill_portal_commerce_quote(
        store,
        normalized_user_id,
        normalized_project_id,
        &order.order_id,
        &settlement_quote,
        payment_order_id,
    )
    .await?;

    order.status = "fulfilled".to_owned();
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub async fn settle_portal_commerce_order_from_session(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    allow_manual_paid_settlement: bool,
) -> CommerceResult<CommerceOrderRecord> {
    let order = load_project_commerce_order(store, user_id, project_id, order_id).await?;
    ensure_portal_session_can_settle(&order, allow_manual_paid_settlement)?;
    settle_portal_commerce_order(store, user_id, project_id, order_id, None).await
}

pub async fn cancel_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "canceled" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be canceled from status {other}"
            )))
        }
    }

    release_marketing_coupon_reservation_if_needed(store, &order, CouponRedemptionStatus::Voided)
        .await?;
    order.status = "canceled".to_owned();
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub async fn apply_portal_commerce_payment_event(
    store: &dyn AdminStore,
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let order = load_commerce_order_by_id(store, order_id).await?;
    let event_type = request.event_type.trim();
    if event_type.is_empty() {
        return Err(CommerceError::InvalidInput(
            "event_type is required".to_owned(),
        ));
    }

    match event_type {
        "settled" => {
            settle_portal_commerce_order(
                store,
                &order.user_id,
                &order.project_id,
                order_id,
                request.payment_order_id.as_deref(),
            )
            .await
        }
        "canceled" => {
            cancel_portal_commerce_order(store, &order.user_id, &order.project_id, order_id).await
        }
        "failed" => {
            fail_portal_commerce_order(store, &order.user_id, &order.project_id, order_id).await
        }
        other => Err(CommerceError::InvalidInput(format!(
            "unsupported payment event_type: {other}"
        ))),
    }
}

pub async fn load_portal_commerce_checkout_session(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    allow_manual_paid_settlement: bool,
) -> CommerceResult<PortalCommerceCheckoutSession> {
    let order = load_project_commerce_order(store, user_id, project_id, order_id).await?;
    Ok(build_checkout_session(&order, allow_manual_paid_settlement))
}

pub async fn load_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    load_project_commerce_order(store, user_id, project_id, order_id).await
}

pub async fn list_project_commerce_orders(
    store: &dyn AdminStore,
    project_id: &str,
) -> CommerceResult<Vec<CommerceOrderRecord>> {
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let mut orders = store
        .list_commerce_orders_for_project(normalized_project_id)
        .await
        .map_err(CommerceError::from)?;
    orders.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.order_id.cmp(&left.order_id))
    });
    Ok(orders)
}

pub async fn load_project_membership(
    store: &dyn AdminStore,
    project_id: &str,
) -> CommerceResult<Option<ProjectMembershipRecord>> {
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    store
        .find_project_membership(normalized_project_id)
        .await
        .map_err(CommerceError::from)
}

fn subscription_plan_catalog() -> Vec<PortalSubscriptionPlan> {
    subscription_plan_seeds()
        .into_iter()
        .map(|seed| PortalSubscriptionPlan {
            id: seed.id.to_owned(),
            name: seed.name.to_owned(),
            price_label: format_catalog_price_label(seed.price_cents),
            cadence: seed.cadence.to_owned(),
            included_units: seed.included_units,
            highlight: seed.highlight.to_owned(),
            features: seed
                .features
                .iter()
                .map(|feature| (*feature).to_owned())
                .collect(),
            cta: seed.cta.to_owned(),
            source: "workspace_seed".to_owned(),
        })
        .collect()
}

fn recharge_pack_catalog() -> Vec<PortalRechargePack> {
    recharge_pack_seeds()
        .into_iter()
        .map(|seed| PortalRechargePack {
            id: seed.id.to_owned(),
            label: seed.label.to_owned(),
            points: seed.points,
            price_label: format_catalog_price_label(seed.price_cents),
            note: seed.note.to_owned(),
            source: "workspace_seed".to_owned(),
        })
        .collect()
}

fn recharge_option_catalog() -> Vec<PortalRechargeOption> {
    recharge_option_seeds()
        .into_iter()
        .map(|seed| PortalRechargeOption {
            id: seed.id.to_owned(),
            label: seed.label.to_owned(),
            amount_cents: seed.amount_cents,
            amount_label: format_quote_price_label(seed.amount_cents),
            granted_units: seed.granted_units,
            effective_ratio_label: format_effective_ratio_label(
                seed.granted_units / seed.amount_cents.max(1),
            ),
            note: seed.note.to_owned(),
            recommended: seed.recommended,
            source: "workspace_seed".to_owned(),
        })
        .collect()
}

fn build_custom_recharge_policy() -> PortalCustomRechargePolicy {
    PortalCustomRechargePolicy {
        enabled: true,
        min_amount_cents: custom_recharge_min_amount_cents(),
        max_amount_cents: custom_recharge_max_amount_cents(),
        step_amount_cents: custom_recharge_step_amount_cents(),
        suggested_amount_cents: custom_recharge_suggested_amount_cents(),
        rules: custom_recharge_rule_seeds()
            .into_iter()
            .map(|rule| PortalCustomRechargeRule {
                id: rule.id.to_owned(),
                label: rule.label.to_owned(),
                min_amount_cents: rule.min_amount_cents,
                max_amount_cents: rule.max_amount_cents,
                units_per_cent: rule.units_per_cent,
                effective_ratio_label: format_effective_ratio_label(rule.units_per_cent),
                note: rule.note.to_owned(),
            })
            .collect(),
        source: "workspace_seed".to_owned(),
    }
}

async fn load_coupon_catalog<S>(store: &S) -> CommerceResult<Vec<PortalCommerceCoupon>>
where
    S: AdminStore + ?Sized,
{
    Ok(load_coupon_definitions(store)
        .await?
        .into_iter()
        .map(|definition| definition.coupon)
        .collect())
}

async fn load_coupon_definitions<S>(store: &S) -> CommerceResult<Vec<CommerceCouponDefinition>>
where
    S: AdminStore + ?Sized,
{
    let mut definitions = seed_coupon_definitions()
        .into_iter()
        .map(|definition| (normalize_coupon_code(&definition.coupon.code), definition))
        .collect::<BTreeMap<_, _>>();

    for coupon in store
        .list_active_coupons()
        .await
        .map_err(CommerceError::from)?
    {
        let code = normalize_coupon_code(&coupon.code);
        let prior = definitions.get(&code).cloned();
        let parsed_benefit = CommerceCouponBenefit {
            discount_percent: parse_discount_percent(&coupon.discount_label),
            fixed_discount_cents: None,
            bonus_units: 0,
        };
        let benefit = merge_coupon_benefit(parsed_benefit, prior.as_ref().map(|item| item.benefit));

        definitions.insert(
            code.clone(),
            CommerceCouponDefinition {
                coupon: PortalCommerceCoupon {
                    id: coupon.id,
                    code,
                    discount_label: coupon.discount_label,
                    audience: coupon.audience,
                    remaining: coupon.remaining,
                    active: coupon.active,
                    note: coupon.note,
                    expires_on: coupon.expires_on,
                    source: "live".to_owned(),
                    discount_percent: benefit.discount_percent,
                    bonus_units: benefit.bonus_units,
                },
                benefit,
            },
        );
    }

    Ok(definitions.into_values().collect())
}

async fn find_coupon_definition<S>(
    store: &S,
    code: &str,
) -> CommerceResult<CommerceCouponDefinition>
where
    S: AdminStore + ?Sized,
{
    let normalized = normalize_coupon_code(code);
    load_coupon_definitions(store)
        .await?
        .into_iter()
        .find(|definition| definition.coupon.code == normalized)
        .ok_or_else(|| CommerceError::NotFound(format!("coupon {normalized} not found")))
}

async fn load_optional_applied_coupon(
    store: &dyn AdminStore,
    user_id: Option<&str>,
    request: &PortalCommerceQuoteRequest,
    reservation_idempotency_key: Option<&str>,
) -> CommerceResult<Option<CommerceCouponDefinition>> {
    match request
        .coupon_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(code) => {
            if let Some(marketing_coupon) = load_marketing_discount_coupon_definition(
                store,
                user_id,
                request,
                code,
                reservation_idempotency_key,
            )
            .await?
            {
                return Ok(Some(marketing_coupon));
            }

            find_coupon_definition(store, code).await.map(Some)
        }
        None => Ok(None),
    }
}

async fn load_marketing_discount_coupon_definition(
    store: &dyn AdminStore,
    user_id: Option<&str>,
    request: &PortalCommerceQuoteRequest,
    coupon_code: &str,
    reservation_idempotency_key: Option<&str>,
) -> CommerceResult<Option<CommerceCouponDefinition>> {
    if !matches!(
        request.target_kind.as_str(),
        "subscription_plan" | "recharge_pack" | "custom_recharge"
    ) {
        return Ok(None);
    }

    let code_lookup_hash = hash_coupon_code_for_lookup(coupon_code);
    let Some(_) = store
        .find_coupon_code_record_by_lookup_hash(&code_lookup_hash)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(None);
    };
    let (subject_type, subject_id) = resolve_marketing_quote_subject_identity(user_id);

    let validation = validate_coupon_for_quote(
        store,
        ValidateCouponForQuoteInput::new(
            &subject_type,
            &subject_id,
            &code_lookup_hash,
            current_time_ms()?,
        )
        .with_target_order_kind(Some(request.target_kind.clone()))
        .with_target_product_id(Some(request.target_id.clone()))
        .with_reservation_idempotency_key(
            reservation_idempotency_key
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
        ),
    )
    .await
    .map_err(|error| CommerceError::InvalidInput(error.to_string()))?;

    let discount_percent = validation
        .percentage_off
        .and_then(convert_percentage_discount_to_u8);
    let fixed_discount_cents = validation
        .fixed_discount_amount
        .and_then(convert_fixed_discount_amount_to_cents);
    let discount_label = format_marketing_discount_label(&validation);

    Ok(Some(CommerceCouponDefinition {
        coupon: PortalCommerceCoupon {
            id: format!("marketing_coupon_code_{}", validation.coupon_code_id),
            code: normalize_coupon_code(coupon_code),
            discount_label,
            audience: "marketing".to_owned(),
            remaining: 0,
            active: true,
            note: "canonical marketing coupon".to_owned(),
            expires_on: "managed".to_owned(),
            source: "marketing".to_owned(),
            discount_percent,
            bonus_units: 0,
        },
        benefit: CommerceCouponBenefit {
            discount_percent,
            fixed_discount_cents,
            bonus_units: 0,
        },
    }))
}

async fn apply_quote_to_project_quota<T>(
    store: &T,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    let target_units = quote.granted_units.saturating_add(quote.bonus_units);
    if target_units == 0 {
        return Ok(());
    }

    let effective_policy = load_effective_quota_policy(store, project_id).await?;
    let current_limit = effective_policy
        .as_ref()
        .map(|policy| policy.max_units)
        .unwrap_or(0);
    let policy_id = effective_policy
        .as_ref()
        .map(|policy| policy.policy_id.clone())
        .unwrap_or_else(|| format!("portal_commerce_{project_id}"));
    let next_limit = match quote.target_kind.as_str() {
        "subscription_plan" => current_limit.max(target_units),
        "coupon_redemption" => current_limit.saturating_add(target_units),
        "recharge_pack" | "custom_recharge" => current_limit,
        _ => current_limit,
    };

    if next_limit == current_limit {
        return Ok(());
    }

    let next_policy = QuotaPolicy::new(policy_id, project_id.to_owned(), next_limit);
    store
        .insert_quota_policy(&next_policy)
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

async fn fulfill_portal_commerce_quote<T>(
    store: &T,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    quote: &PortalCommerceQuote,
    payment_order_id: Option<&str>,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    apply_quote_to_project_quota(store, project_id, quote).await?;
    issue_canonical_recharge_entitlement_if_needed(
        store,
        user_id,
        project_id,
        order_id,
        quote,
        payment_order_id,
    )
    .await?;
    consume_live_coupon_if_needed(store, quote.applied_coupon.as_ref()).await?;
    activate_project_membership_if_needed(store, user_id, project_id, quote).await?;
    redeem_marketing_coupon_if_needed(
        store,
        user_id,
        project_id,
        order_id,
        quote.applied_coupon.as_ref(),
        payment_order_id,
    )
    .await?;
    Ok(())
}

async fn issue_canonical_recharge_entitlement_if_needed<S>(
    store: &S,
    portal_user_id: &str,
    project_id: &str,
    order_id: &str,
    quote: &PortalCommerceQuote,
    payment_order_id: Option<&str>,
) -> CommerceResult<()>
where
    S: AdminStore + ?Sized,
{
    if !matches!(
        quote.target_kind.as_str(),
        "recharge_pack" | "custom_recharge"
    ) {
        return Ok(());
    }

    let granted_quantity = quote.granted_units.saturating_add(quote.bonus_units) as f64;
    if granted_quantity <= f64::EPSILON {
        return Ok(());
    }

    let Some(account_store) = store.account_kernel() else {
        return Err(CommerceError::Conflict(
            "canonical account kernel is unavailable for recharge fulfillment".to_owned(),
        ));
    };
    let Some(identity_store) = store.identity_kernel() else {
        return Err(CommerceError::Conflict(
            "canonical identity kernel is unavailable for recharge fulfillment".to_owned(),
        ));
    };
    let (workspace_tenant_id, workspace_project_id) =
        resolve_portal_workspace_scope_for_recharge(store, portal_user_id, project_id).await?;
    let subject = ensure_portal_workspace_identity(
        store,
        identity_store,
        portal_user_id,
        &workspace_tenant_id,
        &workspace_project_id,
    )
    .await?;

    issue_recharge_grant_to_primary_account(
        account_store,
        &subject,
        order_id,
        payment_order_id,
        granted_quantity,
        quote.payable_price_cents,
        current_time_ms()?,
    )
    .await
}

async fn resolve_portal_workspace_scope_for_recharge<S>(
    store: &S,
    portal_user_id: &str,
    project_id: &str,
) -> CommerceResult<(String, String)>
where
    S: AdminStore + ?Sized,
{
    let normalized_portal_user_id = portal_user_id.trim();
    let normalized_project_id = project_id.trim();

    if let Some(project) = store
        .find_project(normalized_project_id)
        .await
        .map_err(CommerceError::from)?
    {
        return Ok((project.tenant_id, project.id));
    }

    if let Some(membership) = store
        .list_portal_workspace_memberships_for_user(normalized_portal_user_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|record| record.project_id == normalized_project_id)
    {
        return Ok((membership.tenant_id, membership.project_id));
    }

    if let Some(portal_user) = store
        .find_portal_user_by_id(normalized_portal_user_id)
        .await
        .map_err(CommerceError::from)?
    {
        if portal_user.workspace_project_id == normalized_project_id {
            return Ok((
                portal_user.workspace_tenant_id,
                portal_user.workspace_project_id,
            ));
        }
    }

    Ok((
        format!("project_scope:{normalized_project_id}"),
        normalized_project_id.to_owned(),
    ))
}

async fn ensure_portal_workspace_identity<S>(
    store: &S,
    identity_store: &dyn IdentityKernelStore,
    portal_user_id: &str,
    workspace_tenant_id: &str,
    workspace_project_id: &str,
) -> CommerceResult<CanonicalPortalWorkspaceSubject>
where
    S: AdminStore + ?Sized,
{
    let normalized_portal_user_id = portal_user_id.trim();
    let normalized_workspace_tenant_id = workspace_tenant_id.trim();
    let normalized_workspace_project_id = workspace_project_id.trim();
    let binding_subject = portal_workspace_binding_subject(
        normalized_workspace_tenant_id,
        normalized_workspace_project_id,
        normalized_portal_user_id,
    );
    let subject = CanonicalPortalWorkspaceSubject {
        tenant_id: stable_derived_id("portal_workspace_tenant", &[normalized_workspace_tenant_id]),
        organization_id: stable_derived_id(
            "portal_workspace_project",
            &[
                normalized_workspace_tenant_id,
                normalized_workspace_project_id,
            ],
        ),
        user_id: stable_derived_id(
            "portal_workspace_user",
            &[
                normalized_workspace_tenant_id,
                normalized_workspace_project_id,
                normalized_portal_user_id,
            ],
        ),
    };

    if identity_store
        .find_identity_user_record(subject.user_id)
        .await
        .map_err(CommerceError::from)?
        .is_none()
    {
        let portal_user = store
            .find_portal_user_by_id(normalized_portal_user_id)
            .await
            .map_err(CommerceError::from)?;
        let created_at_ms = current_time_ms()?;
        let mut identity_user =
            IdentityUserRecord::new(subject.user_id, subject.tenant_id, subject.organization_id)
                .with_external_user_ref(Some(binding_subject.clone()))
                .with_created_at_ms(created_at_ms)
                .with_updated_at_ms(created_at_ms);
        if let Some(portal_user) = portal_user {
            identity_user = identity_user
                .with_display_name(Some(portal_user.display_name))
                .with_email(Some(portal_user.email));
        }
        identity_store
            .insert_identity_user_record(&identity_user)
            .await
            .map_err(CommerceError::from)?;
    }

    if identity_store
        .find_identity_binding_record(
            PORTAL_WORKSPACE_IDENTITY_BINDING_TYPE,
            Some(PORTAL_WORKSPACE_IDENTITY_BINDING_ISSUER),
            Some(binding_subject.as_str()),
        )
        .await
        .map_err(CommerceError::from)?
        .is_none()
    {
        let created_at_ms = current_time_ms()?;
        let binding = IdentityBindingRecord::new(
            stable_derived_id(
                "portal_workspace_identity_binding",
                &[
                    normalized_workspace_tenant_id,
                    normalized_workspace_project_id,
                    normalized_portal_user_id,
                ],
            ),
            subject.tenant_id,
            subject.organization_id,
            subject.user_id,
            PORTAL_WORKSPACE_IDENTITY_BINDING_TYPE,
        )
        .with_issuer(Some(PORTAL_WORKSPACE_IDENTITY_BINDING_ISSUER.to_owned()))
        .with_subject(Some(binding_subject))
        .with_platform(Some("portal".to_owned()))
        .with_owner(Some(normalized_workspace_project_id.to_owned()))
        .with_external_ref(Some(normalized_portal_user_id.to_owned()))
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
        identity_store
            .insert_identity_binding_record(&binding)
            .await
            .map_err(CommerceError::from)?;
    }

    Ok(subject)
}

async fn issue_recharge_grant_to_primary_account(
    account_store: &dyn AccountKernelStore,
    subject: &CanonicalPortalWorkspaceSubject,
    order_id: &str,
    payment_order_id: Option<&str>,
    granted_quantity: f64,
    payable_price_cents: u64,
    created_at_ms: u64,
) -> CommerceResult<()> {
    let existing_account = account_store
        .find_account_record_by_owner(
            subject.tenant_id,
            subject.organization_id,
            subject.user_id,
            AccountType::Primary,
        )
        .await
        .map_err(CommerceError::from)?;
    let account_id = match existing_account {
        Some(ref account) => account.account_id,
        None => stable_derived_id(
            "portal_workspace_primary_account",
            &[
                &subject.tenant_id.to_string(),
                &subject.organization_id.to_string(),
                &subject.user_id.to_string(),
                "primary",
            ],
        ),
    };
    let source_reference = payment_order_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(order_id);
    let acquired_unit_cost = if granted_quantity > f64::EPSILON {
        Some((payable_price_cents as f64 / 100.0) / granted_quantity)
    } else {
        None
    };

    let mut batch = AccountKernelCommandBatch::default();
    if existing_account.is_none() {
        batch.account_records.push(
            AccountRecord::new(
                account_id,
                subject.tenant_id,
                subject.organization_id,
                subject.user_id,
                AccountType::Primary,
            )
            .with_created_at_ms(created_at_ms)
            .with_updated_at_ms(created_at_ms),
        );
    }

    let lot_id = stable_derived_id("commerce_recharge_lot", &[order_id]);
    let ledger_entry_id = stable_derived_id("commerce_recharge_ledger_entry", &[order_id]);
    batch.benefit_lot_records.push(
        AccountBenefitLotRecord::new(
            lot_id,
            subject.tenant_id,
            subject.organization_id,
            account_id,
            subject.user_id,
            AccountBenefitType::CashCredit,
        )
        .with_source_type(AccountBenefitSourceType::Recharge)
        .with_source_id(Some(stable_derived_id(
            "commerce_recharge_source",
            &[source_reference],
        )))
        .with_original_quantity(granted_quantity)
        .with_remaining_quantity(granted_quantity)
        .with_acquired_unit_cost(acquired_unit_cost)
        .with_issued_at_ms(created_at_ms)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms),
    );
    batch.ledger_entry_records.push(
        AccountLedgerEntryRecord::new(
            ledger_entry_id,
            subject.tenant_id,
            subject.organization_id,
            account_id,
            subject.user_id,
            AccountLedgerEntryType::GrantIssue,
        )
        .with_benefit_type(Some("cash_credit".to_owned()))
        .with_quantity(granted_quantity)
        .with_amount(granted_quantity)
        .with_created_at_ms(created_at_ms),
    );
    batch.ledger_allocation_records.push(
        AccountLedgerAllocationRecord::new(
            stable_derived_id("commerce_recharge_ledger_allocation", &[order_id]),
            subject.tenant_id,
            subject.organization_id,
            ledger_entry_id,
            lot_id,
        )
        .with_quantity_delta(granted_quantity)
        .with_created_at_ms(created_at_ms),
    );

    account_store
        .commit_account_kernel_batch(&batch)
        .await
        .map_err(CommerceError::from)
}

async fn load_effective_quota_policy<T>(
    store: &T,
    project_id: &str,
) -> CommerceResult<Option<QuotaPolicy>>
where
    T: CommerceQuotaStore + ?Sized,
{
    Ok(store
        .list_quota_policies_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|policy| policy.enabled)
        .min_by(|left, right| {
            left.max_units
                .cmp(&right.max_units)
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        }))
}

async fn consume_live_coupon_if_needed<S>(
    store: &S,
    coupon: Option<&PortalAppliedCoupon>,
) -> CommerceResult<()>
where
    S: AdminStore + ?Sized,
{
    let Some(coupon) = coupon else {
        return Ok(());
    };
    if coupon.source != "live" {
        return Ok(());
    }

    let definition = find_coupon_definition(store, &coupon.code).await?;
    if definition.coupon.source != "live" {
        return Ok(());
    }
    if definition.coupon.remaining == 0 {
        return Err(CommerceError::InvalidInput(format!(
            "coupon {} is no longer available",
            definition.coupon.code
        )));
    }

    let persisted_coupon = store
        .find_coupon(&definition.coupon.id)
        .await
        .map_err(CommerceError::from)?
        .ok_or_else(|| {
            CommerceError::NotFound(format!("coupon {} not found", definition.coupon.code))
        })?;
    if persisted_coupon.remaining == 0 {
        return Err(CommerceError::InvalidInput(format!(
            "coupon {} is no longer available",
            persisted_coupon.code
        )));
    }

    store
        .insert_coupon(&CouponCampaign {
            id: persisted_coupon.id,
            code: persisted_coupon.code,
            discount_label: persisted_coupon.discount_label,
            audience: persisted_coupon.audience,
            remaining: persisted_coupon.remaining.saturating_sub(1),
            active: persisted_coupon.active,
            note: persisted_coupon.note,
            expires_on: persisted_coupon.expires_on,
            created_at_ms: persisted_coupon.created_at_ms,
        })
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

async fn redeem_marketing_coupon_if_needed<S>(
    store: &S,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    coupon: Option<&PortalAppliedCoupon>,
    payment_order_id: Option<&str>,
) -> CommerceResult<()>
where
    S: AdminStore + ?Sized,
{
    let Some(coupon) = coupon else {
        return Ok(());
    };
    if coupon.source != "marketing" {
        return Ok(());
    }

    let code_lookup_hash = hash_coupon_code_for_lookup(&coupon.code);
    let redemption_id = generate_marketing_coupon_redemption_id(order_id, &coupon.code);
    let idempotency_key = marketing_coupon_redemption_idempotency_key(order_id, &coupon.code);
    redeem_coupon_code(
        store,
        RedeemCouponCodeInput::new(
            redemption_id,
            "user",
            user_id,
            &code_lookup_hash,
            idempotency_key,
            current_time_ms()?,
        )
        .with_project_id(Some(project_id.to_owned()))
        .with_order_id(Some(order_id.to_owned()))
        .with_payment_order_id(
            payment_order_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
        ),
    )
    .await
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    Ok(())
}

async fn reserve_marketing_coupon_if_needed<S>(
    store: &S,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    coupon: Option<&PortalAppliedCoupon>,
) -> CommerceResult<()>
where
    S: AdminStore + ?Sized,
{
    let Some(coupon) = coupon else {
        return Ok(());
    };
    if coupon.source != "marketing" {
        return Ok(());
    }

    let code_lookup_hash = hash_coupon_code_for_lookup(&coupon.code);
    let redemption_id = generate_marketing_coupon_redemption_id(order_id, &coupon.code);
    let idempotency_key = marketing_coupon_redemption_idempotency_key(order_id, &coupon.code);
    reserve_coupon_redemption(
        store,
        ReserveCouponRedemptionInput::new(
            redemption_id,
            "user",
            user_id,
            &code_lookup_hash,
            idempotency_key,
            current_time_ms()?,
        )
        .with_project_id(Some(project_id.to_owned()))
        .with_order_id(Some(order_id.to_owned())),
    )
    .await
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    Ok(())
}

async fn release_marketing_coupon_reservation_if_needed(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    status: CouponRedemptionStatus,
) -> CommerceResult<()> {
    let Some(applied_coupon_code) = order.applied_coupon_code.as_deref() else {
        return Ok(());
    };
    let idempotency_key =
        marketing_coupon_redemption_idempotency_key(&order.order_id, applied_coupon_code);
    release_coupon_redemption_reservation(
        store,
        ReleaseCouponRedemptionReservationInput::new(idempotency_key, status, current_time_ms()?),
    )
    .await
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    Ok(())
}

async fn backfill_marketing_redemption_payment_order_id_if_needed(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    payment_order_id: Option<&str>,
) -> CommerceResult<()> {
    if order.payable_price_cents == 0 {
        return Ok(());
    }
    let Some(payment_order_id) = payment_order_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let Some(applied_coupon_code) = order.applied_coupon_code.as_deref() else {
        return Ok(());
    };

    let idempotency_key =
        marketing_coupon_redemption_idempotency_key(&order.order_id, applied_coupon_code);
    let Some(redemption) = store
        .find_coupon_redemption_record_by_idempotency_key(&idempotency_key)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(());
    };

    if redemption.payment_order_id.as_deref() == Some(payment_order_id) {
        return Ok(());
    }

    store
        .insert_coupon_redemption_record(
            &redemption
                .with_payment_order_id(Some(payment_order_id.to_owned()))
                .with_updated_at_ms(current_time_ms()?),
        )
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

async fn activate_project_membership_if_needed<S>(
    store: &S,
    user_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()>
where
    S: AdminStore + ?Sized,
{
    if quote.target_kind != "subscription_plan" {
        return Ok(());
    }

    let plan = subscription_plan_seeds()
        .into_iter()
        .find(|candidate| candidate.id.eq_ignore_ascii_case(&quote.target_id))
        .ok_or_else(|| CommerceError::NotFound("subscription plan not found".to_owned()))?;
    let activated_at_ms = current_time_ms()?;

    store
        .upsert_project_membership(&ProjectMembershipRecord::new(
            generate_entity_id("membership")?,
            project_id,
            user_id,
            plan.id,
            plan.name,
            quote.payable_price_cents,
            quote.payable_price_label.clone(),
            plan.cadence,
            plan.included_units,
            "active",
            quote.source.clone(),
            activated_at_ms,
            activated_at_ms,
        ))
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

fn should_fulfill_on_order_create(quote: &PortalCommerceQuote) -> bool {
    quote.target_kind == "coupon_redemption" || quote.payable_price_cents == 0
}

fn initial_order_status(quote: &PortalCommerceQuote) -> &'static str {
    if should_fulfill_on_order_create(quote) {
        "fulfilled"
    } else {
        "pending_payment"
    }
}

async fn load_project_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let order = list_project_commerce_orders(store, project_id)
        .await?
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .ok_or_else(|| CommerceError::NotFound(format!("order {order_id} not found")))?;

    if order.user_id != user_id {
        return Err(CommerceError::NotFound(format!(
            "order {order_id} not found"
        )));
    }

    Ok(order)
}

async fn fail_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "failed" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be marked failed from status {other}"
            )))
        }
    }

    release_marketing_coupon_reservation_if_needed(store, &order, CouponRedemptionStatus::Failed)
        .await?;
    order.status = "failed".to_owned();
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

async fn load_order_settlement_quote(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
) -> CommerceResult<PortalCommerceQuote> {
    let reservation_idempotency_key = order.applied_coupon_code.as_deref().map(|coupon_code| {
        marketing_coupon_redemption_idempotency_key(&order.order_id, coupon_code)
    });
    let settlement_preview = preview_portal_commerce_quote_with_subject_and_reservation(
        store,
        Some(&order.user_id),
        &PortalCommerceQuoteRequest {
            target_kind: order.target_kind.clone(),
            target_id: order.target_id.clone(),
            coupon_code: order.applied_coupon_code.clone(),
            current_remaining_units: None,
            custom_amount_cents: None,
        },
        reservation_idempotency_key.as_deref(),
    )
    .await?;

    Ok(PortalCommerceQuote {
        target_kind: order.target_kind.clone(),
        target_id: order.target_id.clone(),
        target_name: order.target_name.clone(),
        list_price_cents: order.list_price_cents,
        payable_price_cents: order.payable_price_cents,
        list_price_label: order.list_price_label.clone(),
        payable_price_label: order.payable_price_label.clone(),
        granted_units: order.granted_units,
        bonus_units: order.bonus_units,
        amount_cents: settlement_preview.amount_cents,
        projected_remaining_units: None,
        applied_coupon: settlement_preview.applied_coupon,
        pricing_rule_label: settlement_preview.pricing_rule_label,
        effective_ratio_label: settlement_preview.effective_ratio_label,
        source: order.source.clone(),
    })
}

fn build_checkout_session(
    order: &CommerceOrderRecord,
    allow_manual_paid_settlement: bool,
) -> PortalCommerceCheckoutSession {
    let reference = format!("PAY-{}", normalize_payment_reference(&order.order_id));
    let allow_manual_portal_settlement =
        allow_manual_paid_settlement || order.payable_price_cents == 0;
    let guidance = match (order.target_kind.as_str(), order.status.as_str()) {
        ("subscription_plan", "pending_payment") => {
            "Settle this checkout to activate the workspace membership and included monthly units."
        }
        ("recharge_pack", "pending_payment") => {
            "Settle this checkout to apply the recharge pack and restore workspace quota headroom."
        }
        ("custom_recharge", "pending_payment") => {
            "Settle this checkout to apply the custom recharge amount and restore workspace quota headroom."
        }
        ("coupon_redemption", "fulfilled") => {
            "This order required no external payment and was fulfilled immediately at redemption time."
        }
        (_, "fulfilled") => {
            "This checkout session is closed because the order has already been settled."
        }
        (_, "canceled") => {
            "This checkout session is closed because the order was canceled before settlement."
        }
        (_, "failed") => "This checkout session is closed because the payment flow failed.",
        _ => "This checkout session describes how the current order can move through the payment rail.",
    };

    let (session_status, provider, mode, methods) = match order.status.as_str() {
        "pending_payment" if allow_manual_portal_settlement => (
            "open",
            "manual_lab",
            "operator_settlement",
            build_open_checkout_methods(order, true),
        ),
        "pending_payment" => (
            "open",
            "provider_callback",
            "payment_callback_required",
            build_open_checkout_methods(order, false),
        ),
        "fulfilled"
            if order.target_kind == "coupon_redemption" || order.payable_price_cents == 0 =>
        {
            (
                "not_required",
                "no_payment_required",
                "instant_fulfillment",
                Vec::new(),
            )
        }
        "fulfilled" => ("settled", "manual_lab", "closed", Vec::new()),
        "canceled" => ("canceled", "manual_lab", "closed", Vec::new()),
        "failed" => ("failed", "manual_lab", "closed", Vec::new()),
        _ => ("closed", "manual_lab", "closed", Vec::new()),
    };

    PortalCommerceCheckoutSession {
        order_id: order.order_id.clone(),
        order_status: order.status.clone(),
        session_status: session_status.to_owned(),
        provider: provider.to_owned(),
        mode: mode.to_owned(),
        reference,
        checkout_url: None,
        payable_price_label: order.payable_price_label.clone(),
        guidance: guidance.to_owned(),
        methods,
    }
}

fn build_open_checkout_methods(
    order: &CommerceOrderRecord,
    allow_manual_portal_settlement: bool,
) -> Vec<PortalCommerceCheckoutSessionMethod> {
    let mut methods = Vec::new();

    if allow_manual_portal_settlement {
        methods.push(PortalCommerceCheckoutSessionMethod {
            id: "manual_settlement".to_owned(),
            label: "Manual settlement".to_owned(),
            detail:
                "Use the portal settlement action in desktop or lab mode to finalize the order."
                    .to_owned(),
            action: "settle_order".to_owned(),
            availability: "available".to_owned(),
        });
    }

    methods.push(PortalCommerceCheckoutSessionMethod {
        id: "cancel_order".to_owned(),
        label: "Cancel checkout".to_owned(),
        detail: "Close the pending order without applying quota or membership side effects."
            .to_owned(),
        action: "cancel_order".to_owned(),
        availability: "available".to_owned(),
    });

    if order.payable_price_cents > 0 {
        methods.push(PortalCommerceCheckoutSessionMethod {
            id: "provider_handoff".to_owned(),
            label: "Provider handoff".to_owned(),
            detail: "Reserved seam for Stripe, Alipay, WeChat Pay, or other hosted payment providers in server mode.".to_owned(),
            action: "provider_handoff".to_owned(),
            availability: "planned".to_owned(),
        });
    }

    methods
}

fn normalize_payment_reference(order_id: &str) -> String {
    order_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn ensure_portal_session_can_settle(
    order: &CommerceOrderRecord,
    allow_manual_paid_settlement: bool,
) -> CommerceResult<()> {
    if order.payable_price_cents > 0 && !allow_manual_paid_settlement {
        return Err(CommerceError::Forbidden(
            "paid orders must be settled through the payment callback flow".to_owned(),
        ));
    }
    Ok(())
}

async fn load_commerce_order_by_id(
    store: &dyn AdminStore,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_order_id = order_id.trim();
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    store
        .list_commerce_orders()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|order| order.order_id == normalized_order_id)
        .ok_or_else(|| CommerceError::NotFound(format!("order {normalized_order_id} not found")))
}

fn generate_marketing_coupon_redemption_id(order_id: &str, coupon_code: &str) -> u64 {
    stable_derived_id("marketing_coupon_redemption", &[order_id, coupon_code])
}

fn marketing_coupon_redemption_idempotency_key(order_id: &str, coupon_code: &str) -> String {
    format!(
        "marketing_coupon_redemption:{}:{}",
        order_id,
        normalize_coupon_code(coupon_code)
    )
}

fn portal_workspace_binding_subject(
    workspace_tenant_id: &str,
    workspace_project_id: &str,
    portal_user_id: &str,
) -> String {
    format!("{workspace_tenant_id}:{workspace_project_id}:{portal_user_id}")
}

fn stable_derived_id(namespace: &str, components: &[&str]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    for component in components {
        hasher.update(b":");
        hasher.update(component.as_bytes());
    }
    let digest = hasher.finalize();
    let mut id_bytes = [0_u8; 8];
    id_bytes.copy_from_slice(&digest[..8]);
    let id = u64::from_be_bytes(id_bytes) & (i64::MAX as u64);
    id.max(1)
}

fn build_priced_quote(
    target_kind: &str,
    target_id: &str,
    target_name: &str,
    list_price_cents: u64,
    granted_units: u64,
    source: &str,
    current_remaining_units: Option<u64>,
    applied_coupon: Option<CommerceCouponDefinition>,
) -> PortalCommerceQuote {
    let discount_percent = applied_coupon
        .as_ref()
        .and_then(|coupon| coupon.benefit.discount_percent)
        .unwrap_or(0);
    let fixed_discount_cents = applied_coupon
        .as_ref()
        .and_then(|coupon| coupon.benefit.fixed_discount_cents)
        .unwrap_or(0);
    let bonus_units = applied_coupon
        .as_ref()
        .map(|coupon| coupon.benefit.bonus_units)
        .unwrap_or(0);
    let discounted_cents =
        list_price_cents.saturating_mul(u64::from(100_u8.saturating_sub(discount_percent))) / 100;
    let payable_cents = discounted_cents.saturating_sub(fixed_discount_cents);
    let projected_remaining_units = current_remaining_units.map(|units| {
        units
            .saturating_add(granted_units)
            .saturating_add(bonus_units)
    });

    PortalCommerceQuote {
        target_kind: target_kind.to_owned(),
        target_id: target_id.to_owned(),
        target_name: target_name.to_owned(),
        list_price_cents,
        payable_price_cents: payable_cents,
        list_price_label: format_quote_price_label(list_price_cents),
        payable_price_label: format_quote_price_label(payable_cents),
        granted_units,
        bonus_units,
        amount_cents: None,
        projected_remaining_units,
        applied_coupon: applied_coupon.map(|coupon| PortalAppliedCoupon {
            code: coupon.coupon.code,
            discount_label: coupon.coupon.discount_label,
            source: coupon.coupon.source,
            discount_percent: coupon.benefit.discount_percent,
            bonus_units: coupon.benefit.bonus_units,
        }),
        pricing_rule_label: None,
        effective_ratio_label: None,
        source: source.to_owned(),
    }
}

fn build_custom_recharge_quote(
    amount_cents: u64,
    current_remaining_units: Option<u64>,
    applied_coupon: Option<CommerceCouponDefinition>,
) -> CommerceResult<PortalCommerceQuote> {
    let rule = resolve_custom_recharge_rule(amount_cents)?;
    let mut quote = build_priced_quote(
        "custom_recharge",
        &custom_recharge_target_id(amount_cents),
        "Custom recharge",
        amount_cents,
        amount_cents.saturating_mul(rule.units_per_cent),
        "workspace_seed",
        current_remaining_units,
        applied_coupon,
    );
    quote.amount_cents = Some(amount_cents);
    quote.pricing_rule_label = Some("Tiered custom recharge".to_owned());
    quote.effective_ratio_label = Some(format_effective_ratio_label(rule.units_per_cent));
    Ok(quote)
}

fn build_redemption_quote(
    coupon: CommerceCouponDefinition,
    current_remaining_units: Option<u64>,
) -> PortalCommerceQuote {
    let source = coupon.coupon.source.clone();
    let projected_remaining_units =
        current_remaining_units.map(|units| units.saturating_add(coupon.benefit.bonus_units));

    PortalCommerceQuote {
        target_kind: "coupon_redemption".to_owned(),
        target_id: coupon.coupon.code.clone(),
        target_name: coupon.coupon.code.clone(),
        list_price_cents: 0,
        payable_price_cents: 0,
        list_price_label: "$0.00".to_owned(),
        payable_price_label: "$0.00".to_owned(),
        granted_units: 0,
        bonus_units: coupon.benefit.bonus_units,
        amount_cents: None,
        projected_remaining_units,
        applied_coupon: Some(PortalAppliedCoupon {
            code: coupon.coupon.code,
            discount_label: coupon.coupon.discount_label,
            source: source.clone(),
            discount_percent: coupon.benefit.discount_percent,
            bonus_units: coupon.benefit.bonus_units,
        }),
        pricing_rule_label: None,
        effective_ratio_label: None,
        source,
    }
}

fn merge_coupon_benefit(
    current: CommerceCouponBenefit,
    previous: Option<CommerceCouponBenefit>,
) -> CommerceCouponBenefit {
    let fallback = previous.unwrap_or_default();
    CommerceCouponBenefit {
        discount_percent: current.discount_percent.or(fallback.discount_percent),
        fixed_discount_cents: current
            .fixed_discount_cents
            .or(fallback.fixed_discount_cents),
        bonus_units: if current.bonus_units > 0 {
            current.bonus_units
        } else {
            fallback.bonus_units
        },
    }
}

fn normalize_coupon_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn hash_coupon_code_for_lookup(value: &str) -> String {
    let normalized = normalize_coupon_code(value);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn resolve_marketing_quote_subject_identity(user_id: Option<&str>) -> (String, String) {
    match user_id.map(str::trim).filter(|value| !value.is_empty()) {
        Some(user_id) => ("user".to_owned(), user_id.to_owned()),
        None => ("anonymous".to_owned(), "anonymous".to_owned()),
    }
}

fn convert_percentage_discount_to_u8(value: f64) -> Option<u8> {
    if !value.is_finite() || value < 0.0 || value > 100.0 {
        return None;
    }
    if (value.fract()).abs() > f64::EPSILON {
        return None;
    }

    Some(value.round() as u8)
}

fn convert_fixed_discount_amount_to_cents(value: f64) -> Option<u64> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }

    Some((value * 100.0).round() as u64)
}

fn format_marketing_discount_label(validation: &CouponQuoteValidation) -> String {
    if let Some(percentage_off) = validation.percentage_off {
        if (percentage_off.fract()).abs() <= f64::EPSILON {
            return format!("{}% off", percentage_off.round() as u64);
        }

        return format!("{percentage_off:.2}% off");
    }

    if let Some(fixed_discount_amount) = validation.fixed_discount_amount {
        let currency_code = validation
            .currency_code
            .as_deref()
            .unwrap_or("USD")
            .to_ascii_uppercase();
        return if currency_code == "USD" {
            format!("${fixed_discount_amount:.2} off")
        } else {
            format!("{currency_code} {fixed_discount_amount:.2} off")
        };
    }

    "Discount".to_owned()
}

fn format_catalog_price_label(price_cents: u64) -> String {
    if price_cents % 100 == 0 {
        return format!("${}", price_cents / 100);
    }

    format_quote_price_label(price_cents)
}

fn format_quote_price_label(price_cents: u64) -> String {
    format!("${:.2}", price_cents as f64 / 100.0)
}

fn format_integer_with_commas(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(character);
    }

    formatted
}

fn format_effective_ratio_label(units_per_cent: u64) -> String {
    format!(
        "{} units / $1",
        format_integer_with_commas(units_per_cent.saturating_mul(100))
    )
}

fn custom_recharge_min_amount_cents() -> u64 {
    1_000
}

fn custom_recharge_max_amount_cents() -> u64 {
    200_000
}

fn custom_recharge_step_amount_cents() -> u64 {
    500
}

fn custom_recharge_suggested_amount_cents() -> u64 {
    5_000
}

fn custom_recharge_target_id(amount_cents: u64) -> String {
    format!("custom-{amount_cents}")
}

fn parse_custom_recharge_target_amount(target_id: &str) -> Option<u64> {
    target_id
        .strip_prefix("custom-")
        .and_then(|value| value.parse::<u64>().ok())
}

fn resolve_custom_recharge_amount_cents(
    target_id: &str,
    request_amount_cents: Option<u64>,
) -> CommerceResult<u64> {
    let amount_from_target = parse_custom_recharge_target_amount(target_id);

    if let (Some(target_amount_cents), Some(request_amount_cents)) =
        (amount_from_target, request_amount_cents)
    {
        if target_amount_cents != request_amount_cents {
            return Err(CommerceError::InvalidInput(
                "custom recharge amount does not match target_id".to_owned(),
            ));
        }
    }

    let amount_cents = request_amount_cents.or(amount_from_target).ok_or_else(|| {
        CommerceError::InvalidInput(
            "custom_amount_cents is required for custom_recharge".to_owned(),
        )
    })?;

    validate_custom_recharge_amount_cents(amount_cents)?;
    Ok(amount_cents)
}

fn validate_custom_recharge_amount_cents(amount_cents: u64) -> CommerceResult<()> {
    let min_amount_cents = custom_recharge_min_amount_cents();
    let max_amount_cents = custom_recharge_max_amount_cents();
    let step_amount_cents = custom_recharge_step_amount_cents();

    if amount_cents < min_amount_cents || amount_cents > max_amount_cents {
        return Err(CommerceError::InvalidInput(format!(
            "custom_amount_cents must stay between {min_amount_cents} and {max_amount_cents}"
        )));
    }

    if amount_cents % step_amount_cents != 0 {
        return Err(CommerceError::InvalidInput(format!(
            "custom_amount_cents must increase in steps of {step_amount_cents}"
        )));
    }

    Ok(())
}

fn resolve_custom_recharge_rule(amount_cents: u64) -> CommerceResult<CustomRechargeRuleSeed> {
    custom_recharge_rule_seeds()
        .into_iter()
        .find(|rule| amount_cents >= rule.min_amount_cents && amount_cents <= rule.max_amount_cents)
        .ok_or_else(|| {
            CommerceError::InvalidInput(format!(
                "no custom recharge rule applies to amount {amount_cents}"
            ))
        })
}

fn parse_discount_percent(label: &str) -> Option<u8> {
    let percent_index = label.find('%')?;
    let digits = label[..percent_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }

    let value = digits.parse::<u8>().ok()?;
    Some(value.min(100))
}

fn subscription_plan_seeds() -> Vec<SubscriptionPlanSeed> {
    vec![
        SubscriptionPlanSeed {
            id: "starter",
            name: "Starter",
            price_cents: 1_900,
            cadence: "/month",
            included_units: 10_000,
            highlight: "For prototypes and lean internal tools",
            features: &[
                "10k token units included",
                "2 live API keys",
                "Email support",
            ],
            cta: "Start Starter",
        },
        SubscriptionPlanSeed {
            id: "growth",
            name: "Growth",
            price_cents: 7_900,
            cadence: "/month",
            included_units: 100_000,
            highlight: "For production workloads and multi-environment delivery",
            features: &[
                "100k token units included",
                "10 live API keys",
                "Priority support",
            ],
            cta: "Upgrade to Growth",
        },
        SubscriptionPlanSeed {
            id: "scale",
            name: "Scale",
            price_cents: 24_900,
            cadence: "/month",
            included_units: 500_000,
            highlight: "For platform teams optimizing predictable spend",
            features: &[
                "500k token units included",
                "Unlimited keys",
                "Architecture advisory",
            ],
            cta: "Talk to Sales",
        },
    ]
}

fn recharge_pack_seeds() -> Vec<RechargePackSeed> {
    vec![
        RechargePackSeed {
            id: "pack-25k",
            label: "Boost 25k",
            points: 25_000,
            price_cents: 1_200,
            note: "Best for launch spikes and testing windows.",
        },
        RechargePackSeed {
            id: "pack-100k",
            label: "Boost 100k",
            points: 100_000,
            price_cents: 4_000,
            note: "Designed for monthly usage expansion.",
        },
        RechargePackSeed {
            id: "pack-500k",
            label: "Boost 500k",
            points: 500_000,
            price_cents: 16_500,
            note: "For scheduled releases and campaign traffic.",
        },
    ]
}

fn recharge_option_seeds() -> Vec<RechargeOptionSeed> {
    vec![
        RechargeOptionSeed {
            id: "recharge-10",
            label: "Starter top-up",
            amount_cents: 1_000,
            granted_units: 25_000,
            note: "Fastest way to restore balance for prototyping and short tests.",
            recommended: false,
        },
        RechargeOptionSeed {
            id: "recharge-50",
            label: "Growth top-up",
            amount_cents: 5_000,
            granted_units: 140_000,
            note: "Best balance between instant headroom and effective recharge ratio.",
            recommended: true,
        },
        RechargeOptionSeed {
            id: "recharge-100",
            label: "Scale top-up",
            amount_cents: 10_000,
            granted_units: 300_000,
            note: "Designed for sustained production traffic and larger daily workloads.",
            recommended: false,
        },
        RechargeOptionSeed {
            id: "recharge-200",
            label: "Campaign top-up",
            amount_cents: 20_000,
            granted_units: 660_000,
            note: "Most efficient preset for launches, campaigns, and heavy concurrency windows.",
            recommended: false,
        },
    ]
}

fn custom_recharge_rule_seeds() -> Vec<CustomRechargeRuleSeed> {
    vec![
        CustomRechargeRuleSeed {
            id: "tier-entry",
            label: "Entry tier",
            min_amount_cents: 1_000,
            max_amount_cents: 4_500,
            units_per_cent: 25,
            note:
                "Entry custom recharges restore balance quickly while preserving the starter ratio.",
        },
        CustomRechargeRuleSeed {
            id: "tier-growth",
            label: "Growth tier",
            min_amount_cents: 5_000,
            max_amount_cents: 9_500,
            units_per_cent: 28,
            note: "Growth custom recharges match the recommended balance-to-headroom ratio.",
        },
        CustomRechargeRuleSeed {
            id: "tier-scale",
            label: "Scale tier",
            min_amount_cents: 10_000,
            max_amount_cents: 19_500,
            units_per_cent: 30,
            note: "Scale custom recharges keep larger recurring traffic windows cost-efficient.",
        },
        CustomRechargeRuleSeed {
            id: "tier-campaign",
            label: "Campaign tier",
            min_amount_cents: 20_000,
            max_amount_cents: 200_000,
            units_per_cent: 33,
            note: "Campaign custom recharges maximize the effective ratio for larger top-ups.",
        },
    ]
}

fn seed_coupon_definitions() -> Vec<CommerceCouponDefinition> {
    coupon_seeds()
        .into_iter()
        .map(|seed| CommerceCouponDefinition {
            coupon: PortalCommerceCoupon {
                id: seed.id.to_owned(),
                code: seed.code.to_owned(),
                discount_label: seed.discount_label.to_owned(),
                audience: seed.audience.to_owned(),
                remaining: seed.remaining,
                active: true,
                note: seed.note.to_owned(),
                expires_on: seed.expires_on.to_owned(),
                source: "workspace_seed".to_owned(),
                discount_percent: seed.discount_percent,
                bonus_units: seed.bonus_units,
            },
            benefit: CommerceCouponBenefit {
                discount_percent: seed.discount_percent,
                fixed_discount_cents: None,
                bonus_units: seed.bonus_units,
            },
        })
        .collect()
}

fn coupon_seeds() -> Vec<CouponSeed> {
    vec![
        CouponSeed {
            id: "seed_welcome100",
            code: "WELCOME100",
            discount_label: "+100 starter points",
            audience: "new_workspace",
            remaining: 100,
            note: "Apply during onboarding to offset initial exploration traffic.",
            expires_on: "rolling",
            discount_percent: None,
            bonus_units: 100,
        },
        CouponSeed {
            id: "seed_springboost",
            code: "SPRINGBOOST",
            discount_label: "10% off Growth",
            audience: "growth_upgrade",
            remaining: 10_000,
            note: "Use on the next subscription change for a temporary expansion window.",
            expires_on: "rolling",
            discount_percent: Some(10),
            bonus_units: 0,
        },
        CouponSeed {
            id: "seed_teamready",
            code: "TEAMREADY",
            discount_label: "Free staging credits",
            audience: "team_rollout",
            remaining: 25_000,
            note: "Unlocks extra staging budget for launch validation.",
            expires_on: "rolling",
            discount_percent: None,
            bonus_units: 25_000,
        },
    ]
}

fn generate_entity_id(prefix: &str) -> CommerceResult<String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CommerceError::Storage(anyhow::anyhow!("system clock error")))?
        .as_nanos();
    Ok(format!("{prefix}_{nonce:x}"))
}

fn current_time_ms() -> CommerceResult<u64> {
    Ok(u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommerceError::Storage(anyhow::anyhow!("system clock error")))?
            .as_millis(),
    )
    .map_err(|error| CommerceError::Storage(error.into()))?)
}

#[async_trait]
trait CommerceQuotaStore: Send + Sync {
    async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<QuotaPolicy>>;
}

#[async_trait]
impl<T> CommerceQuotaStore for T
where
    T: AdminStore + ?Sized,
{
    async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<QuotaPolicy>> {
        AdminStore::list_quota_policies_for_project(self, project_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use sdkwork_api_domain_billing::{
        AccountBenefitSourceType, AccountBenefitType, AccountLedgerEntryType, AccountType,
    };
    use sdkwork_api_domain_identity::PortalUserRecord;
    use sdkwork_api_domain_marketing::{
        CouponBenefitKind, CouponBenefitRuleRecord, CouponCodeBatchRecord, CouponCodeBatchStatus,
        CouponCodeGenerationMode, CouponCodeKind, CouponCodeRecord, CouponCodeStatus,
        CouponDistributionKind, CouponRedemptionStatus, CouponTemplateRecord, CouponTemplateStatus,
    };
    use sdkwork_api_domain_tenant::{Project, Tenant};
    use sdkwork_api_storage_core::{AccountKernelStore, IdentityKernelStore};
    use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
    use std::sync::Mutex;

    #[test]
    fn parses_percent_discount_suffixes() {
        assert_eq!(parse_discount_percent("20% launch discount"), Some(20));
        assert_eq!(parse_discount_percent("10% off Growth"), Some(10));
        assert_eq!(parse_discount_percent("Free staging credits"), None);
    }

    #[test]
    fn priced_quote_applies_discount_and_bonus_units() {
        let quote = build_priced_quote(
            "recharge_pack",
            "pack-100k",
            "Boost 100k",
            4_000,
            100_000,
            "workspace_seed",
            Some(5_000),
            Some(CommerceCouponDefinition {
                coupon: PortalCommerceCoupon {
                    id: "coupon_spring_launch".to_owned(),
                    code: "SPRING20".to_owned(),
                    discount_label: "20% launch discount".to_owned(),
                    audience: "new_signup".to_owned(),
                    remaining: 120,
                    active: true,
                    note: "Spring launch campaign".to_owned(),
                    expires_on: "2026-05-31".to_owned(),
                    source: "live".to_owned(),
                    discount_percent: Some(20),
                    bonus_units: 0,
                },
                benefit: CommerceCouponBenefit {
                    discount_percent: Some(20),
                    fixed_discount_cents: None,
                    bonus_units: 0,
                },
            }),
        );

        assert_eq!(quote.payable_price_label, "$32.00");
        assert_eq!(quote.projected_remaining_units, Some(105_000));
        assert_eq!(quote.applied_coupon.unwrap().code, "SPRING20");
    }

    #[test]
    fn redemption_quote_uses_bonus_units() {
        let quote = build_redemption_quote(
            CommerceCouponDefinition {
                coupon: PortalCommerceCoupon {
                    id: "seed_welcome100".to_owned(),
                    code: "WELCOME100".to_owned(),
                    discount_label: "+100 starter points".to_owned(),
                    audience: "new_workspace".to_owned(),
                    remaining: 100,
                    active: true,
                    note: "Apply during onboarding to offset initial exploration traffic."
                        .to_owned(),
                    expires_on: "rolling".to_owned(),
                    source: "workspace_seed".to_owned(),
                    discount_percent: None,
                    bonus_units: 100,
                },
                benefit: CommerceCouponBenefit {
                    discount_percent: None,
                    fixed_discount_cents: None,
                    bonus_units: 100,
                },
            },
            Some(5_000),
        );

        assert_eq!(quote.payable_price_label, "$0.00");
        assert_eq!(quote.bonus_units, 100);
        assert_eq!(quote.projected_remaining_units, Some(5_100));
    }

    #[tokio::test]
    async fn load_effective_quota_policy_reads_only_project_scope() {
        let store = RecordingCommerceQuotaStore::new(vec![
            QuotaPolicy::new("policy-project-1-a", "project-1", 300).with_enabled(true),
            QuotaPolicy::new("policy-project-1-b", "project-1", 200).with_enabled(true),
            QuotaPolicy::new("policy-project-2", "project-2", 1).with_enabled(true),
        ]);

        let policy = load_effective_quota_policy(&store, "project-1")
            .await
            .unwrap()
            .expect("expected project policy");

        assert_eq!(policy.policy_id, "policy-project-1-b");
        assert_eq!(
            store.last_project_id.lock().unwrap().as_deref(),
            Some("project-1")
        );
    }

    #[tokio::test]
    async fn preview_quote_for_user_uses_canonical_marketing_discount_codes() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            100,
            1001,
            2002,
            "growth-discount",
            "Growth recharge discount",
            CouponBenefitKind::PercentageDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(false)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            200,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::PercentageDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_percentage_off(Some(15.0))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            300,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            400,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("SPRING15"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Issued)
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let quote = preview_portal_commerce_quote_for_user(
            &store,
            "user_123",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("spring15".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(quote.payable_price_cents, 3_400);
        assert_eq!(quote.payable_price_label, "$34.00");
        assert_eq!(quote.projected_remaining_units, Some(105_000));

        let applied_coupon = quote.applied_coupon.expect("expected applied coupon");
        assert_eq!(applied_coupon.code, "SPRING15");
        assert_eq!(applied_coupon.discount_percent, Some(15));
        assert_eq!(applied_coupon.source, "marketing");
    }

    #[tokio::test]
    async fn preview_quote_for_user_supports_claim_required_marketing_codes_for_string_subject_ids()
    {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            110,
            1001,
            2002,
            "workspace-growth-discount",
            "Workspace growth recharge discount",
            CouponBenefitKind::PercentageDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(true)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            210,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::PercentageDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_percentage_off(Some(12.0))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            310,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            410,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("WORKSPACE12"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Claimed)
        .with_claim_subject_type(Some("user".to_owned()))
        .with_claim_subject_id(Some("1001:2002:user_workspace_owner".to_owned()))
        .with_claimed_at_ms(Some(1_710_000_031))
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let quote = preview_portal_commerce_quote_for_user(
            &store,
            "user_workspace_owner",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("workspace12".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(quote.payable_price_cents, 3_520);
        assert_eq!(quote.payable_price_label, "$35.20");
        assert_eq!(quote.projected_remaining_units, Some(105_000));

        let applied_coupon = quote.applied_coupon.expect("expected applied coupon");
        assert_eq!(applied_coupon.code, "WORKSPACE12");
        assert_eq!(applied_coupon.discount_percent, Some(12));
        assert_eq!(applied_coupon.source, "marketing");
    }

    #[tokio::test]
    async fn preview_quote_for_user_applies_canonical_fixed_amount_discount_codes() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            101,
            1001,
            2002,
            "fixed-discount",
            "Fixed recharge discount",
            CouponBenefitKind::FixedAmountDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(false)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            201,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::FixedAmountDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_fixed_discount_amount(Some(5.0))
        .with_currency_code(Some("USD".to_owned()))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            301,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            401,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("LESS5"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Issued)
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let quote = preview_portal_commerce_quote_for_user(
            &store,
            "user_123",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("less5".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(quote.payable_price_cents, 3_500);
        assert_eq!(quote.payable_price_label, "$35.00");

        let applied_coupon = quote.applied_coupon.expect("expected applied coupon");
        assert_eq!(applied_coupon.code, "LESS5");
        assert_eq!(applied_coupon.discount_label, "$5.00 off");
        assert_eq!(applied_coupon.discount_percent, None);
    }

    #[tokio::test]
    async fn settle_paid_order_redeems_canonical_marketing_coupon() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            120,
            1001,
            2002,
            "paid-workspace-discount",
            "Paid workspace recharge discount",
            CouponBenefitKind::PercentageDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(true)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            220,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::PercentageDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_percentage_off(Some(15.0))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            320,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            420,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("PAID15"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Claimed)
        .with_claim_subject_type(Some("user".to_owned()))
        .with_claim_subject_id(Some("1001:2002:user_workspace_owner".to_owned()))
        .with_claimed_at_ms(Some(1_710_000_031))
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let order = submit_portal_commerce_order(
            &store,
            "user_workspace_owner",
            "project_alpha",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("paid15".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(order.status, "pending_payment");
        let pending_redemptions = store.list_coupon_redemption_records().await.unwrap();
        assert_eq!(pending_redemptions.len(), 1);
        assert_eq!(
            pending_redemptions[0].status,
            CouponRedemptionStatus::Pending
        );
        assert_eq!(
            pending_redemptions[0].order_id.as_deref(),
            Some(order.order_id.as_str())
        );
        let reserved_code = store
            .find_coupon_code_record_by_lookup_hash(&hash_coupon_code_for_lookup("PAID15"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reserved_code.status, CouponCodeStatus::Claimed);

        let settled = settle_portal_commerce_order(
            &store,
            "user_workspace_owner",
            "project_alpha",
            &order.order_id,
            None,
        )
        .await
        .unwrap();

        assert_eq!(settled.status, "fulfilled");

        let redemptions = store.list_coupon_redemption_records().await.unwrap();
        assert_eq!(redemptions.len(), 1);
        assert_eq!(redemptions[0].status, CouponRedemptionStatus::Fulfilled);
        assert_eq!(redemptions[0].subject_type, "user");
        assert_eq!(redemptions[0].subject_id, "user_workspace_owner");
        assert_eq!(
            redemptions[0].order_id.as_deref(),
            Some(order.order_id.as_str())
        );

        let stored_code = store
            .find_coupon_code_record_by_lookup_hash(&hash_coupon_code_for_lookup("PAID15"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored_code.status, CouponCodeStatus::Redeemed);
    }

    #[tokio::test]
    async fn settle_paid_recharge_creates_canonical_identity_and_account_grant() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        store
            .insert_tenant(&Tenant::new("tenant_commerce", "Commerce Workspace"))
            .await
            .unwrap();
        store
            .insert_project(&Project::new(
                "tenant_commerce",
                "project_commerce",
                "default",
            ))
            .await
            .unwrap();
        store
            .insert_portal_user(&PortalUserRecord::new(
                "user_paid_owner",
                "paid-owner@example.com",
                "Paid Owner",
                "salt",
                "hash",
                "tenant_commerce",
                "project_commerce",
                true,
                1_710_000_000,
            ))
            .await
            .unwrap();

        let order = submit_portal_commerce_order(
            &store,
            "user_paid_owner",
            "project_commerce",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: None,
                current_remaining_units: Some(260),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(order.status, "pending_payment");

        let settled = settle_portal_commerce_order(
            &store,
            "user_paid_owner",
            "project_commerce",
            &order.order_id,
            Some("payment_success_100k"),
        )
        .await
        .unwrap();
        assert_eq!(settled.status, "fulfilled");

        let identity_users = store.list_identity_user_records().await.unwrap();
        assert_eq!(identity_users.len(), 1);
        assert_eq!(
            identity_users[0].external_user_ref.as_deref(),
            Some("tenant_commerce:project_commerce:user_paid_owner")
        );
        assert_eq!(
            identity_users[0].email.as_deref(),
            Some("paid-owner@example.com")
        );

        let binding = store
            .find_identity_binding_record(
                PORTAL_WORKSPACE_IDENTITY_BINDING_TYPE,
                Some(PORTAL_WORKSPACE_IDENTITY_BINDING_ISSUER),
                Some("tenant_commerce:project_commerce:user_paid_owner"),
            )
            .await
            .unwrap()
            .expect("expected canonical workspace binding");
        assert_eq!(binding.user_id, identity_users[0].user_id);

        let accounts = store.list_account_records().await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_type, AccountType::Primary);
        assert_eq!(accounts[0].tenant_id, identity_users[0].tenant_id);
        assert_eq!(
            accounts[0].organization_id,
            identity_users[0].organization_id
        );
        assert_eq!(accounts[0].user_id, identity_users[0].user_id);

        let benefit_lots = store.list_account_benefit_lots().await.unwrap();
        assert_eq!(benefit_lots.len(), 1);
        assert_eq!(benefit_lots[0].account_id, accounts[0].account_id);
        assert_eq!(benefit_lots[0].benefit_type, AccountBenefitType::CashCredit);
        assert_eq!(
            benefit_lots[0].source_type,
            AccountBenefitSourceType::Recharge
        );
        assert_eq!(benefit_lots[0].original_quantity, 100_000.0);
        assert_eq!(benefit_lots[0].remaining_quantity, 100_000.0);
        assert_eq!(benefit_lots[0].held_quantity, 0.0);

        let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
        assert_eq!(ledger_entries.len(), 1);
        assert_eq!(ledger_entries[0].account_id, accounts[0].account_id);
        assert_eq!(
            ledger_entries[0].entry_type,
            AccountLedgerEntryType::GrantIssue
        );
        assert_eq!(ledger_entries[0].quantity, 100_000.0);
        assert_eq!(ledger_entries[0].amount, 100_000.0);

        let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
        assert_eq!(ledger_allocations.len(), 1);
        assert_eq!(
            ledger_allocations[0].ledger_entry_id,
            ledger_entries[0].ledger_entry_id
        );
        assert_eq!(ledger_allocations[0].lot_id, benefit_lots[0].lot_id);
        assert_eq!(ledger_allocations[0].quantity_delta, 100_000.0);
    }

    #[tokio::test]
    async fn cancel_pending_paid_marketing_order_voids_coupon_reservation() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            123,
            1001,
            2002,
            "cancel-marketing-discount",
            "Cancelable workspace recharge discount",
            CouponBenefitKind::PercentageDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(true)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            223,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::PercentageDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_percentage_off(Some(15.0))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            323,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            423,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("CANCEL15"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Claimed)
        .with_claim_subject_type(Some("user".to_owned()))
        .with_claim_subject_id(Some("1001:2002:user_cancel_owner".to_owned()))
        .with_claimed_at_ms(Some(1_710_000_031))
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let order = submit_portal_commerce_order(
            &store,
            "user_cancel_owner",
            "project_cancel",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("cancel15".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        let canceled = cancel_portal_commerce_order(
            &store,
            "user_cancel_owner",
            "project_cancel",
            &order.order_id,
        )
        .await
        .unwrap();
        assert_eq!(canceled.status, "canceled");

        let redemptions = store.list_coupon_redemption_records().await.unwrap();
        assert_eq!(redemptions.len(), 1);
        assert_eq!(redemptions[0].status, CouponRedemptionStatus::Voided);

        let stored_code = store
            .find_coupon_code_record_by_lookup_hash(&hash_coupon_code_for_lookup("CANCEL15"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored_code.status, CouponCodeStatus::Claimed);
    }

    #[tokio::test]
    async fn zero_pay_marketing_order_fulfills_and_redeems_coupon_on_create() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            121,
            1001,
            2002,
            "free-workspace-discount",
            "Zero pay workspace recharge discount",
            CouponBenefitKind::FixedAmountDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(true)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            221,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::FixedAmountDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_fixed_discount_amount(Some(40.0))
        .with_currency_code(Some("USD".to_owned()))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            321,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            421,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("FREE100"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Claimed)
        .with_claim_subject_type(Some("user".to_owned()))
        .with_claim_subject_id(Some("1001:2002:user_zero_pay_owner".to_owned()))
        .with_claimed_at_ms(Some(1_710_000_031))
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let order = submit_portal_commerce_order(
            &store,
            "user_zero_pay_owner",
            "project_zero_pay",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("free100".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(order.status, "fulfilled");
        assert_eq!(order.payable_price_cents, 0);

        let redemptions = store.list_coupon_redemption_records().await.unwrap();
        assert_eq!(redemptions.len(), 1);
        assert_eq!(redemptions[0].status, CouponRedemptionStatus::Fulfilled);
        assert_eq!(redemptions[0].subject_type, "user");
        assert_eq!(redemptions[0].subject_id, "user_zero_pay_owner");
        assert_eq!(
            redemptions[0].order_id.as_deref(),
            Some(order.order_id.as_str())
        );

        let stored_code = store
            .find_coupon_code_record_by_lookup_hash(&hash_coupon_code_for_lookup("FREE100"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored_code.status, CouponCodeStatus::Redeemed);
    }

    #[tokio::test]
    async fn fulfilled_marketing_order_can_backfill_payment_order_id_on_settlement_replay() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);

        let template = CouponTemplateRecord::new(
            122,
            1001,
            2002,
            "replay-marketing-discount",
            "Replay-safe marketing recharge discount",
            CouponBenefitKind::PercentageDiscount,
            CouponDistributionKind::UniqueCode,
            1_710_000_000,
        )
        .with_status(CouponTemplateStatus::Active)
        .with_claim_required(true)
        .with_updated_at_ms(1_710_000_100);
        store
            .insert_coupon_template_record(&template)
            .await
            .unwrap();

        let rule = CouponBenefitRuleRecord::new(
            222,
            1001,
            2002,
            template.coupon_template_id,
            CouponBenefitKind::PercentageDiscount,
            1_710_000_010,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned()))
        .with_percentage_off(Some(15.0))
        .with_updated_at_ms(1_710_000_101);
        store
            .insert_coupon_benefit_rule_record(&rule)
            .await
            .unwrap();

        let batch = CouponCodeBatchRecord::new(
            322,
            1001,
            2002,
            template.coupon_template_id,
            None,
            CouponCodeGenerationMode::BulkRandom,
            1_710_000_020,
        )
        .with_status(CouponCodeBatchStatus::Active)
        .with_issued_count(1)
        .with_updated_at_ms(1_710_000_102);
        store.insert_coupon_code_batch_record(&batch).await.unwrap();

        let code = CouponCodeRecord::new(
            422,
            1001,
            2002,
            batch.coupon_code_batch_id,
            template.coupon_template_id,
            None,
            hash_coupon_code_for_lookup("REPLAY15"),
            CouponCodeKind::SingleUseUnique,
            1_710_000_030,
        )
        .with_status(CouponCodeStatus::Claimed)
        .with_claim_subject_type(Some("user".to_owned()))
        .with_claim_subject_id(Some("1001:2002:user_replay_owner".to_owned()))
        .with_claimed_at_ms(Some(1_710_000_031))
        .with_updated_at_ms(1_710_000_103);
        store.insert_coupon_code_record(&code).await.unwrap();

        let order = submit_portal_commerce_order(
            &store,
            "user_replay_owner",
            "project_replay",
            &PortalCommerceQuoteRequest {
                target_kind: "recharge_pack".to_owned(),
                target_id: "pack-100k".to_owned(),
                coupon_code: Some("replay15".to_owned()),
                current_remaining_units: Some(5_000),
                custom_amount_cents: None,
            },
        )
        .await
        .unwrap();

        let settled = settle_portal_commerce_order(
            &store,
            "user_replay_owner",
            "project_replay",
            &order.order_id,
            None,
        )
        .await
        .unwrap();
        assert_eq!(settled.status, "fulfilled");

        let replay = settle_portal_commerce_order(
            &store,
            "user_replay_owner",
            "project_replay",
            &order.order_id,
            Some("stripe_pi_replay"),
        )
        .await
        .unwrap();
        assert_eq!(replay.status, "fulfilled");

        let redemptions = store.list_coupon_redemption_records().await.unwrap();
        assert_eq!(redemptions.len(), 1);
        assert_eq!(
            redemptions[0].payment_order_id.as_deref(),
            Some("stripe_pi_replay")
        );
    }

    struct RecordingCommerceQuotaStore {
        policies: Vec<QuotaPolicy>,
        last_project_id: Mutex<Option<String>>,
    }

    impl RecordingCommerceQuotaStore {
        fn new(policies: Vec<QuotaPolicy>) -> Self {
            Self {
                policies,
                last_project_id: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl CommerceQuotaStore for RecordingCommerceQuotaStore {
        async fn list_quota_policies_for_project(
            &self,
            project_id: &str,
        ) -> anyhow::Result<Vec<QuotaPolicy>> {
            *self.last_project_id.lock().unwrap() = Some(project_id.to_owned());
            Ok(self
                .policies
                .iter()
                .filter(|policy| policy.project_id == project_id)
                .cloned()
                .collect())
        }
    }
}
