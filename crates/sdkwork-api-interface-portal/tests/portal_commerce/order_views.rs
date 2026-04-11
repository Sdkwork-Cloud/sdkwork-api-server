use super::*;

#[tokio::test]
async fn portal_commerce_orders_list_reflects_refunded_recharge_flow() {
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
    let settled_json = read_json(settled_response).await;
    assert_eq!(settled_json["status"], "fulfilled");

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
    assert_eq!(refunded_json["order_id"], order_id);
    assert_eq!(refunded_json["status"], "refunded");

    let response = app
        .clone()
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

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["order_id"], order_id);
    assert_eq!(json[0]["status"], "refunded");

    let checkout_response = app
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

    assert_eq!(checkout_response.status(), StatusCode::OK);
    let checkout_json = read_json(checkout_response).await;
    assert_eq!(checkout_json["order_status"], "refunded");
    assert_eq!(checkout_json["session_status"], "refunded");
    assert_eq!(checkout_json["methods"].as_array().unwrap().len(), 0);

    let events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| {
        event.event_type == "settled"
            && event.provider == "stripe"
            && event.provider_event_id.as_deref() == Some(settled_event_id.as_str())
            && event.processing_status.as_str() == "processed"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
    assert!(events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider == "stripe"
            && event.provider_event_id.as_deref() == Some(refunded_event_id.as_str())
            && event.processing_status.as_str() == "processed"
            && event.order_status_after.as_deref() == Some("refunded")
    }));
}

