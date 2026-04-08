use super::*;

#[tokio::test]
async fn portal_recharge_payment_events_sync_canonical_account_history_and_reconciliation_state() {
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
    let settled_event_id = format!("evt_settled_account_history_{order_id}");
    let refunded_event_id = format!("evt_refunded_account_history_{order_id}");

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

    let settlement_history = app
        .clone()
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
    assert_eq!(settlement_history.status(), StatusCode::OK);
    let settlement_history_json = read_json(settlement_history).await;
    assert_eq!(
        settlement_history_json["account"]["account_id"],
        account.account_id
    );
    assert_eq!(
        settlement_history_json["balance"]["available_balance"],
        100000.0
    );
    assert_eq!(
        settlement_history_json["request_settlements"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        settlement_history_json["benefit_lots"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        settlement_history_json["benefit_lots"][0]["source_type"],
        "order"
    );
    assert_eq!(
        settlement_history_json["benefit_lots"][0]["status"],
        "active"
    );
    assert_eq!(
        settlement_history_json["ledger"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        settlement_history_json["ledger"][0]["entry"]["entry_type"],
        "grant_issue"
    );
    assert_eq!(
        settlement_history_json["ledger"][0]["entry"]["quantity"],
        100000.0
    );
    assert_eq!(
        settlement_history_json["ledger"][0]["entry"]["amount"],
        40.0
    );
    assert_eq!(
        settlement_history_json["ledger"][0]["allocations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        settlement_history_json["ledger"][0]["allocations"][0]["quantity_delta"],
        100000.0
    );

    let settlement_order_center = app
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
    assert_eq!(settlement_order_center.status(), StatusCode::OK);
    let settlement_order_center_json = read_json(settlement_order_center).await;
    assert_eq!(
        settlement_order_center_json["reconciliation"]["account_id"],
        account.account_id
    );
    assert_eq!(
        settlement_order_center_json["reconciliation"]["last_reconciled_order_id"],
        order_id
    );
    assert_eq!(
        settlement_order_center_json["reconciliation"]["backlog_order_count"],
        0
    );
    assert_eq!(
        settlement_order_center_json["reconciliation"]["healthy"],
        true
    );

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

    let refund_history = app
        .clone()
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
    assert_eq!(refund_history.status(), StatusCode::OK);
    let refund_history_json = read_json(refund_history).await;
    assert_eq!(
        refund_history_json["account"]["account_id"],
        account.account_id
    );
    assert_eq!(refund_history_json["balance"]["available_balance"], 0.0);
    assert_eq!(
        refund_history_json["benefit_lots"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(refund_history_json["benefit_lots"][0]["status"], "disabled");
    assert_eq!(refund_history_json["ledger"].as_array().unwrap().len(), 2);
    assert_eq!(
        refund_history_json["ledger"][0]["entry"]["entry_type"],
        "refund"
    );
    assert_eq!(
        refund_history_json["ledger"][0]["entry"]["quantity"],
        100000.0
    );
    assert_eq!(refund_history_json["ledger"][0]["entry"]["amount"], 40.0);
    assert_eq!(
        refund_history_json["ledger"][0]["allocations"][0]["quantity_delta"],
        -100000.0
    );
    assert_eq!(
        refund_history_json["ledger"][1]["entry"]["entry_type"],
        "grant_issue"
    );
    assert_eq!(
        refund_history_json["ledger"][1]["allocations"][0]["quantity_delta"],
        100000.0
    );
    assert_eq!(
        refund_history_json["request_settlements"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let refund_order_center = app
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
    assert_eq!(refund_order_center.status(), StatusCode::OK);
    let refund_order_center_json = read_json(refund_order_center).await;
    assert_eq!(
        refund_order_center_json["reconciliation"]["account_id"],
        account.account_id
    );
    assert_eq!(
        refund_order_center_json["reconciliation"]["last_reconciled_order_id"],
        order_id
    );
    assert_eq!(
        refund_order_center_json["reconciliation"]["backlog_order_count"],
        0
    );
    assert_eq!(refund_order_center_json["reconciliation"]["healthy"], true);
}

#[tokio::test]
async fn portal_recharge_payment_event_replay_repairs_canonical_account_sync_after_ledger_failure()
{
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;
    let _account = seed_portal_workspace_commercial_account(&store, &workspace).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_repair_{order_id}");

    sqlx::query(
        "CREATE TRIGGER fail_grant_issue_ledger_entry
         BEFORE INSERT ON ai_account_ledger_entry
         WHEN NEW.entry_type = 'grant_issue'
         BEGIN
             SELECT RAISE(ABORT, 'fail commerce account issue');
         END;",
    )
    .execute(&pool)
    .await
    .unwrap();

    let failed_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(failed_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let failed_json = read_json(failed_response).await;
    assert!(failed_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("fail commerce account issue"));

    let stored_order = store
        .list_commerce_orders_for_project(&project_id)
        .await
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .unwrap();
    assert_eq!(stored_order.status, "fulfilled");

    let events_after_failure = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(events_after_failure.len(), 1);
    assert_eq!(events_after_failure[0].processing_status.as_str(), "failed");
    assert_eq!(
        events_after_failure[0].order_status_after.as_deref(),
        Some("fulfilled")
    );

    sqlx::query("DROP TRIGGER fail_grant_issue_ledger_entry")
        .execute(&pool)
        .await
        .unwrap();

    let replay_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(replay_response.status(), StatusCode::OK);

    let repaired_history = app
        .clone()
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
    assert_eq!(repaired_history.status(), StatusCode::OK);
    let repaired_history_json = read_json(repaired_history).await;
    assert_eq!(repaired_history_json["ledger"].as_array().unwrap().len(), 1);
    assert_eq!(
        repaired_history_json["ledger"][0]["entry"]["entry_type"],
        "grant_issue"
    );
    assert_eq!(
        repaired_history_json["ledger"][0]["allocations"][0]["quantity_delta"],
        100000.0
    );

    let events_after_replay = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(events_after_replay.len(), 1);
    assert_eq!(
        events_after_replay[0].processing_status.as_str(),
        "processed"
    );
    assert_eq!(
        events_after_replay[0].order_status_after.as_deref(),
        Some("fulfilled")
    );
}
