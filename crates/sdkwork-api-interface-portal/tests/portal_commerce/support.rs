use super::*;

pub(super) async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

pub(super) fn portal_lab_app(pool: SqlitePool) -> axum::Router {
    sdkwork_api_interface_portal::portal_router_with_state(
        sdkwork_api_interface_portal::PortalApiState::new(pool)
            .with_payment_simulation_enabled(true),
    )
}

pub(super) async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
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

pub(super) async fn portal_workspace(app: axum::Router, token: &str) -> Value {
    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await
}

pub(super) fn workspace_request_context(workspace: &Value) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace["tenant"]["id"].as_str().unwrap().to_owned(),
        project_id: workspace["project"]["id"].as_str().unwrap().to_owned(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
    }
}

pub(super) async fn seed_portal_workspace_commercial_account(
    store: &SqliteAdminStore,
    workspace: &Value,
) -> AccountRecord {
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

    store.insert_account_record(&account).await.unwrap();
    account
}

pub(super) async fn create_portal_recharge_order(
    app: axum::Router,
    token: &str,
    body_json: &str,
) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body_json.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await["order_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub(super) async fn apply_portal_payment_event(
    app: axum::Router,
    token: &str,
    order_id: &str,
    body_json: &str,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(&format!(
                "/portal/commerce/orders/{order_id}/payment-events"
            ))
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(body_json.to_owned()))
            .unwrap(),
    )
    .await
    .unwrap()
}

pub(super) async fn seed_portal_recharge_capacity_fixture(pool: &SqlitePool, project_id: &str) {
    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(pool)
    .await
    .unwrap();
}

pub(super) async fn seed_portal_payment_method(
    store: &SqliteAdminStore,
    payment_method: &PaymentMethodRecord,
) -> PaymentMethodRecord {
    store.upsert_payment_method(payment_method).await.unwrap()
}

pub(super) async fn seed_portal_payment_attempt(
    store: &SqliteAdminStore,
    payment_attempt: &CommercePaymentAttemptRecord,
) -> CommercePaymentAttemptRecord {
    store
        .upsert_commerce_payment_attempt(payment_attempt)
        .await
        .unwrap()
}

pub(super) async fn seed_marketing_catalog_coupon(store: &SqliteAdminStore) {
    let template = CouponTemplateRecord::new(
        "template_launch20",
        "launch20",
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name("Launch 20")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_restriction(CouponRestrictionSpec::new(MarketingSubjectScope::Project))
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff).with_discount_percent(Some(20)),
    )
    .with_created_at_ms(1_710_000_000_000)
    .with_updated_at_ms(1_710_000_000_000);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new("campaign_launch20", "template_launch20")
        .with_display_name("Launch Campaign")
        .with_status(MarketingCampaignStatus::Active)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let budget = CampaignBudgetRecord::new("budget_launch20", "campaign_launch20")
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(5_000)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_campaign_budget_record(&budget).await.unwrap();

    let code = CouponCodeRecord::new("code_launch20", "template_launch20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available)
        .with_created_at_ms(1_710_000_000_000)
        .with_updated_at_ms(1_710_000_000_000);
    store.insert_coupon_code_record(&code).await.unwrap();
}

pub(super) async fn seed_pricing_plan(
    store: &SqliteAdminStore,
    pricing_plan_id: u64,
    plan_code: &str,
    plan_version: u64,
    status: &str,
) {
    let pricing_plan = sdkwork_api_domain_billing::PricingPlanRecord::new(
        pricing_plan_id,
        1001,
        2002,
        plan_code,
        plan_version,
    )
    .with_display_name(format!("{plan_code} v{plan_version}"))
    .with_status(status.to_owned())
    .with_effective_from_ms(1_710_000_000_000)
    .with_created_at_ms(1_710_000_000_000 + pricing_plan_id)
    .with_updated_at_ms(1_710_000_000_000 + pricing_plan_id);
    AccountKernelStore::insert_pricing_plan_record(store, &pricing_plan)
        .await
        .unwrap();
}