#[tokio::test]
async fn portal_commerce_order_center_aggregates_order_payment_and_checkout_views() {
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
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["payment_simulation_enabled"], true);
    assert!(json["membership"].is_null());
    assert_eq!(json["orders"].as_array().unwrap().len(), 1);
    assert_eq!(json["orders"][0]["order"]["order_id"], order_id);
    assert_eq!(json["orders"][0]["order"]["status"], "refunded");
    assert_eq!(
        json["orders"][0]["order"]["product_id"],
        "product:recharge_pack:pack-100k"
    );
    assert_eq!(
        json["orders"][0]["order"]["offer_id"],
        "offer:recharge_pack:pack-100k"
    );
    assert_eq!(
        json["orders"][0]["order"]["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(
        json["orders"][0]["order"]["publication_kind"],
        "portal_catalog"
    );
    assert_eq!(
        json["orders"][0]["order"]["publication_status"],
        "published"
    );
    assert_eq!(
        json["orders"][0]["order"]["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
    );
    assert_eq!(json["orders"][0]["order"]["publication_version"], 1);
    assert_eq!(
        json["orders"][0]["order"]["publication_source_kind"],
        "catalog_seed"
    );
    assert_eq!(
        json["orders"][0]["checkout_session"]["payment_simulation_enabled"],
        true
    );
    assert_eq!(
        json["orders"][0]["payment_events"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        json["orders"][0]["latest_payment_event"]["event_type"],
        "refunded"
    );
    assert_eq!(
        json["orders"][0]["checkout_session"]["order_status"],
        "refunded"
    );
    assert_eq!(
        json["orders"][0]["checkout_session"]["session_status"],
        "refunded"
    );
}

#[tokio::test]
async fn portal_commerce_order_center_reports_reconciliation_backlog_for_workspace_account() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;
    let account = seed_portal_workspace_commercial_account(&store, &workspace).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    store
        .insert_account_commerce_reconciliation_state(
            &AccountCommerceReconciliationStateRecord::new(
                account.tenant_id,
                account.organization_id,
                account.account_id,
                &project_id,
                "checkpoint-order",
            )
            .with_last_order_updated_at_ms(1)
            .with_last_order_created_at_ms(1)
            .with_updated_at_ms(1),
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
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["payment_simulation_enabled"], true);
    assert_eq!(json["orders"].as_array().unwrap().len(), 1);
    assert_eq!(json["orders"][0]["order"]["order_id"], order_id);
    assert_eq!(json["reconciliation"]["account_id"], account.account_id);
    assert_eq!(
        json["reconciliation"]["last_reconciled_order_id"],
        "checkpoint-order"
    );
    assert_eq!(
        json["reconciliation"]["last_reconciled_order_updated_at_ms"],
        1
    );
    assert_eq!(
        json["reconciliation"]["last_reconciled_order_created_at_ms"],
        1
    );
    assert_eq!(json["reconciliation"]["last_reconciled_at_ms"], 1);
    assert_eq!(json["reconciliation"]["backlog_order_count"], 1);
    assert_eq!(json["reconciliation"]["healthy"], false);
    assert!(
        json["reconciliation"]["checkpoint_lag_ms"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[tokio::test]
async fn portal_commerce_order_detail_returns_canonical_order_view() {
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

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/portal/commerce/orders/{order_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["order_id"], order_id);
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["target_kind"], "recharge_pack");
    assert_eq!(json["target_id"], "pack-100k");
    assert_eq!(json["status"], "pending_payment");
    assert_eq!(json["product_id"], "product:recharge_pack:pack-100k");
    assert_eq!(json["offer_id"], "offer:recharge_pack:pack-100k");
    assert_eq!(
        json["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(json["publication_kind"], "portal_catalog");
    assert_eq!(json["publication_status"], "published");
    assert_eq!(
        json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
    );
    assert_eq!(json["publication_version"], 1);
    assert_eq!(json["publication_source_kind"], "catalog_seed");
    assert_eq!(
        json["pricing_plan_id"],
        "pricing_plan:recharge_pack:pack-100k"
    );
    assert_eq!(json["pricing_plan_version"], 1);
    assert!(json["latest_payment_attempt_id"].is_null());
}

#[tokio::test]
async fn portal_commerce_payment_methods_return_filtered_configured_methods_for_order() {
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

    seed_portal_payment_method(
        &store,
        &PaymentMethodRecord::new(
            "pm_stripe_checkout",
            "Stripe Checkout",
            "stripe",
            "hosted_checkout",
            1,
        )
        .with_description("Hosted card and wallet checkout")
        .with_capability_codes(vec!["checkout".to_owned(), "refund".to_owned()])
        .with_supported_currency_codes(vec!["USD".to_owned()])
        .with_supported_order_kinds(vec!["recharge_pack".to_owned()]),
    )
    .await;
    seed_portal_payment_method(
        &store,
        &PaymentMethodRecord::new(
            "pm_subscription_only",
            "Subscription Only",
            "stripe",
            "hosted_checkout",
            2,
        )
        .with_description("Not valid for recharge orders")
        .with_supported_currency_codes(vec!["USD".to_owned()])
        .with_supported_order_kinds(vec!["subscription_plan".to_owned()]),
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-methods"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let methods = json.as_array().unwrap();
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0]["payment_method_id"], "pm_stripe_checkout");
    assert_eq!(methods[0]["provider"], "stripe");
    assert_eq!(methods[0]["channel"], "hosted_checkout");
    assert_eq!(methods[0]["supported_order_kinds"][0], "recharge_pack");
}

#[tokio::test]
async fn portal_commerce_payment_attempt_detail_returns_workspace_visible_attempt() {
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

    let order = store
        .list_commerce_orders_for_project(&project_id)
        .await
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .unwrap();
    let payment_attempt_id = "payatt_portal_detail_1";
    let payment_attempt = seed_portal_payment_attempt(
        &store,
        &CommercePaymentAttemptRecord::new(
            payment_attempt_id,
            order.order_id.clone(),
            order.project_id.clone(),
            order.user_id.clone(),
            "pm_stripe_checkout",
            "stripe",
            "hosted_checkout",
            "idem_portal_detail_1",
            1,
            order.payable_price_cents,
            order.currency_code.clone(),
            11,
        )
        .with_status("requires_action")
        .with_provider_payment_intent_id_option(Some("pi_portal_detail_1".to_owned()))
        .with_provider_checkout_session_id_option(Some("cs_portal_detail_1".to_owned()))
        .with_provider_reference_option(Some("ref_portal_detail_1".to_owned()))
        .with_checkout_url_option(Some(
            "https://checkout.stripe.test/session/payatt_portal_detail_1".to_owned(),
        ))
        .with_request_payload_json("{\"payment_method_id\":\"pm_stripe_checkout\"}")
        .with_response_payload_json("{\"status\":\"open\"}")
        .with_expires_at_ms_option(Some(1_717))
        .with_updated_at_ms(12),
    )
    .await;
    store
        .insert_commerce_order(
            &order
                .with_payment_method_id_option(Some("pm_stripe_checkout".to_owned()))
                .with_latest_payment_attempt_id_option(Some(
                    payment_attempt.payment_attempt_id.clone(),
                ))
                .with_settlement_status("requires_action")
                .with_updated_at_ms(12),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/payment-attempts/{}",
                    payment_attempt.payment_attempt_id
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
        json["payment_attempt_id"],
        payment_attempt.payment_attempt_id
    );
    assert_eq!(json["order_id"], order_id);
    assert_eq!(json["payment_method_id"], "pm_stripe_checkout");
    assert_eq!(json["status"], "requires_action");
    assert_eq!(
        json["checkout_url"],
        "https://checkout.stripe.test/session/payatt_portal_detail_1"
    );
    assert_eq!(json["provider_checkout_session_id"], "cs_portal_detail_1");
    assert_eq!(json["provider_payment_intent_id"], "pi_portal_detail_1");
}
