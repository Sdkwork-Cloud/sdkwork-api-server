use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_payment::{
    finalize_refund_order_success, ingest_payment_callback, request_payment_order_refund,
    PaymentCallbackIntakeRequest, PaymentSubjectScope,
};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_payment::{
    PaymentChannelPolicyRecord, PaymentGatewayAccountRecord, PaymentOrderRecord,
    PaymentProviderCode, RefundOrderRecord,
};
use sdkwork_api_storage_core::PaymentKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn order_center_entries(json: &Value) -> &[Value] {
    json["orders"].as_array().unwrap()
}

async fn memory_pool() -> SqlitePool {
    run_migrations("sqlite::memory:").await.unwrap()
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

async fn create_portal_recharge_order(app: axum::Router, token: &str, body_json: &str) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body_json.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await["order_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn load_portal_order(
    pool: &SqlitePool,
    project_id: &str,
    order_id: &str,
) -> CommerceOrderRecord {
    SqliteAdminStore::new(pool.clone())
        .list_commerce_orders_for_project(project_id)
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.order_id == order_id)
        .unwrap()
}

async fn settle_portal_order_via_canonical_payment(
    pool: &SqlitePool,
    project_id: &str,
    order_id: &str,
    created_at_ms: u64,
) -> PaymentOrderRecord {
    let store = SqliteAdminStore::new(pool.clone());
    let _order = load_portal_order(pool, project_id, order_id).await;
    let payment_order = store.list_payment_order_records().await.unwrap();
    let payment_order = payment_order
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            &format!("evt_settled_{order_id}"),
            &format!("dedupe_evt_settled_{order_id}"),
            created_at_ms.saturating_add(120),
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some(format!("pi_{order_id}")))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(format!("{{\"id\":\"evt_settled_{order_id}\"}}"))),
    )
    .await
    .unwrap();

    store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap()
}

async fn authorize_portal_order_via_canonical_payment(
    pool: &SqlitePool,
    project_id: &str,
    order_id: &str,
    created_at_ms: u64,
) -> PaymentOrderRecord {
    let store = SqliteAdminStore::new(pool.clone());
    let _order = load_portal_order(pool, project_id, order_id).await;
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "payment_intent.amount_capturable_updated",
            &format!("evt_authorized_{order_id}"),
            &format!("dedupe_evt_authorized_{order_id}"),
            created_at_ms.saturating_add(120),
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some(format!("pi_auth_{order_id}")))
        .with_signature_status("verified")
        .with_provider_status(Some("requires_capture".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(format!(
            "{{\"id\":\"evt_authorized_{order_id}\",\"status\":\"requires_capture\"}}"
        ))),
    )
    .await
    .unwrap();

    store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap()
}

async fn partially_capture_portal_order_via_canonical_payment(
    pool: &SqlitePool,
    project_id: &str,
    order_id: &str,
    captured_amount_minor: u64,
    created_at_ms: u64,
) -> PaymentOrderRecord {
    let store = SqliteAdminStore::new(pool.clone());
    let _order = load_portal_order(pool, project_id, order_id).await;
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            &format!("evt_partial_{order_id}"),
            &format!("dedupe_evt_partial_{order_id}"),
            created_at_ms.saturating_add(120),
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some(format!("pi_partial_{order_id}")))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(captured_amount_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(format!("{{\"id\":\"evt_partial_{order_id}\"}}"))),
    )
    .await
    .unwrap();

    store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap()
}

async fn capture_portal_order_via_canonical_payment(
    pool: &SqlitePool,
    project_id: &str,
    order_id: &str,
    provider_transaction_id: &str,
    captured_amount_minor: u64,
    created_at_ms: u64,
) -> PaymentOrderRecord {
    let store = SqliteAdminStore::new(pool.clone());
    let _order = load_portal_order(pool, project_id, order_id).await;
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            &format!("evt_capture_{}_{}", order_id, provider_transaction_id),
            &format!("dedupe_capture_{}_{}", order_id, provider_transaction_id),
            created_at_ms.saturating_add(120),
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some(provider_transaction_id.to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(captured_amount_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(format!(
            "{{\"id\":\"evt_capture_{}_{}\"}}",
            order_id, provider_transaction_id
        ))),
    )
    .await
    .unwrap();

    store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap()
}

