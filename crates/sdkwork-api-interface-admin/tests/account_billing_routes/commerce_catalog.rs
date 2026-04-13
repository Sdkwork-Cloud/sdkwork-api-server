use super::*;

const FUTURE_PUBLICATION_EFFECTIVE_FROM_MS: u64 = 4_102_444_800_000;

#[tokio::test]
async fn admin_commerce_catalog_publications_expose_canonical_product_offer_publication_chain() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let draft_pack_plan = PricingPlanRecord::new(9102, 1001, 2002, "recharge_pack:pack-100k", 4)
        .with_display_name("Boost Pack v4")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(FUTURE_PUBLICATION_EFFECTIVE_FROM_MS)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    store
        .insert_pricing_plan_record(&draft_pack_plan)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/catalog-publications")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert!(json.as_array().unwrap().len() >= 3);

    let publication = json
        .as_array()
        .unwrap()
        .iter()
        .find(|item| {
            item["publication"]["publication_id"]
                == "publication:portal_catalog:offer:recharge_pack:pack-100k"
        })
        .expect("pack publication projection should exist");

    assert_eq!(
        publication["product"]["product_id"],
        "product:recharge_pack:pack-100k"
    );
    assert_eq!(publication["product"]["product_kind"], "recharge_pack");
    assert_eq!(
        publication["offer"]["offer_id"],
        "offer:recharge_pack:pack-100k"
    );
    assert_eq!(
        publication["offer"]["pricing_plan_id"],
        "pricing_plan:recharge_pack:pack-100k"
    );
    assert_eq!(publication["offer"]["pricing_plan_version"], 4);
    assert_eq!(
        publication["offer"]["pricing_metric_code"],
        "credit.prepaid_pack"
    );
    assert_eq!(
        publication["publication"]["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v4"
    );
    assert_eq!(publication["publication"]["publication_version"], 4);
    assert_eq!(
        publication["publication"]["publication_source_kind"],
        "pricing_plan"
    );
    assert_eq!(publication["publication"]["status"], "draft");
    assert_eq!(
        publication["publication"]["publication_effective_from_ms"],
        FUTURE_PUBLICATION_EFFECTIVE_FROM_MS
    );
}

