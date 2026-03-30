use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
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
async fn portal_commerce_pending_recharge_can_be_settled_or_canceled() {
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
    assert_eq!(checkout_session_json["provider"], "manual_lab");
    assert_eq!(checkout_session_json["mode"], "operator_settlement");
    assert!(checkout_session_json["reference"]
        .as_str()
        .unwrap()
        .starts_with("PAY-"));
    assert!(checkout_session_json["methods"]
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

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["order_id"], settled_order_id);
    assert_eq!(settle_json["status"], "fulfilled");

    let settled_checkout_session_response = app
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

    assert_eq!(settled_checkout_session_response.status(), StatusCode::OK);
    let settled_checkout_session_json = read_json(settled_checkout_session_response).await;
    assert_eq!(settled_checkout_session_json["session_status"], "settled");
    assert_eq!(
        settled_checkout_session_json["methods"]
            .as_array()
            .unwrap()
            .len(),
        0
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
    assert_eq!(billing_after_settle_json["remaining_units"], 100260);

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
    assert_eq!(billing_after_cancel_json["remaining_units"], 100260);
}

#[tokio::test]
async fn portal_commerce_subscription_checkout_requires_settlement_before_membership_activation() {
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

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

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
    assert_eq!(settled_billing_json["remaining_units"], 99760);

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
    assert_eq!(membership_json["project_id"], project_id);
    assert_eq!(membership_json["user_id"], user_id);
    assert_eq!(membership_json["plan_id"], "growth");
    assert_eq!(membership_json["plan_name"], "Growth");
    assert_eq!(membership_json["included_units"], 100000);
    assert_eq!(membership_json["cadence"], "/month");
    assert_eq!(membership_json["status"], "active");
}

#[tokio::test]
async fn portal_commerce_payment_events_can_fail_checkout_and_block_invalid_recovery() {
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
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
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
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
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
async fn portal_commerce_payment_settlement_event_activates_membership_and_quota() {
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
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
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
