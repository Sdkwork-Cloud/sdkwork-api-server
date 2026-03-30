use async_trait::async_trait;
use sdkwork_api_app_coupon::list_active_coupons;
use sdkwork_api_domain_billing::QuotaPolicy;
use sdkwork_api_domain_commerce::{CommerceOrderRecord, ProjectMembershipRecord};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_storage_core::AdminStore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub use sdkwork_api_domain_commerce::CommerceOrderRecord as PortalCommerceOrderRecord;
pub use sdkwork_api_domain_commerce::ProjectMembershipRecord as PortalProjectMembershipRecord;

type CommerceResult<T> = std::result::Result<T, CommerceError>;

#[derive(Debug)]
pub enum CommerceError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Storage(anyhow::Error),
}

impl std::fmt::Display for CommerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Conflict(message) => write!(f, "{message}"),
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
    pub projected_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon: Option<PortalAppliedCoupon>,
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
    pub payable_price_label: String,
    pub guidance: String,
    pub methods: Vec<PortalCommerceCheckoutSessionMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortalCommercePaymentEventRequest {
    pub event_type: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct CommerceCouponBenefit {
    discount_percent: Option<u8>,
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
        coupons: load_coupon_catalog(store).await?,
    })
}

pub async fn preview_portal_commerce_quote(
    store: &dyn AdminStore,
    request: &PortalCommerceQuoteRequest,
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
                load_optional_applied_coupon(store, request.coupon_code.as_deref()).await?;
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
                load_optional_applied_coupon(store, request.coupon_code.as_deref()).await?;
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

    let quote = preview_portal_commerce_quote(store, request).await?;
    let status = initial_order_status(&quote);

    if should_fulfill_on_order_create(&quote) {
        apply_quote_to_project_quota(store, normalized_project_id, &quote).await?;
        consume_live_coupon_if_needed(store, quote.applied_coupon.as_ref()).await?;
        activate_project_membership_if_needed(
            store,
            normalized_user_id,
            normalized_project_id,
            &quote,
        )
        .await?;
    }

    let order = CommerceOrderRecord::new(
        generate_entity_id("commerce_order")?,
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
        "fulfilled" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be settled from status {other}"
            )))
        }
    }

    let settlement_quote = load_order_settlement_quote(store, &order).await?;
    apply_quote_to_project_quota(store, normalized_project_id, &settlement_quote).await?;
    consume_live_coupon_if_needed(store, settlement_quote.applied_coupon.as_ref()).await?;
    activate_project_membership_if_needed(
        store,
        normalized_user_id,
        normalized_project_id,
        &settlement_quote,
    )
    .await?;

    order.status = "fulfilled".to_owned();
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
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

    order.status = "canceled".to_owned();
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub async fn apply_portal_commerce_payment_event(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let event_type = request.event_type.trim();
    if event_type.is_empty() {
        return Err(CommerceError::InvalidInput(
            "event_type is required".to_owned(),
        ));
    }

    match event_type {
        "settled" => settle_portal_commerce_order(store, user_id, project_id, order_id).await,
        "canceled" => cancel_portal_commerce_order(store, user_id, project_id, order_id).await,
        "failed" => fail_portal_commerce_order(store, user_id, project_id, order_id).await,
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
) -> CommerceResult<PortalCommerceCheckoutSession> {
    let order = load_project_commerce_order(store, user_id, project_id, order_id).await?;
    Ok(build_checkout_session(&order))
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

async fn load_coupon_catalog(store: &dyn AdminStore) -> CommerceResult<Vec<PortalCommerceCoupon>> {
    Ok(load_coupon_definitions(store)
        .await?
        .into_iter()
        .map(|definition| definition.coupon)
        .collect())
}

async fn load_coupon_definitions(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<CommerceCouponDefinition>> {
    let mut definitions = seed_coupon_definitions()
        .into_iter()
        .map(|definition| (normalize_coupon_code(&definition.coupon.code), definition))
        .collect::<BTreeMap<_, _>>();

    for coupon in list_active_coupons(store).await? {
        let code = normalize_coupon_code(&coupon.code);
        let prior = definitions.get(&code).cloned();
        let parsed_benefit = CommerceCouponBenefit {
            discount_percent: parse_discount_percent(&coupon.discount_label),
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

async fn find_coupon_definition(
    store: &dyn AdminStore,
    code: &str,
) -> CommerceResult<CommerceCouponDefinition> {
    let normalized = normalize_coupon_code(code);
    load_coupon_definitions(store)
        .await?
        .into_iter()
        .find(|definition| definition.coupon.code == normalized)
        .ok_or_else(|| CommerceError::NotFound(format!("coupon {normalized} not found")))
}

async fn load_optional_applied_coupon(
    store: &dyn AdminStore,
    coupon_code: Option<&str>,
) -> CommerceResult<Option<CommerceCouponDefinition>> {
    match coupon_code.map(str::trim).filter(|value| !value.is_empty()) {
        Some(code) => find_coupon_definition(store, code).await.map(Some),
        None => Ok(None),
    }
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
        "recharge_pack" | "coupon_redemption" => current_limit.saturating_add(target_units),
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

async fn consume_live_coupon_if_needed(
    store: &dyn AdminStore,
    coupon: Option<&PortalAppliedCoupon>,
) -> CommerceResult<()> {
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

async fn activate_project_membership_if_needed(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()> {
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
    quote.target_kind == "coupon_redemption"
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
    let settlement_preview = preview_portal_commerce_quote(
        store,
        &PortalCommerceQuoteRequest {
            target_kind: order.target_kind.clone(),
            target_id: order.target_id.clone(),
            coupon_code: order.applied_coupon_code.clone(),
            current_remaining_units: None,
        },
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
        projected_remaining_units: None,
        applied_coupon: settlement_preview.applied_coupon,
        source: order.source.clone(),
    })
}

fn build_checkout_session(order: &CommerceOrderRecord) -> PortalCommerceCheckoutSession {
    let reference = format!("PAY-{}", normalize_payment_reference(&order.order_id));
    let guidance = match (order.target_kind.as_str(), order.status.as_str()) {
        ("subscription_plan", "pending_payment") => {
            "Settle this checkout to activate the workspace membership and included monthly units."
        }
        ("recharge_pack", "pending_payment") => {
            "Settle this checkout to apply the recharge pack and restore workspace quota headroom."
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
        "pending_payment" => (
            "open",
            "manual_lab",
            "operator_settlement",
            build_open_checkout_methods(order),
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
        payable_price_label: order.payable_price_label.clone(),
        guidance: guidance.to_owned(),
        methods,
    }
}

fn build_open_checkout_methods(
    order: &CommerceOrderRecord,
) -> Vec<PortalCommerceCheckoutSessionMethod> {
    let mut methods = vec![
        PortalCommerceCheckoutSessionMethod {
            id: "manual_settlement".to_owned(),
            label: "Manual settlement".to_owned(),
            detail:
                "Use the portal settlement action in desktop or lab mode to finalize the order."
                    .to_owned(),
            action: "settle_order".to_owned(),
            availability: "available".to_owned(),
        },
        PortalCommerceCheckoutSessionMethod {
            id: "cancel_order".to_owned(),
            label: "Cancel checkout".to_owned(),
            detail: "Close the pending order without applying quota or membership side effects."
                .to_owned(),
            action: "cancel_order".to_owned(),
            availability: "available".to_owned(),
        },
    ];

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
    let bonus_units = applied_coupon
        .as_ref()
        .map(|coupon| coupon.benefit.bonus_units)
        .unwrap_or(0);
    let payable_cents =
        list_price_cents.saturating_mul(u64::from(100_u8.saturating_sub(discount_percent))) / 100;
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
        projected_remaining_units,
        applied_coupon: applied_coupon.map(|coupon| PortalAppliedCoupon {
            code: coupon.coupon.code,
            discount_label: coupon.coupon.discount_label,
            source: coupon.coupon.source,
            discount_percent: coupon.benefit.discount_percent,
            bonus_units: coupon.benefit.bonus_units,
        }),
        source: source.to_owned(),
    }
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
        projected_remaining_units,
        applied_coupon: Some(PortalAppliedCoupon {
            code: coupon.coupon.code,
            discount_label: coupon.coupon.discount_label,
            source: source.clone(),
            discount_percent: coupon.benefit.discount_percent,
            bonus_units: coupon.benefit.bonus_units,
        }),
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

fn format_catalog_price_label(price_cents: u64) -> String {
    if price_cents % 100 == 0 {
        return format!("${}", price_cents / 100);
    }

    format_quote_price_label(price_cents)
}

fn format_quote_price_label(price_cents: u64) -> String {
    format!("${:.2}", price_cents as f64 / 100.0)
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
