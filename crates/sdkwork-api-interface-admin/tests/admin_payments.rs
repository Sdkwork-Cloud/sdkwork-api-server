use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_payment::{
    approve_refund_order_request, ensure_commerce_payment_checkout, finalize_refund_order_success,
    ingest_payment_callback, request_payment_order_refund, request_portal_commerce_order_refund,
    PaymentCallbackIntakeRequest, PaymentSubjectScope,
};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_payment::{PaymentProviderCode, RefundOrderStatus};
use sdkwork_api_storage_core::PaymentKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    sdkwork_api_app_identity::upsert_admin_user(
        &store,
        Some("admin_local_default"),
        "admin@sdkwork.local",
        "Admin Operator",
        Some("ChangeMe123!"),
        Some(sdkwork_api_domain_identity::AdminUserRole::SuperAdmin),
        true,
    )
    .await
    .unwrap();
    pool
}

async fn login_token(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn build_pending_recharge_order() -> CommerceOrderRecord {
    build_pending_recharge_order_with_suffix("admin-1", 1_710_600_000)
}

fn build_pending_recharge_order_with_suffix(
    suffix: &str,
    created_at_ms: u64,
) -> CommerceOrderRecord {
    CommerceOrderRecord::new(
        format!("commerce-order-{suffix}"),
        format!("project-{suffix}"),
        format!("portal-user-{suffix}"),
        "recharge_pack",
        format!("pack-100k-{suffix}"),
        "Boost 100k",
        4_000,
        4_000,
        "$40.00",
        "$40.00",
        100_000,
        0,
        "pending_payment",
        "workspace_seed",
        created_at_ms,
    )
}

#[tokio::test]
async fn admin_payment_routes_list_orders_and_refunds() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order();
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(51, 0, 91);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "admin_test")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_paid_1",
            "dedupe_admin_paid_1",
            1_710_600_200,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_paid_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_paid_1\"}".to_owned())),
    )
    .await
    .unwrap();

    request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        payment_order.payable_minor,
        "portal_user",
        "portal-user-1",
        1_710_600_400,
    )
    .await
    .unwrap();

    let orders_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/orders")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(orders_response.status(), StatusCode::OK);
    let orders_json = read_json(orders_response).await;
    assert_eq!(orders_json.as_array().unwrap().len(), 1);
    assert_eq!(
        orders_json[0]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(orders_json[0]["payment_status"], "captured");
    assert_eq!(orders_json[0]["refund_status"], "pending");

    let refunds_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/refunds")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refunds_response.status(), StatusCode::OK);
    let refunds_json = read_json(refunds_response).await;
    assert_eq!(refunds_json.as_array().unwrap().len(), 1);
    assert_eq!(
        refunds_json[0]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(refunds_json[0]["refund_status"], "requested");
}

#[tokio::test]
async fn admin_payment_routes_filter_refund_queue_by_status() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);

    let awaiting_order =
        build_pending_recharge_order_with_suffix("admin-refund-filter-awaiting", 1_710_600_500);
    store.insert_commerce_order(&awaiting_order).await.unwrap();
    let awaiting_scope = PaymentSubjectScope::new(151, 0, 191);
    let awaiting_checkout =
        ensure_commerce_payment_checkout(&store, &awaiting_scope, &awaiting_order, "admin_test")
            .await
            .unwrap();
    let awaiting_payment_order = awaiting_checkout.payment_order_opt.unwrap();
    let awaiting_payment_attempt = awaiting_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            awaiting_scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_refund_filter_awaiting_paid_1",
            "dedupe_admin_refund_filter_awaiting_paid_1",
            1_710_600_620,
        )
        .with_payment_order_id(Some(awaiting_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(awaiting_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_refund_filter_awaiting_paid_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(awaiting_payment_order.payable_minor))
        .with_currency_code(Some(awaiting_payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_admin_refund_filter_awaiting_paid_1\"}".to_owned(),
        )),
    )
    .await
    .unwrap();
    request_portal_commerce_order_refund(
        &store,
        "portal-user-admin-refund-filter-awaiting",
        &awaiting_order.project_id,
        &awaiting_order.order_id,
        "customer_request",
        4_000,
        1_710_600_700,
    )
    .await
    .unwrap();

    let approved_order =
        build_pending_recharge_order_with_suffix("admin-refund-filter-approved", 1_710_600_800);
    store.insert_commerce_order(&approved_order).await.unwrap();
    let approved_scope = PaymentSubjectScope::new(152, 0, 192);
    let approved_checkout =
        ensure_commerce_payment_checkout(&store, &approved_scope, &approved_order, "admin_test")
            .await
            .unwrap();
    let approved_payment_order = approved_checkout.payment_order_opt.unwrap();
    let approved_payment_attempt = approved_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            approved_scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_refund_filter_approved_paid_1",
            "dedupe_admin_refund_filter_approved_paid_1",
            1_710_600_920,
        )
        .with_payment_order_id(Some(approved_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(approved_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_refund_filter_approved_paid_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(approved_payment_order.payable_minor))
        .with_currency_code(Some(approved_payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_admin_refund_filter_approved_paid_1\"}".to_owned(),
        )),
    )
    .await
    .unwrap();
    let approved_refund = request_portal_commerce_order_refund(
        &store,
        "portal-user-admin-refund-filter-approved",
        &approved_order.project_id,
        &approved_order.order_id,
        "customer_request",
        4_000,
        1_710_601_000,
    )
    .await
    .unwrap();
    approve_refund_order_request(
        &store,
        &approved_refund.refund_order_id,
        Some(3_000),
        1_710_601_040,
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/refunds?refund_status=approved")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["refund_order_id"], approved_refund.refund_order_id);
    assert_eq!(json[0]["refund_status"], "approved");
}

#[tokio::test]
async fn admin_payment_routes_list_reconciliation_lines() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order();
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(52, 0, 92);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "admin_test")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_paid_conflict_1",
            "dedupe_admin_paid_conflict_1",
            1_710_600_600,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_paid_conflict_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_paid_conflict_1\"}".to_owned())),
    )
    .await
    .unwrap();

    let refund = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        payment_order.payable_minor,
        "portal_user",
        "portal-user-1",
        1_710_600_700,
    )
    .await
    .unwrap();

    finalize_refund_order_success(
        &store,
        &refund.refund_order_id,
        "re_admin_refund_original",
        payment_order.payable_minor,
        1_710_600_760,
    )
    .await
    .unwrap();

    let mut rewound_refund = store
        .find_refund_order_record(&refund.refund_order_id)
        .await
        .unwrap()
        .unwrap();
    rewound_refund.refund_status = RefundOrderStatus::Processing;
    rewound_refund.updated_at_ms = 1_710_600_820;
    store
        .insert_refund_order_record(&rewound_refund)
        .await
        .unwrap();

    finalize_refund_order_success(
        &store,
        &refund.refund_order_id,
        "re_admin_refund_changed",
        payment_order.payable_minor,
        1_710_600_880,
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-lines")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(
        json[0]["provider_transaction_id"],
        Value::String("re_admin_refund_changed".to_owned())
    );
    assert_eq!(
        json[0]["refund_order_id"],
        Value::String(refund.refund_order_id.clone())
    );
    assert_eq!(
        json[0]["payment_order_id"],
        Value::String(payment_order.payment_order_id.clone())
    );
    assert_eq!(json[0]["match_status"], "mismatch_reference");
    assert_eq!(
        json[0]["reason_code"],
        Value::String("refund_provider_transaction_conflict".to_owned())
    );
}

