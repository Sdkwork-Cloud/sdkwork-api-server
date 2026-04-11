use sdkwork_api_domain_catalog::{
    ApiProduct, ApiProductKind, CatalogPublication, CatalogPublicationKind,
    CatalogPublicationStatus, ProductOffer, QuoteKind,
};

#[test]
fn commercial_catalog_records_use_explicit_product_offer_and_publication_semantics() {
    let product = ApiProduct::new(
        "product:subscription_plan:growth",
        ApiProductKind::SubscriptionPlan,
        "growth",
        "Growth",
    )
    .with_source("workspace_seed");
    let offer = ProductOffer::new(
        "offer:subscription_plan:growth",
        product.product_id.clone(),
        ApiProductKind::SubscriptionPlan,
        "Growth",
        QuoteKind::ProductPurchase,
        ApiProductKind::SubscriptionPlan,
        "growth",
    )
    .with_price_label_option(Some("$49 / month".to_owned()))
    .with_source("workspace_seed");
    let publication = CatalogPublication::new(
        "publication:portal_catalog:offer:subscription_plan:growth",
        product.product_id.clone(),
        offer.offer_id.clone(),
        CatalogPublicationKind::PortalCatalog,
    )
    .with_status(CatalogPublicationStatus::Published)
    .with_source("workspace_seed");

    assert_eq!(
        ApiProductKind::SubscriptionPlan.as_str(),
        "subscription_plan"
    );
    assert_eq!(QuoteKind::ProductPurchase.as_str(), "product_purchase");
    assert_eq!(
        CatalogPublicationKind::PortalCatalog.as_str(),
        "portal_catalog"
    );
    assert_eq!(CatalogPublicationStatus::Published.as_str(), "published");
    assert_eq!(product.product_kind, ApiProductKind::SubscriptionPlan);
    assert_eq!(offer.quote_kind, QuoteKind::ProductPurchase);
    assert_eq!(offer.quote_target_kind, ApiProductKind::SubscriptionPlan);
    assert_eq!(
        publication.publication_kind,
        CatalogPublicationKind::PortalCatalog
    );
    assert_eq!(publication.status, CatalogPublicationStatus::Published);
    assert_eq!(
        publication.publication_revision_id,
        "publication:portal_catalog:offer:subscription_plan:growth"
    );
    assert_eq!(publication.publication_version, 1);
    assert_eq!(publication.publication_source_kind, "catalog_seed");
    assert_eq!(publication.publication_effective_from_ms, None);
}
