#![allow(clippy::too_many_arguments)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{
    gateway_auth_subject_from_request_context, hash_gateway_api_key, GatewayRequestContext,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountRecord, AccountStatus, AccountType,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRestrictionSpec, CouponTemplateRecord,
    CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord, MarketingCampaignStatus,
    MarketingSubjectScope,
};
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

fn gateway_request_context(
    tenant_id: &str,
    project_id: &str,
    api_key: &str,
) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: tenant_id.to_owned(),
        project_id: project_id.to_owned(),
        environment: "live".to_owned(),
        api_key_hash: hash_gateway_api_key(api_key),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }
}

async fn seed_gateway_project_account(
    store: &SqliteAdminStore,
    tenant_id: &str,
    project_id: &str,
    api_key: &str,
) -> AccountRecord {
    let subject = gateway_auth_subject_from_request_context(&gateway_request_context(
        tenant_id, project_id, api_key,
    ));
    let account = AccountRecord::new(
        7701,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        AccountType::Primary,
    )
    .with_status(AccountStatus::Active)
    .with_currency_code("USD")
    .with_credit_unit_code("credit")
    .with_created_at_ms(1_710_000_000_000)
    .with_updated_at_ms(1_710_000_000_000);
    let balance_lot = AccountBenefitLotRecord::new(
        8801,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(10.0)
    .with_remaining_quantity(10.0)
    .with_held_quantity(0.0)
    .with_status(AccountBenefitLotStatus::Active)
    .with_created_at_ms(1_710_000_000_001)
    .with_updated_at_ms(1_710_000_000_001);

    store.insert_account_record(&account).await.unwrap();
    AccountKernelStore::insert_account_benefit_lot(store, &balance_lot)
        .await
        .unwrap();
    account
}

async fn seed_marketing_account_entitlement_coupon(store: &SqliteAdminStore) {
    let template = CouponTemplateRecord::new(
        "template_launch20",
        "launch20",
        MarketingBenefitKind::GrantUnits,
    )
    .with_display_name("Launch 20")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_restriction(
        CouponRestrictionSpec::new(MarketingSubjectScope::Project)
            .with_eligible_target_kinds(vec!["coupon_redemption".to_owned()]),
    )
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::GrantUnits).with_grant_units(Some(300)),
    )
    .with_created_at_ms(1_710_000_000_000)
    .with_updated_at_ms(1_710_000_000_000);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new("campaign_launch20", "template_launch20")
        .with_display_name("Launch Campaign")
        .with_status(MarketingCampaignStatus::Active)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let budget = CampaignBudgetRecord::new("budget_launch20", "campaign_launch20")
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(0)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_campaign_budget_record(&budget).await.unwrap();

    let code = CouponCodeRecord::new("code_launch20", "template_launch20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_coupon_code_record(&code).await.unwrap();
}

async fn seed_custom_marketing_coupon(
    store: &SqliteAdminStore,
    template_id: &str,
    campaign_id: &str,
    budget_id: &str,
    code_id: &str,
    code_value: &str,
    benefit_kind: MarketingBenefitKind,
    restriction: CouponRestrictionSpec,
) {
    let benefit = match benefit_kind {
        MarketingBenefitKind::PercentageOff => {
            CouponBenefitSpec::new(benefit_kind).with_discount_percent(Some(20))
        }
        MarketingBenefitKind::FixedAmountOff => {
            CouponBenefitSpec::new(benefit_kind).with_discount_amount_minor(Some(2_000))
        }
        MarketingBenefitKind::GrantUnits => {
            CouponBenefitSpec::new(benefit_kind).with_grant_units(Some(300))
        }
    };

    let template =
        CouponTemplateRecord::new(template_id, code_value.to_ascii_lowercase(), benefit_kind)
            .with_display_name(format!("{code_value} template"))
            .with_status(CouponTemplateStatus::Active)
            .with_distribution_kind(CouponDistributionKind::UniqueCode)
            .with_restriction(restriction)
            .with_benefit(benefit)
            .with_created_at_ms(1_710_000_000_000)
            .with_updated_at_ms(1_710_000_000_000);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new(campaign_id, template_id)
        .with_display_name(format!("{code_value} campaign"))
        .with_status(MarketingCampaignStatus::Active)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let budget = CampaignBudgetRecord::new(budget_id, campaign_id)
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(5_000)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_campaign_budget_record(&budget).await.unwrap();

    let code = CouponCodeRecord::new(code_id, template_id, code_value)
        .with_status(CouponCodeStatus::Available)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_coupon_code_record(&code).await.unwrap();
}