#[tokio::test]
async fn admin_payment_routes_resolve_reconciliation_lines() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order();
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(53, 0, 93);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "admin_test")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_overcapture_1",
            "dedupe_admin_overcapture_1",
            1_710_600_900,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_overcapture_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_500))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_overcapture_1\"}".to_owned())),
    )
    .await
    .unwrap();

    let reconciliation_line = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected overcapture reconciliation line");
    assert_eq!(reconciliation_line.match_status.as_str(), "mismatch_amount");

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/reconciliation-lines/{}/resolve",
                    reconciliation_line.reconciliation_line_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"resolved_at_ms\":1710600960}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolved_json = read_json(resolve_response).await;
    assert_eq!(
        resolved_json["reconciliation_line_id"],
        reconciliation_line.reconciliation_line_id
    );
    assert_eq!(resolved_json["match_status"], "resolved");
    assert_eq!(
        resolved_json["reason_code"],
        "payment_capture_amount_capped"
    );
    assert_eq!(resolved_json["provider_amount_minor"], 4500);
    assert_eq!(resolved_json["local_amount_minor"], 4000);
    assert_eq!(resolved_json["updated_at_ms"], 1710600960u64);

    let list_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-lines")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json.as_array().unwrap().len(), 1);
    assert_eq!(list_json[0]["match_status"], "resolved");
    assert_eq!(list_json[0]["updated_at_ms"], 1710600960u64);
}

