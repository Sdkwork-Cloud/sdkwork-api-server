use super::*;

#[tokio::test]
async fn portal_commerce_subscription_checkout_requires_settlement_before_membership_activation() {
    let pool = memory_pool().await;
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
    assert_eq!(
        subscription_json["pricing_plan_id"],
        "pricing_plan:subscription_plan:growth"
    );
    assert_eq!(subscription_json["pricing_plan_version"], 1);

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