#[tokio::test]
async fn public_market_routes_expose_products_offers_and_quote_pricing() {
    let tenant_id = "tenant-public-market";
    let project_id = "project-public-market";
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let products = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/market/products")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(products.status(), StatusCode::OK);
    let products_json = read_json(products).await;
    assert!(products_json["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["product_kind"] == "subscription_plan"
                && item["target_id"] == "growth"
                && item["display_name"] == "Growth"
        }));

    let offers = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/market/offers")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(offers.status(), StatusCode::OK);
    let offers_json = read_json(offers).await;
    assert!(offers_json["items"].as_array().unwrap().iter().any(|item| {
        item["quote_target_kind"] == "recharge_pack"
            && item["quote_target_id"] == "pack-100k"
            && item["quote_kind"] == "product_purchase"
            && item["publication_status"] == "published"
            && item["pricing_rate_id"] == "pricing_rate:recharge_pack:pack-100k:credit.prepaid_pack"
    }));

    let quote = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/market/quotes")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(quote.status(), StatusCode::OK);
    let quote_json = read_json(quote).await;
    assert_eq!(quote_json["target_kind"], "recharge_pack");
    assert_eq!(quote_json["product_kind"], "recharge_pack");
    assert_eq!(quote_json["quote_kind"], "product_purchase");
    assert_eq!(quote_json["target_name"], "Boost 100k");
    assert_eq!(quote_json["payable_price_label"], "$40.00");
    assert_eq!(quote_json["granted_units"], 100000);
    assert_eq!(quote_json["projected_remaining_units"], 105000);
    assert_eq!(quote_json["product_id"], "product:recharge_pack:pack-100k");
    assert_eq!(quote_json["offer_id"], "offer:recharge_pack:pack-100k");
}

