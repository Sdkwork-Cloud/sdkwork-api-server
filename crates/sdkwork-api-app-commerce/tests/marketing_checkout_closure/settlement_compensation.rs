use super::*;

#[tokio::test]
async fn settlement_write_failure_restores_quota_and_membership_side_effects() {
    let (store, pool) = build_store_with_pool().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-8",
        "project-8",
        &PortalCommerceQuoteRequest {
            target_kind: "subscription_plan".to_owned(),
            target_id: "growth".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(120),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    sqlx::query(
        "CREATE TRIGGER fail_fulfilled_order_update
         BEFORE UPDATE ON ai_commerce_orders
         FOR EACH ROW
         WHEN NEW.status = 'fulfilled'
         BEGIN
             SELECT RAISE(ABORT, 'fail final settle write');
         END;",
    )
    .execute(&pool)
    .await
    .expect("create settle fail trigger");

    let error = settle_portal_commerce_order(&store, "user-8", "project-8", &order.order_id)
        .await
        .expect_err("settlement should fail");
    assert!(
        error.to_string().contains("fail final settle write"),
        "unexpected error: {error}"
    );

    let stored_order = AdminStore::list_commerce_orders_for_project(&store, "project-8")
        .await
        .expect("orders for project")
        .into_iter()
        .find(|candidate| candidate.order_id == order.order_id)
        .expect("persisted order");
    assert_eq!(stored_order.status, "pending_payment");
    assert!(
        AdminStore::find_project_membership(&store, "project-8")
            .await
            .expect("project membership lookup")
            .is_none(),
        "membership should be rolled back when order finalization fails"
    );
    assert!(
        AdminStore::list_quota_policies_for_project(&store, "project-8")
            .await
            .expect("quota policies")
            .is_empty(),
        "quota expansion should be rolled back when order finalization fails"
    );
}

#[tokio::test]
async fn refund_write_failure_restores_quota_side_effects() {
    let (store, pool) = build_store_with_pool().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-9",
        "project-9",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(260),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let settled = settle_portal_commerce_order(&store, "user-9", "project-9", &order.order_id)
        .await
        .expect("settled order");
    assert_eq!(settled.status, "fulfilled");

    sqlx::query(
        "CREATE TRIGGER fail_refunded_order_update
         BEFORE UPDATE ON ai_commerce_orders
         FOR EACH ROW
         WHEN NEW.status = 'refunded'
         BEGIN
             SELECT RAISE(ABORT, 'fail final refund write');
         END;",
    )
    .execute(&pool)
    .await
    .expect("create refund fail trigger");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-9",
        "project-9",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: None,
            provider_event_id: None,
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect_err("refund should fail");
    assert!(
        error.to_string().contains("fail final refund write"),
        "unexpected error: {error}"
    );

    let stored_order = AdminStore::list_commerce_orders_for_project(&store, "project-9")
        .await
        .expect("orders for project")
        .into_iter()
        .find(|candidate| candidate.order_id == order.order_id)
        .expect("persisted order");
    assert_eq!(stored_order.status, "fulfilled");

    let quota_policies = AdminStore::list_quota_policies_for_project(&store, "project-9")
        .await
        .expect("quota policies");
    assert_eq!(quota_policies.len(), 1);
    assert_eq!(quota_policies[0].max_units, 100_000);

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "refunded");
    assert_eq!(events[0].processing_status.as_str(), "failed");
    assert_eq!(events[0].order_status_after.as_deref(), Some("fulfilled"));
}