#[tokio::test]
async fn admin_payment_routes_filter_reconciliation_lines_into_queue_views() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);

    let resolved_order = build_pending_recharge_order_with_suffix("admin-resolved", 1_710_600_000);
    store.insert_commerce_order(&resolved_order).await.unwrap();
    let resolved_scope = PaymentSubjectScope::new(54, 0, 94);
    let resolved_checkout =
        ensure_commerce_payment_checkout(&store, &resolved_scope, &resolved_order, "admin_test")
            .await
            .unwrap();
    let resolved_payment_order = resolved_checkout.payment_order_opt.unwrap();
    let resolved_payment_attempt = resolved_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            resolved_scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_queue_resolved",
            "dedupe_admin_queue_resolved",
            1_710_600_910,
        )
        .with_payment_order_id(Some(resolved_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(resolved_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_queue_resolved".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_500))
        .with_currency_code(Some(resolved_payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_queue_resolved\"}".to_owned())),
    )
    .await
    .unwrap();
    let resolved_line = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            resolved_payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected resolved reconciliation line");
    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/reconciliation-lines/{}/resolve",
                    resolved_line.reconciliation_line_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"resolved_at_ms\":1710601200}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let active_order = build_pending_recharge_order_with_suffix("admin-active", 1_710_600_050);
    store.insert_commerce_order(&active_order).await.unwrap();
    let active_scope = PaymentSubjectScope::new(55, 0, 95);
    let active_checkout =
        ensure_commerce_payment_checkout(&store, &active_scope, &active_order, "admin_test")
            .await
            .unwrap();
    let active_payment_order = active_checkout.payment_order_opt.unwrap();
    let active_payment_attempt = active_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            active_scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_queue_active",
            "dedupe_admin_queue_active",
            1_710_600_980,
        )
        .with_payment_order_id(Some(active_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(active_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_queue_active".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_700))
        .with_currency_code(Some(active_payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_queue_active\"}".to_owned())),
    )
    .await
    .unwrap();
    let active_line = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            active_payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected active reconciliation line");

    let all_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-lines")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(all_response.status(), StatusCode::OK);
    let all_json = read_json(all_response).await;
    assert_eq!(all_json.as_array().unwrap().len(), 2);
    assert_eq!(
        all_json[0]["reconciliation_line_id"],
        active_line.reconciliation_line_id
    );
    assert_eq!(all_json[0]["match_status"], "mismatch_amount");
    assert_eq!(
        all_json[1]["reconciliation_line_id"],
        resolved_line.reconciliation_line_id
    );
    assert_eq!(all_json[1]["match_status"], "resolved");

    let active_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-lines?lifecycle=active")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(active_response.status(), StatusCode::OK);
    let active_json = read_json(active_response).await;
    assert_eq!(active_json.as_array().unwrap().len(), 1);
    assert_eq!(
        active_json[0]["reconciliation_line_id"],
        active_line.reconciliation_line_id
    );
    assert_eq!(active_json[0]["match_status"], "mismatch_amount");

    let resolved_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-lines?lifecycle=resolved")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolved_response.status(), StatusCode::OK);
    let resolved_json = read_json(resolved_response).await;
    assert_eq!(resolved_json.as_array().unwrap().len(), 1);
    assert_eq!(
        resolved_json[0]["reconciliation_line_id"],
        resolved_line.reconciliation_line_id
    );
    assert_eq!(resolved_json[0]["match_status"], "resolved");

    let invalid_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-lines?lifecycle=broken")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_payment_routes_summarize_empty_reconciliation_state() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["total_count"], 0);
    assert_eq!(json["active_count"], 0);
    assert_eq!(json["resolved_count"], 0);
    assert_eq!(json["latest_updated_at_ms"], Value::Null);
    assert_eq!(json["oldest_active_created_at_ms"], Value::Null);
    assert_eq!(json["active_reason_breakdown"], Value::Array(Vec::new()));
}

#[tokio::test]
async fn admin_payment_routes_summarize_reconciliation_state() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);

    let resolved_order =
        build_pending_recharge_order_with_suffix("admin-summary-resolved", 1_710_601_000);
    store.insert_commerce_order(&resolved_order).await.unwrap();
    let resolved_scope = PaymentSubjectScope::new(56, 0, 96);
    let resolved_checkout =
        ensure_commerce_payment_checkout(&store, &resolved_scope, &resolved_order, "admin_test")
            .await
            .unwrap();
    let resolved_payment_order = resolved_checkout.payment_order_opt.unwrap();
    let resolved_payment_attempt = resolved_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            resolved_scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_summary_resolved",
            "dedupe_admin_summary_resolved",
            1_710_601_100,
        )
        .with_payment_order_id(Some(resolved_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(resolved_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_summary_resolved".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_500))
        .with_currency_code(Some(resolved_payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_summary_resolved\"}".to_owned())),
    )
    .await
    .unwrap();
    let resolved_line = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            resolved_payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected resolved reconciliation line");

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/reconciliation-lines/{}/resolve",
                    resolved_line.reconciliation_line_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"resolved_at_ms\":1710602200}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolved_json = read_json(resolve_response).await;

    let active_order =
        build_pending_recharge_order_with_suffix("admin-summary-active", 1_710_601_010);
    store.insert_commerce_order(&active_order).await.unwrap();
    let active_scope = PaymentSubjectScope::new(57, 0, 97);
    let active_checkout =
        ensure_commerce_payment_checkout(&store, &active_scope, &active_order, "admin_test")
            .await
            .unwrap();
    let active_payment_order = active_checkout.payment_order_opt.unwrap();
    let active_payment_attempt = active_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            active_scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_summary_paid",
            "dedupe_admin_summary_paid",
            1_710_601_200,
        )
        .with_payment_order_id(Some(active_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(active_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_summary_paid".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(active_payment_order.payable_minor))
        .with_currency_code(Some(active_payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_summary_paid\"}".to_owned())),
    )
    .await
    .unwrap();

    let refund = request_payment_order_refund(
        &store,
        &active_scope,
        &active_payment_order.payment_order_id,
        "customer_request",
        active_payment_order.payable_minor,
        "portal_user",
        "portal-user-1",
        1_710_601_300,
    )
    .await
    .unwrap();
    finalize_refund_order_success(
        &store,
        &refund.refund_order_id,
        "re_admin_summary_original",
        active_payment_order.payable_minor,
        1_710_601_360,
    )
    .await
    .unwrap();

    let mut rewound_refund = store
        .find_refund_order_record(&refund.refund_order_id)
        .await
        .unwrap()
        .unwrap();
    rewound_refund.refund_status = RefundOrderStatus::Processing;
    rewound_refund.updated_at_ms = 1_710_601_420;
    store
        .insert_refund_order_record(&rewound_refund)
        .await
        .unwrap();

    finalize_refund_order_success(
        &store,
        &refund.refund_order_id,
        "re_admin_summary_changed",
        active_payment_order.payable_minor,
        1_710_601_480,
    )
    .await
    .unwrap();

    let active_line = store
        .list_reconciliation_match_summary_records(&format!(
            "refund_conflict_batch_{}",
            refund.refund_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected active reconciliation line");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/reconciliation-summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["total_count"], 2);
    assert_eq!(json["active_count"], 1);
    assert_eq!(json["resolved_count"], 1);
    assert_eq!(
        json["latest_updated_at_ms"],
        resolved_json["updated_at_ms"].clone()
    );
    assert_eq!(
        json["oldest_active_created_at_ms"],
        Value::from(active_line.created_at_ms)
    );
    assert_eq!(json["active_reason_breakdown"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["active_reason_breakdown"][0]["reason_code"],
        "refund_provider_transaction_conflict"
    );
    assert_eq!(json["active_reason_breakdown"][0]["count"], 1);
    assert_eq!(
        json["active_reason_breakdown"][0]["latest_updated_at_ms"],
        Value::from(active_line.updated_at_ms)
    );
}

#[tokio::test]
async fn admin_metrics_expose_payment_reconciliation_gauges() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);

    let resolved_order =
        build_pending_recharge_order_with_suffix("admin-metrics-resolved", 1_710_602_000);
    store.insert_commerce_order(&resolved_order).await.unwrap();
    let resolved_scope = PaymentSubjectScope::new(58, 0, 98);
    let resolved_checkout =
        ensure_commerce_payment_checkout(&store, &resolved_scope, &resolved_order, "admin_test")
            .await
            .unwrap();
    let resolved_payment_order = resolved_checkout.payment_order_opt.unwrap();
    let resolved_payment_attempt = resolved_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            resolved_scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_metrics_resolved",
            "dedupe_admin_metrics_resolved",
            1_710_602_100,
        )
        .with_payment_order_id(Some(resolved_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(resolved_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_metrics_resolved".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_500))
        .with_currency_code(Some(resolved_payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_metrics_resolved\"}".to_owned())),
    )
    .await
    .unwrap();
    let resolved_line = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            resolved_payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected resolved reconciliation line");
    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/reconciliation-lines/{}/resolve",
                    resolved_line.reconciliation_line_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"resolved_at_ms\":1710603200}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let active_order =
        build_pending_recharge_order_with_suffix("admin-metrics-active", 1_710_602_010);
    store.insert_commerce_order(&active_order).await.unwrap();
    let active_scope = PaymentSubjectScope::new(59, 0, 99);
    let active_checkout =
        ensure_commerce_payment_checkout(&store, &active_scope, &active_order, "admin_test")
            .await
            .unwrap();
    let active_payment_order = active_checkout.payment_order_opt.unwrap();
    let active_payment_attempt = active_checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            active_scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_metrics_active",
            "dedupe_admin_metrics_active",
            1_710_602_200,
        )
        .with_payment_order_id(Some(active_payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(active_payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_metrics_active".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_700))
        .with_currency_code(Some(active_payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_metrics_active\"}".to_owned())),
    )
    .await
    .unwrap();
    let active_line = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            active_payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("expected active reconciliation line");

    let metrics = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);

    let bytes = to_bytes(metrics.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("sdkwork_payment_reconciliation_total{service=\"admin\"} 2"));
    assert!(body.contains("sdkwork_payment_reconciliation_active_total{service=\"admin\"} 1"));
    assert!(body.contains("sdkwork_payment_reconciliation_resolved_total{service=\"admin\"} 1"));
    assert!(body.contains(
        "sdkwork_payment_reconciliation_active_reason_total{service=\"admin\",reason_code=\"payment_capture_amount_capped\"} 1"
    ));
    assert!(body.contains(&format!(
        "sdkwork_payment_reconciliation_latest_updated_at_ms{{service=\"admin\"}} {}",
        1_710_603_200u64
    )));
    assert!(body.contains(&format!(
        "sdkwork_payment_reconciliation_oldest_active_created_at_ms{{service=\"admin\"}} {}",
        active_line.created_at_ms
    )));
}

#[tokio::test]
async fn admin_payment_routes_manage_gateway_accounts() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;

    let first_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/gateway-accounts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "gateway_account_id":"stripe-primary",
                        "tenant_id":71,
                        "organization_id":0,
                        "provider_code":"stripe",
                        "environment":"production",
                        "merchant_id":"merchant-primary",
                        "app_id":"app-primary",
                        "status":"active",
                        "priority":100,
                        "created_at_ms":1710700000,
                        "updated_at_ms":1710700001
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_create.status(), StatusCode::CREATED);

    let second_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/gateway-accounts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "gateway_account_id":"stripe-backup",
                        "tenant_id":71,
                        "organization_id":0,
                        "provider_code":"stripe",
                        "environment":"production",
                        "merchant_id":"merchant-backup",
                        "app_id":"app-backup",
                        "status":"active",
                        "priority":80,
                        "created_at_ms":1710700002,
                        "updated_at_ms":1710700003
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_create.status(), StatusCode::CREATED);

    let unrelated_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/gateway-accounts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "gateway_account_id":"alipay-disabled",
                        "tenant_id":71,
                        "organization_id":0,
                        "provider_code":"alipay",
                        "environment":"production",
                        "merchant_id":"merchant-alipay",
                        "app_id":"app-alipay",
                        "status":"inactive",
                        "priority":120,
                        "created_at_ms":1710700004,
                        "updated_at_ms":1710700005
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unrelated_create.status(), StatusCode::CREATED);

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/gateway-accounts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "gateway_account_id":"stripe-backup",
                        "tenant_id":71,
                        "organization_id":0,
                        "provider_code":"stripe",
                        "environment":"production",
                        "merchant_id":"merchant-backup-updated",
                        "app_id":"app-backup-updated",
                        "status":"active",
                        "priority":110,
                        "created_at_ms":1710700002,
                        "updated_at_ms":1710700010
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::CREATED);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["merchant_id"], "merchant-backup-updated");
    assert_eq!(update_json["priority"], 110);

    let list_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/gateway-accounts?provider_code=stripe&status=active&tenant_id=71")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    let list = list_json.as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["gateway_account_id"], "stripe-backup");
    assert_eq!(list[0]["merchant_id"], "merchant-backup-updated");
    assert_eq!(list[1]["gateway_account_id"], "stripe-primary");
}