#[tokio::test]
async fn public_coupon_and_commercial_routes_expose_coupon_semantics_and_account_arrival() {
    let tenant_id = "tenant-public-coupon";
    let project_id = "project-public-coupon";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account = seed_gateway_project_account(&store, tenant_id, project_id, &api_key).await;
    seed_marketing_account_entitlement_coupon(&store).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let account_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/commercial/account")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_json = read_json(account_response).await;
    assert_eq!(account_json["account"]["account_id"], account.account_id);
    assert_eq!(account_json["balance"]["available_balance"], 10.0);

    let validation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/marketing/coupons/validate")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"coupon_redemption\",\"order_amount_minor\":0,\"reserve_amount_minor\":0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(validation.status(), StatusCode::OK);
    let validation_json = read_json(validation).await;
    assert_eq!(validation_json["decision"]["eligible"], true);
    assert_eq!(validation_json["template"]["template_key"], "launch20");
    assert_eq!(
        validation_json["effect"]["effect_kind"],
        "account_entitlement"
    );
    assert_eq!(validation_json["effect"]["grant_units"], 300);
    assert_eq!(
        validation_json["applicability"]["target_kinds"][0],
        "coupon_redemption"
    );

    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/marketing/coupons/reserve")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"coupon_redemption\",\"reserve_amount_minor\":0,\"ttl_ms\":300000,\"idempotency_key\":\"public_coupon_reserve_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved.status(), StatusCode::CREATED);
    let reserved_json = read_json(reserved).await;
    let reservation_id = reserved_json["reservation"]["coupon_reservation_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        reserved_json["reservation"]["reservation_status"],
        "reserved"
    );
    assert_eq!(
        reserved_json["effect"]["effect_kind"],
        "account_entitlement"
    );

    let confirmed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/marketing/coupons/confirm")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"coupon_reservation_id\":\"{reservation_id}\",\"subsidy_amount_minor\":0,\"order_id\":\"order_public_reward\",\"payment_event_id\":\"payment_public_reward\",\"idempotency_key\":\"public_coupon_confirm_1\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(confirmed.status(), StatusCode::OK);
    let confirmed_json = read_json(confirmed).await;
    let redemption_id = confirmed_json["redemption"]["coupon_redemption_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        confirmed_json["redemption"]["redemption_status"],
        "redeemed"
    );
    assert_eq!(
        confirmed_json["effect"]["effect_kind"],
        "account_entitlement"
    );
    assert_eq!(confirmed_json["code"]["status"], "redeemed");

    let benefit_lot = AccountBenefitLotRecord::new(
        9901,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Order)
    .with_source_id(Some(9902))
    .with_scope_json(Some(format!(
        "{{\"order_id\":\"order_public_reward\",\"project_id\":\"{project_id}\",\"target_kind\":\"coupon_redemption\"}}"
    )))
    .with_original_quantity(300.0)
    .with_remaining_quantity(300.0)
    .with_held_quantity(0.0)
    .with_status(AccountBenefitLotStatus::Active)
    .with_issued_at_ms(1_710_000_000_350)
    .with_expires_at_ms(Some(1_720_000_000_000))
    .with_created_at_ms(1_710_000_000_350)
    .with_updated_at_ms(1_710_000_000_350);
    AccountKernelStore::insert_account_benefit_lot(&store, &benefit_lot)
        .await
        .unwrap();

    let benefit_lots = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/commercial/account/benefit-lots")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(benefit_lots.status(), StatusCode::OK);
    let benefit_lots_json = read_json(benefit_lots).await;
    assert_eq!(
        benefit_lots_json["account"]["account_id"],
        account.account_id
    );
    assert!(benefit_lots_json["benefit_lots"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["lot_id"] == benefit_lot.lot_id
                && item["source_type"] == "order"
                && item["scope_order_id"] == "order_public_reward"
        }));

    let rolled_back = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/marketing/coupons/rollback")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"coupon_redemption_id\":\"{redemption_id}\",\"rollback_type\":\"refund\",\"restored_budget_minor\":0,\"restored_inventory_count\":1,\"idempotency_key\":\"public_coupon_rollback_1\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rolled_back.status(), StatusCode::OK);
    let rolled_back_json = read_json(rolled_back).await;
    assert_eq!(
        rolled_back_json["redemption"]["redemption_status"],
        "rolled_back"
    );
    assert_eq!(rolled_back_json["rollback"]["rollback_status"], "completed");
    assert_eq!(
        rolled_back_json["effect"]["effect_kind"],
        "account_entitlement"
    );
    assert_eq!(rolled_back_json["rollback"]["rollback_type"], "refund");
}

