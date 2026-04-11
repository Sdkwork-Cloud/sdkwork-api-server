use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{gateway_auth_subject_from_request_context, GatewayRequestContext};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountRecord, AccountStatus, AccountType,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationRecord, CouponReservationStatus, CouponRestrictionSpec, CouponTemplateRecord,
    CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord,
    MarketingCampaignStatus, MarketingSubjectScope,
};
use sdkwork_api_domain_rate_limit::RateLimitPolicy;
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
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
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
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

async fn workspace_project_id(app: axum::Router, token: &str) -> String {
    workspace_summary(app, token).await["project"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn workspace_summary(app: axum::Router, token: &str) -> Value {
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

fn workspace_request_context(workspace: &Value) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace["tenant"]["id"].as_str().unwrap().to_owned(),
        project_id: workspace["project"]["id"].as_str().unwrap().to_owned(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
    }
}

async fn seed_portal_workspace_commercial_account(
    store: &SqliteAdminStore,
    workspace: &Value,
) -> AccountRecord {
    let subject = gateway_auth_subject_from_request_context(&workspace_request_context(workspace));
    let account = AccountRecord::new(
        7001,
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
    store.insert_account_record(&account).await.unwrap();
    account
}

fn coupon_rate_limit_policy(
    policy_id: &str,
    project_id: &str,
    route_key: &str,
    actor_bucket: &str,
) -> RateLimitPolicy {
    RateLimitPolicy::new(policy_id, project_id, 1, 3600)
        .with_burst_requests(1)
        .with_enabled(true)
        .with_route_key_option(Some(route_key.to_owned()))
        .with_api_key_hash_option(Some(actor_bucket.to_owned()))
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000)
}

async fn seed_marketing_records(store: &SqliteAdminStore) {
    seed_marketing_records_with_targets(store, &[]).await;
}

async fn seed_marketing_records_with_targets(
    store: &SqliteAdminStore,
    eligible_target_kinds: &[&str],
) {
    seed_marketing_records_with_benefit_and_targets(
        store,
        MarketingBenefitKind::PercentageOff,
        eligible_target_kinds,
    )
    .await;
}

async fn seed_marketing_records_with_benefit_and_targets(
    store: &SqliteAdminStore,
    benefit_kind: MarketingBenefitKind,
    eligible_target_kinds: &[&str],
) {
    let benefit = match benefit_kind {
        MarketingBenefitKind::PercentageOff => CouponBenefitSpec::new(benefit_kind)
            .with_discount_percent(Some(20)),
        MarketingBenefitKind::FixedAmountOff => CouponBenefitSpec::new(benefit_kind)
            .with_discount_amount_minor(Some(2_000)),
        MarketingBenefitKind::GrantUnits => {
            CouponBenefitSpec::new(benefit_kind).with_grant_units(Some(300))
        }
    };
    let template = CouponTemplateRecord::new(
        "template_launch20",
        "launch20",
        benefit_kind,
    )
    .with_display_name("Launch 20")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_restriction(
        CouponRestrictionSpec::new(MarketingSubjectScope::Project).with_eligible_target_kinds(
            eligible_target_kinds
                .iter()
                .map(|kind| (*kind).to_owned())
                .collect(),
        ),
    )
    .with_benefit(benefit)
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
        .with_total_budget_minor(5_000)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_campaign_budget_record(&budget).await.unwrap();

    let code = CouponCodeRecord::new("code_launch20", "template_launch20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_coupon_code_record(&code).await.unwrap();
}

#[tokio::test]
async fn portal_marketing_routes_validate_reserve_confirm_rollback_and_list_assets() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let validation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-validations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"order_amount_minor\":6000,\"reserve_amount_minor\":1200}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(validation.status(), StatusCode::OK);
    let validation_json = read_json(validation).await;
    assert_eq!(validation_json["decision"]["eligible"], true);
    assert_eq!(validation_json["decision"]["reservable_budget_minor"], 1200);
    assert_eq!(validation_json["template"]["template_key"], "launch20");
    assert_eq!(validation_json["code"]["code_value"], "LAUNCH20");

    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000}",
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
    assert_eq!(reserved_json["code"]["status"], "reserved");

    let my_coupons = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/my-coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(my_coupons.status(), StatusCode::OK);
    let my_coupons_json = read_json(my_coupons).await;
    assert_eq!(my_coupons_json["summary"]["total_count"], 1);
    assert_eq!(my_coupons_json["summary"]["available_count"], 0);
    assert_eq!(my_coupons_json["summary"]["reserved_count"], 1);
    assert_eq!(my_coupons_json["items"].as_array().unwrap().len(), 1);
    assert_eq!(my_coupons_json["items"][0]["code"]["code_value"], "LAUNCH20");
    assert_eq!(my_coupons_json["items"][0]["template"]["template_key"], "launch20");
    assert_eq!(
        my_coupons_json["items"][0]["campaign"]["marketing_campaign_id"],
        "campaign_launch20"
    );
    assert_eq!(
        my_coupons_json["items"][0]["applicability"]["all_target_kinds_eligible"],
        true
    );
    assert_eq!(
        my_coupons_json["items"][0]["effect"]["effect_kind"],
        "checkout_discount"
    );
    assert_eq!(my_coupons_json["items"][0]["effect"]["discount_percent"], 20);
    assert_eq!(
        my_coupons_json["items"][0]["ownership"]["owned_by_current_subject"],
        true
    );
    assert_eq!(
        my_coupons_json["items"][0]["ownership"]["claimed_to_current_subject"],
        false
    );
    assert_eq!(
        my_coupons_json["items"][0]["latest_reservation"]["reservation_status"],
        "reserved"
    );

    let confirmed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"coupon_reservation_id\":\"{reservation_id}\",\"subsidy_amount_minor\":1200,\"order_id\":\"order_launch20\",\"payment_event_id\":\"payment_launch20\"}}"
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
        confirmed_json["reservation"]["reservation_status"],
        "confirmed"
    );
    assert_eq!(
        confirmed_json["redemption"]["redemption_status"],
        "redeemed"
    );

    let rolled_back = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"coupon_redemption_id\":\"{redemption_id}\",\"rollback_type\":\"refund\",\"restored_budget_minor\":1200,\"restored_inventory_count\":1}}"
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
    assert_eq!(rolled_back_json["rollback"]["rollback_type"], "refund");

    let reward_history = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/reward-history")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reward_history.status(), StatusCode::OK);
    let reward_history_json = read_json(reward_history).await;
    assert_eq!(reward_history_json.as_array().unwrap().len(), 1);
    assert_eq!(
        reward_history_json[0]["redemption"]["coupon_redemption_id"],
        redemption_id
    );
    assert_eq!(reward_history_json[0]["template"]["template_key"], "launch20");
    assert_eq!(
        reward_history_json[0]["campaign"]["marketing_campaign_id"],
        "campaign_launch20"
    );
    assert_eq!(
        reward_history_json[0]["effect"]["effect_kind"],
        "checkout_discount"
    );
    assert_eq!(reward_history_json[0]["effect"]["discount_percent"], 20);
    assert_eq!(
        reward_history_json[0]["ownership"]["owned_by_current_subject"],
        true
    );
    assert_eq!(
        reward_history_json[0]["rollbacks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        reward_history_json[0]["rollbacks"][0]["rollback_type"],
        "refund"
    );
}

#[tokio::test]
async fn portal_marketing_reservation_reclaims_expired_reservation_inline() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let project_id = workspace_project_id(app.clone(), &token).await;

    let original_code = store
        .find_coupon_code_record_by_value("LAUNCH20")
        .await
        .unwrap()
        .unwrap();
    let reserved_code = original_code
        .clone()
        .with_status(CouponCodeStatus::Reserved)
        .with_updated_at_ms(10);
    store
        .insert_coupon_code_record(&reserved_code)
        .await
        .unwrap();

    let original_budget = store
        .list_campaign_budget_records()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let reserved_budget = original_budget
        .clone()
        .with_reserved_budget_minor(1_200)
        .with_updated_at_ms(10);
    store
        .insert_campaign_budget_record(&reserved_budget)
        .await
        .unwrap();

    let expired_reservation = CouponReservationRecord::new(
        "reservation_portal_expired",
        reserved_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        project_id.clone(),
        1,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(1_200)
    .with_created_at_ms(0)
    .with_updated_at_ms(10);
    store
        .insert_coupon_reservation_record(&expired_reservation)
        .await
        .unwrap();

    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved.status(), StatusCode::CREATED);
    let reserved_json = read_json(reserved).await;
    assert_eq!(
        reserved_json["reservation"]["reservation_status"],
        "reserved"
    );
    assert_eq!(reserved_json["code"]["status"], "reserved");

    let reservations = store.list_coupon_reservation_records().await.unwrap();
    assert_eq!(reservations.len(), 2);
    let stale_reservation = reservations
        .iter()
        .find(|reservation| reservation.coupon_reservation_id == "reservation_portal_expired")
        .unwrap();
    assert_eq!(
        stale_reservation.reservation_status,
        CouponReservationStatus::Expired
    );
    let active_reservation = reservations
        .iter()
        .find(|reservation| reservation.coupon_reservation_id != "reservation_portal_expired")
        .unwrap();
    assert_eq!(
        active_reservation.reservation_status,
        CouponReservationStatus::Reserved
    );
    assert_eq!(active_reservation.budget_reserved_minor, 1_200);

    let refreshed_budget = store
        .list_campaign_budget_records()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(refreshed_budget.reserved_budget_minor, 1_200);
}

