use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_domain_marketing::{
    CouponBenefitKind, CouponBenefitRuleRecord, CouponCodeBatchRecord, CouponCodeBatchStatus,
    CouponCodeGenerationMode, CouponCodeKind, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponTemplateRecord, CouponTemplateStatus,
};
use serde_json::Value;
use serial_test::serial;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"hunter2!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);
    read_json(register_response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn portal_workspace(app: axum::Router, token: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await
}

fn hash_coupon_code_for_lookup(value: &str) -> String {
    let normalized = value.trim().to_ascii_uppercase();
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn payment_callback_secret_env_guard(value: &str) -> PaymentCallbackSecretEnvGuard {
    let previous = std::env::var("SDKWORK_PORTAL_PAYMENT_CALLBACK_SECRET").ok();
    std::env::set_var("SDKWORK_PORTAL_PAYMENT_CALLBACK_SECRET", value);
    PaymentCallbackSecretEnvGuard { previous }
}

struct PaymentCallbackSecretEnvGuard {
    previous: Option<String>,
}

impl Drop for PaymentCallbackSecretEnvGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_PORTAL_PAYMENT_CALLBACK_SECRET", value),
            None => std::env::remove_var("SDKWORK_PORTAL_PAYMENT_CALLBACK_SECRET"),
        }
    }
}

#[tokio::test]
async fn portal_commerce_catalog_exposes_plans_packs_and_active_coupons() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_inactive")
    .bind("INACTIVE10")
    .bind("10% inactive discount")
    .bind("internal")
    .bind(40_i64)
    .bind(0_i64)
    .bind("Inactive campaign")
    .bind("2026-05-31")
    .bind(1_710_000_002_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["plans"].as_array().unwrap().len(), 3);
    assert_eq!(json["packs"].as_array().unwrap().len(), 3);
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|coupon| coupon["code"] == "SPRING20"));
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .all(|coupon| coupon["code"] != "INACTIVE10"));
}

#[tokio::test]
async fn portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["recharge_options"].as_array().unwrap().len(), 4);
    assert_eq!(json["recharge_options"][0]["amount_cents"], 1000);
    assert_eq!(json["recharge_options"][0]["amount_label"], "$10.00");
    assert_eq!(json["recharge_options"][0]["granted_units"], 25000);
    assert_eq!(
        json["recharge_options"][1]["effective_ratio_label"],
        "2,800 units / $1"
    );
    assert_eq!(json["custom_recharge_policy"]["enabled"], true);
    assert_eq!(json["custom_recharge_policy"]["min_amount_cents"], 1000);
    assert_eq!(json["custom_recharge_policy"]["step_amount_cents"], 500);
    assert_eq!(
        json["custom_recharge_policy"]["suggested_amount_cents"],
        5000
    );
    assert_eq!(
        json["custom_recharge_policy"]["rules"][1]["effective_ratio_label"],
        "2,800 units / $1"
    );
}

#[tokio::test]
async fn portal_commerce_catalog_requires_authentication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn portal_commerce_quote_prices_recharge_and_coupon_redemption() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::OK);
    let recharge_json = read_json(recharge_response).await;
    assert_eq!(recharge_json["target_kind"], "recharge_pack");
    assert_eq!(recharge_json["target_name"], "Boost 100k");
    assert_eq!(recharge_json["list_price_label"], "$40.00");
    assert_eq!(recharge_json["payable_price_label"], "$32.00");
    assert_eq!(recharge_json["granted_units"], 100000);
    assert_eq!(recharge_json["projected_remaining_units"], 105000);
    assert_eq!(recharge_json["applied_coupon"]["code"], "SPRING20");

    let coupon_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"coupon_redemption\",\"target_id\":\"WELCOME100\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(coupon_response.status(), StatusCode::OK);
    let coupon_json = read_json(coupon_response).await;
    assert_eq!(coupon_json["target_kind"], "coupon_redemption");
    assert_eq!(coupon_json["target_name"], "WELCOME100");
    assert_eq!(coupon_json["payable_price_label"], "$0.00");
    assert_eq!(coupon_json["bonus_units"], 100);
    assert_eq!(coupon_json["projected_remaining_units"], 5100);
}