async fn request_portal_refund_seed(
    pool: &SqlitePool,
    payment_order: &PaymentOrderRecord,
    portal_user_id: &str,
    requested_amount_minor: u64,
    requested_at_ms: u64,
) -> RefundOrderRecord {
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );
    request_payment_order_refund(
        &SqliteAdminStore::new(pool.clone()),
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        requested_amount_minor,
        "portal_user",
        portal_user_id,
        requested_at_ms,
    )
    .await
    .unwrap()
}

async fn seed_portal_failover_route(
    store: &SqliteAdminStore,
    payment_order: &PaymentOrderRecord,
    provider_code: PaymentProviderCode,
    gateway_account_id: &str,
    priority: i32,
    method_code: &str,
) {
    store
        .insert_payment_gateway_account_record(
            &PaymentGatewayAccountRecord::new(
                gateway_account_id,
                payment_order.tenant_id,
                payment_order.organization_id,
                provider_code,
            )
            .with_environment("production")
            .with_merchant_id(format!("merchant_{gateway_account_id}"))
            .with_app_id(format!("app_{gateway_account_id}"))
            .with_status("active")
            .with_priority(priority)
            .with_created_at_ms(1_710_503_000)
            .with_updated_at_ms(1_710_503_001),
        )
        .await
        .unwrap();

    store
        .insert_payment_channel_policy_record(
            &PaymentChannelPolicyRecord::new(
                format!("policy_{gateway_account_id}"),
                payment_order.tenant_id,
                payment_order.organization_id,
                provider_code,
                method_code,
            )
            .with_scene_code(&payment_order.order_kind)
            .with_currency_code(&payment_order.currency_code)
            .with_client_kind("portal_web")
            .with_status("active")
            .with_priority(priority)
            .with_created_at_ms(1_710_503_002)
            .with_updated_at_ms(1_710_503_003),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn portal_paid_order_prepares_canonical_payment_artifacts() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(order_center_entries(&json)[0]["order"]["order_id"], order_id);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["commerce_order_id"],
        order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "awaiting_customer"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        order_center_entries(&json)[0]["refunds"].as_array().unwrap().len(),
        0
    );
    assert_eq!(order_center_entries(&json)[0]["refundable_amount_minor"], 0);

    let store = SqliteAdminStore::new(pool.clone());
    let payment_orders = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .filter(|payment_order| payment_order.commerce_order_id == order_id)
        .collect::<Vec<_>>();
    assert_eq!(payment_orders.len(), 1);

    let payment_attempts = store
        .list_payment_attempt_records_for_order(&payment_orders[0].payment_order_id)
        .await
        .unwrap();
    assert_eq!(payment_attempts.len(), 1);

    let payment_sessions = store
        .list_payment_session_records_for_attempt(&payment_attempts[0].payment_attempt_id)
        .await
        .unwrap();
    assert_eq!(payment_sessions.len(), 1);
}

#[tokio::test]
async fn portal_order_center_repairs_missing_checkout_artifacts_for_payable_orders() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    let store = SqliteAdminStore::new(pool.clone());
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    sqlx::query("DELETE FROM ai_payment_session WHERE payment_attempt_id = ?")
        .bind(&payment_attempt.payment_attempt_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM ai_payment_attempt WHERE payment_order_id = ?")
        .bind(&payment_order.payment_order_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM ai_payment_order WHERE payment_order_id = ?")
        .bind(&payment_order.payment_order_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(order_center_entries(&json)[0]["order"]["order_id"], order_id);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["commerce_order_id"],
        order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "awaiting_customer"
    );

    let restored_payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let restored_attempts = store
        .list_payment_attempt_records_for_order(&restored_payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(restored_attempts.len(), 1);
    let restored_sessions = store
        .list_payment_session_records_for_attempt(&restored_attempts[0].payment_attempt_id)
        .await
        .unwrap();
    assert_eq!(restored_sessions.len(), 1);
}

#[tokio::test]
async fn portal_order_center_includes_payment_and_refund_state() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let portal_user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order =
        settle_portal_order_via_canonical_payment(&pool, &project_id, &order_id, 1_710_500_000)
            .await;
    let refund_order = request_portal_refund_seed(
        &pool,
        &payment_order,
        &portal_user_id,
        payment_order.payable_minor,
        1_710_500_600,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(order_center_entries(&json)[0]["order"]["order_id"], order_id);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "captured"
    );
    assert_eq!(
        order_center_entries(&json)[0]["refunds"][0]["refund_order_id"],
        refund_order.refund_order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["refunds"][0]["refund_status"],
        "requested"
    );
    assert_eq!(order_center_entries(&json)[0]["refundable_amount_minor"], 0);
}

#[tokio::test]
async fn portal_order_center_includes_authorized_payment_state() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order =
        authorize_portal_order_via_canonical_payment(&pool, &project_id, &order_id, 1_710_500_400)
            .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "authorized"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["fulfillment_status"],
        "authorized_pending_capture"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"][0]["transaction_kind"],
        "authorization"
    );
    assert_eq!(order_center_entries(&json)[0]["refundable_amount_minor"], 0);
}

