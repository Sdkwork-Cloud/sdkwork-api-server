use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{gateway_auth_subject_from_request_context, GatewayRequestContext};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldRecord, AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType, PricingPlanRecord,
    PricingRateRecord, RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"billing-portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Billing Portal User\"}",
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
    let workspace_response = app
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

    assert_eq!(workspace_response.status(), StatusCode::OK);
    read_json(workspace_response).await
}

fn workspace_request_context(workspace: &Value) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace["tenant"]["id"].as_str().unwrap().to_owned(),
        project_id: workspace["project"]["id"].as_str().unwrap().to_owned(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }
}

async fn seed_portal_workspace_canonical_billing_fixture(
    store: &SqliteAdminStore,
    workspace: &Value,
) {
    let subject = gateway_auth_subject_from_request_context(&workspace_request_context(workspace));

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
    let active_lot = AccountBenefitLotRecord::new(
        8001,
        subject.tenant_id,
        subject.organization_id,
        account.account_id,
        subject.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(200.0)
    .with_remaining_quantity(160.0)
    .with_held_quantity(10.0)
    .with_created_at_ms(11)
    .with_updated_at_ms(11);
    let expired_lot = AccountBenefitLotRecord::new(
        8002,
        subject.tenant_id,
        subject.organization_id,
        account.account_id,
        subject.user_id,
        AccountBenefitType::PromoCredit,
    )
    .with_source_type(AccountBenefitSourceType::Coupon)
    .with_original_quantity(40.0)
    .with_remaining_quantity(40.0)
    .with_expires_at_ms(Some(1))
    .with_status(AccountBenefitLotStatus::Expired)
    .with_created_at_ms(12)
    .with_updated_at_ms(12);
    let hold = AccountHoldRecord::new(
        8101,
        subject.tenant_id,
        subject.organization_id,
        account.account_id,
        subject.user_id,
        6001,
    )
    .with_status(AccountHoldStatus::Captured)
    .with_estimated_quantity(10.0)
    .with_captured_quantity(10.0)
    .with_expires_at_ms(20)
    .with_created_at_ms(13)
    .with_updated_at_ms(13);
    let settlement = RequestSettlementRecord::new(
        8301,
        subject.tenant_id,
        subject.organization_id,
        6001,
        account.account_id,
        subject.user_id,
    )
    .with_hold_id(Some(8101))
    .with_status(RequestSettlementStatus::Captured)
    .with_estimated_credit_hold(10.0)
    .with_captured_credit_amount(10.0)
    .with_provider_cost_amount(5.0)
    .with_retail_charge_amount(10.0)
    .with_settled_at_ms(14)
    .with_created_at_ms(14)
    .with_updated_at_ms(14);
    let pricing_plan = PricingPlanRecord::new(
        9101,
        subject.tenant_id,
        subject.organization_id,
        "workspace-retail",
        1,
    )
    .with_display_name("Workspace Retail")
    .with_status("active")
    .with_created_at_ms(15)
    .with_updated_at_ms(15);
    let pricing_rate = PricingRateRecord::new(
        9201,
        subject.tenant_id,
        subject.organization_id,
        9101,
        "token.input",
    )
    .with_model_code(Some("gpt-4.1".to_owned()))
    .with_provider_code(Some("provider-openrouter".to_owned()))
    .with_quantity_step(1000.0)
    .with_unit_price(0.25)
    .with_created_at_ms(16);

    let foreign_account = AccountRecord::new(
        7999,
        subject.tenant_id + 1,
        subject.organization_id + 1,
        subject.user_id + 1,
        AccountType::Primary,
    )
    .with_status(AccountStatus::Active)
    .with_created_at_ms(21)
    .with_updated_at_ms(21);
    let foreign_lot = AccountBenefitLotRecord::new(
        8999,
        foreign_account.tenant_id,
        foreign_account.organization_id,
        foreign_account.account_id,
        foreign_account.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(999.0)
    .with_remaining_quantity(999.0)
    .with_created_at_ms(22)
    .with_updated_at_ms(22);
    let foreign_settlement = RequestSettlementRecord::new(
        8399,
        foreign_account.tenant_id,
        foreign_account.organization_id,
        6999,
        foreign_account.account_id,
        foreign_account.user_id,
    )
    .with_status(RequestSettlementStatus::Captured)
    .with_captured_credit_amount(50.0)
    .with_retail_charge_amount(50.0)
    .with_created_at_ms(23)
    .with_updated_at_ms(23);
    let workspace_capture_ledger_entry = AccountLedgerEntryRecord::new(
        8401,
        subject.tenant_id,
        subject.organization_id,
        account.account_id,
        subject.user_id,
        AccountLedgerEntryType::SettlementCapture,
    )
    .with_request_id(Some(6001))
    .with_hold_id(Some(8101))
    .with_quantity(10.0)
    .with_amount(10.0)
    .with_created_at_ms(14);
    let workspace_capture_ledger_allocation = AccountLedgerAllocationRecord::new(
        8501,
        subject.tenant_id,
        subject.organization_id,
        8401,
        8001,
    )
    .with_quantity_delta(-10.0)
    .with_created_at_ms(14);
    let foreign_ledger_entry = AccountLedgerEntryRecord::new(
        8499,
        foreign_account.tenant_id,
        foreign_account.organization_id,
        foreign_account.account_id,
        foreign_account.user_id,
        AccountLedgerEntryType::SettlementCapture,
    )
    .with_request_id(Some(6999))
    .with_quantity(50.0)
    .with_amount(50.0)
    .with_created_at_ms(23);
    let foreign_ledger_allocation = AccountLedgerAllocationRecord::new(
        8599,
        foreign_account.tenant_id,
        foreign_account.organization_id,
        8499,
        8999,
    )
    .with_quantity_delta(-50.0)
    .with_created_at_ms(23);
    let foreign_plan = PricingPlanRecord::new(
        9199,
        foreign_account.tenant_id,
        foreign_account.organization_id,
        "foreign-retail",
        1,
    )
    .with_display_name("Foreign Retail")
    .with_status("active")
    .with_created_at_ms(24)
    .with_updated_at_ms(24);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&active_lot).await.unwrap();
    store
        .insert_account_benefit_lot(&expired_lot)
        .await
        .unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();
    store
        .insert_account_ledger_entry_record(&workspace_capture_ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&workspace_capture_ledger_allocation)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&pricing_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .unwrap();

    store.insert_account_record(&foreign_account).await.unwrap();
    store
        .insert_account_benefit_lot(&foreign_lot)
        .await
        .unwrap();
    store
        .insert_request_settlement_record(&foreign_settlement)
        .await
        .unwrap();
    store
        .insert_account_ledger_entry_record(&foreign_ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&foreign_ledger_allocation)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&foreign_plan)
        .await
        .unwrap();
}