#[tokio::test]
async fn admin_payment_routes_manage_channel_policies() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;

    let create_primary = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/channel-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "channel_policy_id":"policy-primary",
                        "tenant_id":72,
                        "organization_id":0,
                        "scene_code":"recharge_pack",
                        "country_code":"",
                        "currency_code":"USD",
                        "client_kind":"portal_web",
                        "provider_code":"stripe",
                        "method_code":"hosted_checkout",
                        "priority":100,
                        "status":"active",
                        "created_at_ms":1710800000,
                        "updated_at_ms":1710800001
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_primary.status(), StatusCode::CREATED);

    let create_backup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/channel-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "channel_policy_id":"policy-backup",
                        "tenant_id":72,
                        "organization_id":0,
                        "scene_code":"recharge_pack",
                        "country_code":"",
                        "currency_code":"USD",
                        "client_kind":"portal_web",
                        "provider_code":"stripe",
                        "method_code":"native_qr",
                        "priority":90,
                        "status":"active",
                        "created_at_ms":1710800002,
                        "updated_at_ms":1710800003
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_backup.status(), StatusCode::CREATED);

    let unrelated_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/channel-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "channel_policy_id":"policy-eur",
                        "tenant_id":72,
                        "organization_id":0,
                        "scene_code":"recharge_pack",
                        "country_code":"",
                        "currency_code":"EUR",
                        "client_kind":"portal_web",
                        "provider_code":"stripe",
                        "method_code":"hosted_checkout",
                        "priority":120,
                        "status":"inactive",
                        "created_at_ms":1710800004,
                        "updated_at_ms":1710800005
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unrelated_create.status(), StatusCode::CREATED);

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/channel-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "channel_policy_id":"policy-backup",
                        "tenant_id":72,
                        "organization_id":0,
                        "scene_code":"recharge_pack",
                        "country_code":"",
                        "currency_code":"USD",
                        "client_kind":"portal_web",
                        "provider_code":"stripe",
                        "method_code":"redirect_checkout",
                        "priority":105,
                        "status":"active",
                        "created_at_ms":1710800002,
                        "updated_at_ms":1710800010
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::CREATED);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["method_code"], "redirect_checkout");
    assert_eq!(update_json["priority"], 105);

    let list_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/channel-policies?provider_code=stripe&status=active&tenant_id=72&scene_code=recharge_pack&currency_code=USD&client_kind=portal_web")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    let list = list_json.as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["channel_policy_id"], "policy-backup");
    assert_eq!(list[0]["method_code"], "redirect_checkout");
    assert_eq!(list[1]["channel_policy_id"], "policy-primary");
}

