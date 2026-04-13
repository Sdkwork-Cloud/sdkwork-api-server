use super::*;
use sdkwork_api_app_billing::{
    issue_commerce_order_credits, IssueCommerceOrderCreditsInput, QuotaPolicy,
};
use sdkwork_api_app_identity::{gateway_auth_subject_from_request_context, GatewayRequestContext};
use sdkwork_api_domain_commerce::CommercePaymentAttemptRecord;
use sdkwork_api_domain_tenant::{Project, Tenant};

const DEMO_TENANT_ID: &str = "tenant_local_demo";
const DEMO_PROJECT_ID: &str = "project_local_demo";
const DEMO_SCOPE_KEY_HASH: &str = "portal_workspace_scope";

#[tokio::test]
async fn admin_commerce_refund_route_refunds_manual_recharge_order_and_updates_billing_checkpoint()
{
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let account = seed_demo_workspace_commercial_account(&store).await;
    let order = seed_manual_recharge_order_with_issued_credits(&store, account.account_id).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let refund = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/commerce/orders/{}/refunds", order.order_id))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "reason": "customer_request",
                        "idempotency_key": "refund-admin-recharge-order-001"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refund.status(), StatusCode::OK);
    let refund_json = read_json(refund).await;
    assert_eq!(refund_json["order_id"], order.order_id);
    assert_eq!(refund_json["provider"], "manual_lab");
    assert_eq!(refund_json["payment_method_id"], "manual_lab");
    assert_eq!(refund_json["status"], "succeeded");
    assert_eq!(refund_json["amount_minor"], order.payable_price_cents);
    assert_eq!(refund_json["reason"], "customer_request");

    let refund_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/commerce/orders/{}/refunds", order.order_id))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refund_list.status(), StatusCode::OK);
    let refund_list_json = read_json(refund_list).await;
    assert_eq!(refund_list_json.as_array().unwrap().len(), 1);
    assert_eq!(
        refund_list_json[0]["idempotency_key"],
        "refund-admin-recharge-order-001"
    );

    let stored_order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.order_id == order.order_id)
        .unwrap();
    assert_eq!(stored_order.status, "refunded");
    assert_eq!(stored_order.settlement_status, "refunded");
    assert_eq!(stored_order.refundable_amount_minor, 0);
    assert_eq!(
        stored_order.refunded_amount_minor,
        order.payable_price_cents
    );

    let checkpoint = store
        .find_account_commerce_reconciliation_state(account.account_id, DEMO_PROJECT_ID)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint.last_order_id, order.order_id);
    assert_eq!(checkpoint.project_id, DEMO_PROJECT_ID);

    let quota_policy = store
        .list_quota_policies_for_project(DEMO_PROJECT_ID)
        .await
        .unwrap()
        .into_iter()
        .find(|policy| policy.enabled)
        .unwrap();
    assert_eq!(quota_policy.max_units, 0);
}

#[tokio::test]
async fn admin_commerce_reconciliation_run_route_records_missing_checkout_session_discrepancy() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_reconciliation_attempt_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/commerce/reconciliation-runs")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "provider": "stripe",
                        "scope_started_at_ms": 100,
                        "scope_ended_at_ms": 200
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::OK);
    let create_json = read_json(create).await;
    assert_eq!(create_json["provider"], "stripe");
    assert_eq!(create_json["status"], "completed_with_diff");

    let summary_json: Value =
        serde_json::from_str(create_json["summary_json"].as_str().unwrap()).unwrap();
    assert_eq!(summary_json["checked_attempts"], 1);
    assert_eq!(summary_json["checked_refunds"], 0);
    assert_eq!(summary_json["discrepancy_count"], 1);

    let reconciliation_run_id = create_json["reconciliation_run_id"].as_str().unwrap();
    let runs = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/reconciliation-runs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(runs.status(), StatusCode::OK);
    let runs_json = read_json(runs).await;
    assert_eq!(runs_json.as_array().unwrap().len(), 1);
    assert_eq!(runs_json[0]["reconciliation_run_id"], reconciliation_run_id);

    let items = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/admin/commerce/reconciliation-runs/{reconciliation_run_id}/items"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(items.status(), StatusCode::OK);
    let items_json = read_json(items).await;
    assert_eq!(items_json.as_array().unwrap().len(), 1);
    assert_eq!(
        items_json[0]["discrepancy_type"],
        "missing_checkout_session"
    );
    assert_eq!(
        items_json[0]["payment_attempt_id"],
        "attempt-stripe-missing-checkout"
    );
    assert_eq!(items_json[0]["order_id"], "order-reconcile-001");
    let detail_json: Value =
        serde_json::from_str(items_json[0]["detail_json"].as_str().unwrap()).unwrap();
    assert_eq!(
        detail_json["message"],
        "payment attempt is missing provider_checkout_session_id"
    );
}