#[tokio::test]
async fn portal_billing_account_and_balance_views_are_workspace_scoped() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    seed_portal_workspace_canonical_billing_fixture(&store, &workspace).await;

    let account = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(account.status(), StatusCode::OK);
    let account_json = read_json(account).await;
    assert_eq!(account_json["account"]["account_id"], 7001);
    assert_eq!(account_json["available_balance"], 150.0);
    assert_eq!(account_json["held_balance"], 10.0);
    assert_eq!(account_json["consumed_balance"], 40.0);
    assert_eq!(account_json["grant_balance"], 240.0);
    assert_eq!(account_json["active_lot_count"], 1);

    let balance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account/balance")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(balance.status(), StatusCode::OK);
    let balance_json = read_json(balance).await;
    assert_eq!(balance_json["account_id"], 7001);
    assert_eq!(balance_json["available_balance"], 150.0);
    assert_eq!(balance_json["held_balance"], 10.0);
    assert_eq!(balance_json["active_lot_count"], 1);
    assert_eq!(balance_json["lots"].as_array().unwrap().len(), 1);
    assert_eq!(balance_json["lots"][0]["lot_id"], 8001);

    let lots = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account/benefit-lots")
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
        .all(|lot| lot["account_id"] == 7001));
}

#[tokio::test]
async fn portal_billing_hold_settlement_and_pricing_views_are_workspace_scoped() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    seed_portal_workspace_canonical_billing_fixture(&store, &workspace).await;

    let holds = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account/holds")
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
    assert_eq!(holds_json[0]["account_id"], 7001);

    let settlements = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account/request-settlements")
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
    assert_eq!(settlements_json[0]["account_id"], 7001);

    let plans = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/pricing-plans")
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

    let rates = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/pricing-rates")
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
}

#[tokio::test]
async fn portal_billing_account_ledger_view_is_workspace_scoped() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    seed_portal_workspace_canonical_billing_fixture(&store, &workspace).await;

    let ledger = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account/ledger")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ledger.status(), StatusCode::OK);
    let ledger_json = read_json(ledger).await;
    assert_eq!(ledger_json.as_array().unwrap().len(), 1);
    assert_eq!(ledger_json[0]["entry"]["ledger_entry_id"], 8401);
    assert_eq!(ledger_json[0]["entry"]["account_id"], 7001);
    assert_eq!(ledger_json[0]["entry"]["entry_type"], "settlement_capture");
    assert_eq!(ledger_json[0]["allocations"][0]["lot_id"], 8001);
    assert_eq!(ledger_json[0]["allocations"][0]["quantity_delta"], -10.0);
}

