use super::*;

#[tokio::test]
async fn portal_refund_payment_event_is_idempotent_for_owned_paid_recharge_order() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let refund_event_id = format!("evt_refund_idempotent_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

    let first_refund_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refund_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(first_refund_response.status(), StatusCode::OK);
    let first_refund_json = read_json(first_refund_response).await;
    assert_eq!(first_refund_json["status"], "refunded");

    let replay_response = apply_portal_payment_event(
        app,
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refund_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(replay_response.status(), StatusCode::OK);
    let replay_json = read_json(replay_response).await;
    assert_eq!(replay_json["order_id"], order_id);
    assert_eq!(replay_json["status"], "refunded");

    let events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    let refunded_events = events
        .iter()
        .filter(|event| event.event_type == "refunded")
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert_eq!(refunded_events.len(), 1);
    assert_eq!(
        refunded_events[0].dedupe_key,
        format!("stripe:{refund_event_id}")
    );
    assert_eq!(refunded_events[0].processing_status.as_str(), "processed");
    assert_eq!(
        refunded_events[0].order_status_after.as_deref(),
        Some("refunded")
    );
}

#[tokio::test]
async fn portal_refund_payment_event_restores_quota_and_blocks_unsafe_recovery() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let refunded_event_id = format!("evt_refunded_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

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

    let refunded_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refunded_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(refunded_response.status(), StatusCode::OK);
    let refunded_json = read_json(refunded_response).await;
    assert_eq!(refunded_json["status"], "refunded");

    let billing_after_refund = app
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

    assert_eq!(billing_after_refund.status(), StatusCode::OK);
    let billing_after_refund_json = read_json(billing_after_refund).await;
    assert_eq!(billing_after_refund_json["remaining_units"], 260);

    let unsafe_order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let unsafe_settled_event_id = format!("evt_settled_{unsafe_order_id}");
    let unsafe_refund_event_id = format!("evt_refunded_{unsafe_order_id}");

    let unsafe_settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &unsafe_order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{unsafe_settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(unsafe_settled_response.status(), StatusCode::OK);

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(100_000_i64)
    .bind(175.0_f64)
    .execute(&pool)
    .await
    .unwrap();

    let unsafe_refund_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &unsafe_order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{unsafe_refund_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(unsafe_refund_response.status(), StatusCode::CONFLICT);
    let unsafe_refund_json = read_json(unsafe_refund_response).await;
    assert_eq!(
        unsafe_refund_json["error"]["message"],
        format!(
            "order {unsafe_order_id} cannot be refunded because recharge headroom has already been consumed"
        )
    );

    let unsafe_checkout_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{unsafe_order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unsafe_checkout_response.status(), StatusCode::OK);
    let unsafe_checkout_json = read_json(unsafe_checkout_response).await;
    assert_eq!(unsafe_checkout_json["order_status"], "fulfilled");
    assert_eq!(unsafe_checkout_json["session_status"], "settled");

    let unsafe_events = store
        .list_commerce_payment_events_for_order(&unsafe_order_id)
        .await
        .unwrap();
    assert_eq!(unsafe_events.len(), 2);
    assert!(unsafe_events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider_event_id.as_deref() == Some(unsafe_refund_event_id.as_str())
            && event.processing_status.as_str() == "rejected"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
}

#[tokio::test]
async fn portal_commerce_payment_events_reject_unsupported_provider_names() {
    let pool = memory_pool().await;
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

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

    let invalid_provider_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/payment-events"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"event_type\":\"settled\",\"provider\":\"paypal\",\"provider_event_id\":\"evt_paypal\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid_provider_response.status(), StatusCode::BAD_REQUEST);
    let invalid_provider_json = read_json(invalid_provider_response).await;
    assert_eq!(
        invalid_provider_json["error"]["message"],
        "unsupported commerce payment provider: paypal"
    );
}

#[tokio::test]
async fn portal_refund_payment_event_rejects_provider_mismatch_against_settlement_provider() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let mismatched_refund_event_id = format!("evt_refund_alipay_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\",\"checkout_method_id\":\"stripe_checkout\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

    let mismatched_refund_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"alipay\",\"provider_event_id\":\"{mismatched_refund_event_id}\",\"checkout_method_id\":\"alipay_qr\"}}"
        ),
    )
    .await;
    assert_eq!(mismatched_refund_response.status(), StatusCode::CONFLICT);
    let mismatched_refund_json = read_json(mismatched_refund_response).await;
    assert_eq!(
        mismatched_refund_json["error"]["message"],
        format!(
            "refund provider alipay does not match settled provider stripe for order {order_id}"
        )
    );

    let events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider == "alipay"
            && event.provider_event_id.as_deref() == Some(mismatched_refund_event_id.as_str())
            && event.processing_status.as_str() == "rejected"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
}

#[tokio::test]
async fn portal_provider_backed_payment_events_require_provider_event_id_without_checkout_method_hint(
) {
    let pool = memory_pool().await;
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    let response = apply_portal_payment_event(
        app,
        &token,
        &order_id,
        "{\"event_type\":\"settled\",\"provider\":\"stripe\"}",
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "provider_event_id is required for provider-backed payment events"
    );
}