#[tokio::test]
async fn portal_commerce_quote_supports_canonical_marketing_discount_codes() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());

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

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING15\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["target_kind"], "recharge_pack");
    assert_eq!(json["payable_price_label"], "$34.00");
    assert_eq!(json["projected_remaining_units"], 105000);
    assert_eq!(json["applied_coupon"]["code"], "SPRING15");
    assert_eq!(json["applied_coupon"]["source"], "marketing");
}

#[tokio::test]
async fn portal_commerce_zero_pay_marketing_discount_fulfills_immediately() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    let template = CouponTemplateRecord::new(
        101,
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
        201,
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
        hash_coupon_code_for_lookup("FREE100"),
        CouponCodeKind::SingleUseUnique,
        1_710_000_030,
    )
    .with_status(CouponCodeStatus::Claimed)
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some(format!("1001:2002:{user_id}")))
    .with_claimed_at_ms(Some(1_710_000_031))
    .with_updated_at_ms(1_710_000_103);
    store.insert_coupon_code_record(&code).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"FREE100\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = read_json(response).await;
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["user_id"], user_id);
    assert_eq!(json["status"], "fulfilled");
    assert_eq!(json["payable_price_label"], "$0.00");

    let order_id = json["order_id"].as_str().unwrap().to_owned();
    let redemptions = store.list_coupon_redemption_records().await.unwrap();
    assert_eq!(redemptions.len(), 1);
    assert_eq!(redemptions[0].subject_type, "user");
    assert_eq!(redemptions[0].subject_id, user_id);
    assert_eq!(
        redemptions[0].project_id.as_deref(),
        Some(project_id.as_str())
    );
    assert_eq!(redemptions[0].order_id.as_deref(), Some(order_id.as_str()));

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash(&hash_coupon_code_for_lookup("FREE100"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_code.status, CouponCodeStatus::Redeemed);
}

#[tokio::test]
#[serial]
async fn portal_commerce_provider_callback_writes_payment_order_id_to_marketing_redemption() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let _callback_secret_guard = payment_callback_secret_env_guard("test-payment-callback-secret");
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let template = CouponTemplateRecord::new(
        102,
        1001,
        2002,
        "paid-marketing-discount",
        "Paid marketing recharge discount",
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
        202,
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
        302,
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
        402,
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
    .with_claim_subject_id(Some(format!("1001:2002:{user_id}")))
    .with_claimed_at_ms(Some(1_710_000_031))
    .with_updated_at_ms(1_710_000_103);
    store.insert_coupon_code_record(&code).await.unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"PAID15\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["status"], "pending_payment");
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();

    let settle_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/internal/commerce/orders/{order_id}/payment-events"
                ))
                .header(
                    "x-sdkwork-payment-callback-secret",
                    "test-payment-callback-secret",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"event_type\":\"settled\",\"payment_order_id\":\"stripe_pi_123\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

    let redemptions = store.list_coupon_redemption_records().await.unwrap();
    assert_eq!(redemptions.len(), 1);
    assert_eq!(redemptions[0].subject_id, user_id);
    assert_eq!(redemptions[0].order_id.as_deref(), Some(order_id.as_str()));
    assert_eq!(
        redemptions[0].payment_order_id.as_deref(),
        Some("stripe_pi_123")
    );
}