#[tokio::test]
async fn portal_marketing_validation_rejects_ineligible_target_kind() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records_with_targets(&store, &["coupon_redemption"]).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let validation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-validations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"subscription_plan\",\"order_amount_minor\":6000,\"reserve_amount_minor\":1200}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(validation.status(), StatusCode::OK);
    let validation_json = read_json(validation).await;
    assert_eq!(validation_json["decision"]["eligible"], false);
    assert_eq!(
        validation_json["decision"]["rejection_reason"],
        "target_kind_not_eligible"
    );
}

#[tokio::test]
async fn portal_marketing_reservation_rejects_ineligible_target_kind() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records_with_targets(&store, &["coupon_redemption"]).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"subscription_plan\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn portal_marketing_routes_replay_same_idempotency_key_without_duplicate_records() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let reserve_body = "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000,\"idempotency_key\":\"reserve_launch20_project_1\"}";
    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(reserve_body))
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

    let reserved_replay = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(reserve_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved_replay.status(), StatusCode::OK);
    let reserved_replay_json = read_json(reserved_replay).await;
    assert_eq!(
        reserved_replay_json["reservation"]["coupon_reservation_id"],
        reservation_id
    );

    let confirm_body = format!(
        "{{\"coupon_reservation_id\":\"{reservation_id}\",\"subsidy_amount_minor\":1200,\"order_id\":\"order_launch20\",\"payment_event_id\":\"payment_launch20\",\"idempotency_key\":\"confirm_launch20_project_1\"}}"
    );
    let confirmed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(confirm_body.clone()))
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

    let confirmed_replay = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(confirm_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(confirmed_replay.status(), StatusCode::OK);
    let confirmed_replay_json = read_json(confirmed_replay).await;
    assert_eq!(
        confirmed_replay_json["redemption"]["coupon_redemption_id"],
        redemption_id
    );

    let rollback_body = format!(
        "{{\"coupon_redemption_id\":\"{redemption_id}\",\"rollback_type\":\"refund\",\"restored_budget_minor\":1200,\"restored_inventory_count\":1,\"idempotency_key\":\"rollback_launch20_project_1_refund\"}}"
    );
    let rolled_back = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(rollback_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rolled_back.status(), StatusCode::OK);
    let rolled_back_json = read_json(rolled_back).await;
    let rollback_id = rolled_back_json["rollback"]["coupon_rollback_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let rolled_back_replay = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(rollback_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rolled_back_replay.status(), StatusCode::OK);
    let rolled_back_replay_json = read_json(rolled_back_replay).await;
    assert_eq!(
        rolled_back_replay_json["rollback"]["coupon_rollback_id"],
        rollback_id
    );

    let reservations = AdminStore::list_coupon_reservation_records(&store)
        .await
        .unwrap();
    let redemptions = AdminStore::list_coupon_redemption_records(&store)
        .await
        .unwrap();
    let rollbacks = AdminStore::list_coupon_rollback_records(&store)
        .await
        .unwrap();
    assert_eq!(reservations.len(), 1);
    assert_eq!(redemptions.len(), 1);
    assert_eq!(rollbacks.len(), 1);
}

#[tokio::test]
async fn portal_marketing_routes_replay_idempotency_key_header_without_duplicate_records() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let reserve_body =
        "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000}";
    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "reserve_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(reserve_body))
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

    let reserved_replay = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "reserve_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(reserve_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved_replay.status(), StatusCode::OK);
    let reserved_replay_json = read_json(reserved_replay).await;
    assert_eq!(
        reserved_replay_json["reservation"]["coupon_reservation_id"],
        reservation_id
    );

    let confirm_body = format!(
        "{{\"coupon_reservation_id\":\"{reservation_id}\",\"subsidy_amount_minor\":1200,\"order_id\":\"order_launch20\",\"payment_event_id\":\"payment_launch20\"}}"
    );
    let confirmed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "confirm_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(confirm_body.clone()))
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

    let confirmed_replay = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "confirm_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(confirm_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(confirmed_replay.status(), StatusCode::OK);
    let confirmed_replay_json = read_json(confirmed_replay).await;
    assert_eq!(
        confirmed_replay_json["redemption"]["coupon_redemption_id"],
        redemption_id
    );

    let rollback_body = format!(
        "{{\"coupon_redemption_id\":\"{redemption_id}\",\"rollback_type\":\"refund\",\"restored_budget_minor\":1200,\"restored_inventory_count\":1}}"
    );
    let rolled_back = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "rollback_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(rollback_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rolled_back.status(), StatusCode::OK);
    let rolled_back_json = read_json(rolled_back).await;
    let rollback_id = rolled_back_json["rollback"]["coupon_rollback_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let rolled_back_replay = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "rollback_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(rollback_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rolled_back_replay.status(), StatusCode::OK);
    let rolled_back_replay_json = read_json(rolled_back_replay).await;
    assert_eq!(
        rolled_back_replay_json["rollback"]["coupon_rollback_id"],
        rollback_id
    );

    let reservations = AdminStore::list_coupon_reservation_records(&store)
        .await
        .unwrap();
    let redemptions = AdminStore::list_coupon_redemption_records(&store)
        .await
        .unwrap();
    let rollbacks = AdminStore::list_coupon_rollback_records(&store)
        .await
        .unwrap();
    assert_eq!(reservations.len(), 1);
    assert_eq!(redemptions.len(), 1);
    assert_eq!(rollbacks.len(), 1);
}

#[tokio::test]
async fn portal_marketing_routes_reject_conflicting_body_and_header_idempotency_keys() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let reserved = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("idempotency-key", "reserve_launch20_project_header_1")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000,\"idempotency_key\":\"reserve_launch20_project_body_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn portal_marketing_routes_enforce_coupon_rate_limits_on_validate_and_reserve() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let project_id = workspace_project_id(app.clone(), &token).await;
    let actor_bucket = format!("project:{project_id}");

    store
        .insert_rate_limit_policy(&coupon_rate_limit_policy(
            "coupon_validate_project_limit",
            &project_id,
            "marketing.coupon.validate",
            &actor_bucket,
        ))
        .await
        .unwrap();
    store
        .insert_rate_limit_policy(&coupon_rate_limit_policy(
            "coupon_reserve_project_limit",
            &project_id,
            "marketing.coupon.reserve",
            &actor_bucket,
        ))
        .await
        .unwrap();

    let validation_body = "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"order_amount_minor\":6000,\"reserve_amount_minor\":1200}";
    let validation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-validations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(validation_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(validation.status(), StatusCode::OK);

    let throttled_validation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-validations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(validation_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(throttled_validation.status(), StatusCode::TOO_MANY_REQUESTS);

    let reserve_body = "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000}";
    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(reserve_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reserved.status(), StatusCode::CREATED);

    let throttled_reservation = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(reserve_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        throttled_reservation.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn portal_marketing_routes_enforce_coupon_rate_limits_on_confirm_and_rollback() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records(&store).await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let project_id = workspace_project_id(app.clone(), &token).await;
    let actor_bucket = format!("project:{project_id}");

    store
        .insert_rate_limit_policy(&coupon_rate_limit_policy(
            "coupon_confirm_project_limit",
            &project_id,
            "marketing.coupon.confirm",
            &actor_bucket,
        ))
        .await
        .unwrap();
    store
        .insert_rate_limit_policy(&coupon_rate_limit_policy(
            "coupon_rollback_project_limit",
            &project_id,
            "marketing.coupon.rollback",
            &actor_bucket,
        ))
        .await
        .unwrap();

    let reserved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-reservations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"coupon_code\":\"LAUNCH20\",\"subject_scope\":\"project\",\"target_kind\":\"recharge_pack\",\"reserve_amount_minor\":1200,\"ttl_ms\":300000}",
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

    let confirm_body = format!(
        "{{\"coupon_reservation_id\":\"{reservation_id}\",\"subsidy_amount_minor\":1200,\"order_id\":\"order_launch20\",\"payment_event_id\":\"payment_launch20\"}}"
    );
    let confirmed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(confirm_body.clone()))
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

    let throttled_confirm = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/confirm")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(confirm_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(throttled_confirm.status(), StatusCode::TOO_MANY_REQUESTS);

    let rollback_body = format!(
        "{{\"coupon_redemption_id\":\"{redemption_id}\",\"rollback_type\":\"refund\",\"restored_budget_minor\":1200,\"restored_inventory_count\":1}}"
    );
    let rolled_back = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(rollback_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rolled_back.status(), StatusCode::OK);

    let throttled_rollback = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/marketing/coupon-redemptions/rollback")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(rollback_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(throttled_rollback.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn portal_marketing_my_coupons_exposes_account_entitlement_coupon_semantics() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records_with_benefit_and_targets(
        &store,
        MarketingBenefitKind::GrantUnits,
        &["coupon_redemption"],
    )
    .await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let project_id = workspace_project_id(app.clone(), &token).await;

    let original_code = store
        .find_coupon_code_record_by_value("LAUNCH20")
        .await
        .unwrap()
        .unwrap();
    let claimed_code = original_code
        .clone()
        .with_claimed_subject(Some(MarketingSubjectScope::Project), Some(project_id))
        .with_updated_at_ms(1_710_000_000_500);
    store.insert_coupon_code_record(&claimed_code).await.unwrap();

    let my_coupons = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/my-coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(my_coupons.status(), StatusCode::OK);
    let my_coupons_json = read_json(my_coupons).await;
    assert_eq!(my_coupons_json["summary"]["total_count"], 1);
    assert_eq!(
        my_coupons_json["items"][0]["effect"]["effect_kind"],
        "account_entitlement"
    );
    assert_eq!(my_coupons_json["items"][0]["effect"]["grant_units"], 300);
    assert_eq!(
        my_coupons_json["items"][0]["applicability"]["all_target_kinds_eligible"],
        false
    );
    assert_eq!(
        my_coupons_json["items"][0]["applicability"]["target_kinds"][0],
        "coupon_redemption"
    );
    assert_eq!(
        my_coupons_json["items"][0]["ownership"]["owned_by_current_subject"],
        true
    );
    assert_eq!(
        my_coupons_json["items"][0]["ownership"]["claimed_to_current_subject"],
        true
    );
}

#[tokio::test]
async fn portal_marketing_reward_history_exposes_account_arrival_evidence_for_entitlement_coupon() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_records_with_benefit_and_targets(
        &store,
        MarketingBenefitKind::GrantUnits,
        &["coupon_redemption"],
    )
    .await;

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = workspace_summary(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let account = seed_portal_workspace_commercial_account(&store, &workspace).await;

    let reservation = CouponReservationRecord::new(
        "reservation_reward_arrival",
        "code_launch20",
        MarketingSubjectScope::Project,
        project_id.clone(),
        1_710_000_030_000,
    )
    .with_status(CouponReservationStatus::Confirmed)
    .with_budget_reserved_minor(0)
    .with_created_at_ms(1_710_000_000_100)
    .with_updated_at_ms(1_710_000_000_200);
    store
        .insert_coupon_reservation_record(&reservation)
        .await
        .unwrap();

    let redemption = CouponRedemptionRecord::new(
        "redemption_reward_arrival",
        reservation.coupon_reservation_id.clone(),
        "code_launch20",
        "template_launch20",
        1_710_000_000_300,
    )
    .with_status(CouponRedemptionStatus::Redeemed)
    .with_order_id(Some("order_reward_arrival".to_owned()))
    .with_updated_at_ms(1_710_000_000_300);
    store.insert_coupon_redemption_record(&redemption).await.unwrap();

    let benefit_lot = AccountBenefitLotRecord::new(
        8001,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Order)
    .with_source_id(Some(9001))
    .with_scope_json(Some(format!(
        "{{\"order_id\":\"order_reward_arrival\",\"project_id\":\"{project_id}\",\"target_kind\":\"coupon_redemption\"}}"
    )))
    .with_original_quantity(300.0)
    .with_remaining_quantity(300.0)
    .with_held_quantity(0.0)
    .with_issued_at_ms(1_710_000_000_350)
    .with_expires_at_ms(Some(1_720_000_000_000))
    .with_status(AccountBenefitLotStatus::Active)
    .with_created_at_ms(1_710_000_000_350)
    .with_updated_at_ms(1_710_000_000_350);
    AccountKernelStore::insert_account_benefit_lot(&store, &benefit_lot)
        .await
        .unwrap();

    let reward_history = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/marketing/reward-history")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reward_history.status(), StatusCode::OK);
    let reward_history_json = read_json(reward_history).await;
    assert_eq!(reward_history_json.as_array().unwrap().len(), 1);
    assert_eq!(
        reward_history_json[0]["effect"]["effect_kind"],
        "account_entitlement"
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["order_id"],
        "order_reward_arrival"
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["account_id"],
        account.account_id
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lot_count"],
        1
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["lot_id"],
        benefit_lot.lot_id
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["benefit_type"],
        "cash_credit"
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["source_type"],
        "order"
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["status"],
        "active"
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["source_id"],
        9001
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["scope_order_id"],
        "order_reward_arrival"
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["credited_quantity"]
            .as_f64()
            .unwrap(),
        300.0
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["original_quantity"]
            .as_f64()
            .unwrap(),
        300.0
    );
    assert_eq!(
        reward_history_json[0]["account_arrival"]["benefit_lots"][0]["remaining_quantity"]
            .as_f64()
            .unwrap(),
        300.0
    );
}