fn demo_workspace_request_context() -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: DEMO_TENANT_ID.to_owned(),
        project_id: DEMO_PROJECT_ID.to_owned(),
        environment: "portal".to_owned(),
        api_key_hash: DEMO_SCOPE_KEY_HASH.to_owned(),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }
}

async fn seed_demo_workspace_commercial_account(store: &SqliteAdminStore) -> AccountRecord {
    if store.find_tenant(DEMO_TENANT_ID).await.unwrap().is_none() {
        store
            .insert_tenant(&Tenant::new(DEMO_TENANT_ID, "Local Demo Tenant"))
            .await
            .unwrap();
    }
    if store.find_project(DEMO_PROJECT_ID).await.unwrap().is_none() {
        store
            .insert_project(&Project::new(
                DEMO_TENANT_ID,
                DEMO_PROJECT_ID,
                "Local Demo Project",
            ))
            .await
            .unwrap();
    }

    let subject = gateway_auth_subject_from_request_context(&demo_workspace_request_context());
    let account = AccountRecord::new(
        7001,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        AccountType::Primary,
    )
    .with_status(AccountStatus::Active)
    .with_currency_code("USD")
    .with_credit_unit_code("credit")
    .with_created_at_ms(10)
    .with_updated_at_ms(10);

    store.insert_account_record(&account).await.unwrap();
    account
}

async fn seed_manual_recharge_order_with_issued_credits(
    store: &SqliteAdminStore,
    account_id: u64,
) -> CommerceOrderRecord {
    let order = CommerceOrderRecord::new(
        "order-admin-recharge-001",
        DEMO_PROJECT_ID,
        "portal-user-demo",
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        4_000,
        "$40.00",
        "$40.00",
        100_000,
        0,
        "fulfilled",
        "live",
        100,
    )
    .with_payment_method_id_option(Some("manual_lab".to_owned()))
    .with_updated_at_ms(120);

    store.insert_commerce_order(&order).await.unwrap();
    store
        .insert_quota_policy(&QuotaPolicy::new(
            "quota-project-local-demo",
            DEMO_PROJECT_ID,
            100_000,
        ))
        .await
        .unwrap();
    issue_commerce_order_credits(
        store,
        IssueCommerceOrderCreditsInput {
            account_id,
            order_id: &order.order_id,
            project_id: &order.project_id,
            target_kind: &order.target_kind,
            granted_quantity: order.granted_units as f64,
            payable_amount: order.payable_price_cents as f64 / 100.0,
            issued_at_ms: order.updated_at_ms,
        },
    )
    .await
    .unwrap();

    order
}

async fn seed_reconciliation_attempt_fixture(store: &SqliteAdminStore) {
    if store.find_tenant(DEMO_TENANT_ID).await.unwrap().is_none() {
        store
            .insert_tenant(&Tenant::new(DEMO_TENANT_ID, "Local Demo Tenant"))
            .await
            .unwrap();
    }
    if store.find_project(DEMO_PROJECT_ID).await.unwrap().is_none() {
        store
            .insert_project(&Project::new(
                DEMO_TENANT_ID,
                DEMO_PROJECT_ID,
                "Local Demo Project",
            ))
            .await
            .unwrap();
    }

    let order = CommerceOrderRecord::new(
        "order-reconcile-001",
        DEMO_PROJECT_ID,
        "portal-user-demo",
        "recharge_pack",
        "pack-50k",
        "Boost 50k",
        2_000,
        2_000,
        "$20.00",
        "$20.00",
        50_000,
        0,
        "fulfilled",
        "live",
        100,
    )
    .with_payment_method_id_option(Some("pm-stripe-main".to_owned()))
    .with_latest_payment_attempt_id_option(Some("attempt-stripe-missing-checkout".to_owned()))
    .with_updated_at_ms(120);
    let payment_attempt = CommercePaymentAttemptRecord::new(
        "attempt-stripe-missing-checkout",
        "order-reconcile-001",
        DEMO_PROJECT_ID,
        "portal-user-demo",
        "pm-stripe-main",
        "stripe",
        "card",
        "idem-attempt-stripe-missing-checkout",
        1,
        2_000,
        "USD",
        150,
    )
    .with_status("succeeded")
    .with_captured_amount_minor(2_000)
    .with_updated_at_ms(160);

    store.insert_commerce_order(&order).await.unwrap();
    store
        .upsert_commerce_payment_attempt(&payment_attempt)
        .await
        .unwrap();
}
