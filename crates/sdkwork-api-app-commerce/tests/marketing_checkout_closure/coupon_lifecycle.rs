use super::*;

#[tokio::test]
async fn discounted_order_reserves_confirms_and_rolls_back_marketing_coupon() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "LAUNCH20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-1",
        "project-1",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("LAUNCH20".to_owned()),
            current_remaining_units: Some(3_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    assert_eq!(order.status, "pending_payment");

    let reservations = MarketingStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records");
    assert_eq!(reservations.len(), 1);
    assert_eq!(
        reservations[0].reservation_status,
        CouponReservationStatus::Reserved
    );

    let reserved_code = MarketingStore::find_coupon_code_record_by_value(&store, "LAUNCH20")
        .await
        .expect("coupon code lookup")
        .expect("coupon code exists");
    assert_eq!(reserved_code.status, CouponCodeStatus::Reserved);

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-1",
        "project-1",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_launch20_paid".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect("settled order");

    assert_eq!(settled.status, "fulfilled");

    let redemptions = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("redemption records");
    assert_eq!(redemptions.len(), 1);
    assert_eq!(
        redemptions[0].redemption_status,
        CouponRedemptionStatus::Redeemed
    );
    assert_eq!(redemptions[0].subsidy_amount_minor, 800);

    let refunded = apply_portal_commerce_payment_event(
        &store,
        "user-1",
        "project-1",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_launch20_refund".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect("refunded order");

    assert_eq!(refunded.status, "refunded");

    let rollbacks = MarketingStore::list_coupon_rollback_records(&store)
        .await
        .expect("rollback records");
    assert_eq!(rollbacks.len(), 1);
    assert_eq!(
        rollbacks[0].rollback_status,
        CouponRollbackStatus::Completed
    );

    let rolled_back_redemption = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("redemptions after refund");
    assert_eq!(
        rolled_back_redemption[0].redemption_status,
        CouponRedemptionStatus::RolledBack
    );

    let recycled_code = MarketingStore::find_coupon_code_record_by_value(&store, "LAUNCH20")
        .await
        .expect("coupon code lookup after refund")
        .expect("coupon code exists after refund");
    assert_eq!(recycled_code.status, CouponCodeStatus::Available);
}

#[tokio::test]
async fn failed_payment_releases_marketing_coupon_reservation() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "FAILSAFE20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-2",
        "project-2",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("FAILSAFE20".to_owned()),
            current_remaining_units: Some(0),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let failed = apply_portal_commerce_payment_event(
        &store,
        "user-2",
        "project-2",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "failed".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_failsafe20_failed".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect("failed order");

    assert_eq!(failed.status, "failed");

    let reservations = MarketingStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records");
    assert_eq!(reservations.len(), 1);
    assert_eq!(
        reservations[0].reservation_status,
        CouponReservationStatus::Released
    );

    let released_code = MarketingStore::find_coupon_code_record_by_value(&store, "FAILSAFE20")
        .await
        .expect("released coupon code lookup")
        .expect("coupon code exists after failure");
    assert_eq!(released_code.status, CouponCodeStatus::Available);
}

#[tokio::test]
async fn expired_coupon_reservation_is_reclaimed_inline_for_new_order() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "RECLAIM20", 20).await;

    let original_code = MarketingStore::find_coupon_code_record_by_value(&store, "RECLAIM20")
        .await
        .expect("coupon code lookup")
        .expect("coupon code exists");
    let reserved_code = original_code
        .clone()
        .with_status(CouponCodeStatus::Reserved)
        .with_updated_at_ms(10);
    MarketingStore::insert_coupon_code_record(&store, &reserved_code)
        .await
        .expect("persist reserved code");

    let original_budget = MarketingStore::list_campaign_budget_records(&store)
        .await
        .expect("campaign budgets")
        .into_iter()
        .next()
        .expect("campaign budget");
    let reserved_budget = original_budget
        .clone()
        .with_reserved_budget_minor(800)
        .with_updated_at_ms(10);
    MarketingStore::insert_campaign_budget_record(&store, &reserved_budget)
        .await
        .expect("persist reserved budget");

    let expired_reservation = CouponReservationRecord::new(
        "reservation_reclaim_expired",
        reserved_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-12",
        1,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(800)
    .with_created_at_ms(0)
    .with_updated_at_ms(10);
    MarketingStore::insert_coupon_reservation_record(&store, &expired_reservation)
        .await
        .expect("persist expired reservation");

    let order = submit_portal_commerce_order(
        &store,
        "user-12",
        "project-12",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("RECLAIM20".to_owned()),
            current_remaining_units: Some(250),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should succeed after reclaiming expired reservation");

    assert_eq!(order.status, "pending_payment");
    assert!(
        order.coupon_reservation_id.is_some(),
        "new order should hold a fresh coupon reservation"
    );

    let reservations = MarketingStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records");
    assert_eq!(reservations.len(), 2);
    let stale_reservation = reservations
        .iter()
        .find(|reservation| reservation.coupon_reservation_id == "reservation_reclaim_expired")
        .expect("stale reservation");
    assert_eq!(
        stale_reservation.reservation_status,
        CouponReservationStatus::Expired
    );
    let fresh_reservation = reservations
        .iter()
        .find(|reservation| {
            Some(&reservation.coupon_reservation_id) == order.coupon_reservation_id.as_ref()
        })
        .expect("fresh reservation");
    assert_eq!(
        fresh_reservation.reservation_status,
        CouponReservationStatus::Reserved
    );
    assert_eq!(fresh_reservation.budget_reserved_minor, 800);

    let refreshed_code = MarketingStore::find_coupon_code_record_by_value(&store, "RECLAIM20")
        .await
        .expect("coupon code lookup after reclaim")
        .expect("coupon code exists");
    assert_eq!(refreshed_code.status, CouponCodeStatus::Reserved);

    let refreshed_budget = MarketingStore::list_campaign_budget_records(&store)
        .await
        .expect("campaign budgets after reclaim")
        .into_iter()
        .next()
        .expect("campaign budget after reclaim");
    assert_eq!(refreshed_budget.reserved_budget_minor, 800);
}