#[tokio::test]
async fn portal_marketing_redemptions_return_current_user_history_and_summary() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    store
        .insert_coupon_redemption_record(
            &sdkwork_api_domain_marketing::CouponRedemptionRecord::new(
                801,
                1001,
                2002,
                501,
                101,
                Some(301),
                "user",
                &user_id,
                1_710_000_100,
            )
            .with_status(sdkwork_api_domain_marketing::CouponRedemptionStatus::Fulfilled)
            .with_project_id(Some(project_id.clone()))
            .with_order_id(Some("order_alpha".to_owned()))
            .with_payment_order_id(Some("stripe_pi_alpha".to_owned()))
            .with_subsidy_amount(Some(8.5))
            .with_currency_code(Some("USD".to_owned()))
            .with_updated_at_ms(1_710_000_200),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &sdkwork_api_domain_marketing::CouponRedemptionRecord::new(
                802,
                1001,
                2002,
                502,
                101,
                Some(301),
                "user",
                &user_id,
                1_710_000_120,
            )
            .with_status(sdkwork_api_domain_marketing::CouponRedemptionStatus::Failed)
            .with_project_id(Some(project_id.clone()))
            .with_order_id(Some("order_beta".to_owned()))
            .with_updated_at_ms(1_710_000_220),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &sdkwork_api_domain_marketing::CouponRedemptionRecord::new(
                803,
                1001,
                2002,
                503,
                101,
                Some(301),
                "user",
                "other_user",
                1_710_000_130,
            )
            .with_status(sdkwork_api_domain_marketing::CouponRedemptionStatus::Fulfilled)
            .with_project_id(Some(project_id))
            .with_order_id(Some("order_gamma".to_owned()))
            .with_updated_at_ms(1_710_000_230),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/redemptions")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["coupon_redemption_id"], 802);
    assert_eq!(items[1]["coupon_redemption_id"], 801);
    assert_eq!(json["summary"]["total_count"], 2);
    assert_eq!(json["summary"]["fulfilled_count"], 1);
    assert_eq!(json["summary"]["failed_count"], 1);
    assert_eq!(json["summary"]["payment_linked_count"], 1);
    assert_eq!(json["summary"]["subsidized_count"], 1);
    assert_eq!(json["summary"]["total_subsidy_amount"], 8.5);
}

