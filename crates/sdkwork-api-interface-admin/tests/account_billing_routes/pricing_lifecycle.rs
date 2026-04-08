use super::*;

#[tokio::test]
async fn admin_billing_pricing_management_routes_publish_cloned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let draft_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let draft_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store.insert_pricing_plan_record(&draft_plan).await.unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store.insert_pricing_rate_record(&draft_rate).await.unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let publish_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9102/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(publish_plan.status(), StatusCode::OK);
    let published_plan_json = read_json(publish_plan).await;
    assert_eq!(published_plan_json["pricing_plan_id"], 9102);
    assert_eq!(published_plan_json["status"], "active");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let published_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(published_plan.status, "active");
    assert_eq!(archived_plan.status, "archived");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let published_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(published_rate.status, "active");
    assert_eq!(archived_rate.status, "archived");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_reject_publish_for_future_effective_plan_versions()
{
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let future_draft_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let future_draft_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_draft_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_draft_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let publish_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9102/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(publish_plan.status(), StatusCode::BAD_REQUEST);
    let error_json = read_json(publish_plan).await;
    assert_eq!(
        error_json["error"]["message"],
        "pricing plan 9102 cannot be published before effective_from_ms"
    );

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let stored_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let stored_active_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(stored_future_plan.status, "draft");
    assert_eq!(stored_active_plan.status, "active");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let stored_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let stored_active_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(stored_future_rate.status, "draft");
    assert_eq!(stored_active_rate.status, "active");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_schedule_future_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1_717_171_700_000)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let future_draft_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro Summer")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let future_draft_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_draft_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_draft_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let schedule_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9102/schedule")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(schedule_plan.status(), StatusCode::OK);
    let scheduled_plan_json = read_json(schedule_plan).await;
    assert_eq!(scheduled_plan_json["pricing_plan_id"], 9102);
    assert_eq!(scheduled_plan_json["status"], "planned");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let scheduled_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_active_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(scheduled_plan.status, "planned");
    assert_eq!(still_active_plan.status, "active");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let scheduled_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_active_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(scheduled_rate.status, "planned");
    assert_eq!(still_active_rate.status, "active");
}

#[tokio::test]
async fn admin_billing_pricing_reads_auto_activate_due_planned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let due_planned_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(2)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let future_planned_plan = PricingPlanRecord::new(9103, 1001, 2002, "retail-pro", 3)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);
    let due_planned_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let future_planned_rate = PricingRateRecord::new(9203, 1001, 2002, 9103, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(3.1)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&due_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&due_planned_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_planned_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let plans = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(plans.status(), StatusCode::OK);
    let plans_json = read_json(plans).await;
    assert_eq!(plans_json.as_array().unwrap().len(), 3);
    assert_eq!(plans_json[0]["status"], "archived");
    assert_eq!(plans_json[1]["status"], "active");
    assert_eq!(plans_json[2]["status"], "planned");

    let rates = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(rates.status(), StatusCode::OK);
    let rates_json = read_json(rates).await;
    assert_eq!(rates_json.as_array().unwrap().len(), 3);
    assert_eq!(rates_json[0]["status"], "archived");
    assert_eq!(rates_json[1]["status"], "active");
    assert_eq!(rates_json[2]["status"], "planned");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    let activated_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9103)
        .unwrap();
    assert_eq!(archived_plan.status, "archived");
    assert_eq!(activated_plan.status, "active");
    assert_eq!(still_future_plan.status, "planned");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    let activated_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9203)
        .unwrap();
    assert_eq!(archived_rate.status, "archived");
    assert_eq!(activated_rate.status, "active");
    assert_eq!(still_future_rate.status, "planned");
}

#[tokio::test]
async fn admin_billing_pricing_lifecycle_sync_route_activates_due_planned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let due_planned_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(2)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let future_planned_plan = PricingPlanRecord::new(9103, 1001, 2002, "retail-pro", 3)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);
    let due_planned_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let future_planned_rate = PricingRateRecord::new(9203, 1001, 2002, 9103, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(3.1)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&due_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&due_planned_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_planned_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-lifecycle/synchronize")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["changed"], true);
    assert_eq!(response_json["due_group_count"], 1);
    assert_eq!(response_json["activated_plan_count"], 1);
    assert_eq!(response_json["archived_plan_count"], 1);
    assert_eq!(response_json["activated_rate_count"], 1);
    assert_eq!(response_json["archived_rate_count"], 1);
    assert_eq!(response_json["skipped_plan_count"], 0);
    assert!(
        response_json["synchronized_at_ms"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    let activated_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9103)
        .unwrap();
    assert_eq!(archived_plan.status, "archived");
    assert_eq!(activated_plan.status, "active");
    assert_eq!(still_future_plan.status, "planned");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    let activated_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9203)
        .unwrap();
    assert_eq!(archived_rate.status, "archived");
    assert_eq!(activated_rate.status, "active");
    assert_eq!(still_future_rate.status, "planned");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_retire_plan_versions_and_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let retire_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9101/retire")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retire_plan.status(), StatusCode::OK);
    let retired_plan_json = read_json(retire_plan).await;
    assert_eq!(retired_plan_json["pricing_plan_id"], 9101);
    assert_eq!(retired_plan_json["status"], "archived");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    assert_eq!(stored_plans.len(), 1);
    let retired_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(retired_plan.status, "archived");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    assert_eq!(stored_rates.len(), 1);
    let retired_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(retired_rate.status, "archived");
}
