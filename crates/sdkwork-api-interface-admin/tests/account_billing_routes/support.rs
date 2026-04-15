use super::*;

pub(super) async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) async fn memory_pool() -> SqlitePool {
    let pool = sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap();
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    sdkwork_api_app_identity::upsert_admin_user(
        &store,
        Some("admin_local_default"),
        "admin@sdkwork.local",
        "Admin Operator",
        Some("ChangeMe123!"),
        Some(sdkwork_api_domain_identity::AdminUserRole::SuperAdmin),
        true,
    )
    .await
    .unwrap();
    pool
}

pub(super) async fn login_token(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub(super) async fn seed_canonical_billing_fixture(store: &SqliteAdminStore) {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_status(AccountStatus::Active)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let active_credit_lot =
        AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(120.0)
            .with_remaining_quantity(95.0)
            .with_held_quantity(5.0)
            .with_priority(20)
            .with_created_at_ms(11)
            .with_updated_at_ms(11);
    let expired_promo_lot = AccountBenefitLotRecord::new(
        8002,
        1001,
        2002,
        7001,
        9001,
        AccountBenefitType::PromoCredit,
    )
    .with_source_type(AccountBenefitSourceType::Coupon)
    .with_original_quantity(30.0)
    .with_remaining_quantity(30.0)
    .with_expires_at_ms(Some(1))
    .with_status(AccountBenefitLotStatus::Expired)
    .with_created_at_ms(12)
    .with_updated_at_ms(12);
    let hold = AccountHoldRecord::new(8101, 1001, 2002, 7001, 9001, 6001)
        .with_status(AccountHoldStatus::Captured)
        .with_estimated_quantity(5.0)
        .with_captured_quantity(5.0)
        .with_expires_at_ms(20)
        .with_created_at_ms(13)
        .with_updated_at_ms(13);
    let settlement = RequestSettlementRecord::new(8301, 1001, 2002, 6001, 7001, 9001)
        .with_hold_id(Some(8101))
        .with_status(RequestSettlementStatus::Captured)
        .with_estimated_credit_hold(5.0)
        .with_captured_credit_amount(5.0)
        .with_provider_cost_amount(2.5)
        .with_retail_charge_amount(5.0)
        .with_settled_at_ms(14)
        .with_created_at_ms(14)
        .with_updated_at_ms(14);
    let capture_ledger_entry = AccountLedgerEntryRecord::new(
        8401,
        1001,
        2002,
        7001,
        9001,
        AccountLedgerEntryType::SettlementCapture,
    )
    .with_request_id(Some(6001))
    .with_hold_id(Some(8101))
    .with_quantity(5.0)
    .with_amount(5.0)
    .with_created_at_ms(14);
    let capture_ledger_allocation =
        AccountLedgerAllocationRecord::new(8501, 1001, 2002, 8401, 8001)
            .with_quantity_delta(-5.0)
            .with_created_at_ms(14);
    let refund_ledger_entry =
        AccountLedgerEntryRecord::new(8402, 1001, 2002, 7001, 9001, AccountLedgerEntryType::Refund)
            .with_request_id(Some(6001))
            .with_hold_id(Some(8101))
            .with_quantity(2.0)
            .with_amount(2.0)
            .with_created_at_ms(15);
    let refund_ledger_allocation = AccountLedgerAllocationRecord::new(8502, 1001, 2002, 8402, 8001)
        .with_quantity_delta(2.0)
        .with_created_at_ms(15);
    let pricing_plan = PricingPlanRecord::new(9101, 1001, 2002, "default-retail", 3)
        .with_display_name("Default Retail v3")
        .with_status("active")
        .with_effective_from_ms(10)
        .with_effective_to_ms(Some(100))
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let pricing_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_capability_code(Some("responses".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000.0)
        .with_unit_price(0.25)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail text input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(16)
        .with_updated_at_ms(16);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&active_credit_lot)
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(&expired_promo_lot)
        .await
        .unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();
    store
        .insert_account_ledger_entry_record(&capture_ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&capture_ledger_allocation)
        .await
        .unwrap();
    store
        .insert_account_ledger_entry_record(&refund_ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&refund_ledger_allocation)
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
}

pub(super) async fn seed_commerce_audit_fixture(store: &SqliteAdminStore) {
    let template = CouponTemplateRecord::new(
        "template-launch20",
        "launch20",
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name("Spring launch 20%")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::SharedCode)
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff).with_discount_percent(Some(20)),
    )
    .with_restriction(
        CouponRestrictionSpec::new(MarketingSubjectScope::Project)
            .with_stacking_policy(MarketingStackingPolicy::Exclusive)
            .with_eligible_target_kinds(vec!["recharge_pack".to_owned()]),
    )
    .with_created_at_ms(80)
    .with_updated_at_ms(80);
    let campaign = MarketingCampaignRecord::new("campaign-launch20", "template-launch20")
        .with_display_name("Spring launch")
        .with_status(MarketingCampaignStatus::Active)
        .with_start_at_ms(Some(80))
        .with_created_at_ms(80)
        .with_updated_at_ms(80);
    let code = CouponCodeRecord::new("code-launch20", "template-launch20", "SPRING20")
        .with_status(CouponCodeStatus::Redeemed)
        .with_created_at_ms(90)
        .with_updated_at_ms(210);
    let reservation = CouponReservationRecord::new(
        "reservation-order-refunded",
        "code-launch20",
        MarketingSubjectScope::Project,
        "project-a",
        600,
    )
    .with_status(CouponReservationStatus::Confirmed)
    .with_budget_reserved_minor(800)
    .with_created_at_ms(95)
    .with_updated_at_ms(160);
    let redemption = CouponRedemptionRecord::new(
        "redemption-order-refunded",
        "reservation-order-refunded",
        "code-launch20",
        "template-launch20",
        160,
    )
    .with_status(CouponRedemptionStatus::PartiallyRolledBack)
    .with_subsidy_amount_minor(800)
    .with_order_id(Some("order-refunded".to_owned()))
    .with_payment_event_id(Some("payevt-order-refunded-settled".to_owned()))
    .with_updated_at_ms(310);
    let rollback = CouponRollbackRecord::new(
        "rollback-order-refunded",
        "redemption-order-refunded",
        CouponRollbackType::Refund,
        320,
    )
    .with_status(CouponRollbackStatus::Completed)
    .with_restored_budget_minor(800)
    .with_restored_inventory_count(1)
    .with_updated_at_ms(330);
    let refunded_order = CommerceOrderRecord::new(
        "order-refunded",
        "project-a",
        "user-a",
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        3_200,
        "$40.00",
        "$32.00",
        100_000,
        0,
        "refunded",
        "live",
        100,
    )
    .with_applied_coupon_code_option(Some("SPRING20".to_owned()))
    .with_coupon_reservation_id_option(Some("reservation-order-refunded".to_owned()))
    .with_coupon_redemption_id_option(Some("redemption-order-refunded".to_owned()))
    .with_marketing_campaign_id_option(Some("campaign-launch20".to_owned()))
    .with_subsidy_amount_minor(800)
    .with_updated_at_ms(300);
    let pending_order = CommerceOrderRecord::new(
        "order-pending",
        "project-b",
        "user-b",
        "subscription_plan",
        "growth",
        "Growth",
        7_900,
        7_900,
        "$79.00",
        "$79.00",
        100_000,
        0,
        "pending_payment",
        "live",
        190,
    )
    .with_updated_at_ms(200);
    let older_order = CommerceOrderRecord::new(
        "order-old",
        "project-c",
        "user-c",
        "coupon_redemption",
        "TEAMREADY",
        "TEAMREADY",
        0,
        0,
        "$0.00",
        "$0.00",
        0,
        25_000,
        "fulfilled",
        "live",
        50,
    )
    .with_updated_at_ms(60);

    let settled_event = CommercePaymentEventRecord::new(
        "payevt-order-refunded-settled",
        "order-refunded",
        "project-a",
        "user-a",
        "stripe",
        "stripe:evt_stripe_1",
        "settled",
        "{\"event_type\":\"settled\"}",
        150,
    )
    .with_provider_event_id(Some("evt_stripe_1".to_owned()))
    .with_processing_status(CommercePaymentEventProcessingStatus::Processed)
    .with_processed_at_ms(Some(160))
    .with_order_status_after(Some("fulfilled".to_owned()));
    let failed_event = CommercePaymentEventRecord::new(
        "payevt-order-refunded-failed",
        "order-refunded",
        "project-a",
        "user-a",
        "stripe",
        "stripe:evt_stripe_fail",
        "failed",
        "{\"event_type\":\"failed\"}",
        120,
    )
    .with_provider_event_id(Some("evt_stripe_fail".to_owned()))
    .with_processing_status(CommercePaymentEventProcessingStatus::Rejected)
    .with_processing_message(Some("provider signature mismatch".to_owned()));

    AdminStore::insert_coupon_template_record(store, &template)
        .await
        .unwrap();
    AdminStore::insert_marketing_campaign_record(store, &campaign)
        .await
        .unwrap();
    AdminStore::insert_coupon_code_record(store, &code)
        .await
        .unwrap();
    AdminStore::insert_coupon_reservation_record(store, &reservation)
        .await
        .unwrap();
    AdminStore::insert_coupon_redemption_record(store, &redemption)
        .await
        .unwrap();
    AdminStore::insert_coupon_rollback_record(store, &rollback)
        .await
        .unwrap();
    store.insert_commerce_order(&refunded_order).await.unwrap();
    store.insert_commerce_order(&pending_order).await.unwrap();
    store.insert_commerce_order(&older_order).await.unwrap();
    store
        .upsert_commerce_payment_event(&settled_event)
        .await
        .unwrap();
    store
        .upsert_commerce_payment_event(&failed_event)
        .await
        .unwrap();
}