#[tokio::test]
async fn portal_marketing_codes_return_current_user_wallet_and_latest_redemption() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    store
        .insert_coupon_code_record(
            &sdkwork_api_domain_marketing::CouponCodeRecord::new(
                901,
                1001,
                2002,
                301,
                101,
                Some(401),
                hash_coupon_code_for_lookup("WALLET15"),
                sdkwork_api_domain_marketing::CouponCodeKind::SingleUseUnique,
                1_710_000_100,
            )
            .with_status(sdkwork_api_domain_marketing::CouponCodeStatus::Claimed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some(format!("1001:2002:{user_id}")))
            .with_display_code_prefix(Some("WAL".to_owned()))
            .with_display_code_suffix(Some("15".to_owned()))
            .with_updated_at_ms(1_710_000_200),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &sdkwork_api_domain_marketing::CouponRedemptionRecord::new(
                951,
                1001,
                2002,
                901,
                101,
                Some(401),
                "user",
                &user_id,
                1_710_000_210,
            )
            .with_status(sdkwork_api_domain_marketing::CouponRedemptionStatus::Pending)
            .with_order_id(Some("order_wallet".to_owned()))
            .with_updated_at_ms(1_710_000_220),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_record(
            &sdkwork_api_domain_marketing::CouponCodeRecord::new(
                902,
                1001,
                2002,
                301,
                101,
                Some(401),
                hash_coupon_code_for_lookup("DONE20"),
                sdkwork_api_domain_marketing::CouponCodeKind::SingleUseUnique,
                1_710_000_110,
            )
            .with_status(sdkwork_api_domain_marketing::CouponCodeStatus::Redeemed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some(format!("1001:2002:{user_id}")))
            .with_display_code_prefix(Some("DON".to_owned()))
            .with_display_code_suffix(Some("20".to_owned()))
            .with_updated_at_ms(1_710_000_230),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &sdkwork_api_domain_marketing::CouponRedemptionRecord::new(
                952,
                1001,
                2002,
                902,
                101,
                Some(401),
                "user",
                &user_id,
                1_710_000_231,
            )
            .with_status(sdkwork_api_domain_marketing::CouponRedemptionStatus::Fulfilled)
            .with_order_id(Some("order_done".to_owned()))
            .with_updated_at_ms(1_710_000_240),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_record(
            &sdkwork_api_domain_marketing::CouponCodeRecord::new(
                903,
                1001,
                2002,
                301,
                101,
                Some(401),
                hash_coupon_code_for_lookup("OTHER30"),
                sdkwork_api_domain_marketing::CouponCodeKind::SingleUseUnique,
                1_710_000_120,
            )
            .with_status(sdkwork_api_domain_marketing::CouponCodeStatus::Claimed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some("1001:2002:other_user".to_owned()))
            .with_updated_at_ms(1_710_000_250),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["code"]["coupon_code_id"], 902);
    assert_eq!(items[0]["latest_redemption"]["status"], "fulfilled");
    assert_eq!(items[1]["code"]["coupon_code_id"], 901);
    assert_eq!(items[1]["latest_redemption"]["status"], "pending");
    assert_eq!(json["summary"]["total_count"], 2);
    assert_eq!(json["summary"]["claimed_count"], 1);
    assert_eq!(json["summary"]["redeemed_count"], 1);
    assert_eq!(json["summary"]["reserved_count"], 1);
}

#[tokio::test]
async fn portal_marketing_codes_support_status_filter() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    for (coupon_code_id, status, lookup_hash) in [
        (
            911_u64,
            sdkwork_api_domain_marketing::CouponCodeStatus::Claimed,
            hash_coupon_code_for_lookup("FILTER15"),
        ),
        (
            912_u64,
            sdkwork_api_domain_marketing::CouponCodeStatus::Redeemed,
            hash_coupon_code_for_lookup("FILTER20"),
        ),
    ] {
        store
            .insert_coupon_code_record(
                &sdkwork_api_domain_marketing::CouponCodeRecord::new(
                    coupon_code_id,
                    1001,
                    2002,
                    301,
                    101,
                    Some(401),
                    lookup_hash,
                    sdkwork_api_domain_marketing::CouponCodeKind::SingleUseUnique,
                    1_710_000_100 + coupon_code_id,
                )
                .with_status(status)
                .with_claim_subject_type(Some("user".to_owned()))
                .with_claim_subject_id(Some(format!("1001:2002:{user_id}")))
                .with_updated_at_ms(1_710_000_300 + coupon_code_id),
            )
            .await
            .unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/codes?status=claimed")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["code"]["coupon_code_id"], 911);
    assert_eq!(json["summary"]["total_count"], 1);
    assert_eq!(json["summary"]["claimed_count"], 1);
    assert_eq!(json["summary"]["redeemed_count"], 0);
}

#[tokio::test]
async fn portal_commerce_quote_and_order_support_custom_recharge_but_reject_portal_self_settlement()
{
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let quote_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"custom_recharge\",\"target_id\":\"custom\",\"custom_amount_cents\":5000,\"current_remaining_units\":260}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(quote_response.status(), StatusCode::OK);
    let quote_json = read_json(quote_response).await;
    assert_eq!(quote_json["target_kind"], "custom_recharge");
    assert_eq!(quote_json["target_id"], "custom-5000");
    assert_eq!(quote_json["target_name"], "Custom recharge");
    assert_eq!(quote_json["amount_cents"], 5000);
    assert_eq!(quote_json["list_price_label"], "$50.00");
    assert_eq!(quote_json["payable_price_label"], "$50.00");
    assert_eq!(quote_json["granted_units"], 140000);
    assert_eq!(quote_json["projected_remaining_units"], 140260);
    assert_eq!(quote_json["pricing_rule_label"], "Tiered custom recharge");
    assert_eq!(quote_json["effective_ratio_label"], "2,800 units / $1");

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"custom_recharge\",\"target_id\":\"custom\",\"custom_amount_cents\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["target_kind"], "custom_recharge");
    assert_eq!(create_json["target_id"], "custom-5000");
    assert_eq!(create_json["target_name"], "Custom recharge");
    assert_eq!(create_json["payable_price_label"], "$50.00");
    assert_eq!(create_json["granted_units"], 140000);
    assert_eq!(create_json["status"], "pending_payment");

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/settle"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::FORBIDDEN);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        "paid orders must be settled through the payment callback flow"
    );

    let billing_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 260);
}