#[tokio::test]
async fn portal_order_center_uses_captured_amount_for_partial_capture_refunds() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order = partially_capture_portal_order_via_canonical_payment(
        &pool,
        &project_id,
        &order_id,
        1_000,
        1_710_500_700,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "partially_captured"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["fulfillment_status"],
        "partial_capture_pending_review"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"][0]["transaction_kind"],
        "sale"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"][0]["amount_minor"],
        1000
    );
    assert_eq!(order_center_entries(&json)[0]["refundable_amount_minor"], 1000);
}

#[tokio::test]
async fn portal_order_center_lists_multiple_capture_transactions_with_aggregated_refund_capacity() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let _payment_order = capture_portal_order_via_canonical_payment(
        &pool,
        &project_id,
        &order_id,
        "pi_portal_multi_1",
        1_000,
        1_710_500_760,
    )
    .await;
    let payment_order = capture_portal_order_via_canonical_payment(
        &pool,
        &project_id,
        &order_id,
        "pi_portal_multi_2",
        1_500,
        1_710_500_820,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "partially_captured"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["captured_amount_minor"],
        2500
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["fulfillment_status"],
        "partial_capture_pending_review"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(order_center_entries(&json)[0]["refundable_amount_minor"], 2500);
}

#[tokio::test]
async fn portal_order_center_caps_refundable_amount_after_overcapture() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order = capture_portal_order_via_canonical_payment(
        &pool,
        &project_id,
        &order_id,
        "pi_portal_overcap_1",
        4_500,
        1_710_500_900,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(order_center_entries(&json).len(), 1);
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["payment_status"],
        "captured"
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_order"]["captured_amount_minor"],
        4000
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        order_center_entries(&json)[0]["payment_transactions"][0]["amount_minor"],
        4000
    );
    assert_eq!(order_center_entries(&json)[0]["refundable_amount_minor"], 4000);
}

#[tokio::test]
async fn portal_can_request_refund_for_owned_paid_recharge_order() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order =
        settle_portal_order_via_canonical_payment(&pool, &project_id, &order_id, 1_710_501_000)
            .await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/refunds"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"refund_reason_code\":\"customer_request\",\"requested_amount_minor\":4000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = read_json(response).await;
    assert_eq!(json["payment_order_id"], payment_order.payment_order_id);
    assert_eq!(json["refund_reason_code"], "customer_request");
    assert_eq!(json["requested_amount_minor"], 4000);
    assert_eq!(json["refund_status"], "awaiting_approval");
    assert!(json["approved_amount_minor"].is_null());

    let refunds = SqliteAdminStore::new(pool.clone())
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(refunds.len(), 1);
    assert_eq!(refunds[0].requested_amount_minor, 4_000);
    assert_eq!(refunds[0].refund_status.as_str(), "awaiting_approval");
    assert_eq!(refunds[0].approved_amount_minor, None);
}

#[tokio::test]
async fn portal_repeated_refund_submission_reuses_pending_refund_order() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order =
        settle_portal_order_via_canonical_payment(&pool, &project_id, &order_id, 1_710_501_500)
            .await;

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/refunds"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"refund_reason_code\":\"customer_request\",\"requested_amount_minor\":1000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let second = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/refunds"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"refund_reason_code\":\"customer_request\",\"requested_amount_minor\":1000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first.status(), StatusCode::CREATED);
    assert_eq!(second.status(), StatusCode::CREATED);
    let first_json = read_json(first).await;
    let second_json = read_json(second).await;
    assert_eq!(
        second_json["refund_order_id"],
        first_json["refund_order_id"]
    );
    assert_eq!(second_json["requested_amount_minor"], 1000);
    assert_eq!(second_json["refund_status"], "awaiting_approval");
    assert!(second_json["approved_amount_minor"].is_null());

    let refunds = SqliteAdminStore::new(pool.clone())
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(refunds.len(), 1);
    assert_eq!(refunds[0].requested_amount_minor, 1_000);
    assert_eq!(refunds[0].refund_status.as_str(), "awaiting_approval");
    assert_eq!(refunds[0].approved_amount_minor, None);
}