#[tokio::test]
async fn admin_payment_routes_reject_invalid_routing_payloads() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;

    let invalid_gateway_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/gateway-accounts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "gateway_account_id":"gateway-invalid",
                        "tenant_id":73,
                        "organization_id":0,
                        "provider_code":"stripe",
                        "environment":"production",
                        "merchant_id":"",
                        "app_id":"app-invalid",
                        "status":"broken",
                        "priority":1
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_gateway_response.status(), StatusCode::BAD_REQUEST);

    let invalid_policy_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/payments/channel-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "channel_policy_id":"policy-invalid",
                        "tenant_id":73,
                        "organization_id":0,
                        "scene_code":"recharge_pack",
                        "country_code":"",
                        "currency_code":"USD",
                        "client_kind":"portal_web",
                        "provider_code":"stripe",
                        "method_code":"",
                        "priority":1,
                        "status":"broken"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_policy_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_payment_routes_get_order_dossier() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order_with_suffix("admin-detail-1", 1_710_602_000);
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(74, 0, 94);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_detail_paid_1",
            "dedupe_admin_detail_paid_1",
            1_710_602_120,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_detail_paid_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_detail_paid_1\"}".to_owned())),
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/admin/payments/orders/{}",
                    payment_order.payment_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["payment_order"]["payment_order_id"],
        payment_order.payment_order_id
    );
    assert_eq!(json["commerce_order"]["order_id"], order.order_id);
    assert_eq!(json["payment_attempts"].as_array().unwrap().len(), 1);
    assert_eq!(json["payment_sessions"].as_array().unwrap().len(), 1);
    assert_eq!(json["payment_callback_events"].as_array().unwrap().len(), 1);
    assert_eq!(json["payment_transactions"].as_array().unwrap().len(), 1);
    assert_eq!(json["payment_transactions"][0]["transaction_kind"], "sale");
    assert_eq!(json["refund_orders"].as_array().unwrap().len(), 0);
    assert_eq!(json["reconciliation_lines"].as_array().unwrap().len(), 0);
    assert_eq!(json["account"]["user_id"], scope.user_id);
    assert_eq!(json["account_ledger_entries"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["account_ledger_entries"][0]["entry_type"],
        "grant_issue"
    );
    assert_eq!(
        json["account_ledger_allocations"].as_array().unwrap().len(),
        1
    );
    assert_eq!(json["finance_journal_entries"].as_array().unwrap().len(), 0);
    assert_eq!(json["finance_journal_lines"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn admin_payment_routes_get_order_dossier_includes_refund_finance_and_reconciliation() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order_with_suffix("admin-detail-2", 1_710_602_300);
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(75, 0, 95);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_detail_paid_2",
            "dedupe_admin_detail_paid_2",
            1_710_602_420,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_detail_paid_2".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some("{\"id\":\"evt_admin_detail_paid_2\"}".to_owned())),
    )
    .await
    .unwrap();

    let refund = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        payment_order.payable_minor,
        "portal_user",
        "portal-user-detail-2",
        1_710_602_500,
    )
    .await
    .unwrap();

    finalize_refund_order_success(
        &store,
        &refund.refund_order_id,
        "re_admin_detail_original",
        payment_order.payable_minor,
        1_710_602_620,
    )
    .await
    .unwrap();

    let mut rewound_refund = store
        .find_refund_order_record(&refund.refund_order_id)
        .await
        .unwrap()
        .unwrap();
    rewound_refund.refund_status = RefundOrderStatus::Processing;
    rewound_refund.updated_at_ms = 1_710_602_660;
    store
        .insert_refund_order_record(&rewound_refund)
        .await
        .unwrap();

    finalize_refund_order_success(
        &store,
        &refund.refund_order_id,
        "re_admin_detail_changed",
        payment_order.payable_minor,
        1_710_602_700,
    )
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/admin/payments/orders/{}",
                    payment_order.payment_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["refund_orders"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["refund_orders"][0]["refund_order_id"],
        refund.refund_order_id
    );
    assert_eq!(json["refund_orders"][0]["refund_status"], "succeeded");
    assert_eq!(json["payment_transactions"].as_array().unwrap().len(), 2);
    assert_eq!(
        json["payment_transactions"][0]["transaction_kind"],
        "refund"
    );
    assert_eq!(json["payment_transactions"][1]["transaction_kind"], "sale");
    assert_eq!(json["reconciliation_lines"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["reconciliation_lines"][0]["refund_order_id"],
        refund.refund_order_id
    );
    assert_eq!(
        json["reconciliation_lines"][0]["reason_code"],
        "refund_provider_transaction_conflict"
    );
    assert_eq!(json["account"]["user_id"], scope.user_id);
    assert_eq!(json["account_ledger_entries"].as_array().unwrap().len(), 2);
    assert_eq!(json["account_ledger_entries"][0]["entry_type"], "refund");
    assert_eq!(
        json["account_ledger_entries"][1]["entry_type"],
        "grant_issue"
    );
    assert_eq!(
        json["account_ledger_allocations"].as_array().unwrap().len(),
        2
    );
    assert_eq!(json["finance_journal_entries"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["finance_journal_entries"][0]["source_id"],
        refund.refund_order_id
    );
    assert_eq!(
        json["finance_journal_entries"][0]["source_kind"],
        "refund_order"
    );
    assert_eq!(json["finance_journal_lines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn admin_payment_routes_get_order_dossier_returns_not_found_for_missing_order() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app).await;

    let response = sdkwork_api_interface_admin::admin_router_with_pool(pool)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/payments/orders/payment-order-missing")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_payment_routes_approve_and_cancel_refund_requests() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order_with_suffix("admin-approve-1", 1_710_603_000);
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(76, 0, 96);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_refund_approval_paid_1",
            "dedupe_admin_refund_approval_paid_1",
            1_710_603_120,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_refund_approval_paid_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_admin_refund_approval_paid_1\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    let refund = request_portal_commerce_order_refund(
        &store,
        "portal-user-admin-approve-1",
        &order.project_id,
        &order.order_id,
        "customer_request",
        4_000,
        1_710_603_220,
    )
    .await
    .unwrap();
    assert_eq!(refund.refund_status, RefundOrderStatus::AwaitingApproval);

    let approve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/refunds/{}/approve",
                    refund.refund_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"approved_amount_minor\":3000,\"approved_at_ms\":1710603240}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approve_response.status(), StatusCode::OK);
    let approve_json = read_json(approve_response).await;
    assert_eq!(approve_json["refund_status"], "approved");
    assert_eq!(approve_json["approved_amount_minor"], 3000);

    let cancel_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/refunds/{}/cancel",
                    refund.refund_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"canceled_at_ms\":1710603300}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["refund_status"], "canceled");

    let payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(payment_order.refund_status.as_str(), "not_requested");
}

#[tokio::test]
async fn admin_payment_routes_start_refund_execution() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;
    let store = SqliteAdminStore::new(pool);
    let order = build_pending_recharge_order_with_suffix("admin-start-refund-1", 1_710_603_400);
    store.insert_commerce_order(&order).await.unwrap();

    let scope = PaymentSubjectScope::new(77, 0, 97);
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();
    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_admin_refund_start_paid_1",
            "dedupe_admin_refund_start_paid_1",
            1_710_603_520,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_admin_refund_start_paid_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(
            "{\"id\":\"evt_admin_refund_start_paid_1\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    let refund = request_portal_commerce_order_refund(
        &store,
        &order.user_id,
        &order.project_id,
        &order.order_id,
        "customer_request",
        4_000,
        1_710_603_620,
    )
    .await
    .unwrap();
    assert_eq!(refund.refund_status, RefundOrderStatus::AwaitingApproval);

    let approve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/refunds/{}/approve",
                    refund.refund_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"approved_amount_minor\":3000,\"approved_at_ms\":1710603640}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approve_response.status(), StatusCode::OK);

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/refunds/{}/start",
                    refund.refund_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"started_at_ms\":1710603700}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_json = read_json(start_response).await;
    assert_eq!(start_json["refund_status"], "processing");
    assert_eq!(start_json["approved_amount_minor"], 3000);

    let repeat_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/payments/refunds/{}/start",
                    refund.refund_order_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"started_at_ms\":1710603750}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(repeat_response.status(), StatusCode::OK);
    let repeat_json = read_json(repeat_response).await;
    assert_eq!(repeat_json["refund_status"], "processing");
    assert_eq!(repeat_json["updated_at_ms"], start_json["updated_at_ms"]);
}