#[tokio::test]
async fn portal_commerce_orders_queue_paid_checkout_and_fulfill_coupon_redemption() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::CREATED);
    let recharge_json = read_json(recharge_response).await;
    assert_eq!(recharge_json["project_id"], project_id);
    assert_eq!(recharge_json["user_id"], user_id);
    assert_eq!(recharge_json["target_kind"], "recharge_pack");
    assert_eq!(recharge_json["target_name"], "Boost 100k");
    assert_eq!(recharge_json["payable_price_label"], "$32.00");
    assert_eq!(recharge_json["status"], "pending_payment");

    let billing_after_recharge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_after_recharge.status(), StatusCode::OK);
    let billing_after_recharge_json = read_json(billing_after_recharge).await;
    assert_eq!(billing_after_recharge_json["remaining_units"], 260);

    let coupon_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"coupon_redemption\",\"target_id\":\"WELCOME100\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(coupon_response.status(), StatusCode::CREATED);
    let coupon_json = read_json(coupon_response).await;
    assert_eq!(coupon_json["target_kind"], "coupon_redemption");
    assert_eq!(coupon_json["bonus_units"], 100);
    assert_eq!(coupon_json["status"], "fulfilled");

    let billing_after_coupon = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_after_coupon.status(), StatusCode::OK);
    let billing_after_coupon_json = read_json(billing_after_coupon).await;
    assert_eq!(billing_after_coupon_json["remaining_units"], 360);

    let history_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(history_response.status(), StatusCode::OK);
    let history_json = read_json(history_response).await;
    assert_eq!(history_json.as_array().unwrap().len(), 2);
    assert_eq!(history_json[0]["status"], "fulfilled");
    assert_eq!(history_json[0]["project_id"], project_id);
    assert_eq!(history_json[1]["status"], "pending_payment");
    assert_eq!(history_json[1]["project_id"], project_id);
}

#[tokio::test]
async fn portal_commerce_pending_recharge_requires_payment_callback_or_cancel() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::CREATED);
    let recharge_json = read_json(recharge_response).await;
    let settled_order_id = recharge_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(recharge_json["status"], "pending_payment");

    let checkout_session_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{settled_order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(checkout_session_response.status(), StatusCode::OK);
    let checkout_session_json = read_json(checkout_session_response).await;
    assert_eq!(checkout_session_json["order_id"], settled_order_id);
    assert_eq!(checkout_session_json["order_status"], "pending_payment");
    assert_eq!(checkout_session_json["session_status"], "open");
    assert!(checkout_session_json["reference"]
        .as_str()
        .unwrap()
        .starts_with("PAY-"));
    assert!(!checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["action"] == "settle_order"));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["action"] == "provider_handoff"));

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{settled_order_id}/settle"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::FORBIDDEN);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        "paid orders must be settled through the payment callback flow"
    );

    let billing_after_settle = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_after_settle.status(), StatusCode::OK);
    let billing_after_settle_json = read_json(billing_after_settle).await;
    assert_eq!(billing_after_settle_json["remaining_units"], 260);

    let cancel_create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-500k\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_create_response.status(), StatusCode::CREATED);
    let cancel_create_json = read_json(cancel_create_response).await;
    let canceled_order_id = cancel_create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(cancel_create_json["status"], "pending_payment");

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{canceled_order_id}/cancel"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["order_id"], canceled_order_id);
    assert_eq!(cancel_json["status"], "canceled");

    let billing_after_cancel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_after_cancel.status(), StatusCode::OK);
    let billing_after_cancel_json = read_json(billing_after_cancel).await;
    assert_eq!(billing_after_cancel_json["remaining_units"], 260);
}

#[tokio::test]
async fn portal_commerce_subscription_checkout_requires_payment_callback_before_membership_activation(
) {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let subscription_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(subscription_response.status(), StatusCode::CREATED);
    let subscription_json = read_json(subscription_response).await;
    assert_eq!(subscription_json["project_id"], project_id);
    assert_eq!(subscription_json["user_id"], user_id);
    assert_eq!(subscription_json["target_kind"], "subscription_plan");
    assert_eq!(subscription_json["target_name"], "Growth");
    assert_eq!(subscription_json["payable_price_label"], "$79.00");
    assert_eq!(subscription_json["status"], "pending_payment");

    let billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 260);

    let membership_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/membership")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(membership_response.status(), StatusCode::OK);
    let membership_json = read_json(membership_response).await;
    assert!(membership_json.is_null());

    let order_id = subscription_json["order_id"].as_str().unwrap();
    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/settle"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::FORBIDDEN);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        "paid orders must be settled through the payment callback flow"
    );

    let settled_billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settled_billing_response.status(), StatusCode::OK);
    let settled_billing_json = read_json(settled_billing_response).await;
    assert_eq!(settled_billing_json["remaining_units"], 260);

    let settled_membership_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/membership")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settled_membership_response.status(), StatusCode::OK);
    let membership_json = read_json(settled_membership_response).await;
    assert!(membership_json.is_null());
}

