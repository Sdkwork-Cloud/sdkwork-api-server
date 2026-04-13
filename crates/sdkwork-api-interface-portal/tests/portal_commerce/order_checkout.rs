use super::*;

#[tokio::test]
async fn portal_manual_payment_simulation_is_disabled_by_default() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
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
    assert_eq!(checkout_session_json["payment_simulation_enabled"], false);

    let order_center_response = app
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
    assert_eq!(order_center_response.status(), StatusCode::OK);
    let order_center_json = read_json(order_center_response).await;
    assert_eq!(order_center_json["payment_simulation_enabled"], false);

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
    assert_eq!(settle_response.status(), StatusCode::CONFLICT);
    assert_eq!(
        read_json(settle_response).await["error"]["message"],
        "portal payment simulation is disabled; use payment attempts and provider callbacks"
    );

    let payment_event_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/payment-events"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"evt_default_disabled\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(payment_event_response.status(), StatusCode::CONFLICT);
    assert_eq!(
        read_json(payment_event_response).await["error"]["message"],
        "portal payment simulation is disabled; use payment attempts and provider callbacks"
    );
}

#[tokio::test]
async fn portal_manual_payment_simulation_can_be_enabled_for_lab_compatibility() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_state(
        sdkwork_api_interface_portal::PortalApiState::new(pool.clone())
            .with_payment_simulation_enabled(true),
    );
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

    let settle_response = app
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
    assert_eq!(read_json(settle_response).await["status"], "fulfilled");
}

#[tokio::test]
async fn portal_commerce_orders_queue_paid_checkout_and_fulfill_coupon_redemption() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_catalog_coupon_code(
        &store,
        "spring20",
        "SPRING20",
        MarketingCampaignStatus::Active,
        CouponCodeStatus::Available,
    )
    .await;

    let app = portal_lab_app(pool.clone());
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
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_catalog_coupon_code(
        &store,
        "spring20",
        "SPRING20",
        MarketingCampaignStatus::Active,
        CouponCodeStatus::Available,
    )
    .await;

    let app = portal_lab_app(pool.clone());
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
        .any(|method| {
            method["action"] == "settle_order"
                && method["provider"] == "manual_lab"
                && method["channel"] == "operator_settlement"
                && method["session_kind"] == "operator_action"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("MANUAL-")
                && method["qr_code_payload"].is_null()
                && method["webhook_verification"] == "manual"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == false
                && method["supports_webhook"] == false
        }));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["action"] == "provider_handoff"
                && method["provider"] == "stripe"
                && method["channel"] == "hosted_checkout"
                && method["session_kind"] == "hosted_checkout"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("STRIPE-")
                && method["qr_code_payload"].is_null()
                && method["webhook_verification"] == "stripe_signature"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == true
                && method["recommended"] == true
                && method["supports_webhook"] == true
        }));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["provider"] == "alipay"
                && method["channel"] == "scan_qr"
                && method["session_kind"] == "qr_code"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("ALIPAY-")
                && method["qr_code_payload"]
                    .as_str()
                    .unwrap()
                    .contains("sdkworkpay://alipay_qr/")
                && method["webhook_verification"] == "alipay_rsa_sha256"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == false
                && method["supports_webhook"] == true
        }));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["provider"] == "wechat_pay"
                && method["channel"] == "scan_qr"
                && method["session_kind"] == "qr_code"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("WECHAT-")
                && method["qr_code_payload"]
                    .as_str()
                    .unwrap()
                    .contains("sdkworkpay://wechat_pay_qr/")
                && method["webhook_verification"] == "wechatpay_rsa_sha256"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == false
                && method["supports_webhook"] == true
        }));

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
