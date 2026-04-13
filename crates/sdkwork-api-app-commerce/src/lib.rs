mod constants;
mod coupon_catalog;
mod coupon_state;
mod error;
mod order;
mod payment_attempt;
mod payment_event;
mod payment_method;
mod payment_provider;
mod reconciliation;
mod refund;
mod settlement;
mod types;
mod webhook;

use async_trait::async_trait;
pub use constants::*;
pub use coupon_catalog::reclaim_expired_coupon_reservations_for_code_if_needed;
use coupon_catalog::*;
use coupon_state::*;
pub use error::{commerce_atomic_coupon_error, CommerceError, CommerceResult};
pub use order::{
    apply_portal_commerce_payment_event, cancel_portal_commerce_order,
    list_project_commerce_orders, load_portal_commerce_catalog,
    load_portal_commerce_checkout_session, load_portal_commerce_checkout_session_with_policy,
    load_portal_commerce_order, load_project_membership, preview_portal_commerce_quote,
    settle_portal_commerce_order, settle_portal_commerce_order_from_verified_payment,
    settle_portal_commerce_order_with_billing,
    submit_portal_commerce_order,
};
pub(crate) use order::{
    refund_portal_commerce_order, settle_portal_commerce_order_with_payment_event,
};
pub use payment_attempt::{
    create_portal_commerce_payment_attempt, list_payment_attempts_for_order,
    list_portal_commerce_payment_attempts, load_portal_commerce_payment_attempt,
};
pub use payment_event::{
    apply_portal_commerce_payment_event_with_billing, list_order_commerce_payment_events,
};
use payment_method::build_checkout_session;
pub use payment_method::{
    delete_admin_payment_method, list_admin_payment_method_credential_bindings,
    list_admin_payment_methods, list_portal_commerce_payment_methods, persist_admin_payment_method,
    replace_admin_payment_method_credential_bindings,
};
pub use reconciliation::{
    create_admin_commerce_reconciliation_run, list_admin_commerce_reconciliation_items,
    list_admin_commerce_reconciliation_runs, list_admin_commerce_webhook_delivery_attempts,
    list_admin_commerce_webhook_inbox,
};
pub use refund::{create_admin_commerce_refund, list_admin_commerce_refunds_for_order};
use sdkwork_api_app_billing::{
    CommercialBillingAdminKernel, IssueCommerceOrderCreditsInput, RefundCommerceOrderCreditsInput,
};
use sdkwork_api_app_catalog::{
    build_canonical_commercial_catalog_with_pricing_plans, CanonicalCommercialCatalog,
    CommercialApiProductKind, CommercialCatalogSeedProduct,
};
use sdkwork_api_app_identity::GatewayRequestContext;
use sdkwork_api_app_marketing::{
    confirm_coupon_redemption, reserve_coupon_redemption, rollback_coupon_redemption,
    validate_coupon_stack, CouponValidationDecision,
};
use sdkwork_api_domain_billing::{
    AccountCommerceReconciliationStateRecord, AccountRecord, PricingPlanRecord, QuotaPolicy,
};
pub use sdkwork_api_domain_commerce::{
    CommerceOrderRecord as PortalCommerceOrderRecord, CommercePaymentAttemptRecord,
    CommercePaymentEventRecord as PortalCommercePaymentEventRecord,
    CommerceReconciliationItemRecord, CommerceReconciliationRunRecord, CommerceRefundRecord,
    CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord, PaymentMethodRecord,
    ProjectMembershipRecord as PortalProjectMembershipRecord,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentEventProcessingStatus, ProjectMembershipRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationStatus, CouponRollbackRecord, CouponRollbackStatus, CouponRollbackType,
    CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord,
    MarketingSubjectScope,
};
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_core::{
    AtomicCouponConfirmationCommand, AtomicCouponReleaseCommand, AtomicCouponReservationCommand,
    AtomicCouponRollbackCommand, AtomicCouponRollbackCompensationCommand,
};
use settlement::*;
pub(crate) use settlement::{fail_portal_commerce_order, load_project_commerce_order};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
pub use types::{
    AdminCommerceReconciliationRunCreateRequest, AdminCommerceRefundCreateRequest,
    PortalApiProduct, PortalAppliedCoupon, PortalCommerceCatalog, PortalCommerceCatalogBinding,
    PortalCommerceCheckoutSession, PortalCommerceCheckoutSessionMethod, PortalCommerceCoupon,
    PortalCommercePaymentAttemptCreateRequest, PortalCommercePaymentEventRequest,
    PortalCommerceQuote, PortalCommerceQuoteRequest, PortalCommerceWebhookAck,
    PortalCustomRechargePolicy, PortalCustomRechargeRule, PortalProductOffer, PortalRechargeOption,
    PortalRechargePack, PortalSubscriptionPlan,
};
pub use webhook::process_portal_stripe_webhook;

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