#[tokio::test]
async fn portal_commerce_payment_events_reject_direct_portal_settlement_events() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();

    let settle_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"settled\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::FORBIDDEN);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        "payment provider callbacks must use the internal callback endpoint"
    );
}

#[tokio::test]
#[serial]
async fn portal_commerce_provider_callbacks_can_fail_checkout_and_block_invalid_recovery() {
    let pool = memory_pool().await;
    let _callback_secret_guard = payment_callback_secret_env_guard("test-payment-callback-secret");
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["status"], "pending_payment");

    let failed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/internal/commerce/orders/{order_id}/payment-events"
                ))
                .header(
                    "x-sdkwork-payment-callback-secret",
                    "test-payment-callback-secret",
                )
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"failed\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(failed_response.status(), StatusCode::OK);
    let failed_json = read_json(failed_response).await;
    assert_eq!(failed_json["order_id"], order_id);
    assert_eq!(failed_json["status"], "failed");

    let checkout_session_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(checkout_session_response.status(), StatusCode::OK);
    let checkout_session_json = read_json(checkout_session_response).await;
    assert_eq!(checkout_session_json["order_status"], "failed");
    assert_eq!(checkout_session_json["session_status"], "failed");
    assert_eq!(
        checkout_session_json["methods"].as_array().unwrap().len(),
        0
    );

    let billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 260);

    let invalid_recovery_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/internal/commerce/orders/{order_id}/payment-events"
                ))
                .header(
                    "x-sdkwork-payment-callback-secret",
                    "test-payment-callback-secret",
                )
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"settled\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid_recovery_response.status(), StatusCode::CONFLICT);
    let invalid_recovery_json = read_json(invalid_recovery_response).await;
    assert_eq!(
        invalid_recovery_json["error"]["message"],
        format!("order {order_id} cannot be settled from status failed")
    );
}

#[tokio::test]
#[serial]
async fn portal_commerce_provider_settlement_event_activates_membership_and_quota() {
    let pool = memory_pool().await;
    let _callback_secret_guard = payment_callback_secret_env_guard("test-payment-callback-secret");
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["status"], "pending_payment");

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/internal/commerce/orders/{order_id}/payment-events"
                ))
                .header(
                    "x-sdkwork-payment-callback-secret",
                    "test-payment-callback-secret",
                )
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"settled\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

    let billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 99760);

    let membership_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/membership")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(membership_response.status(), StatusCode::OK);
    let membership_json = read_json(membership_response).await;
    assert_eq!(membership_json["project_id"], project_id);
    assert_eq!(membership_json["user_id"], user_id);
    assert_eq!(membership_json["plan_id"], "growth");
    assert_eq!(membership_json["status"], "active");
}

#[tokio::test]
#[serial]
async fn portal_billing_summary_prefers_canonical_account_balance_after_recharge_settlement() {
    let pool = memory_pool().await;
    let _callback_secret_guard = payment_callback_secret_env_guard("test-payment-callback-secret");
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["status"], "pending_payment");

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/internal/commerce/orders/{order_id}/payment-events"
                ))
                .header(
                    "x-sdkwork-payment-callback-secret",
                    "test-payment-callback-secret",
                )
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"settled\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

    let billing_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["balance_source"], "canonical_account");
    assert_eq!(billing_json["remaining_units"], 100000);
    assert_eq!(billing_json["canonical_available_balance"], 100000.0);
    assert_eq!(billing_json["canonical_grant_balance"], 100000.0);
    assert_eq!(billing_json["canonical_consumed_balance"], 0.0);
    assert_eq!(billing_json["canonical_held_balance"], 0.0);
    assert_eq!(billing_json["quota_remaining_units"], 260);
}