#[tokio::test]
async fn portal_billing_account_history_aggregates_workspace_finance_timeline() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    seed_portal_workspace_canonical_billing_fixture(&store, &workspace).await;

    let history = app
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

    assert_eq!(history.status(), StatusCode::OK);
    let history_json = read_json(history).await;
    assert_eq!(history_json["account"]["account_id"], 7001);
    assert_eq!(history_json["balance"]["account_id"], 7001);
    assert_eq!(history_json["balance"]["available_balance"], 150.0);
    assert_eq!(history_json["benefit_lots"].as_array().unwrap().len(), 2);
    assert_eq!(history_json["holds"].as_array().unwrap().len(), 1);
    assert_eq!(
        history_json["request_settlements"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(history_json["ledger"].as_array().unwrap().len(), 1);
    assert_eq!(history_json["ledger"][0]["entry"]["ledger_entry_id"], 8401);
    assert_eq!(history_json["ledger_entries"].as_array().unwrap().len(), 1);
    assert_eq!(history_json["ledger_entries"][0]["ledger_entry_id"], 8401);
    assert_eq!(
        history_json["ledger_entries"][0]["entry_type"],
        "settlement_capture"
    );
    assert_eq!(
        history_json["ledger_allocations"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        history_json["ledger_allocations"][0]["ledger_entry_id"],
        8401
    );
    assert_eq!(history_json["ledger_allocations"][0]["lot_id"], 8001);
    assert_eq!(
        history_json["ledger_allocations"][0]["quantity_delta"],
        -10.0
    );
}

#[tokio::test]
async fn portal_billing_pricing_reads_auto_activate_due_planned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let subject = gateway_auth_subject_from_request_context(&workspace_request_context(&workspace));

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
    let active_plan = PricingPlanRecord::new(
        9101,
        subject.tenant_id,
        subject.organization_id,
        "workspace-retail",
        1,
    )
    .with_display_name("Workspace Retail")
    .with_currency_code("USD")
    .with_credit_unit_code("credit")
    .with_status("active")
    .with_effective_from_ms(1)
    .with_created_at_ms(11)
    .with_updated_at_ms(11);
    let due_planned_plan = PricingPlanRecord::new(
        9102,
        subject.tenant_id,
        subject.organization_id,
        "workspace-retail",
        2,
    )
    .with_display_name("Workspace Retail v2")
    .with_currency_code("USD")
    .with_credit_unit_code("credit")
    .with_status("planned")
    .with_effective_from_ms(2)
    .with_created_at_ms(12)
    .with_updated_at_ms(12);
    let future_planned_plan = PricingPlanRecord::new(
        9103,
        subject.tenant_id,
        subject.organization_id,
        "workspace-retail",
        3,
    )
    .with_display_name("Workspace Retail Future")
    .with_currency_code("USD")
    .with_credit_unit_code("credit")
    .with_status("planned")
    .with_effective_from_ms(4_102_444_800_000)
    .with_created_at_ms(13)
    .with_updated_at_ms(13);
    let active_rate = PricingRateRecord::new(
        9201,
        subject.tenant_id,
        subject.organization_id,
        9101,
        "token.input",
    )
    .with_charge_unit("input_token")
    .with_pricing_method("per_unit")
    .with_quantity_step(1000.0)
    .with_unit_price(0.25)
    .with_status("active")
    .with_created_at_ms(14)
    .with_updated_at_ms(14);
    let due_planned_rate = PricingRateRecord::new(
        9202,
        subject.tenant_id,
        subject.organization_id,
        9102,
        "token.input",
    )
    .with_charge_unit("input_token")
    .with_pricing_method("per_unit")
    .with_quantity_step(1000.0)
    .with_unit_price(0.28)
    .with_status("planned")
    .with_created_at_ms(15)
    .with_updated_at_ms(15);
    let future_planned_rate = PricingRateRecord::new(
        9203,
        subject.tenant_id,
        subject.organization_id,
        9103,
        "token.input",
    )
    .with_charge_unit("input_token")
    .with_pricing_method("per_unit")
    .with_quantity_step(1000.0)
    .with_unit_price(0.31)
    .with_status("planned")
    .with_created_at_ms(16)
    .with_updated_at_ms(16);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&due_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&due_planned_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_planned_rate)
        .await
        .unwrap();

    let rates = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(rates.status(), StatusCode::OK);
    let rates_json = read_json(rates).await;
    assert_eq!(rates_json.as_array().unwrap().len(), 3);
    assert_eq!(rates_json[0]["status"], "archived");
    assert_eq!(rates_json[1]["status"], "active");
    assert_eq!(rates_json[2]["status"], "planned");

    let plans = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(plans.status(), StatusCode::OK);
    let plans_json = read_json(plans).await;
    assert_eq!(plans_json.as_array().unwrap().len(), 3);
    assert_eq!(plans_json[0]["status"], "archived");
    assert_eq!(plans_json[1]["status"], "active");
    assert_eq!(plans_json[2]["status"], "planned");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    let activated_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9103)
        .unwrap();
    assert_eq!(archived_plan.status, "archived");
    assert_eq!(activated_plan.status, "active");
    assert_eq!(still_future_plan.status, "planned");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    let activated_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9203)
        .unwrap();
    assert_eq!(archived_rate.status, "archived");
    assert_eq!(activated_rate.status, "active");
    assert_eq!(still_future_rate.status, "planned");
}

#[tokio::test]
async fn portal_billing_account_returns_not_found_when_workspace_account_is_missing() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/account")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "workspace commercial account is not provisioned"
    );
}
