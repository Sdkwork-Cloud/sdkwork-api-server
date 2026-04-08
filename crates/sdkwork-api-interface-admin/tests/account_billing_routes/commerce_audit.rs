use super::*;

#[tokio::test]
async fn admin_commerce_routes_expose_recent_orders_and_payment_events() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_commerce_audit_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let recent_orders = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders?limit=2")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recent_orders.status(), StatusCode::OK);
    let recent_orders_json = read_json(recent_orders).await;
    assert_eq!(recent_orders_json.as_array().unwrap().len(), 2);
    assert_eq!(recent_orders_json[0]["order_id"], "order-refunded");
    assert_eq!(recent_orders_json[1]["order_id"], "order-pending");

    let payment_events = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders/order-refunded/payment-events")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(payment_events.status(), StatusCode::OK);
    let payment_events_json = read_json(payment_events).await;
    assert_eq!(payment_events_json.as_array().unwrap().len(), 2);
    assert_eq!(
        payment_events_json[0]["payment_event_id"],
        "payevt-order-refunded-settled"
    );
    assert_eq!(payment_events_json[0]["provider"], "stripe");
    assert_eq!(payment_events_json[0]["provider_event_id"], "evt_stripe_1");
    assert_eq!(payment_events_json[0]["processing_status"], "processed");
    assert_eq!(
        payment_events_json[1]["payment_event_id"],
        "payevt-order-refunded-failed"
    );
    assert_eq!(payment_events_json[1]["processing_status"], "rejected");
}

#[tokio::test]
async fn admin_commerce_order_audit_route_returns_coupon_and_payment_evidence_chain() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_commerce_audit_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let audit = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders/order-refunded/audit")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(audit.status(), StatusCode::OK);
    let audit_json = read_json(audit).await;
    assert_eq!(audit_json["order"]["order_id"], "order-refunded");
    assert_eq!(audit_json["order"]["applied_coupon_code"], "SPRING20");
    assert_eq!(audit_json["payment_events"].as_array().unwrap().len(), 2);
    assert_eq!(
        audit_json["coupon_reservation"]["coupon_reservation_id"],
        "reservation-order-refunded"
    );
    assert_eq!(
        audit_json["coupon_redemption"]["coupon_redemption_id"],
        "redemption-order-refunded"
    );
    assert_eq!(
        audit_json["coupon_redemption"]["order_id"],
        "order-refunded"
    );
    assert_eq!(audit_json["coupon_rollbacks"].as_array().unwrap().len(), 1);
    assert_eq!(
        audit_json["coupon_rollbacks"][0]["coupon_rollback_id"],
        "rollback-order-refunded"
    );
    assert_eq!(audit_json["coupon_code"]["code_value"], "SPRING20");
    assert_eq!(
        audit_json["coupon_template"]["display_name"],
        "Spring launch 20%"
    );
    assert_eq!(
        audit_json["marketing_campaign"]["display_name"],
        "Spring launch"
    );
}

#[tokio::test]
async fn admin_commerce_order_audit_route_returns_not_found_for_unknown_order() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_commerce_audit_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let audit = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders/order-missing/audit")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(audit.status(), StatusCode::NOT_FOUND);
    let audit_json = read_json(audit).await;
    assert_eq!(
        audit_json["error"]["message"],
        "commerce order order-missing not found"
    );
}