#[derive(Debug, Clone)]
struct ResolvedCouponDefinition {
    definition: CommerceCouponDefinition,
    marketing: Option<MarketingCouponContext>,
}

#[derive(Debug, Clone)]
struct MarketingCouponContext {
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
    source: String,
}

#[derive(Debug, Clone)]
struct ReservedMarketingCouponState {
    coupon_reservation_id: String,
    marketing_campaign_id: String,
    subsidy_amount_minor: u64,
}

#[derive(Debug, Clone)]
struct CommerceSettlementSideEffectSnapshot {
    previous_quota_policy: Option<QuotaPolicy>,
    previous_membership: Option<ProjectMembershipRecord>,
}

#[derive(Debug, Clone)]
struct CouponRollbackCompensationSnapshot {
    previous_budget: CampaignBudgetRecord,
    previous_code: CouponCodeRecord,
    previous_redemption: CouponRedemptionRecord,
    applied_budget: CampaignBudgetRecord,
    applied_code: CouponCodeRecord,
    applied_redemption: CouponRedemptionRecord,
    applied_rollback: CouponRollbackRecord,
}

impl PortalCommerceCatalogBinding {
    fn from_quote(quote: &PortalCommerceQuote) -> Self {
        Self {
            product_id: quote.product_id.clone(),
            offer_id: quote.offer_id.clone(),
            publication_id: quote.publication_id.clone(),
            publication_kind: quote.publication_kind.clone(),
            publication_status: quote.publication_status.clone(),
            publication_revision_id: quote.publication_revision_id.clone(),
            publication_version: quote.publication_version,
            publication_source_kind: quote.publication_source_kind.clone(),
            publication_effective_from_ms: quote.publication_effective_from_ms,
            pricing_plan_id: quote.pricing_plan_id.clone(),
            pricing_plan_version: quote.pricing_plan_version,
            pricing_rate_id: quote.pricing_rate_id.clone(),
            pricing_metric_code: quote.pricing_metric_code.clone(),
        }
    }

