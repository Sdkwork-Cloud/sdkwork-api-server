use super::*;

#[tokio::test]
async fn preview_quote_uses_marketing_coupon_discount() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "LAUNCH20", 20).await;

    let quote = preview_portal_commerce_quote(
        &store,
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("launch20".to_owned()),
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("marketing coupon quote");

    assert_eq!(quote.payable_price_cents, 3_200);
    assert_eq!(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.code.as_str()),
        Some("LAUNCH20")
    );
    assert_eq!(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.source.as_str()),
        Some("marketing")
    );
}

#[tokio::test]
async fn preview_quote_rejects_coupon_when_target_kind_is_not_eligible() {
    let store = build_store().await;
    seed_percent_off_coupon_for_targets(&store, "PLANONLY20", 20, &["subscription_plan"]).await;

    let error = preview_portal_commerce_quote(
        &store,
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("planonly20".to_owned()),
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect_err("coupon should reject ineligible target kind");

    assert!(
        error
            .to_string()
            .contains("coupon PLANONLY20 is not eligible: target_kind_not_eligible"),
        "unexpected error: {error}"
    );
}