#[tokio::test]
async fn portal_account_history_shows_grant_and_refund_reversal() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let portal_user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let payment_order =
        settle_portal_order_via_canonical_payment(&pool, &project_id, &order_id, 1_710_502_000)
            .await;
    let refund_order = request_portal_refund_seed(
        &pool,
        &payment_order,
        &portal_user_id,
        payment_order.payable_minor,
        1_710_502_400,
    )
    .await;
    finalize_refund_order_success(
        &SqliteAdminStore::new(pool.clone()),
        &refund_order.refund_order_id,
        "re_portal_refund_1",
        payment_order.payable_minor,
        1_710_502_800,
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account-history")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert!(json["account"].is_object());
    assert!(json["balance"].is_object());
    assert_eq!(json["refunds"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["refunds"][0]["refund_order_id"],
        refund_order.refund_order_id
    );
    assert_eq!(json["ledger_entries"].as_array().unwrap().len(), 2);
    assert!(json["ledger_entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["entry_type"] == "grant_issue"));
    assert!(json["ledger_entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["entry_type"] == "refund"));
    assert_eq!(json["ledger_allocations"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn portal_order_center_exposes_failover_attempts_and_active_retry_session() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    let store = SqliteAdminStore::new(pool.clone());
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    seed_portal_failover_route(
        &store,
        &payment_order,
        PaymentProviderCode::Stripe,
        "stripe-main",
        100,
        "hosted_checkout",
    )
    .await;
    seed_portal_failover_route(
        &store,
        &payment_order,
        PaymentProviderCode::Alipay,
        "alipay-backup",
        90,
        "native_qr",
    )
    .await;

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "payment_intent.payment_failed",
            "evt_portal_failover_1",
            "dedupe_portal_failover_1",
            1_710_503_200,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_portal_failover_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("failed".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_portal_failover_1\",\"status\":\"failed\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let entry = json
        ["orders"].as_array().unwrap()
        .iter()
        .find(|entry| entry["order"]["order_id"] == order_id)
        .unwrap();
    assert_eq!(
        entry["payment_order"]["payment_status"],
        "awaiting_customer"
    );
    assert_eq!(entry["payment_attempts"].as_array().unwrap().len(), 2);
    assert_eq!(entry["payment_attempts"][0]["attempt"]["attempt_no"], 2);
    assert_eq!(
        entry["payment_attempts"][0]["attempt"]["gateway_account_id"],
        "alipay-backup"
    );
    assert_eq!(
        entry["payment_attempts"][0]["sessions"][0]["session_status"],
        "open"
    );
    assert_eq!(
        entry["active_payment_session"]["payment_attempt_id"],
        entry["payment_attempts"][0]["attempt"]["payment_attempt_id"]
    );
    assert_eq!(entry["payment_attempts"][1]["attempt"]["attempt_no"], 1);
    assert_eq!(
        entry["payment_attempts"][1]["attempt"]["attempt_status"],
        "failed"
    );
}

#[tokio::test]
async fn portal_payment_events_get_returns_newest_first_callback_history() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    let store = SqliteAdminStore::new(pool.clone());
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    seed_portal_failover_route(
        &store,
        &payment_order,
        PaymentProviderCode::Stripe,
        "stripe-main-history",
        100,
        "hosted_checkout",
    )
    .await;
    seed_portal_failover_route(
        &store,
        &payment_order,
        PaymentProviderCode::Stripe,
        "stripe-backup-history",
        90,
        "hosted_checkout",
    )
    .await;

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main-history",
            "payment_intent.payment_failed",
            "evt_portal_history_failed",
            "dedupe_portal_history_failed",
            1_710_503_400,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_portal_history_failed".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("failed".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_portal_history_failed\",\"status\":\"failed\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    let retry_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-backup-history",
            "checkout.session.expired",
            "evt_portal_history_expired",
            "dedupe_portal_history_expired",
            1_710_503_500,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(retry_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_portal_history_expired".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("expired".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_portal_history_expired\",\"status\":\"expired\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let events = json.as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event_identity"], "evt_portal_history_expired");
    assert_eq!(events[0]["gateway_account_id"], "stripe-backup-history");
    assert_eq!(events[1]["event_identity"], "evt_portal_history_failed");
    assert_eq!(events[1]["gateway_account_id"], "stripe-main-history");
}