#[tokio::test]
async fn refund_write_failure_restores_coupon_rollback_side_effects() {
    let (store, pool) = build_store_with_pool().await;
    seed_percent_off_coupon(&store, "SAFEBACK20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-10",
        "project-10",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("SAFEBACK20".to_owned()),
            current_remaining_units: Some(1_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-10",
        "project-10",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_safeback20_paid".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect("settled order");
    assert_eq!(settled.status, "fulfilled");

    sqlx::query(
        "CREATE TRIGGER fail_refunded_order_update
         BEFORE UPDATE ON ai_commerce_orders
         FOR EACH ROW
         WHEN NEW.status = 'refunded'
         BEGIN
             SELECT RAISE(ABORT, 'fail final refund write');
         END;",
    )
    .execute(&pool)
    .await
    .expect("create refund fail trigger");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-10",
        "project-10",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_safeback20_refund".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect_err("refund should fail");
    assert!(
        error.to_string().contains("fail final refund write"),
        "unexpected error: {error}"
    );

    let stored_order = AdminStore::list_commerce_orders_for_project(&store, "project-10")
        .await
        .expect("orders for project")
        .into_iter()
        .find(|candidate| candidate.order_id == order.order_id)
        .expect("persisted order");
    assert_eq!(stored_order.status, "fulfilled");

    let quota_policies = AdminStore::list_quota_policies_for_project(&store, "project-10")
        .await
        .expect("quota policies");
    assert_eq!(quota_policies.len(), 1);
    assert_eq!(quota_policies[0].max_units, 100_000);

    let budgets = MarketingStore::list_campaign_budget_records(&store)
        .await
        .expect("campaign budgets");
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].reserved_budget_minor, 0);
    assert_eq!(budgets[0].consumed_budget_minor, 800);

    let coupon_code = MarketingStore::find_coupon_code_record_by_value(&store, "SAFEBACK20")
        .await
        .expect("coupon code lookup")
        .expect("coupon code exists");
    assert_eq!(coupon_code.status, CouponCodeStatus::Redeemed);

    let redemptions = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("redemptions");
    assert_eq!(redemptions.len(), 1);
    assert_eq!(
        redemptions[0].redemption_status,
        CouponRedemptionStatus::Redeemed
    );

    let rollbacks = MarketingStore::list_coupon_rollback_records(&store)
        .await
        .expect("rollback records");
    assert_eq!(rollbacks.len(), 1);
    assert_eq!(rollbacks[0].rollback_status, CouponRollbackStatus::Failed);

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 2);
    let refund_event = events
        .iter()
        .find(|event| event.event_type == "refunded")
        .expect("refunded payment event");
    assert_eq!(refund_event.processing_status.as_str(), "failed");
    assert_eq!(
        refund_event.order_status_after.as_deref(),
        Some("fulfilled")
    );
}

#[tokio::test]
async fn refund_replay_after_coupon_rollback_compensation_succeeds() {
    let (store, pool) = build_store_with_pool().await;
    seed_percent_off_coupon(&store, "RETRY20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-11",
        "project-11",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("RETRY20".to_owned()),
            current_remaining_units: Some(1_500),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-11",
        "project-11",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_retry20_paid".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect("settled order");
    assert_eq!(settled.status, "fulfilled");

    sqlx::query(
        "CREATE TRIGGER fail_refunded_order_update
         BEFORE UPDATE ON ai_commerce_orders
         FOR EACH ROW
         WHEN NEW.status = 'refunded'
         BEGIN
             SELECT RAISE(ABORT, 'fail final refund write');
         END;",
    )
    .execute(&pool)
    .await
    .expect("create refund fail trigger");

    let _ = apply_portal_commerce_payment_event(
        &store,
        "user-11",
        "project-11",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_retry20_refund".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect_err("refund should fail while trigger is active");

    sqlx::query("DROP TRIGGER fail_refunded_order_update;")
        .execute(&pool)
        .await
        .expect("drop refund fail trigger");

    let refunded = apply_portal_commerce_payment_event(
        &store,
        "user-11",
        "project-11",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_retry20_refund".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect("refund replay should succeed");
    assert_eq!(refunded.status, "refunded");

    let budgets = MarketingStore::list_campaign_budget_records(&store)
        .await
        .expect("campaign budgets");
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].reserved_budget_minor, 0);
    assert_eq!(budgets[0].consumed_budget_minor, 0);

    let coupon_code = MarketingStore::find_coupon_code_record_by_value(&store, "RETRY20")
        .await
        .expect("coupon code lookup")
        .expect("coupon code exists");
    assert_eq!(coupon_code.status, CouponCodeStatus::Available);

    let redemptions = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("redemptions");
    assert_eq!(redemptions.len(), 1);
    assert_eq!(
        redemptions[0].redemption_status,
        CouponRedemptionStatus::RolledBack
    );

    let rollbacks = MarketingStore::list_coupon_rollback_records(&store)
        .await
        .expect("rollback records");
    assert_eq!(rollbacks.len(), 1);
    assert_eq!(
        rollbacks[0].rollback_status,
        CouponRollbackStatus::Completed
    );

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 2);
    let refund_event = events
        .iter()
        .find(|event| event.event_type == "refunded")
        .expect("refunded payment event");
    assert_eq!(refund_event.processing_status.as_str(), "processed");
    assert_eq!(refund_event.order_status_after.as_deref(), Some("refunded"));
}
