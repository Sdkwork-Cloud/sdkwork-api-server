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
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
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

async fn seed_marketing_coupon_fixture(
    store: &SqliteAdminStore,
    coupon_template_id: &str,
    template_key: &str,
    display_name: &str,
    campaign_display_name: &str,
    coupon_code_id: &str,
    coupon_code_value: &str,
    benefit: CouponBenefitSpec,
    template_status: CouponTemplateStatus,
    campaign_status: MarketingCampaignStatus,
) {
    let campaign_id = format!("campaign_{template_key}");
    let budget_id = format!("budget_{template_key}");
    let created_at_ms = 1_710_000_000_000;

    let template =
        CouponTemplateRecord::new(coupon_template_id, template_key, benefit.benefit_kind)
            .with_display_name(display_name)
            .with_status(template_status)
            .with_distribution_kind(CouponDistributionKind::UniqueCode)
            .with_restriction(CouponRestrictionSpec::new(MarketingSubjectScope::Project))
            .with_benefit(benefit)
            .with_created_at_ms(created_at_ms)
            .with_updated_at_ms(created_at_ms);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new(&campaign_id, coupon_template_id)
        .with_display_name(campaign_display_name)
        .with_status(campaign_status)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let budget = CampaignBudgetRecord::new(&budget_id, &campaign_id)
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(5_000)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
    store.insert_campaign_budget_record(&budget).await.unwrap();

    let code = CouponCodeRecord::new(coupon_code_id, coupon_template_id, coupon_code_value)
        .with_status(CouponCodeStatus::Available)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
    store.insert_coupon_code_record(&code).await.unwrap();
}

pub(super) async fn seed_marketing_catalog_coupon_code(
    store: &SqliteAdminStore,
    slug: &str,
    code_value: &str,
    campaign_status: MarketingCampaignStatus,
    code_status: CouponCodeStatus,
) {
    let created_at_ms = 1_710_000_000_000;
    let template_id = format!("template_{slug}");
    let campaign_id = format!("campaign_{slug}");
    let budget_id = format!("budget_{slug}");
    let code_id = format!("code_{slug}");

    let template =
        CouponTemplateRecord::new(&template_id, slug, MarketingBenefitKind::PercentageOff)
            .with_display_name(format!("{code_value} Campaign"))
            .with_status(CouponTemplateStatus::Active)
            .with_distribution_kind(CouponDistributionKind::UniqueCode)
            .with_restriction(CouponRestrictionSpec::new(MarketingSubjectScope::Project))
            .with_benefit(
                CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
                    .with_discount_percent(Some(20)),
            )
            .with_created_at_ms(created_at_ms)
            .with_updated_at_ms(created_at_ms);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new(&campaign_id, &template_id)
        .with_display_name(format!("{code_value} Campaign"))
        .with_status(campaign_status)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let budget = CampaignBudgetRecord::new(&budget_id, &campaign_id)
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(5_000)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
    store.insert_campaign_budget_record(&budget).await.unwrap();

    let code = CouponCodeRecord::new(&code_id, &template_id, code_value)
        .with_status(code_status)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(created_at_ms);
    store.insert_coupon_code_record(&code).await.unwrap();
}

pub(super) async fn seed_marketing_catalog_coupon(store: &SqliteAdminStore) {
    seed_marketing_catalog_coupon_code(
        store,
        "launch20",
        "LAUNCH20",
        MarketingCampaignStatus::Active,
        CouponCodeStatus::Available,
    )
    .await;
}

pub(super) async fn seed_marketing_bonus_coupon(store: &SqliteAdminStore) {
    seed_marketing_coupon_fixture(
        store,
        "template_welcome100",
        "welcome100",
        "Welcome 100",
        "Welcome Credits",
        "code_welcome100",
        "WELCOME100",
        CouponBenefitSpec::new(MarketingBenefitKind::GrantUnits).with_grant_units(Some(100)),
        CouponTemplateStatus::Active,
        MarketingCampaignStatus::Active,
    )
    .await;
}

pub(super) async fn seed_inactive_marketing_catalog_coupon(store: &SqliteAdminStore) {
    seed_marketing_coupon_fixture(
        store,
        "template_inactive10",
        "inactive10",
        "Inactive 10",
        "Inactive Campaign",
        "code_inactive10",
        "INACTIVE10",
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff).with_discount_percent(Some(10)),
        CouponTemplateStatus::Active,
        MarketingCampaignStatus::Paused,
    )
    .await;
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
