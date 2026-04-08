use super::*;

#[tokio::test]
async fn admin_billing_accounts_expose_canonical_balance_summaries_and_lot_details() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_canonical_billing_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let accounts = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(accounts.status(), StatusCode::OK);
    let accounts_json = read_json(accounts).await;
    assert_eq!(accounts_json.as_array().unwrap().len(), 1);
    assert_eq!(accounts_json[0]["account"]["account_id"], 7001);
    assert_eq!(accounts_json[0]["available_balance"], 90.0);
    assert_eq!(accounts_json[0]["held_balance"], 5.0);
    assert_eq!(accounts_json[0]["consumed_balance"], 25.0);
    assert_eq!(accounts_json[0]["grant_balance"], 150.0);
    assert_eq!(accounts_json[0]["active_lot_count"], 1);

    let balance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/7001/balance")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(balance.status(), StatusCode::OK);
    let balance_json = read_json(balance).await;
    assert_eq!(balance_json["account_id"], 7001);
    assert_eq!(balance_json["available_balance"], 90.0);
    assert_eq!(balance_json["held_balance"], 5.0);
    assert_eq!(balance_json["consumed_balance"], 25.0);
    assert_eq!(balance_json["grant_balance"], 150.0);
    assert_eq!(balance_json["active_lot_count"], 1);
    assert_eq!(balance_json["lots"].as_array().unwrap().len(), 1);
    assert_eq!(balance_json["lots"][0]["lot_id"], 8001);

    let lots = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/7001/benefit-lots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(lots.status(), StatusCode::OK);
    let lots_json = read_json(lots).await;
    assert_eq!(lots_json.as_array().unwrap().len(), 2);
    assert!(lots_json
        .as_array()
        .unwrap()
        .iter()
        .any(|lot| lot["lot_id"] == 8001 && lot["status"] == "active"));
    assert!(lots_json
        .as_array()
        .unwrap()
        .iter()
        .any(|lot| lot["lot_id"] == 8002 && lot["status"] == "expired"));
}

#[tokio::test]
async fn admin_billing_investigation_routes_list_holds_settlements_and_pricing() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_canonical_billing_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let holds = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/account-holds")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(holds.status(), StatusCode::OK);
    let holds_json = read_json(holds).await;
    assert_eq!(holds_json.as_array().unwrap().len(), 1);
    assert_eq!(holds_json[0]["hold_id"], 8101);
    assert_eq!(holds_json[0]["status"], "captured");

    let settlements = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/request-settlements")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settlements.status(), StatusCode::OK);
    let settlements_json = read_json(settlements).await;
    assert_eq!(settlements_json.as_array().unwrap().len(), 1);
    assert_eq!(settlements_json[0]["request_settlement_id"], 8301);
    assert_eq!(settlements_json[0]["status"], "captured");

    let plans = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(plans.status(), StatusCode::OK);
    let plans_json = read_json(plans).await;
    assert_eq!(plans_json.as_array().unwrap().len(), 1);
    assert_eq!(plans_json[0]["pricing_plan_id"], 9101);
    assert_eq!(plans_json[0]["status"], "active");
    assert_eq!(plans_json[0]["effective_from_ms"], 10);
    assert_eq!(plans_json[0]["effective_to_ms"], 100);

    let rates = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rates.status(), StatusCode::OK);
    let rates_json = read_json(rates).await;
    assert_eq!(rates_json.as_array().unwrap().len(), 1);
    assert_eq!(rates_json[0]["pricing_rate_id"], 9201);
    assert_eq!(rates_json[0]["metric_code"], "token.input");
    assert_eq!(rates_json[0]["charge_unit"], "input_token");
    assert_eq!(rates_json[0]["pricing_method"], "per_unit");
    assert_eq!(rates_json[0]["display_price_unit"], "USD / 1M input tokens");
    assert_eq!(rates_json[0]["status"], "active");
}

#[tokio::test]
async fn admin_billing_account_ledger_route_exposes_account_history_with_allocations() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_canonical_billing_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let ledger = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/7001/ledger")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ledger.status(), StatusCode::OK);
    let ledger_json = read_json(ledger).await;
    assert_eq!(ledger_json.as_array().unwrap().len(), 2);
    assert_eq!(ledger_json[0]["entry"]["ledger_entry_id"], 8402);
    assert_eq!(ledger_json[0]["entry"]["entry_type"], "refund");
    assert_eq!(ledger_json[0]["entry"]["account_id"], 7001);
    assert_eq!(ledger_json[0]["allocations"][0]["lot_id"], 8001);
    assert_eq!(ledger_json[0]["allocations"][0]["quantity_delta"], 2.0);
    assert_eq!(ledger_json[1]["entry"]["ledger_entry_id"], 8401);
    assert_eq!(ledger_json[1]["entry"]["entry_type"], "settlement_capture");
    assert_eq!(ledger_json[1]["allocations"][0]["quantity_delta"], -5.0);
}

#[tokio::test]
async fn admin_billing_balance_returns_not_found_for_unknown_account() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/9999/balance")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "account 9999 does not exist");
}
