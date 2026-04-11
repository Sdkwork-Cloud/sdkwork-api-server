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
    let quote_json = serde_json::to_value(&quote).expect("serialize quote");
    assert_eq!(quote_json["target_kind"], "recharge_pack");
    assert_eq!(quote_json["product_kind"], "recharge_pack");
    assert_eq!(quote_json["quote_kind"], "product_purchase");
    assert_eq!(
        quote_json["pricing_plan_id"],
        "pricing_plan:recharge_pack:pack-100k"
    );
    assert_eq!(quote_json["pricing_plan_version"], 1);
    assert_eq!(
        quote_json["pricing_rate_id"],
        "pricing_rate:recharge_pack:pack-100k:credit.prepaid_pack"
    );
    assert_eq!(quote_json["pricing_metric_code"], "credit.prepaid_pack");
    assert_eq!(quote_json["product_id"], "product:recharge_pack:pack-100k");
    assert_eq!(quote_json["offer_id"], "offer:recharge_pack:pack-100k");
    assert_eq!(
        quote_json["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(quote_json["publication_kind"], "portal_catalog");
    assert_eq!(quote_json["publication_status"], "published");
    assert_eq!(
        quote_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
    );
    assert_eq!(quote_json["publication_version"], 1);
    assert_eq!(quote_json["publication_source_kind"], "catalog_seed");
    assert!(quote_json["publication_effective_from_ms"].is_null());
}

#[tokio::test]
async fn preview_quote_prefers_active_pricing_governance_from_account_kernel() {
    let store = build_store().await;
    seed_pricing_plan(&store, 9103, "recharge_pack:pack-100k", 3, "active").await;
    seed_pricing_plan(&store, 9104, "recharge_pack:pack-100k", 4, "planned").await;

    let quote = preview_portal_commerce_quote(
        &store,
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("governed pricing quote");

    let quote_json = serde_json::to_value(&quote).expect("serialize quote");
    assert_eq!(quote_json["pricing_plan_version"], 3);
    assert_eq!(quote_json["publication_status"], "published");
    assert_eq!(
        quote_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v3"
    );
    assert_eq!(quote_json["publication_version"], 3);
    assert_eq!(quote_json["publication_source_kind"], "pricing_plan");
    assert_eq!(
        quote_json["publication_effective_from_ms"],
        1_710_000_000_000_u64
    );
}

#[tokio::test]
async fn preview_quote_marks_governed_planned_catalog_as_draft_when_no_active_plan_exists() {
    let store = build_store().await;
    seed_pricing_plan(&store, 9204, "recharge_pack:pack-100k", 4, "planned").await;

    let quote = preview_portal_commerce_quote(
        &store,
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("planned pricing quote");

    let quote_json = serde_json::to_value(&quote).expect("serialize quote");
    assert_eq!(quote_json["pricing_plan_version"], 4);
    assert_eq!(quote_json["publication_status"], "draft");
    assert_eq!(
        quote_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v4"
    );
    assert_eq!(quote_json["publication_version"], 4);
    assert_eq!(quote_json["publication_source_kind"], "pricing_plan");
    assert_eq!(
        quote_json["publication_effective_from_ms"],
        1_710_000_000_000_u64
    );
}

#[tokio::test]
async fn submitted_order_freezes_quote_pricing_binding_in_snapshot() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "LAUNCH20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-1",
        "project-1",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("launch20".to_owned()),
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("marketing coupon order");

    assert_eq!(
        order.pricing_plan_id.as_deref(),
        Some("pricing_plan:recharge_pack:pack-100k")
    );
    assert_eq!(order.pricing_plan_version, Some(1));

    let snapshot: serde_json::Value =
        serde_json::from_str(&order.pricing_snapshot_json).expect("deserialize pricing snapshot");
    assert_eq!(
        snapshot["quote"]["pricing_plan_id"],
        "pricing_plan:recharge_pack:pack-100k"
    );
    assert_eq!(snapshot["quote"]["pricing_plan_version"], 1);
    assert_eq!(
        snapshot["quote"]["pricing_rate_id"],
        "pricing_rate:recharge_pack:pack-100k:credit.prepaid_pack"
    );
    assert_eq!(
        snapshot["quote"]["pricing_metric_code"],
        "credit.prepaid_pack"
    );
    assert_eq!(
        snapshot["pricing_binding"]["pricing_plan_id"],
        "pricing_plan:recharge_pack:pack-100k"
    );
    assert_eq!(snapshot["pricing_binding"]["pricing_plan_version"], 1);
    assert_eq!(
        snapshot["pricing_binding"]["pricing_rate_id"],
        "pricing_rate:recharge_pack:pack-100k:credit.prepaid_pack"
    );
    assert_eq!(
        snapshot["pricing_binding"]["pricing_metric_code"],
        "credit.prepaid_pack"
    );
    assert_eq!(
        snapshot["quote"]["product_id"],
        "product:recharge_pack:pack-100k"
    );
    assert_eq!(
        snapshot["quote"]["offer_id"],
        "offer:recharge_pack:pack-100k"
    );
    assert_eq!(
        snapshot["quote"]["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(snapshot["quote"]["publication_kind"], "portal_catalog");
    assert_eq!(snapshot["quote"]["publication_status"], "published");
    assert_eq!(
        snapshot["quote"]["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
    );
    assert_eq!(snapshot["quote"]["publication_version"], 1);
    assert_eq!(snapshot["quote"]["publication_source_kind"], "catalog_seed");
    assert_eq!(
        snapshot["catalog_binding"]["product_id"],
        "product:recharge_pack:pack-100k"
    );
    assert_eq!(
        snapshot["catalog_binding"]["offer_id"],
        "offer:recharge_pack:pack-100k"
    );
    assert_eq!(
        snapshot["catalog_binding"]["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(
        snapshot["catalog_binding"]["publication_kind"],
        "portal_catalog"
    );
    assert_eq!(
        snapshot["catalog_binding"]["publication_status"],
        "published"
    );
    assert_eq!(
        snapshot["catalog_binding"]["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
    );
    assert_eq!(snapshot["catalog_binding"]["publication_version"], 1);
    assert_eq!(
        snapshot["catalog_binding"]["publication_source_kind"],
        "catalog_seed"
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
