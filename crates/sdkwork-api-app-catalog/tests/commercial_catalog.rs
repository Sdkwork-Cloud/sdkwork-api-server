use sdkwork_api_app_catalog::{
    build_canonical_commercial_catalog, build_canonical_commercial_catalog_with_pricing_plans,
    normalize_commercial_pricing_plan_code, CommercialCatalogSeedProduct,
};
use sdkwork_api_domain_billing::PricingPlanRecord;
use sdkwork_api_domain_catalog::{
    ApiProductKind, CatalogPublicationKind, CatalogPublicationStatus, QuoteKind,
};

#[test]
fn builds_canonical_commercial_catalog_from_seed_products() {
    let catalog = build_canonical_commercial_catalog(&[
        CommercialCatalogSeedProduct::new(
            ApiProductKind::SubscriptionPlan,
            "growth",
            "Growth",
            "workspace_seed",
        )
        .with_price_label_option(Some("$49 / month".to_owned())),
        CommercialCatalogSeedProduct::new(
            ApiProductKind::RechargePack,
            "pack-100k",
            "100k Units",
            "workspace_seed",
        )
        .with_price_label_option(Some("$40".to_owned())),
        CommercialCatalogSeedProduct::new(
            ApiProductKind::CustomRecharge,
            "custom_recharge",
            "Custom recharge",
            "workspace_seed",
        ),
    ]);

    assert_eq!(catalog.products.len(), 3);
    assert_eq!(catalog.offers.len(), 3);
    assert_eq!(catalog.publications.len(), 3);
    assert_eq!(
        catalog.products[0].product_id,
        "product:subscription_plan:growth"
    );
    assert_eq!(catalog.offers[1].offer_id, "offer:recharge_pack:pack-100k");
    assert_eq!(catalog.offers[0].quote_kind, QuoteKind::ProductPurchase);
    assert_eq!(catalog.offers[2].price_label, None);
    let offer_json = serde_json::to_value(&catalog.offers[0]).expect("serialize offer");
    assert_eq!(
        offer_json["pricing_plan_id"],
        "pricing_plan:subscription_plan:growth"
    );
    assert_eq!(offer_json["pricing_plan_version"], 1);
    assert_eq!(
        offer_json["pricing_rate_id"],
        "pricing_rate:subscription_plan:growth:subscription.base"
    );
    assert_eq!(offer_json["pricing_metric_code"], "subscription.base");
    let publication_json =
        serde_json::to_value(&catalog.publications[0]).expect("serialize publication");
    assert_eq!(
        publication_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:subscription_plan:growth:v1"
    );
    assert_eq!(publication_json["publication_version"], 1);
    assert_eq!(publication_json["publication_source_kind"], "catalog_seed");
    assert!(publication_json["publication_effective_from_ms"].is_null());
    assert!(catalog.publications.iter().any(|publication| {
        publication.publication_kind == CatalogPublicationKind::PortalCatalog
            && publication.status == CatalogPublicationStatus::Published
            && publication.offer_id == "offer:custom_recharge:custom_recharge"
    }));
}

#[test]
fn canonical_catalog_prefers_active_pricing_governance_over_newer_planned_version() {
    let catalog = build_canonical_commercial_catalog_with_pricing_plans(
        &[CommercialCatalogSeedProduct::new(
            ApiProductKind::RechargePack,
            "pack-100k",
            "100k Units",
            "workspace_seed",
        )],
        &[
            PricingPlanRecord::new(9103, 1001, 2002, "recharge_pack:pack-100k", 3)
                .with_status("active")
                .with_effective_from_ms(10)
                .with_created_at_ms(10)
                .with_updated_at_ms(10),
            PricingPlanRecord::new(9104, 1001, 2002, "recharge_pack:pack-100k", 4)
                .with_status("planned")
                .with_effective_from_ms(20)
                .with_created_at_ms(20)
                .with_updated_at_ms(20),
        ],
    );

    assert_eq!(catalog.offers[0].pricing_plan_version, Some(3));
    assert_eq!(
        catalog.publications[0].status,
        CatalogPublicationStatus::Published
    );
    let publication_json =
        serde_json::to_value(&catalog.publications[0]).expect("serialize governed publication");
    assert_eq!(
        publication_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v3"
    );
    assert_eq!(publication_json["publication_version"], 3);
    assert_eq!(publication_json["publication_source_kind"], "pricing_plan");
    assert_eq!(publication_json["publication_effective_from_ms"], 10);
}

#[test]
fn canonical_catalog_marks_publication_draft_when_only_planned_governance_exists() {
    let catalog = build_canonical_commercial_catalog_with_pricing_plans(
        &[CommercialCatalogSeedProduct::new(
            ApiProductKind::RechargePack,
            "pack-100k",
            "100k Units",
            "workspace_seed",
        )],
        &[
            PricingPlanRecord::new(9204, 1001, 2002, "recharge_pack:pack-100k", 4)
                .with_status("planned")
                .with_effective_from_ms(20)
                .with_created_at_ms(20)
                .with_updated_at_ms(20),
        ],
    );

    assert_eq!(catalog.offers[0].pricing_plan_version, Some(4));
    assert_eq!(
        catalog.publications[0].status,
        CatalogPublicationStatus::Draft
    );
    let publication_json =
        serde_json::to_value(&catalog.publications[0]).expect("serialize planned publication");
    assert_eq!(
        publication_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v4"
    );
    assert_eq!(publication_json["publication_version"], 4);
    assert_eq!(publication_json["publication_source_kind"], "pricing_plan");
    assert_eq!(publication_json["publication_effective_from_ms"], 20);
}

#[test]
fn canonical_catalog_matches_normalized_commercial_plan_code_variants() {
    let catalog = build_canonical_commercial_catalog_with_pricing_plans(
        &[CommercialCatalogSeedProduct::new(
            ApiProductKind::SubscriptionPlan,
            "growth",
            "Growth",
            "workspace_seed",
        )],
        &[
            PricingPlanRecord::new(9105, 1001, 2002, " Subscription-Plan : growth ", 2)
                .with_status("active")
                .with_effective_from_ms(10)
                .with_created_at_ms(10)
                .with_updated_at_ms(10),
        ],
    );

    assert_eq!(catalog.offers[0].pricing_plan_version, Some(2));
    assert_eq!(
        catalog.publications[0].status,
        CatalogPublicationStatus::Published
    );
}

#[test]
fn normalizes_canonical_commercial_pricing_plan_code_variants() {
    assert_eq!(
        normalize_commercial_pricing_plan_code(" Subscription-Plan : growth ")
            .expect("normalize commercial code"),
        Some("subscription_plan:growth".to_owned())
    );
    assert_eq!(
        normalize_commercial_pricing_plan_code("retail-pro").expect("preserve generic code"),
        None
    );
}

#[test]
fn rejects_canonical_commercial_pricing_plan_codes_without_target_id() {
    let error = normalize_commercial_pricing_plan_code("recharge_pack:   ")
        .expect_err("empty target id should be rejected");
    assert_eq!(
        error.to_string(),
        "canonical commercial pricing plan code requires a non-empty target_id"
    );
}