#[tokio::test]
async fn public_commercial_benefit_lots_route_is_cursor_paginated() {
    let tenant_id = "tenant-public-benefit-lots-page";
    let project_id = "project-public-benefit-lots-page";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account = seed_gateway_project_account(&store, tenant_id, project_id, &api_key).await;
    let other_account = AccountRecord::new(7702, 1001, 2002, 9902, AccountType::Primary)
        .with_status(AccountStatus::Active)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(1_710_000_000_100)
        .with_updated_at_ms(1_710_000_000_100);
    store.insert_account_record(&other_account).await.unwrap();
    AccountKernelStore::insert_account_benefit_lot(
        &store,
        &AccountBenefitLotRecord::new(
            7700,
            other_account.tenant_id,
            other_account.organization_id,
            other_account.account_id,
            other_account.user_id,
            AccountBenefitType::CashCredit,
        )
        .with_source_type(AccountBenefitSourceType::Recharge)
        .with_original_quantity(1.0)
        .with_remaining_quantity(1.0)
        .with_held_quantity(0.0)
        .with_status(AccountBenefitLotStatus::Active)
        .with_created_at_ms(1_710_000_000_101)
        .with_updated_at_ms(1_710_000_000_101),
    )
    .await
    .unwrap();

    for (lot_id, quantity, created_at_ms) in [
        (9901_u64, 100.0_f64, 1_710_000_000_301_u64),
        (9902_u64, 200.0_f64, 1_710_000_000_302_u64),
        (9903_u64, 300.0_f64, 1_710_000_000_303_u64),
    ] {
        AccountKernelStore::insert_account_benefit_lot(
            &store,
            &AccountBenefitLotRecord::new(
                lot_id,
                account.tenant_id,
                account.organization_id,
                account.account_id,
                account.user_id,
                AccountBenefitType::CashCredit,
            )
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(quantity)
            .with_remaining_quantity(quantity)
            .with_held_quantity(0.0)
            .with_status(AccountBenefitLotStatus::Active)
            .with_created_at_ms(created_at_ms)
            .with_updated_at_ms(created_at_ms),
        )
        .await
        .unwrap();
    }

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);
    let first_page = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/commercial/account/benefit-lots?limit=2")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_page.status(), StatusCode::OK);
    let first_page_json = read_json(first_page).await;
    assert_eq!(first_page_json["account"]["account_id"], account.account_id);
    assert_eq!(first_page_json["page"]["limit"], 2);
    assert_eq!(first_page_json["page"]["after_lot_id"], Value::Null);
    assert_eq!(first_page_json["page"]["returned_count"], 2);
    assert_eq!(first_page_json["page"]["has_more"], true);
    assert_eq!(first_page_json["page"]["next_after_lot_id"], 9901);
    assert_eq!(
        first_page_json["benefit_lots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["lot_id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![8801, 9901]
    );
    assert!(first_page_json["benefit_lots"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["lot_id"] != 7700));

    let second_page = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/commercial/account/benefit-lots?limit=2&after_lot_id=9901")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_page.status(), StatusCode::OK);
    let second_page_json = read_json(second_page).await;
    assert_eq!(second_page_json["page"]["limit"], 2);
    assert_eq!(second_page_json["page"]["after_lot_id"], 9901);
    assert_eq!(second_page_json["page"]["returned_count"], 2);
    assert_eq!(second_page_json["page"]["has_more"], false);
    assert_eq!(second_page_json["page"]["next_after_lot_id"], Value::Null);
    assert_eq!(
        second_page_json["benefit_lots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["lot_id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![9902, 9903]
    );
}

#[tokio::test]
async fn public_coupon_reservation_uses_order_amount_minor_instead_of_reserve_amount_minor() {
    let tenant_id = "tenant-public-min-order";
    let project_id = "project-public-min-order";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    seed_custom_marketing_coupon(
        &store,
        "template_public_min_order",
        "campaign_public_min_order",
        "budget_public_min_order",
        "code_public_min_order",
        "PUBLICMIN20",
        MarketingBenefitKind::PercentageOff,
        CouponRestrictionSpec::new(MarketingSubjectScope::Project)
            .with_min_order_amount_minor(Some(1_000))
            .with_eligible_target_kinds(vec!["coupon_redemption".to_owned()]),
    )
    .await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);
    let reserved = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/marketing/coupons/reserve")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"PUBLICMIN20\",\"subject_scope\":\"project\",\"target_kind\":\"coupon_redemption\",\"order_amount_minor\":5000,\"reserve_amount_minor\":200,\"ttl_ms\":300000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reserved.status(), StatusCode::CREATED);
}