#[tokio::test]
async fn admin_commerce_catalog_publication_detail_exposes_governed_pricing_context() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let draft_pack_plan = PricingPlanRecord::new(9102, 1001, 2002, "recharge_pack:pack-100k", 4)
        .with_display_name("Boost Pack v4")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(FUTURE_PUBLICATION_EFFECTIVE_FROM_MS)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let input_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "credit.prepaid_pack")
        .with_charge_unit("credit")
        .with_pricing_method("flat")
        .with_quantity_step(1.0)
        .with_unit_price(10.0)
        .with_display_price_unit("USD / pack")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let output_rate = PricingRateRecord::new(9203, 1001, 2002, 9102, "credit.bonus")
        .with_charge_unit("credit")
        .with_pricing_method("flat")
        .with_quantity_step(1.0)
        .with_unit_price(0.0)
        .with_display_price_unit("bonus")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);
    store
        .insert_pricing_plan_record(&draft_pack_plan)
        .await
        .unwrap();
    store.insert_pricing_rate_record(&input_rate).await.unwrap();
    store
        .insert_pricing_rate_record(&output_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:recharge_pack:pack-100k")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["projection"]["publication"]["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(json["governed_pricing_plan"]["pricing_plan_id"], 9102);
    assert_eq!(json["governed_pricing_plan"]["plan_version"], 4);
    assert_eq!(
        json["governed_pricing_plan"]["effective_from_ms"],
        FUTURE_PUBLICATION_EFFECTIVE_FROM_MS
    );
    assert_eq!(
        json["projection"]["publication"]["publication_effective_from_ms"],
        FUTURE_PUBLICATION_EFFECTIVE_FROM_MS
    );
    assert_eq!(json["governed_pricing_rates"].as_array().unwrap().len(), 2);
    assert!(json["governed_pricing_rates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rate| rate["pricing_rate_id"] == 9202));
    assert!(json["governed_pricing_rates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rate| rate["pricing_rate_id"] == 9203));
    assert_eq!(json["actionability"]["publish"]["allowed"], false);
    assert!(
        json["actionability"]["publish"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason
                == "publication effective_from_ms is in the future; schedule instead")
    );
    assert_eq!(json["actionability"]["schedule"]["allowed"], true);
    assert!(json["actionability"]["schedule"]["reasons"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(json["actionability"]["retire"]["allowed"], true);
    assert!(json["actionability"]["retire"]["reasons"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn admin_commerce_catalog_publication_detail_marks_catalog_seed_publication_not_actionable() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:subscription_plan:growth")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert!(json["governed_pricing_plan"].is_null());
    assert!(json["governed_pricing_rates"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        json["projection"]["publication"]["publication_source_kind"],
        "catalog_seed"
    );
    assert_eq!(json["actionability"]["publish"]["allowed"], false);
    assert_eq!(json["actionability"]["schedule"]["allowed"], false);
    assert_eq!(json["actionability"]["retire"]["allowed"], false);
    assert!(json["actionability"]["publish"]["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason
            == "publication is derived from catalog_seed and has no governed pricing plan"));
}

#[tokio::test]
async fn admin_commerce_catalog_publication_detail_returns_not_found_for_unknown_publication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:recharge_pack:missing-pack")
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
        "publication publication:portal_catalog:offer:recharge_pack:missing-pack does not exist"
    );
}

#[tokio::test]
async fn admin_commerce_catalog_publication_publish_mutation_updates_governed_publication_and_records_audit(
) {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let draft_pack_plan = PricingPlanRecord::new(9102, 1001, 2002, "recharge_pack:pack-100k", 4)
        .with_display_name("Boost Pack v4")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(10)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let input_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "credit.prepaid_pack")
        .with_charge_unit("credit")
        .with_pricing_method("flat")
        .with_quantity_step(1.0)
        .with_unit_price(10.0)
        .with_display_price_unit("USD / pack")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    store
        .insert_pricing_plan_record(&draft_pack_plan)
        .await
        .unwrap();
    store.insert_pricing_rate_record(&input_rate).await.unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:recharge_pack:pack-100k/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-publication-publish-1")
                .body(Body::from(
                    r#"{"reason":"publish canonical recharge pack revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["detail"]["projection"]["publication"]["status"],
        "published"
    );
    assert_eq!(json["detail"]["governed_pricing_plan"]["status"], "active");
    assert_eq!(json["detail"]["actionability"]["publish"]["allowed"], false);
    assert_eq!(json["detail"]["actionability"]["retire"]["allowed"], true);
    assert_eq!(json["audit"]["action"], "publish");
    assert_eq!(json["audit"]["outcome"], "applied");
    assert_eq!(json["audit"]["operator_id"], "admin_local_default");
    assert_eq!(
        json["audit"]["request_id"],
        "sdkw-test-publication-publish-1"
    );
    assert_eq!(
        json["audit"]["operator_reason"],
        "publish canonical recharge pack revision"
    );
    assert_eq!(json["audit"]["publication_status_before"], "draft");
    assert_eq!(json["audit"]["publication_status_after"], "published");
    assert_eq!(json["audit"]["governed_pricing_status_before"], "draft");
    assert_eq!(json["audit"]["governed_pricing_status_after"], "active");

    let audit_records = store
        .list_catalog_publication_lifecycle_audit_records()
        .await
        .unwrap();
    assert_eq!(audit_records.len(), 1);
    assert_eq!(audit_records[0].action.as_str(), "publish");
    assert_eq!(audit_records[0].outcome.as_str(), "applied");
    assert_eq!(
        audit_records[0].request_id,
        "sdkw-test-publication-publish-1"
    );
}

#[tokio::test]
async fn admin_commerce_catalog_publication_schedule_mutation_marks_publication_as_already_scheduled(
) {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let future_pack_plan = PricingPlanRecord::new(9102, 1001, 2002, "recharge_pack:pack-100k", 4)
        .with_display_name("Boost Pack v4")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(FUTURE_PUBLICATION_EFFECTIVE_FROM_MS)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let input_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "credit.prepaid_pack")
        .with_charge_unit("credit")
        .with_pricing_method("flat")
        .with_quantity_step(1.0)
        .with_unit_price(10.0)
        .with_display_price_unit("USD / pack")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    store
        .insert_pricing_plan_record(&future_pack_plan)
        .await
        .unwrap();
    store.insert_pricing_rate_record(&input_rate).await.unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:recharge_pack:pack-100k/schedule")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-publication-schedule-1")
                .body(Body::from(
                    r#"{"reason":"schedule canonical recharge pack revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["detail"]["projection"]["publication"]["status"],
        "draft"
    );
    assert_eq!(json["detail"]["governed_pricing_plan"]["status"], "planned");
    assert_eq!(json["detail"]["actionability"]["publish"]["allowed"], false);
    assert_eq!(
        json["detail"]["actionability"]["schedule"]["allowed"],
        false
    );
    assert!(json["detail"]["actionability"]["schedule"]["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "publication is already scheduled"));
    assert_eq!(json["audit"]["action"], "schedule");
    assert_eq!(json["audit"]["outcome"], "applied");
    assert_eq!(json["audit"]["publication_status_before"], "draft");
    assert_eq!(json["audit"]["publication_status_after"], "draft");
    assert_eq!(json["audit"]["governed_pricing_status_before"], "draft");
    assert_eq!(json["audit"]["governed_pricing_status_after"], "planned");
}

#[tokio::test]
async fn admin_commerce_catalog_publication_retire_mutation_archives_publication_and_records_audit()
{
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_pack_plan = PricingPlanRecord::new(9102, 1001, 2002, "recharge_pack:pack-100k", 4)
        .with_display_name("Boost Pack v4")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(10)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let input_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "credit.prepaid_pack")
        .with_charge_unit("credit")
        .with_pricing_method("flat")
        .with_quantity_step(1.0)
        .with_unit_price(10.0)
        .with_display_price_unit("USD / pack")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    store
        .insert_pricing_plan_record(&active_pack_plan)
        .await
        .unwrap();
    store.insert_pricing_rate_record(&input_rate).await.unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:recharge_pack:pack-100k/retire")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-publication-retire-1")
                .body(Body::from(
                    r#"{"reason":"retire canonical recharge pack revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["detail"]["projection"]["publication"]["status"],
        "archived"
    );
    assert_eq!(
        json["detail"]["governed_pricing_plan"]["status"],
        "archived"
    );
    assert_eq!(json["detail"]["actionability"]["retire"]["allowed"], false);
    assert_eq!(json["audit"]["action"], "retire");
    assert_eq!(json["audit"]["outcome"], "applied");
    assert_eq!(json["audit"]["publication_status_before"], "published");
    assert_eq!(json["audit"]["publication_status_after"], "archived");
    assert_eq!(json["audit"]["governed_pricing_status_before"], "active");
    assert_eq!(json["audit"]["governed_pricing_status_after"], "archived");
}

#[tokio::test]
async fn admin_commerce_catalog_publication_publish_mutation_rejects_catalog_seed_and_records_rejected_audit(
) {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/commerce/catalog-publications/publication:portal_catalog:offer:subscription_plan:growth/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-publication-publish-rejected-1")
                .body(Body::from(
                    r#"{"reason":"attempt to publish catalog seed publication"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "publication publication:portal_catalog:offer:subscription_plan:growth cannot be published: publication is derived from catalog_seed and has no governed pricing plan"
    );

    let audit_records = store
        .list_catalog_publication_lifecycle_audit_records()
        .await
        .unwrap();
    assert_eq!(audit_records.len(), 1);
    assert_eq!(audit_records[0].action.as_str(), "publish");
    assert_eq!(audit_records[0].outcome.as_str(), "rejected");
    assert_eq!(
        audit_records[0].request_id,
        "sdkw-test-publication-publish-rejected-1"
    );
    assert_eq!(
        audit_records[0].decision_reasons,
        vec!["publication is derived from catalog_seed and has no governed pricing plan"]
    );
}
