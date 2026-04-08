use super::*;

#[tokio::test]
async fn admin_billing_pricing_management_routes_create_canonical_plans_and_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let existing_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    store
        .insert_pricing_plan_record(&existing_plan)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_plan = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"plan_code":"media-studio","plan_version":2,"display_name":"Media Studio","currency_code":"USD","credit_unit_code":"credit","status":"draft","effective_from_ms":1717171730000,"effective_to_ms":1719773730000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_plan.status(), StatusCode::CREATED);
    let created_plan_json = read_json(create_plan).await;
    assert_eq!(created_plan_json["plan_code"], "media-studio");
    assert_eq!(created_plan_json["plan_version"], 2);
    assert_eq!(created_plan_json["display_name"], "Media Studio");
    assert_eq!(created_plan_json["effective_from_ms"], 1717171730000u64);
    assert_eq!(created_plan_json["effective_to_ms"], 1719773730000u64);
    assert!(created_plan_json["pricing_plan_id"].as_u64().unwrap() > 0);

    let create_rate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"pricing_plan_id":9101,"metric_code":"image.output","capability_code":"image_generation","model_code":"gpt-image-1","provider_code":"provider-openai-official","charge_unit":"image","pricing_method":"per_unit","quantity_step":1.0,"unit_price":0.08,"display_price_unit":"USD / image","minimum_billable_quantity":1.0,"minimum_charge":0.08,"rounding_increment":1.0,"rounding_mode":"ceil","included_quantity":0.0,"priority":200,"notes":"Image generation retail pricing","status":"active"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_rate.status(), StatusCode::CREATED);
    let created_rate_json = read_json(create_rate).await;
    assert_eq!(created_rate_json["pricing_plan_id"], 9101);
    assert_eq!(created_rate_json["metric_code"], "image.output");
    assert_eq!(created_rate_json["capability_code"], "image_generation");
    assert_eq!(created_rate_json["charge_unit"], "image");
    assert_eq!(created_rate_json["pricing_method"], "per_unit");
    assert_eq!(created_rate_json["display_price_unit"], "USD / image");
    assert_eq!(created_rate_json["rounding_mode"], "ceil");
    assert_eq!(created_rate_json["status"], "active");
    assert!(created_rate_json["pricing_rate_id"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_update_canonical_plans_and_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let existing_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1717171730000)
        .with_effective_to_ms(Some(1718035730000))
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let existing_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_capability_code(Some("responses".to_owned()))
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
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

    store
        .insert_pricing_plan_record(&existing_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&existing_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let update_plan = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/admin/billing/pricing-plans/9101")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"plan_code":"retail-pro","plan_version":2,"display_name":"Retail Pro Updated","currency_code":"USD","credit_unit_code":"credit","status":"draft","effective_from_ms":1718035730000,"effective_to_ms":1720627730000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_plan.status(), StatusCode::OK);
    let updated_plan_json = read_json(update_plan).await;
    assert_eq!(updated_plan_json["pricing_plan_id"], 9101);
    assert_eq!(updated_plan_json["plan_version"], 2);
    assert_eq!(updated_plan_json["display_name"], "Retail Pro Updated");
    assert_eq!(updated_plan_json["status"], "draft");
    assert_eq!(updated_plan_json["effective_from_ms"], 1718035730000u64);
    assert_eq!(updated_plan_json["effective_to_ms"], 1720627730000u64);

    let update_rate = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/admin/billing/pricing-rates/9201")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"pricing_plan_id":9101,"metric_code":"image.output","capability_code":"images","model_code":"gpt-image-1","provider_code":"provider-openai-official","charge_unit":"image","pricing_method":"flat","quantity_step":1.0,"unit_price":0.08,"display_price_unit":"USD / image","minimum_billable_quantity":1.0,"minimum_charge":0.08,"rounding_increment":1.0,"rounding_mode":"ceil","included_quantity":0.0,"priority":200,"notes":"Updated image pricing","status":"draft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_rate.status(), StatusCode::OK);
    let updated_rate_json = read_json(update_rate).await;
    assert_eq!(updated_rate_json["pricing_rate_id"], 9201);
    assert_eq!(updated_rate_json["metric_code"], "image.output");
    assert_eq!(updated_rate_json["capability_code"], "images");
    assert_eq!(updated_rate_json["charge_unit"], "image");
    assert_eq!(updated_rate_json["pricing_method"], "flat");
    assert_eq!(updated_rate_json["display_price_unit"], "USD / image");
    assert_eq!(updated_rate_json["minimum_charge"], 0.08);
    assert_eq!(updated_rate_json["priority"], 200);
    assert_eq!(updated_rate_json["status"], "draft");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_clone_canonical_plan_versions_with_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let existing_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let existing_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_capability_code(Some("responses".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
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

    store
        .insert_pricing_plan_record(&existing_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&existing_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let clone_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9101/clone")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(clone_plan.status(), StatusCode::CREATED);
    let cloned_plan_json = read_json(clone_plan).await;
    let cloned_plan_id = cloned_plan_json["pricing_plan_id"].as_u64().unwrap();
    assert!(cloned_plan_id > 9101);
    assert_eq!(cloned_plan_json["plan_code"], "retail-pro");
    assert_eq!(cloned_plan_json["plan_version"], 2);
    assert_eq!(cloned_plan_json["status"], "draft");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    assert_eq!(stored_plans.len(), 2);

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    assert_eq!(stored_rates.len(), 2);
    let cloned_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_plan_id == cloned_plan_id)
        .unwrap();
    assert_eq!(cloned_rate.metric_code, "token.input");
    assert_eq!(cloned_rate.charge_unit, "input_token");
    assert_eq!(cloned_rate.pricing_method, "per_unit");
    assert_eq!(cloned_rate.display_price_unit, "USD / 1M input tokens");
    assert_eq!(cloned_rate.status, "draft");
}