    fn from_order(order: &CommerceOrderRecord) -> Self {
        let snapshot_value =
            serde_json::from_str::<serde_json::Value>(&order.pricing_snapshot_json).ok();
        let catalog_binding = snapshot_value
            .as_ref()
            .and_then(|snapshot| snapshot.get("catalog_binding"));
        let pricing_binding = snapshot_value
            .as_ref()
            .and_then(|snapshot| snapshot.get("pricing_binding"));
        let quote_binding = snapshot_value
            .as_ref()
            .and_then(|snapshot| snapshot.get("quote"));
        let live_binding =
            current_quote_target_catalog_binding(&order.target_kind, &order.target_id);

        Self {
            product_id: catalog_binding
                .and_then(|binding| binding.get("product_id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("product_id"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.product_id),
            offer_id: catalog_binding
                .and_then(|binding| binding.get("offer_id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("offer_id"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.offer_id),
            publication_id: catalog_binding
                .and_then(|binding| binding.get("publication_id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_id"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.publication_id),
            publication_kind: catalog_binding
                .and_then(|binding| binding.get("publication_kind"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_kind"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.publication_kind),
            publication_status: catalog_binding
                .and_then(|binding| binding.get("publication_status"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_status"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.publication_status),
            publication_revision_id: catalog_binding
                .and_then(|binding| binding.get("publication_revision_id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_revision_id"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.publication_revision_id),
            publication_version: catalog_binding
                .and_then(|binding| binding.get("publication_version"))
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_version"))
                        .and_then(serde_json::Value::as_u64)
                })
                .or(live_binding.publication_version),
            publication_source_kind: catalog_binding
                .and_then(|binding| binding.get("publication_source_kind"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_source_kind"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or(live_binding.publication_source_kind),
            publication_effective_from_ms: catalog_binding
                .and_then(|binding| binding.get("publication_effective_from_ms"))
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("publication_effective_from_ms"))
                        .and_then(serde_json::Value::as_u64)
                })
                .or(live_binding.publication_effective_from_ms),
            pricing_plan_id: order
                .pricing_plan_id
                .clone()
                .or_else(|| {
                    pricing_binding
                        .and_then(|binding| binding.get("pricing_plan_id"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("pricing_plan_id"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                }),
            pricing_plan_version: order
                .pricing_plan_version
                .or_else(|| {
                    pricing_binding
                        .and_then(|binding| binding.get("pricing_plan_version"))
                        .and_then(serde_json::Value::as_u64)
                })
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("pricing_plan_version"))
                        .and_then(serde_json::Value::as_u64)
                }),
            pricing_rate_id: pricing_binding
                .and_then(|binding| binding.get("pricing_rate_id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            pricing_metric_code: pricing_binding
                .and_then(|binding| binding.get("pricing_metric_code"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    quote_binding
                        .and_then(|binding| binding.get("pricing_metric_code"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                }),
        }
    }
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

fn commercial_catalog_seed_products(
    plans: &[PortalSubscriptionPlan],
    packs: &[PortalRechargePack],
    custom_recharge_policy: Option<&PortalCustomRechargePolicy>,
) -> Vec<CommercialCatalogSeedProduct> {
    let mut seed_products = Vec::with_capacity(
        plans.len() + packs.len() + usize::from(custom_recharge_policy.is_some()),
    );

    seed_products.extend(plans.iter().map(|plan| {
        CommercialCatalogSeedProduct::new(
            CommercialApiProductKind::SubscriptionPlan,
            plan.id.clone(),
            plan.name.clone(),
            plan.source.clone(),
        )
        .with_price_label_option(Some(plan.price_label.clone()))
    }));
    seed_products.extend(packs.iter().map(|pack| {
        CommercialCatalogSeedProduct::new(
            CommercialApiProductKind::RechargePack,
            pack.id.clone(),
            pack.label.clone(),
            pack.source.clone(),
        )
        .with_price_label_option(Some(pack.price_label.clone()))
    }));

    if let Some(policy) = custom_recharge_policy {
        seed_products.push(CommercialCatalogSeedProduct::new(
            CommercialApiProductKind::CustomRecharge,
            "custom_recharge",
            "Custom recharge",
            policy.source.clone(),
        ));
    }

    seed_products
}

fn current_canonical_commercial_catalog_with_pricing_plans(
    pricing_plans: &[PricingPlanRecord],
) -> CanonicalCommercialCatalog {
    let plans = subscription_plan_catalog();
    let packs = recharge_pack_catalog();
    let custom_recharge_policy = Some(build_custom_recharge_policy());
    build_canonical_commercial_catalog_with_pricing_plans(
        &commercial_catalog_seed_products(&plans, &packs, custom_recharge_policy.as_ref()),
        pricing_plans,
    )
}

pub(crate) fn current_canonical_commercial_catalog() -> CanonicalCommercialCatalog {
    current_canonical_commercial_catalog_with_pricing_plans(&[])
}

pub async fn current_canonical_commercial_catalog_for_store(
    store: &dyn AdminStore,
) -> CommerceResult<CanonicalCommercialCatalog> {
    let pricing_plans = match store.account_kernel_store() {
        Some(kernel) => kernel.list_pricing_plan_records().await?,
        None => Vec::new(),
    };
    Ok(current_canonical_commercial_catalog_with_pricing_plans(
        &pricing_plans,
    ))
}

fn project_quote_target_catalog_binding_from_catalog(
    catalog: &CanonicalCommercialCatalog,
    target_kind: &str,
    target_id: &str,
) -> PortalCommerceCatalogBinding {
    let offer = catalog.offers.iter().find(|offer| {
        if offer.quote_target_kind.as_str() != target_kind {
            return false;
        }

        if target_kind == "custom_recharge" {
            return offer.quote_target_id == "custom_recharge";
        }

        offer.quote_target_id.eq_ignore_ascii_case(target_id)
    });

    match offer {
        Some(offer) => {
            let publication = catalog
                .publications
                .iter()
                .find(|publication| publication.offer_id == offer.offer_id);
            PortalCommerceCatalogBinding {
                product_id: Some(offer.product_id.clone()),
                offer_id: Some(offer.offer_id.clone()),
                publication_id: publication.map(|item| item.publication_id.clone()),
                publication_kind: publication.map(|item| item.publication_kind.as_str().to_owned()),
                publication_status: publication.map(|item| item.status.as_str().to_owned()),
                publication_revision_id: publication
                    .map(|item| item.publication_revision_id.clone()),
                publication_version: publication.map(|item| item.publication_version),
                publication_source_kind: publication
                    .map(|item| item.publication_source_kind.clone()),
                publication_effective_from_ms: publication
                    .and_then(|item| item.publication_effective_from_ms),
                pricing_plan_id: offer.pricing_plan_id.clone(),
                pricing_plan_version: offer.pricing_plan_version,
                pricing_rate_id: offer.pricing_rate_id.clone(),
                pricing_metric_code: offer.pricing_metric_code.clone(),
            }
        }
        None => PortalCommerceCatalogBinding::default(),
    }
}

pub(crate) fn current_quote_target_catalog_binding(
    target_kind: &str,
    target_id: &str,
) -> PortalCommerceCatalogBinding {
    project_quote_target_catalog_binding_from_catalog(
        &current_canonical_commercial_catalog(),
        target_kind,
        target_id,
    )
}

pub(crate) async fn current_quote_target_catalog_binding_for_store(
    store: &dyn AdminStore,
    target_kind: &str,
    target_id: &str,
) -> CommerceResult<PortalCommerceCatalogBinding> {
    let catalog = current_canonical_commercial_catalog_for_store(store).await?;
    Ok(project_quote_target_catalog_binding_from_catalog(
        &catalog,
        target_kind,
        target_id,
    ))
}

pub fn project_portal_commerce_order_catalog_binding(
    order: &PortalCommerceOrderRecord,
) -> PortalCommerceCatalogBinding {
    PortalCommerceCatalogBinding::from_order(order)
}

fn portal_api_products_from_canonical_catalog(
    catalog: &CanonicalCommercialCatalog,
) -> Vec<PortalApiProduct> {
    catalog
        .products
        .iter()
        .map(|product| PortalApiProduct {
            product_id: product.product_id.clone(),
            product_kind: product.product_kind.as_str().to_owned(),
            target_id: product.target_id.clone(),
            display_name: product.display_name.clone(),
            source: product.source.clone(),
        })
        .collect()
}

fn portal_product_offers_from_canonical_catalog(
    catalog: &CanonicalCommercialCatalog,
) -> Vec<PortalProductOffer> {
    catalog
        .offers
        .iter()
        .map(|offer| {
            let publication = catalog
                .publications
                .iter()
                .find(|publication| publication.offer_id == offer.offer_id);
            PortalProductOffer {
                offer_id: offer.offer_id.clone(),
                product_id: offer.product_id.clone(),
                product_kind: offer.product_kind.as_str().to_owned(),
                display_name: offer.display_name.clone(),
                quote_kind: offer.quote_kind.as_str().to_owned(),
                quote_target_kind: offer.quote_target_kind.as_str().to_owned(),
                quote_target_id: offer.quote_target_id.clone(),
                publication_id: publication.map(|item| item.publication_id.clone()),
                publication_kind: publication.map(|item| item.publication_kind.as_str().to_owned()),
                publication_status: publication.map(|item| item.status.as_str().to_owned()),
                publication_revision_id: publication
                    .map(|item| item.publication_revision_id.clone()),
                publication_version: publication.map(|item| item.publication_version),
                publication_source_kind: publication
                    .map(|item| item.publication_source_kind.clone()),
                publication_effective_from_ms: publication
                    .and_then(|item| item.publication_effective_from_ms),
                pricing_plan_id: offer.pricing_plan_id.clone(),
                pricing_plan_version: offer.pricing_plan_version,
                pricing_rate_id: offer.pricing_rate_id.clone(),
                pricing_metric_code: offer.pricing_metric_code.clone(),
                price_label: offer.price_label.clone(),
                source: offer.source.clone(),
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
    catalog_binding: PortalCommerceCatalogBinding,
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
        product_kind: portal_commerce_product_kind(target_kind).map(str::to_owned),
        quote_kind: portal_commerce_quote_kind(target_kind).to_owned(),
        target_id: target_id.to_owned(),
        target_name: target_name.to_owned(),
        product_id: catalog_binding.product_id,
        offer_id: catalog_binding.offer_id,
        publication_id: catalog_binding.publication_id,
        publication_kind: catalog_binding.publication_kind,
        publication_status: catalog_binding.publication_status,
        publication_revision_id: catalog_binding.publication_revision_id,
        publication_version: catalog_binding.publication_version,
        publication_source_kind: catalog_binding.publication_source_kind,
        publication_effective_from_ms: catalog_binding.publication_effective_from_ms,
        list_price_cents,
        payable_price_cents: payable_cents,
        list_price_label: format_quote_price_label(list_price_cents),
        payable_price_label: format_quote_price_label(payable_cents),
        granted_units,
        bonus_units,
        amount_cents: None,
        projected_remaining_units,
        pricing_plan_id: catalog_binding.pricing_plan_id,
        pricing_plan_version: catalog_binding.pricing_plan_version,
        pricing_rate_id: catalog_binding.pricing_rate_id,
        pricing_metric_code: catalog_binding.pricing_metric_code,
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
    catalog_binding: PortalCommerceCatalogBinding,
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
        catalog_binding,
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
    catalog_binding: PortalCommerceCatalogBinding,
) -> PortalCommerceQuote {
    let source = coupon.coupon.source.clone();
    let projected_remaining_units =
        current_remaining_units.map(|units| units.saturating_add(coupon.benefit.bonus_units));

    PortalCommerceQuote {
        target_kind: "coupon_redemption".to_owned(),
        product_kind: None,
        quote_kind: portal_commerce_quote_kind("coupon_redemption").to_owned(),
        target_id: coupon.coupon.code.clone(),
        target_name: coupon.coupon.code.clone(),
        product_id: catalog_binding.product_id,
        offer_id: catalog_binding.offer_id,
        publication_id: catalog_binding.publication_id,
        publication_kind: catalog_binding.publication_kind,
        publication_status: catalog_binding.publication_status,
        publication_revision_id: catalog_binding.publication_revision_id,
        publication_version: catalog_binding.publication_version,
        publication_source_kind: catalog_binding.publication_source_kind,
        publication_effective_from_ms: catalog_binding.publication_effective_from_ms,
        list_price_cents: 0,
        payable_price_cents: 0,
        list_price_label: "$0.00".to_owned(),
        payable_price_label: "$0.00".to_owned(),
        granted_units: 0,
        bonus_units: coupon.benefit.bonus_units,
        amount_cents: None,
        projected_remaining_units,
        pricing_plan_id: catalog_binding.pricing_plan_id,
        pricing_plan_version: catalog_binding.pricing_plan_version,
        pricing_rate_id: catalog_binding.pricing_rate_id,
        pricing_metric_code: catalog_binding.pricing_metric_code,
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

pub fn portal_commerce_product_kind(target_kind: &str) -> Option<&'static str> {
    match target_kind {
        "subscription_plan" => Some("subscription_plan"),
        "recharge_pack" => Some("recharge_pack"),
        "custom_recharge" => Some("custom_recharge"),
        _ => None,
    }
}

pub fn portal_commerce_quote_kind(target_kind: &str) -> &'static str {
    match target_kind {
        "coupon_redemption" => "coupon_redemption",
        _ => "product_purchase",
    }
}

pub fn portal_commerce_transaction_kind(target_kind: &str) -> &'static str {
    match target_kind {
        "coupon_redemption" => "coupon_redemption",
        _ => "product_purchase",
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

    #[test]
    fn stripe_payment_provider_sources_do_not_keep_dead_warning_fields() {
        let payment_provider_mod = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/payment_provider/mod.rs"),
        )
        .expect("read payment_provider/mod.rs");
        let stripe_provider = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/payment_provider/stripe.rs"),
        )
        .expect("read payment_provider/stripe.rs");

        for snippet in [
            "pub publishable_key: Option<String>,",
            "pub(crate) struct StripeRefundCreateResult {\n    pub provider_refund_id: String,\n    pub status: String,\n    pub amount_minor: u64,",
            "pub(crate) struct StripeCheckoutSessionSnapshot {\n    pub status: String,\n    pub payment_status: Option<String>,\n    pub amount_total_minor: Option<u64>,\n    pub currency_code: Option<String>,",
            "pub(crate) struct StripeRefundSnapshot {\n    pub status: String,\n    pub amount_minor: u64,\n    pub currency_code: String,",
            "event_type: String,",
            "pub(crate) fn event_type(&self) -> &str {",
        ] {
            let source = if snippet == "pub publishable_key: Option<String>," {
                &payment_provider_mod
            } else {
                &stripe_provider
            };
            assert!(
                !source.contains(snippet),
                "stripe payment provider source should not keep dead warning snippet `{snippet}`",
            );
        }
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
